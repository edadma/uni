# uni-core

![Version](https://img.shields.io/badge/version-0.0.10-blue)
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
uni-core = "0.0.10"
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
- `datetime` - Date/time operations (requires `std`)
- `hardware-microbit` - micro:bit v2 hardware primitives
- `hardware-pico` - Raspberry Pi Pico W hardware primitives

## Documentation

For complete documentation, examples, and the command-line REPL, see the main repository:

**GitHub:** https://github.com/edadma/uni

## License

Dual-licensed under MIT OR Unlicense.
