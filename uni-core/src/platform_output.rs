//! Platform-specific output channel for STM32
//! This is shared between the main REPL and spawned tasks

#[cfg(feature = "target-stm32h753zi")]
use embassy_sync::channel::Channel;
#[cfg(feature = "target-stm32h753zi")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;

#[cfg(feature = "target-stm32h753zi")]
pub static WRITE_CHANNEL: Channel<CriticalSectionRawMutex, heapless::Vec<u8, 256>, 4> = Channel::new();
