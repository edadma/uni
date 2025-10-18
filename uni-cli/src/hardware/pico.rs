//! Raspberry Pi Pico W hardware primitives
//!
//! Provides GPIO and WiFi access (STUB - to be implemented)

use crate::interpreter::Interpreter;

/// Register RP Pico W-specific primitives with the interpreter
///
/// TODO: Implement Pico-specific primitives:
/// - gpio-set, gpio-read (digital I/O)
/// - gpio-pwm (analog output via PWM)
/// - wifi-connect, wifi-disconnect
/// - wifi-scan, wifi-status
/// - I2C, SPI primitives for sensors/displays
pub fn register_pico_primitives(_interp: &mut Interpreter) {
    // Placeholder - to be implemented when Pico support is added
    // For now, this is just a stub to allow compilation
}

// Future primitive implementations will go here
// Example:
// pub fn gpio_set_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> { ... }
// pub fn gpio_read_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> { ... }
