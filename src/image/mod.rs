mod loader;
mod metadata;
mod processing;
mod types;

pub use loader::ImageLoader;
pub use processing::ImageProcessor;
pub use types::{ImageData, ImageError, ImageFormatType};

pub struct ImageBackend;

impl ImageBackend {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Vec<ImageData>> {
        ImageLoader::load(path)
    }

    pub fn load_and_downsample(
        path: &std::path::Path,
        max_w: u32,
        max_h: u32,
    ) -> anyhow::Result<Vec<ImageData>> {
        ImageProcessor::load_and_downsample(path, max_w, max_h)
    }

    pub fn save(
        path: &std::path::Path,
        image: &image::DynamicImage,
        quality: u8,
    ) -> anyhow::Result<()> {
        ImageProcessor::save(path, image, quality)
    }

    pub fn is_supported(path: &std::path::Path) -> bool {
        ImageProcessor::is_supported(path)
    }

    pub fn supported_extensions() -> Vec<&'static str> {
        ImageProcessor::supported_extensions()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            ImageFormatType::from_extension("jpg"),
            ImageFormatType::Jpeg
        );
        assert_eq!(ImageFormatType::from_extension("PNG"), ImageFormatType::Png);
        assert_eq!(
            ImageFormatType::from_extension("heic"),
            ImageFormatType::Heic
        );
        #[cfg(feature = "svg")]
        assert_eq!(ImageFormatType::from_extension("svg"), ImageFormatType::Svg);
        #[cfg(feature = "raw")]
        {
            assert_eq!(ImageFormatType::from_extension("dng"), ImageFormatType::Raw);
            assert_eq!(ImageFormatType::from_extension("arw"), ImageFormatType::Raw);
            assert_eq!(ImageFormatType::from_extension("cr2"), ImageFormatType::Raw);
            assert_eq!(ImageFormatType::from_extension("nef"), ImageFormatType::Raw);
        }
        assert_eq!(
            ImageFormatType::from_extension("unknown"),
            ImageFormatType::Unknown
        );
    }

    #[test]
    fn test_load_and_downsample() {
        let temp_dir = std::env::temp_dir();
        let test_img_path = temp_dir.join("test_spedimage_downsample.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(1000, 1000));
        img.save(&test_img_path).unwrap();

        let result = ImageBackend::load_and_downsample(&test_img_path, 500, 500).unwrap();
        assert_eq!(result.len(), 1);
        let data = &result[0];

        assert_eq!(data.width, 500);
        assert_eq!(data.height, 500);
        assert_eq!(data.rgba_data.len(), 500 * 500 * 4);

        let result = ImageBackend::load_and_downsample(&test_img_path, 400, 300).unwrap();
        let data = &result[0];

        assert_eq!(data.width, 300);
        assert_eq!(data.height, 300);

        std::fs::remove_file(&test_img_path).unwrap();
    }

    #[test]
    fn test_format_is_supported() {
        assert!(ImageFormatType::Jpeg.is_supported());
        assert!(ImageFormatType::Png.is_supported());
        assert!(ImageFormatType::Gif.is_supported());
        assert!(ImageFormatType::Bmp.is_supported());
        assert!(ImageFormatType::Tiff.is_supported());
        assert!(ImageFormatType::WebP.is_supported());
        assert!(ImageFormatType::Heic.is_supported());
        assert!(ImageFormatType::Avif.is_supported());
        assert!(!ImageFormatType::Unknown.is_supported());

        #[cfg(feature = "raw")]
        assert!(ImageFormatType::Raw.is_supported());
        #[cfg(not(feature = "raw"))]
        assert!(!ImageFormatType::Raw.is_supported());

        #[cfg(feature = "svg")]
        assert!(ImageFormatType::Svg.is_supported());
        #[cfg(not(feature = "svg"))]
        assert!(!ImageFormatType::Svg.is_supported());
    }

    #[test]
    fn test_image_data_methods() {
        let data = ImageData {
            width: 800,
            height: 600,
            format: ImageFormatType::Png,
            rgba_data: vec![0u8; 800 * 600 * 4],
            path: "/test/image.png".into(),
            file_size_bytes: 1024,
            frame_delay_ms: 0,
            exif_info: None,
            histogram: None,
            exif_loaded: false,
        };

        assert_eq!(data.as_rgba().len(), 800 * 600 * 4);
        assert!((data.aspect_ratio() - 800.0 / 600.0).abs() < 0.001);
        assert_eq!(data.pixel_count(), 800 * 600);
    }

    #[test]
    fn test_save_jpeg() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_save_jpeg.jpg");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageBackend::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_save_png() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_save_png.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageBackend::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_save_bmp() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_save_bmp.bmp");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        ImageBackend::save(&test_path, &img, 90).unwrap();

        assert!(test_path.exists());
        std::fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_is_supported() {
        let temp_dir = std::env::temp_dir();
        let jpg_path = temp_dir.join("test_is_supported_1.jpg");
        let png_path = temp_dir.join("test_is_supported_2.png");
        let unknown_path = temp_dir.join("test_is_supported_3.xyz");

        std::fs::File::create(&jpg_path).unwrap();
        std::fs::File::create(&png_path).unwrap();
        std::fs::File::create(&unknown_path).unwrap();

        assert!(ImageBackend::is_supported(&jpg_path));
        assert!(ImageBackend::is_supported(&png_path));
        assert!(!ImageBackend::is_supported(&unknown_path));

        std::fs::remove_file(&jpg_path).unwrap();
        std::fs::remove_file(&png_path).unwrap();
        std::fs::remove_file(&unknown_path).unwrap();
    }

    #[test]
    fn test_supported_extensions() {
        let exts = ImageBackend::supported_extensions();
        assert!(exts.contains(&"jpg"));
        assert!(exts.contains(&"jpeg"));
        assert!(exts.contains(&"png"));
        assert!(exts.contains(&"gif"));
        assert!(exts.contains(&"bmp"));
        assert!(exts.contains(&"tiff"));
        assert!(exts.contains(&"webp"));
        assert!(exts.contains(&"heic"));
        assert!(exts.contains(&"avif"));

        #[cfg(feature = "raw")]
        {
            assert!(exts.contains(&"dng"));
            assert!(exts.contains(&"arw"));
            assert!(exts.contains(&"cr2"));
        }

        #[cfg(feature = "svg")]
        assert!(exts.contains(&"svg"));
    }

    #[test]
    fn test_load_standard_image() {
        let temp_dir = std::env::temp_dir();
        let test_img_path = temp_dir.join("test_load.png");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        img.save(&test_img_path).unwrap();

        let result = ImageBackend::load(&test_img_path).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].width, 100);
        assert_eq!(result[0].height, 100);

        std::fs::remove_file(&test_img_path).unwrap();
    }

    #[test]
    fn test_gif_loading() {
        let temp_dir = std::env::temp_dir();
        let test_gif_path = temp_dir.join("test_animated.gif");

        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(10, 10));
        img.save(&test_gif_path).unwrap();

        let result = ImageBackend::load(&test_gif_path).unwrap();
        assert!(!result.is_empty());

        std::fs::remove_file(&test_gif_path).unwrap();
    }
}
