//! Generates the deterministic benchmark corpus used by bench_viewers.ps1
//! and open_latency.rs. Run: cargo run --release --example gen_corpus

use std::path::Path;

fn pixel_value(x: u32, y: u32) -> u8 {
    // Deterministic high-entropy pattern so decoders do real work.
    ((x ^ y).wrapping_mul(2_654_435_761) >> 23) as u8
}

fn build_rgba(w: u32, h: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let v = pixel_value(x, y);
            let i = ((y * w + x) * 4) as usize;
            buf[i] = v;
            buf[i + 1] = v.wrapping_add(37);
            buf[i + 2] = v.rotate_left(3);
            buf[i + 3] = 255;
        }
    }
    buf
}

fn main() {
    let dir = Path::new("target/bench_corpus");
    std::fs::create_dir_all(dir).expect("mkdir");

    // ---- 4K JPEG / PNG / TIFF ----
    let (w, h) = (3840u32, 2160u32);
    let rgba = build_rgba(w, h);

    use zune_image::image::Image;
    let img = Image::from_u8(
        &rgba,
        w as usize,
        h as usize,
        zune_core::colorspace::ColorSpace::RGBA,
    );
    img.save(dir.join("bench.jpg")).expect("save jpg");
    img.save(dir.join("bench.png")).expect("save png");

    {
        // TIFF writer wants planar RGB.
        use tiff::encoder::colortype::RGB8;
        let mut rgb = Vec::with_capacity((w * h * 3) as usize);
        for px in rgba.as_chunks::<4>().0 {
            rgb.extend_from_slice(&px[..3]);
        }
        let file = std::fs::File::create(dir.join("bench.tiff")).expect("create tiff");
        let mut enc = tiff::encoder::TiffEncoder::new(file).expect("tiff enc");
        enc.write_image::<RGB8>(w, h, &rgb).expect("write tiff");
    }

    // ---- 1080p GIF (single frame keeps viewer open-times comparable) ----
    {
        use gif::Frame;
        let (gw, gh) = (1920u16, 1080u16);
        let rgba = build_rgba(gw as u32, gh as u32);
        let file = std::fs::File::create(dir.join("bench.gif")).expect("create gif");
        let mut encoder = gif::Encoder::new(file, gw, gh, &[]).expect("gif enc");
        encoder.set_repeat(gif::Repeat::Infinite).ok();
        let frame = Frame::from_rgba_speed(gw, gh, &mut rgba.clone(), 30);
        encoder.write_frame(&frame).expect("write gif");
    }

    for f in [
        "bench.jpg",
        "bench.png",
        "bench.tiff",
        "bench.gif",
        "bench.heic",
    ] {
        let p = dir.join(f);
        if p.exists() {
            println!("{f}: {} KB", p.metadata().unwrap().len() / 1024);
        } else {
            println!("{f}: MISSING");
        }
    }
}
