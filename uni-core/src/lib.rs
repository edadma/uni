//! # Uni Core (Async)
//!
//! Async interpreter library for the Uni programming language.
//!
//! This is a fully async rewrite of Uni designed for embedded systems with
//! async runtimes like Embassy. The core evaluator uses continuation-based
//! execution which maps naturally to async/await.
//!
//! ## Features
//!
//! - **Async I/O**: All I/O primitives are truly async (non-blocking)
//! - **Continuation-based**: Tail-call optimization preserved
//! - **Homoiconic**: Code and data have identical representation
//! - **Multiple numeric types**: Int32, BigInt, Rational, Complex
//! - **Optional REPL**: Enable with `repl` feature
//!
//! ## Example
//!
//! ```ignore
//! use uni_core::{AsyncInterpreter, execute_string_async};
//!
//! #[tokio::main]  // or embassy_executor::main
//! async fn main() {
//!     let mut interp = AsyncInterpreter::new();
//!
//!     // Execute async code
//!     execute_string_async("5 3 +", &mut interp).await.unwrap();
//!
//!     // Check result
//!     assert_eq!(interp.stack.len(), 1);
//! }
//! ```

#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
extern crate alloc;

// Public modules
pub mod output;
pub mod value;
pub mod interpreter;
pub mod tokenizer;
pub mod parser;
pub mod builtins;
pub mod evaluator;
pub mod primitives;
// pub mod prelude;    // TODO: After evaluator

// Internal module
mod compat;

// Re-exports for convenience
pub use interpreter::{AsyncInterpreter, DictEntry};
pub use value::{Value, RuntimeError};
pub use output::AsyncOutput;
pub use evaluator::{execute, execute_string};
