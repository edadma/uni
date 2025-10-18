//! Hardware abstraction layer for different targets
//!
//! Each target provides its own primitives and hardware access.
//! This module selects the appropriate platform based on Cargo feature flags.

#[cfg(feature = "target-linux")]
pub mod linux;

#[cfg(feature = "target-microbit")]
pub mod microbit;

#[cfg(feature = "target-pico")]
pub mod pico;

// Compile-time checks to ensure exactly one target is selected
#[cfg(not(any(
    feature = "target-linux",
    feature = "target-microbit",
    feature = "target-pico"
)))]
compile_error!("Must select exactly one target: target-linux, target-microbit, or target-pico");

#[cfg(all(feature = "target-linux", feature = "target-microbit"))]
compile_error!("Cannot select multiple targets: target-linux and target-microbit");

#[cfg(all(feature = "target-linux", feature = "target-pico"))]
compile_error!("Cannot select multiple targets: target-linux and target-pico");

#[cfg(all(feature = "target-microbit", feature = "target-pico"))]
compile_error!("Cannot select multiple targets: target-microbit and target-pico");
