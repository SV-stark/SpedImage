use std::time::Duration;

/// Max number of images to prefetch in either direction
pub const PREFETCH_CACHE_SIZE: u64 = 100;

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
