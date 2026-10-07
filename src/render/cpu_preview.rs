//! A CPU-painted first frame for the seconds before the GPU is ready.
//!
//! Creating the DX12 adapter and device takes 0.6-2.5 s on integrated GPUs,
//! while the first image is decoded in ~150 ms. Waiting for the GPU meant
//! staring at an empty window for most of a second or more. Until the
//! renderer arrives, the decoded preview is blitted with GDI `StretchDIBits`
//! instead; the swapchain simply replaces it on its first present.

use crate::image::{ImageData, ImageProcessor};

/// The letterbox colour of the GPU path at fit-to-window zoom: the render
/// pass clears to black and the image quad covers only the image. Matching it
/// avoids a visible jump in the border colour when the GPU takes over.
pub const BACKGROUND: [u8; 3] = [0, 0, 0];

/// Aspect-fit `img` into `win`, centred: `(x, y, w, h)`.
pub fn fit_rect(img: (u32, u32), win: (u32, u32)) -> (i32, i32, i32, i32) {
    let (iw, ih) = (img.0.max(1) as f64, img.1.max(1) as f64);
    let (ww, wh) = (win.0 as f64, win.1 as f64);
    let scale = (ww / iw).min(wh / ih);
    let (w, h) = ((iw * scale).round().max(1.0), (ih * scale).round().max(1.0));
    (
        ((ww - w) / 2.0).round() as i32,
        ((wh - h) / 2.0).round() as i32,
        w as i32,
        h as i32,
    )
}

/// Straight-alpha RGBA -> opaque BGRA composited onto `bg`, the layout a
/// 32-bit `BI_RGB` DIB expects.
pub fn to_bgra_over(rgba: &[u8], bg: [u8; 3]) -> Vec<u8> {
    let mut out = Vec::with_capacity(rgba.len());
    for p in rgba.as_chunks::<4>().0 {
        let a = p[3] as u32;
        let mix = |c: u8, b: u8| ((c as u32 * a + b as u32 * (255 - a) + 127) / 255) as u8;
        out.extend_from_slice(&[mix(p[2], bg[2]), mix(p[1], bg[1]), mix(p[0], bg[0]), 255]);
    }
    out
}

/// Pixels as they should appear: deferred EXIF rotation applied.
fn upright(img: &ImageData) -> (std::borrow::Cow<'_, [u8]>, u32, u32) {
    if img.orientation_deg == 0 {
        return (img.rgba_data.as_slice().into(), img.width, img.height);
    }
    let (data, w, h) = ImageProcessor::rotate_rgba(
        &img.rgba_data,
        img.width,
        img.height,
        img.orientation_deg as i32,
    );
    (data.into(), w, h)
}

/// Paint `img` into `window` with GDI. Returns true when something was drawn.
#[cfg(windows)]
pub fn paint(window: &winit::window::Window, img: &ImageData) -> bool {
    use windows::Win32::Foundation::{COLORREF, HWND, RECT};
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateSolidBrush, DIB_RGB_COLORS, DeleteObject,
        FillRect, GetDC, HALFTONE, HGDIOBJ, ReleaseDC, SRCCOPY, SetStretchBltMode, StretchDIBits,
    };
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    let Ok(handle) = window.window_handle() else {
        return false;
    };
    let RawWindowHandle::Win32(h) = handle.as_raw() else {
        return false;
    };
    let hwnd = HWND(h.hwnd.get() as *mut _);

    let (pixels, w, h) = upright(img);
    if w == 0 || h == 0 || pixels.len() < (w as usize) * (h as usize) * 4 {
        return false;
    }
    let bg = BACKGROUND;
    let bgra = to_bgra_over(&pixels, bg);

    unsafe {
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_err() {
            return false;
        }
        let (cw, ch) = (
            (client.right - client.left).max(0) as u32,
            (client.bottom - client.top).max(0) as u32,
        );
        if cw == 0 || ch == 0 {
            return false;
        }
        let hdc = GetDC(Some(hwnd));
        if hdc.is_invalid() {
            return false;
        }

        // COLORREF is 0x00BBGGRR.
        let brush = CreateSolidBrush(COLORREF(
            bg[0] as u32 | (bg[1] as u32) << 8 | (bg[2] as u32) << 16,
        ));
        FillRect(hdc, &client, brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));

        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w as i32,
                // Negative height = top-down rows, matching the buffer.
                biHeight: -(h as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let (x, y, dw, dh) = fit_rect((w, h), (cw, ch));
        SetStretchBltMode(hdc, HALFTONE);
        let lines = StretchDIBits(
            hdc,
            x,
            y,
            dw,
            dh,
            0,
            0,
            w as i32,
            h as i32,
            Some(bgra.as_ptr().cast()),
            &info,
            DIB_RGB_COLORS,
            SRCCOPY,
        );
        ReleaseDC(Some(hwnd), hdc);
        lines > 0
    }
}

/// Other platforms wait for the GPU as before.
#[cfg(not(windows))]
pub fn paint(_window: &winit::window::Window, _img: &ImageData) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_rect_letterboxes_and_pillarboxes() {
        // Wide image in a tall window: full width, centred vertically.
        assert_eq!(fit_rect((200, 100), (100, 100)), (0, 25, 100, 50));
        // Tall image in a wide window: full height, centred horizontally.
        assert_eq!(fit_rect((100, 200), (200, 100)), (75, 0, 50, 100));
        // Small images scale up to fit, like the GPU path.
        assert_eq!(fit_rect((10, 10), (100, 50)), (25, 0, 50, 50));
    }

    #[test]
    fn bgra_swizzles_and_composites_alpha() {
        let bg = [10, 20, 30];
        let out = to_bgra_over(&[1, 2, 3, 255, 200, 200, 200, 0], bg);
        assert_eq!(&out[..4], &[3, 2, 1, 255], "opaque pixel swizzled");
        assert_eq!(&out[4..], &[30, 20, 10, 255], "transparent pixel shows bg");
    }
}
