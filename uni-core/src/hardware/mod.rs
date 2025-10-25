//! Hardware-specific primitives for embedded targets
//!
//! Each hardware platform provides time sources and potentially other
//! hardware access. These are feature-gated and only compiled
//! when the corresponding hardware feature is enabled.

// Linux/desktop time source (requires std)
#[cfg(feature = "std")]
pub mod linux;

// STM32H753ZI hardware support
#[cfg(feature = "target-stm32h753zi")]
pub mod stm32h753zi;
