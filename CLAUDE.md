# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Uni is a homoiconic stack-based programming language interpreter written in Rust. It combines Forth's immediate execution model with Lisp's cons cells. The project is designed as a Rust learning exercise with extensive educational comments.

## Development Commands

### Building and Running
```bash
# Install Rust toolchain if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build

# Run the interpreter
cargo run

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_atom_interning
```

### Development Tools
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check compilation without building
cargo check
```

## Architecture

### Core Components

1. **Value Enum** (`src/main.rs:8-30`): Central data type representing all Uni values
   - `Number(f64)`: Stack-allocated floating point numbers
   - `Atom(Rc<str>)`: Interned reference-counted strings
   - `Pair(Rc<Value>, Rc<Value>)`: Cons cells for list structures
   - `Nil`: Empty list marker
   - `Builtin`: Function pointers for built-in operations

2. **Interpreter Struct** (`src/main.rs:42-55`): Main execution context
   - `stack: Vec<Value>`: Computation stack
   - `dictionary: HashMap<Rc<str>, Value>`: Word definitions (Forth dictionary)
   - `atoms: HashMap<String, Rc<str>>`: Atom interning table for memory efficiency

3. **Error Handling** (`src/main.rs:34-39`): Type-safe runtime errors
   - Uses `Result<T, RuntimeError>` pattern throughout
   - No exceptions, explicit error propagation with `?` operator

### Memory Management Strategy

- **Reference Counting**: Uses `Rc<T>` for shared ownership without garbage collection
- **Atom Interning**: Reuses identical atoms to save memory (see `intern_atom` at `src/main.rs:70-88`)
- **Structural Sharing**: Lists share common tails through `Rc` pointers

## Implementation Status

### Completed
- Value enum with all data types
- Basic interpreter structure
- Atom interning system
- Stack operations (push, pop, pop_number)
- List construction helpers
- Example builtin function (+)
- Comprehensive test suite foundation

### Next Priority Tasks
1. **Tokenizer**: Parse text into tokens (numbers, atoms, brackets)
2. **Parser**: Convert tokens to Value structures
3. **Core Builtins**: Stack operations (dup, swap, drop), arithmetic, list operations
4. **Evaluation Engine**: Execute atoms and lists
5. **REPL**: Interactive read-eval-print loop

## Language Design

- **Homoiconic**: Code and data have the same representation
- **Stack-based**: Operations manipulate a central stack
- **Immediate execution**: Atoms execute when encountered unless quoted
- **Cons cells**: Lists are pairs like in Lisp/Scheme

Example Uni code (future syntax):
```uni
5 3 +           # Push 5, push 3, add -> 8
'square [dup *] def     # Define word
[1 2 3] head    # List operations
```

## Testing Approach

Tests are in the same file under `#[cfg(test)]` module (`src/main.rs:215-273`). Each major component has focused unit tests. Run all tests before committing changes.

## Rust Learning Focus

This codebase includes extensive educational comments explaining:
- Ownership and borrowing patterns
- Error handling with Result
- Smart pointers (Rc)
- Pattern matching
- Iterator methods
- Trait derivation

Pay attention to comments explaining Rust concepts, especially around the borrow checker (see `src/main.rs:204-206` for a common pattern).