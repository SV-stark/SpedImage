//! Clipboard access.
//!
//! Windows is handled here directly with Win32, the same way
//! [`set_file_drop`] was already handled, because `arboard`'s default
//! `image-data` feature drags the entire `image` crate (plus a fourth PNG
//! decoder and, on macOS, a second TIFF crate) into the build for two call
//! sites. Three clipboard formats are needed:
//!
//! * `CF_UNICODETEXT` — "copy file path".
//! * `CF_DIB` — the image itself, which is what every image viewer pastes.
//! * `CF_HDROP` — the source file, so Explorer and Office can paste a real
//!   file rather than a bitmap.
//!
//! On other platforms this defers to `arboard` for text, and for images only
//! when the `clipboard-image` feature is enabled.
//!
//! The DIB pack/unpack logic is pure and unit-tested at the bottom of this
//! file; the Win32 calls around it are the only part that cannot run in CI.

use std::path::Path;

/// True when this build can put an image on / take an image off the clipboard.
pub const IMAGE_SUPPORTED: bool = cfg!(any(windows, feature = "clipboard-image"));

// ── Windows ────────────────────────────────────────────────────────────────

#[cfg(windows)]
mod win {
    use super::parse_dib;
    use std::path::Path;
    use windows::Win32::Foundation::{HANDLE, HGLOBAL};
    use windows::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
        OpenClipboard, SetClipboardData,
    };
    use windows::Win32::System::Memory::{
        GLOBAL_ALLOC_FLAGS, GMEM_MOVEABLE, GMEM_ZEROINIT, GlobalAlloc, GlobalLock, GlobalSize,
        GlobalUnlock,
    };

    pub const CF_DIB: u32 = 8;
    pub const CF_UNICODETEXT: u32 = 13;
    pub const CF_HDROP: u32 = 15;
    pub const CF_DIBV5: u32 = 17;

    /// `BI_RGB` — the only compression a plain CF_DIB uses.
    pub const BI_RGB: u32 = 0;
    /// `BI_BITFIELDS` — 32bpp with explicit channel masks; treated as BGRA.
    pub const BI_BITFIELDS: u32 = 3;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct BitMapInfoHeader {
        pub size: u32,
        pub width: i32,
        /// Positive for the bottom-up rows Windows itself produces; negative
        /// for the top-down rows most other producers write.
        pub height: i32,
        pub planes: u16,
        pub bit_count: u16,
        pub compression: u32,
        pub size_image: u32,
        pub x_pels_per_meter: i32,
        pub y_pels_per_meter: i32,
        pub clr_used: u32,
        pub clr_important: u32,
    }

    unsafe impl Sync for BitMapInfoHeader {}

    /// A `DROPFILES` header, as laid out by the shell. Only the two fields we
    /// set are named; the rest of the struct is padding the shell ignores.
    #[repr(C)]
    struct DropFilesHeader {
        p_files: u32,
        point: windows::Win32::Foundation::POINT,
        f_nc: u32,
        f_wide: u32,
    }

    fn alloc(size: usize) -> Option<HGLOBAL> {
        let flags = GLOBAL_ALLOC_FLAGS(GMEM_MOVEABLE.0 | GMEM_ZEROINIT.0);
        let h = unsafe { GlobalAlloc(flags, size) }.ok()?;
        if h.is_invalid() {
            return None;
        }
        Some(h)
    }

    /// Run `f` with the clipboard open, retrying while another process holds it.
    ///
    /// `OpenClipboard` fails with `CLIPRUSHIGHCOUNT` when the owner of the
    /// clipboard has it open, which is common while a user is dragging files
    /// around, so a short bounded retry beats a spurious error.
    fn with_open_clipboard<T>(mut f: impl FnMut() -> Result<T, String>) -> Result<T, String> {
        let mut last = String::from("could not open the clipboard");
        for attempt in 0..8u32 {
            if let Err(e) = unsafe { OpenClipboard(None) } {
                last = format!("OpenClipboard failed: {e:?}");
                std::thread::sleep(std::time::Duration::from_millis(10 * (attempt as u64 + 1)));
                continue;
            }
            // The clipboard must be closed on every path out of the loop body.
            let out = f();
            let _ = unsafe { CloseClipboard() };
            return out;
        }
        Err(last)
    }

    /// Put a moveable global block on the clipboard in `format`.
    ///
    /// The clipboard takes ownership: after a successful `SetClipboardData` the
    /// block must not be freed, and on failure it must be.
    fn set_block(format: u32, block: HGLOBAL) -> Result<(), String> {
        let ok = unsafe { SetClipboardData(format, Some(HANDLE(block.0))) }.is_ok();
        if ok {
            Ok(())
        } else {
            Err(format!(
                "SetClipboardData({format}) failed: {:?}",
                std::io::Error::last_os_error()
            ))
        }
    }

    pub fn set_text(text: &str) -> Result<(), String> {
        let mut wide: Vec<u16> = text.encode_utf16().collect();
        wide.push(0);
        let bytes = wide.len() * 2;
        let h = alloc(bytes).ok_or("GlobalAlloc failed for clipboard text")?;

        let ptr = unsafe { GlobalLock(h) };
        if ptr.is_null() {
            return Err("GlobalLock failed for clipboard text".to_string());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
        }
        let _ = unsafe { GlobalUnlock(h) };

        with_open_clipboard(|| {
            // Replacing the text means replacing the clipboard, not adding to it.
            unsafe { EmptyClipboard() }.ok();
            set_block(CF_UNICODETEXT, h)
        })
    }

    pub fn set_image(rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
        let (h, header) = build_dib(rgba, width, height);
        let ptr = unsafe { GlobalLock(h) };
        if ptr.is_null() {
            return Err("GlobalLock failed for clipboard image".to_string());
        }
        unsafe {
            std::ptr::copy_nonoverlapping(
                &header as *const BitMapInfoHeader as *const u8,
                ptr as *mut u8,
                std::mem::size_of::<BitMapInfoHeader>(),
            );
            write_bgra(
                (ptr as *mut u8).add(std::mem::size_of::<BitMapInfoHeader>()),
                rgba,
                width as usize,
                height as usize,
            );
        }
        let _ = unsafe { GlobalUnlock(h) };

        with_open_clipboard(|| {
            unsafe { EmptyClipboard() }.ok();
            set_block(CF_DIB, h)
        })
    }

    /// Add `CF_HDROP` alongside whatever is already on the clipboard.
    ///
    /// Unlike the image write this does not empty the clipboard first, so
    /// `Ctrl+C` still yields both a bitmap and the file it came from.
    pub fn set_file_drop(paths: &[&Path]) -> Result<(), String> {
        use std::os::windows::ffi::OsStrExt;

        let mut wide: Vec<u16> = Vec::new();
        for p in paths {
            let abs = if p.is_absolute() {
                p.to_path_buf()
            } else {
                std::env::current_dir().unwrap_or_default().join(p)
            };
            wide.extend(abs.as_os_str().encode_wide());
            wide.push(0);
        }
        if wide.is_empty() {
            return Err("no path to put on the clipboard".to_string());
        }
        wide.push(0); // Double null-terminate the list.

        let header_size = std::mem::size_of::<DropFilesHeader>();
        let total = header_size + wide.len() * 2;
        let h = alloc(total).ok_or("GlobalAlloc failed for clipboard file drop")?;

        let ptr = unsafe { GlobalLock(h) };
        if ptr.is_null() {
            return Err("GlobalLock failed for clipboard file drop".to_string());
        }
        unsafe {
            let header = ptr as *mut DropFilesHeader;
            (*header).p_files = header_size as u32;
            (*header).f_wide = 1; // Unicode
            std::ptr::copy_nonoverlapping(
                wide.as_ptr(),
                (ptr as usize + header_size) as *mut u16,
                wide.len(),
            );
        }
        let _ = unsafe { GlobalUnlock(h) };

        with_open_clipboard(|| set_block(CF_HDROP, h))
    }

    /// Read a bitmap off the clipboard and return it as tightly packed RGBA8.
    pub fn get_image() -> Result<(Vec<u8>, u32, u32), String> {
        let raw = with_open_clipboard(|| {
            let format = if unsafe { IsClipboardFormatAvailable(CF_DIB) }.is_ok() {
                CF_DIB
            } else if unsafe { IsClipboardFormatAvailable(CF_DIBV5) }.is_ok() {
                CF_DIBV5
            } else {
                return Err("the clipboard holds no bitmap".to_string());
            };

            let h = unsafe { GetClipboardData(format) }
                .map_err(|_| "GetClipboardData failed".to_string())?;
            let size = unsafe { GlobalSize(HGLOBAL(h.0)) } as usize;
            let ptr = unsafe { GlobalLock(HGLOBAL(h.0)) };
            if ptr.is_null() {
                return Err("GlobalLock failed on the clipboard bitmap".to_string());
            }
            let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) }.to_vec();
            let _ = unsafe { GlobalUnlock(HGLOBAL(h.0)) };
            Ok(bytes)
        })?;

        parse_dib(&raw)
    }

    // ── DIB encoding, isolated for testing ────────────────────────────────

    /// Size a `BITMAPINFOHEADER` + 32bpp BGRA block for `rgba`.
    fn build_dib(_rgba: &[u8], width: u32, height: u32) -> (HGLOBAL, BitMapInfoHeader) {
        let header = BitMapInfoHeader {
            size: std::mem::size_of::<BitMapInfoHeader>() as u32,
            width: width as i32,
            // Bottom-up: the layout every Windows API documents, and the one
            // arboard and every other CF_DIB producer has always emitted.
            height: height as i32,
            planes: 1,
            bit_count: 32,
            compression: BI_RGB,
            size_image: width.saturating_mul(height).saturating_mul(4),
            x_pels_per_meter: 0,
            y_pels_per_meter: 0,
            clr_used: 0,
            clr_important: 0,
        };
        let bytes = std::mem::size_of::<BitMapInfoHeader>() + width as usize * height as usize * 4;
        (
            alloc(bytes).expect("GlobalAlloc for the clipboard bitmap"),
            header,
        )
    }

    pub(super) fn write_bgra(dst: *mut u8, rgba: &[u8], width: usize, height: usize) {
        let row = width * 4;
        for y in 0..height {
            // DIB rows run bottom-up, so destination row 0 is source row n-1.
            let src = &rgba[(height - 1 - y) * row..][..row];
            let out = unsafe { std::slice::from_raw_parts_mut(dst.add(y * row), row) };
            let (src_pixels, _) = src.as_chunks::<4>();
            let (out_pixels, _) = out.as_chunks_mut::<4>();
            for (d, px) in out_pixels.iter_mut().zip(src_pixels) {
                // RGBA source, BGRA destination.
                (d[0], d[1], d[2], d[3]) = (px[2], px[1], px[0], px[3]);
            }
        }
    }
}

/// Everything above is Windows-only; this is the shared surface.
#[cfg(windows)]
pub use win::{get_image, set_image, set_text};

/// Put `text` on the clipboard as `CF_UNICODETEXT`.
#[cfg(all(not(windows), feature = "clipboard-image"))]
pub fn set_text(text: &str) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    cb.set_text(text.to_string()).map_err(|e| e.to_string())
}

/// Put `text` on the clipboard.
///
/// Without the `clipboard-image` feature the non-Windows build has no arboard
/// at all, so this is a no-op that reports why.
#[cfg(all(not(windows), not(feature = "clipboard-image")))]
pub fn set_text(_text: &str) -> Result<(), String> {
    Err("clipboard support requires the `clipboard-image` feature".to_string())
}

/// Copy an RGBA8 image to the clipboard.
#[cfg(all(not(windows), feature = "clipboard-image"))]
pub fn set_image(rgba: &[u8], width: u32, height: u32) -> Result<(), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let data = arboard::ImageData {
        width: width as usize,
        height: height as usize,
        bytes: std::borrow::Cow::from(rgba),
    };
    cb.set_image(data).map_err(|e| e.to_string())
}

/// Copy an RGBA8 image to the clipboard (unsupported without the feature).
#[cfg(all(not(windows), not(feature = "clipboard-image")))]
pub fn set_image(_rgba: &[u8], _width: u32, _height: u32) -> Result<(), String> {
    Err("clipboard images require the `clipboard-image` feature".to_string())
}

/// Read an RGBA8 image from the clipboard.
#[cfg(all(not(windows), feature = "clipboard-image"))]
pub fn get_image() -> Result<(Vec<u8>, u32, u32), String> {
    let mut cb = arboard::Clipboard::new().map_err(|e| e.to_string())?;
    let img = cb.get_image().map_err(|e| e.to_string())?;
    Ok((img.bytes.into_owned(), img.width as u32, img.height as u32))
}

/// Read an RGBA8 image from the clipboard (unsupported without the feature).
#[cfg(all(not(windows), not(feature = "clipboard-image")))]
pub fn get_image() -> Result<(Vec<u8>, u32, u32), String> {
    Err("clipboard images require the `clipboard-image` feature".to_string())
}

/// Offer `paths` as files on the clipboard, alongside any other format.
#[cfg(not(windows))]
pub fn set_file_drop(_paths: &[&Path]) -> Result<(), String> {
    Err("file drops on the clipboard are only implemented on Windows".to_string())
}

/// Offer `paths` as files on the clipboard, alongside any other format.
#[cfg(windows)]
pub fn set_file_drop(paths: &[&Path]) -> Result<(), String> {
    win::set_file_drop(paths)
}

// ── DIB decoding, shared and unit-tested ──────────────────────────────────

/// Parse a `CF_DIB` payload into tightly packed RGBA8.
///
/// Handles the 32bpp and 24bpp bottom-up and top-down images that clipboard
/// producers actually emit. Anything else (palettized, 16bpp, RLE) is refused
/// by name rather than converted into the wrong colours.
///
/// A fully-zero alpha channel is treated as opaque: a 32bpp CF_DIB has no
/// defined alpha, and most producers leave that byte at 0, which would
/// otherwise paste as a fully transparent image.
#[cfg(windows)]
pub(super) fn parse_dib(raw: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    use win::{BI_BITFIELDS, BI_RGB, BitMapInfoHeader};

    const HEADER_MIN: usize = 40;
    if raw.len() < HEADER_MIN {
        return Err("clipboard bitmap is too small to hold a header".to_string());
    }
    let h: BitMapInfoHeader =
        unsafe { std::ptr::read_unaligned(raw.as_ptr() as *const BitMapInfoHeader) };
    if h.size < HEADER_MIN as u32 || h.size as usize > raw.len() {
        return Err(format!(
            "clipboard bitmap header size {} is invalid",
            h.size
        ));
    }
    let width = h.width.unsigned_abs();
    let height = h.height.unsigned_abs();
    if width == 0 || height == 0 {
        return Err("clipboard bitmap reports a zero dimension".to_string());
    }

    // Rows run bottom-up unless the height is negative.
    let top_down = h.height < 0;
    let channels = match (h.bit_count, h.compression) {
        (32, BI_RGB) | (32, BI_BITFIELDS) => 4usize,
        (24, BI_RGB) => 3usize,
        (bits, comp) => {
            return Err(format!(
                "clipboard bitmap is {bits}bpp compression {comp}, which is not supported"
            ));
        }
    };

    // Rows are padded out to a 4-byte boundary, which only matters for 24bpp
    // where a 1- or 2-pixel-wide row does not divide evenly.
    let row_bytes = width as usize * channels;
    let stride = (row_bytes + 3) & !3;
    let needed = stride
        .checked_mul(height as usize)
        .ok_or("clipboard bitmap size overflow")?;
    let pixels = raw
        .get(h.size as usize..)
        .ok_or("clipboard bitmap is shorter than its header claims")?;
    if pixels.len() < needed {
        return Err("clipboard bitmap is truncated".to_string());
    }

    let mut out = vec![0u8; width as usize * height as usize * 4];
    let mut alpha_all_zero = channels == 4;
    for row in 0..height as usize {
        let src_row = if top_down {
            row
        } else {
            height as usize - 1 - row
        };
        let src = &pixels[src_row * stride..][..row_bytes];
        for col in 0..width as usize {
            let p = &src[col * channels..][..channels];
            let alpha = if channels == 4 { p[3] } else { 255 };
            alpha_all_zero &= alpha == 0;
            // BGR(A) source, RGBA destination.
            let d = &mut out[(row * width as usize + col) * 4..][..4];
            (d[0], d[1], d[2], d[3]) = (p[2], p[1], p[0], alpha);
        }
    }
    if alpha_all_zero {
        for px in out.as_chunks_mut::<4>().0.iter_mut() {
            px[3] = 255;
        }
    }
    Ok((out, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 2x2 image whose pixels are distinguishable in every channel.
    fn sample_rgba() -> Vec<u8> {
        vec![
            10, 11, 12, 13, // top-left
            20, 21, 22, 23, // top-right
            30, 31, 32, 33, // bottom-left
            40, 41, 42, 43, // bottom-right
        ]
    }

    #[cfg(windows)]
    fn header_bytes(width: i32, height: i32, bit_count: u16, compression: u32) -> Vec<u8> {
        let h = win::BitMapInfoHeader {
            size: std::mem::size_of::<win::BitMapInfoHeader>() as u32,
            width,
            height,
            planes: 1,
            bit_count,
            compression,
            size_image: 0,
            x_pels_per_meter: 0,
            y_pels_per_meter: 0,
            clr_used: 0,
            clr_important: 0,
        };
        // SAFETY: `BitMapInfoHeader` is `repr(C)` with no padding holes the
        // compiler can reorder, and every field is `Copy`.
        unsafe {
            std::slice::from_raw_parts(
                &h as *const win::BitMapInfoHeader as *const u8,
                std::mem::size_of::<win::BitMapInfoHeader>(),
            )
            .to_vec()
        }
    }

    #[cfg(windows)]
    #[test]
    fn dib_round_trips_a_bottom_up_32bpp_image() {
        let rgba = sample_rgba();
        let (w, h) = (2u32, 2u32);

        // Build the payload the same way `set_image` does.
        let mut dib = header_bytes(w as i32, h as i32, 32, win::BI_RGB);
        let mut pixels = vec![0u8; (w * h * 4) as usize];
        win::write_bgra(pixels.as_mut_ptr(), &rgba, w as usize, h as usize);
        dib.extend_from_slice(&pixels);

        let (out, ow, oh) = parse_dib(&dib).expect("parse");
        assert_eq!((ow, oh), (w, h));
        // Nothing here has alpha 0, so alpha must survive untouched.
        assert_eq!(out, rgba);
    }

    #[cfg(windows)]
    #[test]
    fn dib_round_trips_a_top_down_32bpp_image() {
        let rgba = sample_rgba();
        // Negative height: rows arrive top-down rather than bottom-up.
        let mut dib = header_bytes(2, -2, 32, win::BI_RGB);
        let mut pixels = vec![0u8; 16];
        win::write_bgra(pixels.as_mut_ptr(), &rgba, 2, 2);
        // write_bgra emitted bottom-up; swap the two 8-byte rows to get top-down.
        pixels.swap(0, 8);
        pixels.swap(1, 9);
        pixels.swap(2, 10);
        pixels.swap(3, 11);
        pixels.swap(4, 12);
        pixels.swap(5, 13);
        pixels.swap(6, 14);
        pixels.swap(7, 15);
        dib.extend_from_slice(&pixels);

        assert_eq!(parse_dib(&dib).expect("parse").0, rgba);
    }

    #[cfg(windows)]
    #[test]
    fn zero_alpha_is_read_as_opaque() {
        // A fully transparent 32bpp DIB would paste as nothing at all.
        let rgba = vec![10, 11, 12, 0, 20, 21, 22, 0];
        let mut dib = header_bytes(2, 1, 32, win::BI_RGB);
        let mut pixels = vec![0u8; 8];
        win::write_bgra(pixels.as_mut_ptr(), &rgba, 2, 1);
        dib.extend_from_slice(&pixels);

        let (out, _, _) = parse_dib(&dib).expect("parse");
        assert_eq!(&out[3..4], &[255]);
        assert_eq!(&out[7..8], &[255]);
    }

    #[cfg(windows)]
    #[test]
    fn partial_alpha_is_preserved() {
        let rgba = vec![10, 11, 12, 0, 20, 21, 22, 128];
        let mut dib = header_bytes(2, 1, 32, win::BI_RGB);
        let mut pixels = vec![0u8; 8];
        win::write_bgra(pixels.as_mut_ptr(), &rgba, 2, 1);
        dib.extend_from_slice(&pixels);

        let (out, _, _) = parse_dib(&dib).expect("parse");
        assert_eq!(out, rgba);
    }

    #[cfg(windows)]
    #[test]
    fn dib_round_trips_24bpp() {
        let mut dib = header_bytes(2, 1, 24, win::BI_RGB);
        // 6 bytes of BGR, padded out to the 4-byte row boundary.
        dib.extend_from_slice(&[12, 11, 10, 22, 21, 20, 0, 0]);

        let (out, w, h) = parse_dib(&dib).expect("parse");
        assert_eq!((w, h), (2, 1));
        // BGR in, RGBA out, alpha synthesised.
        assert_eq!(out, vec![10, 11, 12, 255, 20, 21, 22, 255]);
    }

    #[cfg(windows)]
    #[test]
    fn unsupported_bit_depth_is_named_not_guessed() {
        let mut dib = header_bytes(2, 1, 8, win::BI_RGB);
        dib.extend_from_slice(&[0u8; 16]);
        let err = parse_dib(&dib).expect_err("8bpp must be refused");
        assert!(err.contains("8bpp"), "unhelpful error: {err}");
    }

    #[cfg(windows)]
    #[test]
    fn truncated_and_oversized_inputs_are_rejected() {
        assert!(parse_dib(&[0u8; 8]).is_err(), "shorter than a header");
        let mut dib = header_bytes(4, 4, 32, win::BI_RGB);
        dib.extend_from_slice(&[0u8; 16]); // 4*4*4 = 64 bytes expected
        assert!(parse_dib(&dib).is_err(), "truncated pixels");

        let mut zero = header_bytes(0, 0, 32, win::BI_RGB);
        zero.extend_from_slice(&[0u8; 8]);
        assert!(parse_dib(&zero).is_err(), "zero dimension");
    }

    #[test]
    fn image_support_flag_matches_the_build() {
        // `IMAGE_SUPPORTED` gates the Ctrl+C / Ctrl+V UI, so it has to agree
        // with which `set_image` implementation was actually compiled in.
        #[cfg(windows)]
        const _: () = assert!(IMAGE_SUPPORTED, "the Windows build always has CF_DIB");
        #[cfg(not(windows))]
        assert_eq!(IMAGE_SUPPORTED, cfg!(feature = "clipboard-image"));
    }
}
