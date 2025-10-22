//! # Uni Core
//!
//! Core interpreter for the Uni programming language - a homoiconic stack-based language
//! that unifies code and data through cons cells and immediate execution.
//!
//! This library provides the core language implementation without platform-specific I/O,
//! making it suitable for embedding in other applications or for use on embedded systems.
//!
//! ## Features
//!
//! - **no_std compatible**: Works on embedded systems without the Rust standard library
//! - **Homoiconic**: Code and data have identical representation
//! - **Stack-based**: All operations work with a central computation stack
//! - **Tail-call optimized**: Continuation-based evaluator enables infinite recursion
//! - **Multiple numeric types**: Integers, rationals, floats, and complex numbers
//!
//! ## Example
//!
//! ```
//! use uni_core::{Interpreter, execute_string};
//!
//! let mut interp = Interpreter::new();
//!
//! // Execute some Uni code
//! execute_string("5 3 +", &mut interp).unwrap();
//!
//! // Check the result
//! assert_eq!(interp.stack.len(), 1);
//! ```
//!
//! ## Optional Features
//!
//! - `std` - Enables standard library support (required for desktop platforms)
//! - `advanced_math` - Trigonometric functions, exp/log, rounding operations
//! - `complex_numbers` - Complex number and Gaussian integer support
//! - `repl` - Enables REPL (Read-Eval-Print Loop) functionality with line editing

#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
extern crate alloc;

// Hardware platform support
#[cfg(feature = "hardware-microbit")]
extern crate microbit;
#[cfg(any(feature = "hardware-microbit", feature = "hardware-pico2"))]
extern crate embedded_hal;
#[cfg(any(feature = "hardware-microbit", feature = "hardware-pico2"))]
extern crate cortex_m;
#[cfg(any(feature = "hardware-microbit", feature = "hardware-pico2"))]
extern crate cortex_m_rt;

// Public modules
pub mod output;
pub mod time_source;
pub mod value;
pub mod interpreter;
pub mod evaluator;
pub mod parser;
pub mod tokenizer;
pub mod builtins;
pub mod prelude;
pub mod primitives;
pub mod hardware;

// Optional REPL module (only with "repl" feature)
#[cfg(feature = "repl")]
pub mod repl;

// Internal module
mod compat;

// Re-exports for convenience
pub use interpreter::{Interpreter, DictEntry};
pub use value::{Value, RuntimeError};
pub use output::Output;
pub use time_source::{TimeSource, DateComponents};
pub use evaluator::execute_string;
