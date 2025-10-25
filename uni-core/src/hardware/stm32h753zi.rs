// STM32H753ZI hardware support - RTC time source
// This will be implemented when we add STM32 RTC support

use crate::time_source::{TimeSource, DateComponents};

/// STM32H753ZI RTC time source
///
/// This is a placeholder for STM32 RTC support.
/// TODO: Implement actual RTC reading using embassy-stm32
pub struct Stm32RtcTimeSource;

impl Stm32RtcTimeSource {
    pub fn new() -> Self {
        Stm32RtcTimeSource
    }
}

impl Default for Stm32RtcTimeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeSource for Stm32RtcTimeSource {
    fn now(&self) -> DateComponents {
        // TODO: Read from actual RTC
        // For now, return a fixed value
        DateComponents {
            year: 2025,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 0,
            offset_minutes: 0,  // UTC
        }
    }
}
