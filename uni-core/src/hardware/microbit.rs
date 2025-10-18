//! micro:bit v2 hardware primitives
//!
//! Provides LED matrix (5×5) and button (A, B) access

use crate::interpreter::{Interpreter, DictEntry};
use crate::value::{RuntimeError, Value};
use crate::compat::format;

#[cfg(feature = "hardware-microbit")]
use core::cell::RefCell;
#[cfg(feature = "hardware-microbit")]
use cortex_m::interrupt::Mutex;

/// Global display for interrupt handler (micro:bit only)
/// This is accessed by both the TIMER1 interrupt handler (defined in binary) and LED primitives
///
/// Note: The interrupt handler itself must be defined in the binary crate (uni-cli), not here,
/// because Rust only allows interrupt handlers in binary crates, not libraries.
#[cfg(feature = "hardware-microbit")]
pub static DISPLAY: Mutex<RefCell<Option<microbit::display::nonblocking::Display<microbit::pac::TIMER1>>>> =
    Mutex::new(RefCell::new(None));

/// Register micro:bit-specific primitives with the interpreter
pub fn register_microbit_primitives(interp: &mut Interpreter) {
    // Register button-read primitive
    let button_read = interp.intern_atom("button-read");
    interp.dictionary.insert(button_read.clone(), DictEntry {
        value: Value::Builtin(button_read_builtin),
        is_executable: true,
        doc: Some("( button-id -- bool ) Read button state (0=A, 1=B)".into()),
    });

    // Register LED primitives
    let led_on = interp.intern_atom("led-on");
    interp.dictionary.insert(led_on.clone(), DictEntry {
        value: Value::Builtin(led_on_builtin),
        is_executable: true,
        doc: Some("( x y brightness -- ) Set LED at (x,y) to brightness (0-9)".into()),
    });

    let led_off = interp.intern_atom("led-off");
    interp.dictionary.insert(led_off.clone(), DictEntry {
        value: Value::Builtin(led_off_builtin),
        is_executable: true,
        doc: Some("( x y -- ) Turn off LED at (x,y)".into()),
    });

    let led_clear = interp.intern_atom("led-clear");
    interp.dictionary.insert(led_clear.clone(), DictEntry {
        value: Value::Builtin(led_clear_builtin),
        is_executable: true,
        doc: Some("( -- ) Clear all LEDs".into()),
    });

    let led_show = interp.intern_atom("led-show");
    interp.dictionary.insert(led_show.clone(), DictEntry {
        value: Value::Builtin(led_show_builtin),
        is_executable: true,
        doc: Some("( vector -- ) Display 25-element vector on LED matrix".into()),
    });
}

/// Read button state (0=A, 1=B)
/// Usage: button-id button-read => boolean
/// Example: 0 button-read => true  (button A pressed)
#[cfg(feature = "hardware-microbit")]
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

/// Placeholder for non-microbit builds
#[cfg(not(feature = "hardware-microbit"))]
pub fn button_read_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("button-read only available on micro:bit".into()))
}

/// Set an LED on the 5x5 matrix (x y brightness)
/// Coordinates: x=0-4 (left to right), y=0-4 (top to bottom)
/// Brightness: 0-9 (0=off, 9=brightest)
/// Usage: x y brightness led-on
/// Example: 2 2 9 led-on  (center LED at full brightness)
#[cfg(feature = "hardware-microbit")]
pub fn led_on_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    use microbit::display::nonblocking::GreyscaleImage;

    // Pop brightness (0-9)
    let brightness = interp.pop_integer()?;
    if brightness > 9 {
        return Err(RuntimeError::TypeError(format!("Brightness must be 0-9, got {}", brightness)));
    }

    // Pop y coordinate (0-4)
    let y = interp.pop_integer()?;
    if y > 4 {
        return Err(RuntimeError::TypeError(format!("Y coordinate must be 0-4, got {}", y)));
    }

    // Pop x coordinate (0-4)
    let x = interp.pop_integer()?;
    if x > 4 {
        return Err(RuntimeError::TypeError(format!("X coordinate must be 0-4, got {}", x)));
    }

    // Update the pixel buffer
    interp.display_buffer[y][x] = brightness as u8;

    // Create image from buffer and show it on global display
    let image = GreyscaleImage::new(&interp.display_buffer);
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(&image);
        }
    });

    Ok(())
}

/// Placeholder for non-microbit builds
#[cfg(not(feature = "hardware-microbit"))]
pub fn led_on_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("led-on only available on micro:bit".into()))
}

/// Turn off an LED on the 5x5 matrix (x y)
/// Usage: x y led-off
/// Example: 2 2 led-off  (turn off center LED)
#[cfg(feature = "hardware-microbit")]
pub fn led_off_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    use microbit::display::nonblocking::GreyscaleImage;

    // Pop y coordinate (0-4)
    let y = interp.pop_integer()?;
    if y > 4 {
        return Err(RuntimeError::TypeError(format!("Y coordinate must be 0-4, got {}", y)));
    }

    // Pop x coordinate (0-4)
    let x = interp.pop_integer()?;
    if x > 4 {
        return Err(RuntimeError::TypeError(format!("X coordinate must be 0-4, got {}", x)));
    }

    // Update the pixel buffer (set to 0 = off)
    interp.display_buffer[y][x] = 0;

    // Create image from buffer and show it on global display
    let image = GreyscaleImage::new(&interp.display_buffer);
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(&image);
        }
    });

    Ok(())
}

/// Placeholder for non-microbit builds
#[cfg(not(feature = "hardware-microbit"))]
pub fn led_off_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("led-off only available on micro:bit".into()))
}

/// Clear all LEDs on the 5x5 matrix
/// Usage: led-clear
/// Example: led-clear  (turn off all LEDs)
#[cfg(feature = "hardware-microbit")]
pub fn led_clear_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    use microbit::display::nonblocking::GreyscaleImage;

    // Reset the pixel buffer to all zeros
    interp.display_buffer = [[0u8; 5]; 5];

    // Create blank image and show it on global display
    let image = GreyscaleImage::blank();
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(&image);
        }
    });

    Ok(())
}

/// Placeholder for non-microbit builds
#[cfg(not(feature = "hardware-microbit"))]
pub fn led_clear_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("led-clear only available on micro:bit".into()))
}

/// Display a 5x5 image from a vector of 25 brightness values
/// Usage: vector led-show
/// Vector must contain exactly 25 integers (0-9), arranged in row-major order:
/// [row0-col0, row0-col1, ..., row0-col4, row1-col0, ..., row4-col4]
/// Example: 25 make-vector 'img val  12 9 img vector-set!  img led-show
#[cfg(feature = "hardware-microbit")]
pub fn led_show_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    use microbit::display::nonblocking::GreyscaleImage;
    use num_traits::ToPrimitive;

    // Pop the vector from stack
    let array = interp.pop()?;

    // Ensure it's an array
    let arr_data = match array {
        Value::Array(ref v) => v,
        _ => return Err(RuntimeError::TypeError(format!("led-show expects a vector, got {}", array.type_name()))),
    };

    // Ensure it has exactly 25 elements
    if arr_data.borrow().len() != 25 {
        return Err(RuntimeError::TypeError(format!("led-show expects vector of 25 elements, got {}", arr_data.borrow().len())));
    }

    // Convert vector elements to display buffer
    for (i, value) in arr_data.borrow().iter().enumerate() {
        let brightness = match value {
            Value::Int32(n) => *n as i64,
            Value::Integer(n) => n.to_i64().unwrap_or(0),
            Value::Number(f) => *f as i64,
            _ => return Err(RuntimeError::TypeError(format!("led-show expects numeric brightness values, got {}", value.type_name()))),
        };

        if brightness < 0 || brightness > 9 {
            return Err(RuntimeError::TypeError(format!("Brightness must be 0-9, got {} at position {}", brightness, i)));
        }

        let row = i / 5;
        let col = i % 5;
        interp.display_buffer[row][col] = brightness as u8;
    }

    // Create image from buffer and show it on global display
    let image = GreyscaleImage::new(&interp.display_buffer);
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.show(&image);
        }
    });

    Ok(())
}

/// Placeholder for non-microbit builds
#[cfg(not(feature = "hardware-microbit"))]
pub fn led_show_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("led-show only available on micro:bit".into()))
}
