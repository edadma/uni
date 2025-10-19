// RUST CONCEPT: Platform-specific TimeSource implementation for Linux/desktop
// Uses std::time::SystemTime to provide real time information

use crate::time_source::TimeSource;
use std::time::{SystemTime, UNIX_EPOCH};

/// Linux/desktop time source using system clock
pub struct LinuxTimeSource;

impl LinuxTimeSource {
    pub fn new() -> Self {
        LinuxTimeSource
    }
}

impl TimeSource for LinuxTimeSource {
    fn now_timestamp_millis(&self) -> i64 {
        // Get current system time
        let now = SystemTime::now();

        // Calculate milliseconds since Unix epoch
        match now.duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as i64,
            Err(_) => {
                // System time is before Unix epoch (very unlikely)
                // Return 0 as fallback
                0
            }
        }
    }

    fn now_offset_minutes(&self) -> i32 {
        // Get local timezone offset by comparing local time to UTC
        // We use libc's localtime_r for this
        #[cfg(unix)]
        {
            use std::time::SystemTime;

            let now = SystemTime::now();

            // Get Unix timestamp
            let unix_time = now.duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            // Convert to local time and UTC time to find offset
            use std::mem::MaybeUninit;
            unsafe {
                let mut local_tm = MaybeUninit::uninit();
                let time_t = unix_time as libc::time_t;

                if !libc::localtime_r(&time_t, local_tm.as_mut_ptr()).is_null() {
                    let local_tm = local_tm.assume_init();
                    // tm_gmtoff is offset from UTC in seconds (GNU extension, available on Linux/BSD)
                    #[cfg(target_os = "linux")]
                    {
                        return (local_tm.tm_gmtoff / 60) as i32;
                    }
                    #[cfg(not(target_os = "linux"))]
                    {
                        // On other Unix systems without tm_gmtoff, return 0
                        return 0;
                    }
                }
            }
        }

        // Fallback to UTC if we can't determine offset
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linux_time_source() {
        let time_source = LinuxTimeSource::new();

        // Should return a reasonable timestamp (after year 2000)
        let timestamp = time_source.now_timestamp_millis();
        assert!(timestamp > 946_684_800_000); // Jan 1, 2000
        assert!(timestamp < 4_000_000_000_000); // Sometime before year 2096
    }

    #[test]
    fn test_linux_offset() {
        let time_source = LinuxTimeSource::new();

        // Should return a valid timezone offset (-720 to +840 minutes)
        // UTC-12 (Baker Island) to UTC+14 (Kiribati)
        let offset = time_source.now_offset_minutes();
        assert!(offset >= -720 && offset <= 840);
    }
}
