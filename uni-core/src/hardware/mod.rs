//! Hardware-specific primitives for embedded targets
//!
//! Each hardware platform provides primitives for direct hardware access
//! (buttons, LEDs, GPIO, etc.). These are feature-gated and only compiled
//! when the corresponding hardware feature is enabled.
//!
//! Hardware features are optional - you can build uni-core without any
//! hardware support for use in desktop/server environments.

#[cfg(feature = "hardware-microbit")]
pub mod microbit;

#[cfg(feature = "hardware-pico")]
pub mod pico;
