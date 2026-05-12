use std::time::Duration;

/// Max number of images to prefetch in either direction
pub const PREFETCH_CACHE_SIZE: u64 = 100;
/// Max memory budget for prefetch cache (bytes)
pub const PREFETCH_CACHE_BYTES: u64 = 256 * 1024 * 1024; // 256 MB

/// Default interval between images in slideshow mode
pub const DEFAULT_SLIDESHOW_INTERVAL: Duration = Duration::from_secs(3);

/// Minimum slideshow interval in seconds
pub const MIN_SLIDESHOW_INTERVAL_SECS: u64 = 1;
/// Maximum slideshow interval in seconds
pub const MAX_SLIDESHOW_INTERVAL_SECS: u64 = 120;

/// Duration for image transitions (cross-fade)
pub const TRANSITION_DURATION_MS: f32 = 150.0;

/// Interval for auto-advancing when a navigation key is held
pub const KEY_REPEAT_DELAY: Duration = Duration::from_millis(200);

/// Friction applied to momentum scrolling (0.0 to 1.0)
pub const SCROLL_FRICTION: f32 = 0.92;

/// Factor for linear interpolation in UI transitions (e.g., cropping)
pub const LERP_FACTOR: f32 = 0.2;

/// Default maximum resolution when window size is unavailable
pub const DEFAULT_MAX_WIDTH: u32 = 3840;
pub const DEFAULT_MAX_HEIGHT: u32 = 2160;

/// Max number of events to process in a single frame to prevent starvation
pub const MAX_EVENTS_PER_FRAME: usize = 100;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefetch_cache_size_reasonable() {
        assert!(PREFETCH_CACHE_SIZE > 0);
        assert!(PREFETCH_CACHE_SIZE <= 1000);
    }

    #[test]
    fn test_prefetch_cache_bytes_reasonable() {
        // Should be at least 1MB and at most 2GB
        assert!(PREFETCH_CACHE_BYTES >= 1024 * 1024);
        assert!(PREFETCH_CACHE_BYTES <= 2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_slideshow_interval_reasonable() {
        assert!(DEFAULT_SLIDESHOW_INTERVAL >= Duration::from_secs(1));
        assert!(DEFAULT_SLIDESHOW_INTERVAL <= Duration::from_secs(30));
    }

    #[test]
    fn test_slideshow_interval_bounds() {
        assert!(MIN_SLIDESHOW_INTERVAL_SECS >= 1);
        assert!(MAX_SLIDESHOW_INTERVAL_SECS >= MIN_SLIDESHOW_INTERVAL_SECS);
        assert!(MAX_SLIDESHOW_INTERVAL_SECS <= 300);
    }

    #[test]
    fn test_transition_duration_positive() {
        assert!(TRANSITION_DURATION_MS > 0.0);
        assert!(TRANSITION_DURATION_MS <= 1000.0);
    }

    #[test]
    fn test_key_repeat_delay_positive() {
        assert!(KEY_REPEAT_DELAY >= Duration::from_millis(50));
        assert!(KEY_REPEAT_DELAY <= Duration::from_millis(1000));
    }

    #[test]
    fn test_scroll_friction_bounds() {
        assert!(SCROLL_FRICTION > 0.0);
        assert!(SCROLL_FRICTION < 1.0);
    }

    #[test]
    fn test_lerp_factor_bounds() {
        assert!(LERP_FACTOR > 0.0);
        assert!(LERP_FACTOR <= 1.0);
    }

    #[test]
    fn test_default_max_resolution_reasonable() {
        assert!(DEFAULT_MAX_WIDTH > 0);
        assert!(DEFAULT_MAX_HEIGHT > 0);
        assert!(DEFAULT_MAX_WIDTH <= 7680); // 8K width
        assert!(DEFAULT_MAX_HEIGHT <= 4320); // 8K height
    }

    #[test]
    fn test_max_events_per_frame_reasonable() {
        assert!(MAX_EVENTS_PER_FRAME > 0);
        assert!(MAX_EVENTS_PER_FRAME <= 1000);
    }
}
