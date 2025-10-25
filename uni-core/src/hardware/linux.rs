// Platform-specific TimeSource implementation for Linux/desktop
// Uses chrono to provide date/time components from system clock

use crate::time_source::{TimeSource, DateComponents};
use chrono::{Local, Datelike, Timelike};

/// Linux/desktop time source using system clock
pub struct LinuxTimeSource;

impl LinuxTimeSource {
    pub fn new() -> Self {
        LinuxTimeSource
    }
}

impl Default for LinuxTimeSource {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeSource for LinuxTimeSource {
    fn now(&self) -> DateComponents {
        // Get current local time using chrono
        let now = Local::now();

        // Get timezone offset in seconds, convert to minutes
        let offset_seconds = now.offset().local_minus_utc();
        let offset_minutes = offset_seconds / 60;

        // Extract date/time components
        DateComponents {
            year: now.year(),
            month: now.month() as u8,
            day: now.day() as u8,
            hour: now.hour() as u8,
            minute: now.minute() as u8,
            second: now.second() as u8,
            offset_minutes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_time_source() {
        let time_source = LinuxTimeSource::new();
        let components = time_source.now();

        // Should return reasonable values
        assert!(components.year >= 2000 && components.year <= 2100);
        assert!(components.month >= 1 && components.month <= 12);
        assert!(components.day >= 1 && components.day <= 31);
        assert!(components.hour <= 23);
        assert!(components.minute <= 59);
        assert!(components.second <= 59);

        // Timezone offset should be reasonable (-12 to +14 hours = -720 to +840 minutes)
        assert!(components.offset_minutes >= -720 && components.offset_minutes <= 840);
    }
}
