//! Platform-specific state and hardware access
//!
//! This module defines the Platform enum which encapsulates platform-specific
//! hardware state (RTC, SPI, I2C, GPIO, etc.) in a type-safe way.

/// Platform-specific state for STM32H753ZI
#[cfg(feature = "target-stm32h753zi")]
pub struct Stm32Platform {
    /// Real-Time Clock peripheral
    pub rtc: Option<Arc<RefCell<embassy_stm32::rtc::Rtc>>>,
    // Future: SPI, I2C, GPIO, SD card, etc.
}

/// Platform-specific state for Linux/desktop
#[cfg(feature = "std")]
pub struct LinuxPlatform {
    // Future: file handles, sockets, etc.
}

/// Platform enum - holds platform-specific hardware state
pub enum Platform {
    #[cfg(feature = "target-stm32h753zi")]
    Stm32(Stm32Platform),

    #[cfg(feature = "std")]
    Linux(LinuxPlatform),

    /// No platform-specific features
    None,
}

impl Default for Platform {
    fn default() -> Self {
        #[cfg(feature = "target-stm32h753zi")]
        return Platform::Stm32(Stm32Platform { rtc: None });

        #[cfg(all(feature = "std", not(feature = "target-stm32h753zi")))]
        return Platform::Linux(LinuxPlatform {});

        #[cfg(not(any(feature = "target-stm32h753zi", feature = "std")))]
        Platform::None
    }
}
