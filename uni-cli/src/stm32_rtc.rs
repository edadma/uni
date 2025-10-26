//! STM32 RTC TimeSource implementation
//! Depends on embassy_stm32, so implemented in uni-cli rather than uni-core

#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
use embassy_stm32::rtc::Rtc;
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
use alloc::sync::Arc;
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
use core::cell::RefCell;
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
use uni_core::time_source::{TimeSource, DateComponents};

/// STM32H753ZI RTC time source
///
/// Reads date/time from the STM32 hardware RTC
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
pub struct Stm32RtcTimeSource {
    rtc: Arc<RefCell<Rtc>>,
}

#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
impl Stm32RtcTimeSource {
    pub fn new(rtc: Arc<RefCell<Rtc>>) -> Self {
        Stm32RtcTimeSource { rtc }
    }
}

#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
impl TimeSource for Stm32RtcTimeSource {
    fn now(&self) -> DateComponents {
        let rtc = self.rtc.borrow();

        // If RTC read fails, return a default date
        let dt = match rtc.now() {
            Ok(datetime) => datetime,
            Err(_) => {
                defmt::warn!("Failed to read RTC, returning default date");
                // Return a default date
                return DateComponents {
                    year: 2025,
                    month: 1,
                    day: 1,
                    hour: 0,
                    minute: 0,
                    second: 0,
                    offset_minutes: 0,
                };
            }
        };

        DateComponents {
            year: dt.year() as i32,
            month: dt.month(),
            day: dt.day(),
            hour: dt.hour(),
            minute: dt.minute(),
            second: dt.second(),
            offset_minutes: 0,  // UTC - can be configured based on timezone
        }
    }
}
