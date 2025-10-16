//! Hardware primitives for micro:bit
//!
//! These primitives provide access to physical hardware like buttons and LEDs.
//! They are only available when building for embedded targets (micro:bit).

use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

#[cfg(target_os = "none")]
use crate::compat::format;
#[cfg(target_os = "none")]
use crate::value::Value;

/// Read button state (0=A, 1=B)
/// Usage: button-id button-read => boolean
/// Example: 0 button-read => true  (button A pressed)
#[cfg(target_os = "none")]
pub fn button_read_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    use embedded_hal::digital::InputPin;

    // Pop button ID (0=A, 1=B)
    let button_id = interp.pop_integer()?;

    // Get buttons from interpreter (need mutable access for is_low())
    let buttons = interp.buttons.as_mut()
        .ok_or_else(|| RuntimeError::TypeError("Hardware not initialized".into()))?;

    // Read button state (active-low: pressed = low/false)
    // We return true when pressed for easier use
    // Buttons on micro:bit are active-low (pressed connects to GND)
    let pressed = match button_id {
        0 => {
            // is_low() returns Result<bool, Infallible>, so we can unwrap safely
            buttons.button_a.is_low().unwrap_or(false)
        }
        1 => {
            buttons.button_b.is_low().unwrap_or(false)
        }
        _ => return Err(RuntimeError::TypeError(format!("Invalid button ID: {}. Use 0 for A, 1 for B", button_id))),
    };

    interp.push(Value::Boolean(pressed));
    Ok(())
}

/// Placeholder for non-embedded builds
#[cfg(not(target_os = "none"))]
#[allow(dead_code)]
pub fn button_read_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("button-read only available on micro:bit".into()))
}
