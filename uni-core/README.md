# uni-core

![Version](https://img.shields.io/badge/version-0.0.11-blue)
![License](https://img.shields.io/badge/license-MIT%20OR%20Unlicense-green)

The core interpreter library for the Uni programming language - a homoiconic stack-based language that unifies code and data.

## Features

- **Homoiconic** - Code and data have identical representation
- **Stack-based** - All operations work with a central computation stack
- **Tail-call optimized** - Continuation-based evaluator enables infinite recursion
- **Multiple numeric types** - Integers, rationals, floats, and complex numbers
- **no_std compatible** - Works on embedded systems without the Rust standard library

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
uni-core = "0.0.11"
```

Basic usage:

```rust
use uni_core::{Interpreter, execute_string};

fn main() {
    let mut interp = Interpreter::new();

    // Execute Uni code
    execute_string("5 3 +", &mut interp).unwrap();

    // Get result from stack
    if let Some(result) = interp.stack.last() {
        println!("Result: {}", result);  // Output: 8
    }
}
```

## Example: Calculator

```rust
use uni_core::{Interpreter, execute_string};

let mut interp = Interpreter::new();

// Define a square function
execute_string("'square [dup *] def", &mut interp).unwrap();

// Use it
execute_string("5 square", &mut interp).unwrap();

println!("{}", interp.stack.last().unwrap());  // Output: 25
```

## Optional Features

- `std` - Standard library support (default: disabled)
- `advanced_math` - Trigonometric functions, exp/log, rounding
- `complex_numbers` - Complex number and Gaussian integer support
- `repl` - REPL (Read-Eval-Print Loop) with line editing support
- `datetime` - Date/time operations (requires `std`)
- `hardware-microbit` - micro:bit v2 hardware primitives
- `hardware-pico` - Raspberry Pi Pico hardware primitives

## Using the REPL

To build products with custom primitives and an interactive REPL:

```rust
use uni_core::{Interpreter, repl::run_repl, Value, RuntimeError};
use uni_core::interpreter::DictEntry;
use editline::terminals::StdioTerminal;
use std::rc::Rc;

// Your custom primitive
fn my_op(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n * 42.0));
    Ok(())
}

fn main() {
    let mut interp = Interpreter::new();

    // Register custom primitive
    let atom = interp.intern_atom("my-op");
    interp.dictionary.insert(atom, DictEntry {
        value: Value::Builtin(my_op),
        is_executable: true,
        doc: Some(Rc::from("Multiply by 42\nUsage: n my-op => result")),
    });

    // Start REPL with your custom primitives
    run_repl(interp, StdioTerminal::new());
}
```

Enable the `repl` feature in your `Cargo.toml`:

```toml
[dependencies]
uni-core = { version = "0.0.11", features = ["repl"] }
editline = "0.0.19"
```

## Documentation

For complete documentation, examples, and the command-line REPL, see the main repository:

**GitHub:** https://github.com/edadma/uni

## License

Dual-licensed under MIT OR Unlicense.
