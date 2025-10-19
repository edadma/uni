/// Platform-agnostic time source abstraction for the Uni interpreter.
///
/// This trait allows embedding applications to provide time information
/// without requiring the interpreter to depend on specific time libraries.
/// Different platforms can implement this trait using their available
/// hardware timers, RTCs, or system clocks.
///
/// # Examples
///
/// ```
/// use uni_core::TimeSource;
///
/// struct SystemTime;
///
/// impl TimeSource for SystemTime {
///     fn now_timestamp_millis(&self) -> i64 {
///         // On std platforms, use std::time
///         std::time::SystemTime::now()
///             .duration_since(std::time::UNIX_EPOCH)
///             .unwrap()
///             .as_millis() as i64
///     }
///
///     fn now_offset_minutes(&self) -> i32 {
///         // Simple implementation: UTC offset
///         0  // UTC
///     }
/// }
/// ```
pub trait TimeSource {
    /// Get current time as milliseconds since Unix epoch (1970-01-01 00:00:00 UTC).
    ///
    /// For embedded systems without a real-time clock, this may return:
    /// - Milliseconds since power-on
    /// - Time from an external RTC chip
    /// - Time from NTP (if network available)
    /// - A fixed value (for systems without time)
    fn now_timestamp_millis(&self) -> i64;

    /// Get current timezone offset in minutes from UTC.
    ///
    /// Positive values are east of UTC, negative values are west.
    /// Examples:
    /// - UTC: 0
    /// - EST (UTC-5): -300
    /// - JST (UTC+9): +540
    ///
    /// For embedded systems, this typically returns 0 (UTC).
    fn now_offset_minutes(&self) -> i32;
}
