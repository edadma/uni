//! Raspberry Pi Pico 2 (RP2350) hardware primitives
//!
//! Provides GPIO access using direct PAC register access for runtime flexibility

use crate::interpreter::{Interpreter, DictEntry};
use crate::value::{RuntimeError, Value};
use crate::compat::format;

#[cfg(feature = "hardware-pico2")]
use rp235x_hal::pac;

/// Register Pico 2 GPIO primitives with the interpreter
pub fn register_pico2_primitives(interp: &mut Interpreter) {
    // Register gpio-mode primitive (configure pin as input or output)
    let gpio_mode = interp.intern_atom("gpio-mode");
    interp.dictionary.insert(gpio_mode.clone(), DictEntry {
        value: Value::Builtin(gpio_mode_builtin),
        is_executable: true,
        doc: Some("( pin-num mode -- ) Set GPIO mode: 0=input 1=output".into()),
    });

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

    // Register sleep-ms primitive (millisecond delay)
    let sleep_ms = interp.intern_atom("sleep-ms");
    interp.dictionary.insert(sleep_ms.clone(), DictEntry {
        value: Value::Builtin(sleep_ms_builtin),
        is_executable: true,
        doc: Some("( ms -- ) Sleep for specified milliseconds".into()),
    });
}

/// Configure GPIO pin mode (input or output)
/// Usage: pin-num mode gpio-mode
/// Example: 25 1 gpio-mode  (set pin 25 as output)
///          15 0 gpio-mode  (set pin 15 as input)
#[cfg(feature = "hardware-pico2")]
pub fn gpio_mode_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Pop mode (0=input, 1=output)
    let mode = interp.pop_integer()?;

    // Pop pin number
    let pin_num = interp.pop_integer()?;

    if pin_num > 47 {
        return Err(RuntimeError::TypeError(format!("Invalid pin number: {}. Must be 0-47", pin_num)));
    }

    let pin = pin_num as usize;

    unsafe {
        // Get peripheral handles
        let io_bank0 = &*pac::IO_BANK0::ptr();
        let pads_bank0 = &*pac::PADS_BANK0::ptr();
        let sio = &*pac::SIO::ptr();

        // Configure the pad: enable input, disable output disable, disable isolation
        // RP2350 requires ISO=0 for GPIO to work (different from RP2040)
        pads_bank0.gpio(pin).modify(|_, w| w
            .od().clear_bit()      // Disable output disable
            .ie().set_bit()        // Enable input
            .iso().clear_bit()     // Disable isolation (RP2350 requirement)
        );

        // Set function to SIO (Software I/O)
        io_bank0.gpio(pin).gpio_ctrl().write(|w| w.funcsel().sio());

        // Set direction based on mode
        if mode == 1 {
            // Output mode
            sio.gpio_oe_set().write(|w| w.bits(1 << pin));
        } else {
            // Input mode
            sio.gpio_oe_clr().write(|w| w.bits(1 << pin));
        }
    }

    Ok(())
}

/// Placeholder for non-pico2 builds
#[cfg(not(feature = "hardware-pico2"))]
pub fn gpio_mode_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("gpio-mode only available on Pico 2".into()))
}

/// Set GPIO pin high or low
/// Usage: pin-num state gpio-set
/// Example: 25 true gpio-set  (turn on onboard LED)
#[cfg(feature = "hardware-pico2")]
pub fn gpio_set_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Pop state (true=high, false=low)
    let state = interp.pop()?;
    let high = match state {
        Value::Boolean(b) => b,
        _ => return Err(RuntimeError::TypeError(format!("gpio-set expects boolean state, got {:?}", state))),
    };

    // Pop pin number
    let pin_num = interp.pop_integer()?;

    if pin_num > 47 {
        return Err(RuntimeError::TypeError(format!("Invalid pin number: {}. Must be 0-47", pin_num)));
    }

    let pin = pin_num as usize;

    unsafe {
        let sio = &*pac::SIO::ptr();

        if high {
            // Set pin high
            sio.gpio_out_set().write(|w| w.bits(1 << pin));
        } else {
            // Set pin low
            sio.gpio_out_clr().write(|w| w.bits(1 << pin));
        }
    }

    Ok(())
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

    if pin_num > 47 {
        return Err(RuntimeError::TypeError(format!("Invalid pin number: {}. Must be 0-47", pin_num)));
    }

    let pin = pin_num as usize;

    let state = unsafe {
        let sio = &*pac::SIO::ptr();
        let gpio_in = sio.gpio_in().read().bits();
        (gpio_in & (1 << pin)) != 0
    };

    interp.push(Value::Boolean(state));
    Ok(())
}

/// Placeholder for non-pico2 builds
#[cfg(not(feature = "hardware-pico2"))]
pub fn gpio_get_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("gpio-get only available on Pico 2".into()))
}

/// Sleep for specified milliseconds
/// Usage: ms sleep-ms
/// Example: 500 sleep-ms  (sleep for 500ms)
#[cfg(feature = "hardware-pico2")]
pub fn sleep_ms_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    use cortex_m::asm;

    // Pop milliseconds to sleep
    let ms = interp.pop_integer()?;

    // RP2350 runs at 150MHz by default
    // Each cycle is ~6.67ns
    // For 1ms we need ~150,000 cycles
    const CYCLES_PER_MS: u32 = 150_000;

    // Calculate total cycles needed and do it in one call (no loop overhead)
    let total_cycles = CYCLES_PER_MS * (ms as u32);
    asm::delay(total_cycles);

    Ok(())
}

/// Placeholder for non-pico2 builds
#[cfg(not(feature = "hardware-pico2"))]
pub fn sleep_ms_builtin(_interp: &mut Interpreter) -> Result<(), RuntimeError> {
    Err(RuntimeError::TypeError("sleep-ms only available on Pico 2".into()))
}
