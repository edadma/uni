/// Platform-agnostic time source abstraction for the Uni interpreter.
///
/// This trait allows embedding applications to provide date/time information
/// without requiring the interpreter to depend on specific time libraries.
/// Different platforms can implement this trait using their available
/// hardware timers, RTCs, or system clocks.
///
/// The interface matches what RTC (Real-Time Clock) chips typically provide:
/// date components (year, month, day, hour, minute, second) plus timezone offset.
///
/// # Examples
///
/// ```ignore
/// use uni_core::{TimeSource, DateComponents};
///
/// struct SystemTime;
///
/// impl TimeSource for SystemTime {
///     fn now(&self) -> DateComponents {
///         // On std platforms, use chrono or std::time
///         DateComponents {
///             year: 2025,
///             month: 10,
///             day: 18,
///             hour: 14,
///             minute: 30,
///             second: 0,
///             offset_minutes: 0,  // UTC
///         }
///     }
/// }
/// ```

/// Date and time components, matching what RTC chips provide
#[derive(Debug, Clone, Copy)]
pub struct DateComponents {
    pub year: i32,
    pub month: u8,   // 1-12
    pub day: u8,     // 1-31
    pub hour: u8,    // 0-23
    pub minute: u8,  // 0-59
    pub second: u8,  // 0-59
    pub offset_minutes: i32,  // Timezone offset from UTC in minutes
}

pub trait TimeSource {
    /// Get current date and time as components.
    ///
    /// For embedded systems without a real-time clock, this may return:
    /// - Time from an external RTC chip
    /// - Time from NTP (if network available)
    /// - A fixed value (e.g., 2000-01-01 for systems without time)
    ///
    /// Timezone offset is positive east of UTC, negative west:
    /// - UTC: 0
    /// - EST (UTC-5): -300
    /// - JST (UTC+9): +540
    fn now(&self) -> DateComponents;
}
