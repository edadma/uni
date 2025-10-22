//! Raspberry Pi Pico 2 (RP2350) hardware primitives
//!
//! Provides GPIO access for the Pico 2

use crate::interpreter::{Interpreter, DictEntry};
use crate::value::{RuntimeError, Value};
use crate::compat::format;

/// Register Pico 2 GPIO primitives with the interpreter
pub fn register_pico2_primitives(interp: &mut Interpreter) {
    // Register gpio-set primitive (set pin high or low)
    let gpio_set = interp.intern_atom("gpio-set");
    interp.dictionary.insert(gpio_set.clone(), DictEntry {
        value: Value::Builtin(gpio_set_builtin),
        is_executable: true,
        doc: Some("( pin-num state -- ) Set GPIO pin high (true) or low (false)".into()),
    });

    // Register gpio-get primitive (read pin state)
    let gpio_get = interp.intern_atom("gpio-get");
    interp.dictionary.insert(gpio_get.clone(), DictEntry {
        value: Value::Builtin(gpio_get_builtin),
        is_executable: true,
        doc: Some("( pin-num -- state ) Read GPIO pin state (true=high, false=low)".into()),
    });
}

/// Set GPIO pin high or low
/// Usage: pin-num state gpio-set
/// Example: 25 true gpio-set  (turn on onboard LED)
#[cfg(feature = "hardware-pico2")]
pub fn gpio_set_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Pop state (true=high, false=low)
    let state = interp.pop()?;
    let _high = match state {
        Value::Boolean(b) => b,
        _ => return Err(RuntimeError::TypeError(format!("gpio-set expects boolean state, got {:?}", state))),
    };

    // Pop pin number
    let pin_num = interp.pop_integer()?;

    // Get GPIO pins from interpreter
    let _pins = interp.gpio_pins.as_mut()
        .ok_or_else(|| RuntimeError::TypeError("GPIO not initialized".into()))?;

    // TODO: Set the appropriate pin
    // The challenge is that pins need to be individually configured as outputs
    // and the Pins struct doesn't allow dynamic access by pin number.
    // We'll need to store configured pins differently.

    Err(RuntimeError::TypeError(format!(
        "gpio-set for pin {} not yet fully implemented",
        pin_num
    )))
}

/// Placeholder for non-pico2 builds
#[cfg(not(feature = "hardware-pico2"))]
pub fn gpio_set_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("gpio-set only available on Pico 2".into()))
}

/// Read GPIO pin state
/// Usage: pin-num gpio-get => state
/// Example: 15 gpio-get  (read pin 15)
#[cfg(feature = "hardware-pico2")]
pub fn gpio_get_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Pop pin number
    let pin_num = interp.pop_integer()?;

    // TODO: Read the appropriate pin
    Err(RuntimeError::TypeError(format!(
        "gpio-get for pin {} not yet fully implemented",
        pin_num
    )))
}

/// Placeholder for non-pico2 builds
#[cfg(not(feature = "hardware-pico2"))]
pub fn gpio_get_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("gpio-get only available on Pico 2".into()))
}
