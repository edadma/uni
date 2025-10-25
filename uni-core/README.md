# uni-core

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT%20OR%20Unlicense-green)

The core async interpreter library for the Uni programming language - a fully asynchronous rewrite designed for embedded systems with async runtimes.

## Features

- **Fully Async** - All I/O primitives are non-blocking
- **Continuation-based** - Tail-call optimization preserved
- **Homoiconic** - Code and data have identical representation
- **Stack-based** - All operations work with a central computation stack
- **Multiple numeric types** - Int32, BigInt, Rational, Complex
- **no_std compatible** - Works on embedded systems

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
uni-core = "0.1.0"
```

Basic usage:

```rust
use uni_core::{AsyncInterpreter, execute_string_async};

#[tokio::main]  // or embassy_executor::main
async fn main() {
    let mut interp = AsyncInterpreter::new();

    // Execute async Uni code
    execute_string_async("5 3 +", &mut interp).await.unwrap();

    // Get result from stack
    if let Some(result) = interp.stack.last() {
        println!("Result: {}", result);  // Output: 8
    }
}
```

## Optional Features

- `std` - Standard library support (default: disabled)
- `advanced_math` - Trigonometric functions, exp/log, rounding
- `complex_numbers` - Complex number and Gaussian integer support
- `repl` - REPL (Read-Eval-Print Loop) with line editing support
- `datetime` - Date/time operations (requires `std`)
- `target-stm32h753zi` - STM32H753ZI hardware support with Embassy

## Async REPL

To build applications with custom primitives and an async REPL:

```rust
use uni_core::{AsyncInterpreter, Value, RuntimeError};
use uni_core::interpreter::DictEntry;
use editline::async_terminals::AsyncTerminal;

// Your custom async primitive
async fn my_delay(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let ms = interp.pop_integer()?;
    embassy_time::Timer::after(
        embassy_time::Duration::from_millis(ms as u64)
    ).await;
    Ok(())
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut interp = AsyncInterpreter::new();

    // Register custom primitive
    let atom = interp.intern_atom("delay");
    interp.dictionary.insert(atom, DictEntry {
        value: Value::AsyncBuiltin(my_delay),
        is_executable: true,
        doc: Some("Async delay in milliseconds".into()),
    });

    // Start async REPL with your custom primitives
    // run_async_repl(interp, terminal).await;
}
```

## Documentation

For complete documentation and examples, see the main repository:

**GitHub:** https://github.com/edadma/uni

## Comparison with Sync Uni

The original Uni (now in `uni-old/`) is synchronous. This async version provides:

- ✅ True streaming output (no buffering needed)
- ✅ Non-blocking delays that yield control
- ✅ Concurrent task execution
- ✅ Native async hardware integration

The continuation-based evaluator is identical in both versions - just the primitives are async.

## License

Dual-licensed under MIT OR Unlicense.
