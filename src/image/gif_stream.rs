//! Streaming GIF decoder: composites frames on a background thread and feeds
//! them to the renderer through a bounded channel so memory stays bounded by
//! a small look-ahead window instead of the whole animation.

use color_eyre::eyre::{Result, eyre};
use crossbeam_channel::Sender;
use gif::DecodeOptions;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// A single decoded, composited (and optionally downsampled) GIF frame.
pub struct GifFrameMsg {
    pub index: usize,
    pub delay_ms: u32,
    pub width: u32,
    pub height: u32,
    pub data: Arc<Vec<u8>>,
}

/// Messages sent from the streamer thread to the UI.
pub enum GifStreamMsg {
    Frame(GifFrameMsg),
    /// The streamer wrapped around and is starting a new playback pass.
    Restarted,
}

/// Decode `path` from `start_index` (inclusive), looping forever until the
/// generation counter moves on or the channel receiver is dropped.
///
/// `max_w`/`max_h` bound the composited canvas size; pass 0 to disable
/// downsampling. Blocks on `tx.send` when the look-ahead window is full.
pub fn stream_gif(
    path: std::path::PathBuf,
    start_index: usize,
    max_w: u32,
    max_h: u32,
    generation: u64,
    gen_flag: Arc<AtomicU64>,
    tx: Sender<GifStreamMsg>,
) {
    let mut pass = 0usize;
    while gen_flag.load(Ordering::Relaxed) == generation {
        if let Err(e) = stream_one_pass(&path, start_index, pass, max_w, max_h, &tx) {
            if !matches!(e.kind, SendErrorKind::Disconnected) {
                tracing::warn!("GIF stream stopped: {e:?}");
                return;
            }
            return; // receiver dropped: app moved on or shut down
        }
        if gen_flag.load(Ordering::Relaxed) != generation {
            return;
        }
        pass += 1;
        // Loop restart marker; ignored by consumers that exited.
        if tx.send(GifStreamMsg::Restarted).is_err() {
            return;
        }
    }
}

#[derive(Debug)]
enum SendErrorKind {
    Disconnected,
    Other,
}

#[derive(Debug)]
struct StreamError {
    kind: SendErrorKind,
}

impl From<crossbeam_channel::SendError<GifStreamMsg>> for StreamError {
    fn from(_: crossbeam_channel::SendError<GifStreamMsg>) -> Self {
        Self {
            kind: SendErrorKind::Disconnected,
        }
    }
}

fn stream_one_pass(
    path: &Path,
    start_index: usize,
    pass: usize,
    max_w: u32,
    max_h: u32,
    tx: &Sender<GifStreamMsg>,
) -> Result<(), StreamError> {
    let inner = (|| -> Result<()> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        let cursor = std::io::Cursor::new(&mmap[..]);

        let mut options = DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let mut decoder = options
            .read_info(cursor)
            .map_err(|e| eyre!("Failed to read GIF info: {e:?}"))?;

        let w = decoder.width() as u32;
        let h = decoder.height() as u32;

        let (dst_w, dst_h) = if max_w > 0 && max_h > 0 && (w > max_w || h > max_h) {
            let ratio = (w as f32 / max_w as f32).max(h as f32 / max_h as f32);
            (
                ((w as f32 / ratio).round() as u32).max(1),
                ((h as f32 / ratio).round() as u32).max(1),
            )
        } else {
            (w, h)
        };
        let is_downsampled = dst_w != w || dst_h != h;

        let mut canvas = vec![0u8; (w * h * 4) as usize];
        let mut prev_canvas = canvas.clone();
        let mut prev_disposal = gif::DisposalMethod::Keep;
        let mut prev_frame_rect = (0usize, 0usize, 0usize, 0usize);
        let mut resizer = if is_downsampled {
            Some(crate::image::ImageProcessor::create_simd_resizer())
        } else {
            None
        };

        let mut index = 0usize;
        while let Ok(Some(frame)) = decoder.read_next_frame() {
            match prev_disposal {
                gif::DisposalMethod::Background => {
                    let (fl, ft, fw, fh) = prev_frame_rect;
                    for row in 0..fh {
                        let y = ft + row;
                        if y < h as usize {
                            let start = (y * w as usize + fl) * 4;
                            let end = start + fw * 4;
                            if end <= canvas.len() {
                                canvas[start..end].fill(0);
                            }
                        }
                    }
                }
                gif::DisposalMethod::Previous => {
                    canvas.copy_from_slice(&prev_canvas);
                }
                _ => {}
            }

            if frame.dispose == gif::DisposalMethod::Previous {
                prev_canvas.copy_from_slice(&canvas);
            }

            let delay_ms = (frame.delay as u32 * 10).max(10);
            let fl = frame.left as usize;
            let ft = frame.top as usize;
            let fw = frame.width as usize;
            let fh = frame.height as usize;

            let line_len = fw * 4;
            for (i, line) in frame.buffer.chunks_exact(line_len).enumerate() {
                let y = ft + i;
                if y < h as usize {
                    let canvas_start = (y * w as usize + fl) * 4;
                    for (p, pixel) in line.as_chunks::<4>().0.iter().enumerate() {
                        let dst_idx = canvas_start + p * 4;
                        if dst_idx + 4 <= canvas.len() && pixel[3] > 0 {
                            canvas[dst_idx..dst_idx + 4].copy_from_slice(pixel);
                        }
                    }
                }
            }

            prev_disposal = frame.dispose;
            prev_frame_rect = (fl, ft, fw, fh);

            if index >= start_index || pass > 0 {
                let data: Vec<u8> = if is_downsampled {
                    use fast_image_resize as fr;
                    let src_image =
                        fr::images::ImageRef::new(w, h, &canvas, fr::PixelType::U8x4)
                            .map_err(|e| eyre!("Failed to create src image for resize: {e:?}"))?;
                    let mut dst_image = fr::images::Image::new(dst_w, dst_h, fr::PixelType::U8x4);
                    if let Some(ref mut r) = resizer {
                        r.resize(&src_image, &mut dst_image, None)
                            .map_err(|e| eyre!("Resize failed: {e:?}"))?;
                    }
                    dst_image.into_vec()
                } else {
                    canvas.clone()
                };

                tx.send(GifStreamMsg::Frame(GifFrameMsg {
                    index,
                    delay_ms,
                    width: dst_w,
                    height: dst_h,
                    data: Arc::new(data),
                }))?;
            }

            index += 1;
        }

        Ok(())
    })();

    match inner {
        Ok(()) => Ok(()),
        Err(_) => Err(StreamError {
            kind: SendErrorKind::Other,
        }),
    }
}
