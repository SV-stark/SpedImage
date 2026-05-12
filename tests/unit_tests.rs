#[cfg(test)]
mod tests {
    use spedimage_lib::image::ImageData;

    #[test]
    fn test_histogram_computation() {
        let rgba = vec![
            255, 0, 0, 255, // red
            0, 255, 0, 255, // green
            0, 0, 255, 255, // blue
            255, 0, 0, 255, // red again
        ];
        let mut r = [0u32; 256];
        let mut g = [0u32; 256];
        let mut b = [0u32; 256];
        ImageData::compute_rgb_histogram(&rgba, &mut r, &mut g, &mut b);

        assert_eq!(r[255], 2); // two red pixels
        assert_eq!(g[255], 1); // one green pixel
        assert_eq!(b[255], 1); // one blue pixel
    }
}
