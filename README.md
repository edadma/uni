# Uni Programming Language

![Version](https://img.shields.io/badge/version-0.0.3-blue)
![License](https://img.shields.io/badge/license-MIT%20OR%20Unlicense-green)

A homoiconic stack-based programming language that unifies code and data, featuring immediate execution, powerful list-based data structures, and precise numeric computing.

**Architecture:** Uni is split into `uni-core` (embeddable library) and `uni-cli` (command-line interface).

## Quick Start

```bash
# Clone the repository
git clone https://github.com/edadma/uni.git
cd uni

# Build and run (Linux/macOS/Windows)
./build_linux
./target/release/uni
```

## Using Uni as a Library

Add to your `Cargo.toml`:
```toml
[dependencies]
uni-core = { path = "../uni/uni-core", features = ["std"] }
```

Example usage:
```rust
use uni_core::{Interpreter, execute_string};

let mut interp = Interpreter::new();
execute_string("5 3 +", &mut interp).unwrap();
println!("Result: {}", interp.stack.last().unwrap()); // Output: 8
```

See `uni-core/examples/simple_calculator.rs` for a complete example.

## Platform Support

Uni runs on desktop platforms (Linux, macOS, Windows) and embedded systems (micro:bit v2, Raspberry Pi Pico W).

### Desktop (Linux/macOS/Windows)

**Build:**
```bash
./build_linux          # Builds release binary with all features
# Or manually:
cargo build --features target-linux --release
```

**Run REPL:**
```bash
./target/release/uni
```

**Run a file:**
```bash
./target/release/uni examples/fibonacci.uni
```

**Available features:**
- `target-linux` - Linux/desktop target support
- `std` - Standard library support (required for desktop)
- `advanced_math` - Trigonometric functions, exp/log, rounding
- `complex_numbers` - Complex number support
- `datetime` - Date/time operations (requires std)

### Embedded: micro:bit v2

**Prerequisites:**
- `arm-none-eabi-objcopy` and `arm-none-eabi-size`: `sudo apt install gcc-arm-none-eabi`
- micro:bit v2 board with USB cable

**Build and flash:**
```bash
./build_microbit       # Build release binary
./flash_microbit       # Build and flash to connected micro:bit
```

**Connect to REPL:**
```bash
picocom /dev/ttyACM0 -b 115200
```

**Specifications:**
- **Target:** ARM Cortex-M4 (thumbv7em-none-eabihf)
- **Binary size:** ~344KB (65.5% of 512KB flash)
- **Heap size:** ~112KB (out of 128KB RAM)
- Interactive REPL over USB serial (115200 baud)
- Full line editing with history (20 entries)
- All core language features with exact arithmetic

### Embedded: Raspberry Pi Pico W

**Prerequisites:**
- `elf2uf2-rs`: `cargo install elf2uf2-rs`
- `arm-none-eabi-size`: `sudo apt install gcc-arm-none-eabi`
- Raspberry Pi Pico W board with USB cable

**Build and flash:**
```bash
./build_pico           # Build release binary
./flash_pico           # Build and flash to connected Pico
```

**Connect to REPL:**
```bash
picocom /dev/ttyACM0 -b 115200
```

**Specifications:**
- **Target:** ARM Cortex-M0+ (thumbv6m-none-eabi)
- **MCU:** RP2040 dual-core @ 133MHz
- **Flash:** 2MB
- **RAM:** 264KB
- **USB:** Native USB 1.1 device support

**Features:**
- Interactive REPL over USB serial (115200 baud)
- Full line editing with history
- All core language features
- Exact arithmetic with arbitrary-precision integers and rationals
- Larger memory footprint than micro:bit (264KB RAM vs 128KB)

## Language Overview

**Uni** is designed around core principles that make it both powerful and elegant:

- **Homoiconic**: Code and data have identical representation - programs can manipulate themselves
- **Stack-based**: All operations work with a central computation stack
- **Immediate execution**: Atoms execute when encountered unless explicitly quoted
- **Tail-call optimized**: Continuation-based evaluator enables infinite recursion without stack overflow
- **Multiple numeric types**: Exact arithmetic with integers, rationals, floats, and complex numbers

### Data Types

| Type | Example | Description |
|------|---------|-------------|
| **Integer** | `42`, `-17`, `999999999999` | Arbitrary-precision integers |
| **Rational** | `1/2`, `22/7` | Exact fractions (auto-simplified) |
| **Number** | `3.14`, `2.5e10` | 64-bit floating point |
| **Complex** | `3+4i`, `-2.5+1.7i` | Complex numbers |
| **GaussianInt** | `3+4i` | Gaussian integers (complex with integer parts) |
| **Atom** | `hello`, `+`, `print` | Interned symbols that execute |
| **String** | `"Hello, World!"` | Reference-counted UTF-8 text |
| **List** | `[1 2 3]`, `[a | b]` | Cons cells (pairs + nil) |
| **Vector** | `#[1 2 3]` | Dense arrays with O(1) indexing |
| **Boolean** | `true`, `false` | Boolean values |
| **Null** | `null` | Null/nil value |

### Numeric Type System

Uni features automatic type promotion and demotion for clean numeric computing:

```uni
1 2 +           \ Integer + Integer = Integer (3)
1 2 /           \ Integer / Integer = Rational (1/2) - exact!
10 2 /          \ Exact division demotes: 5 (Integer, not 5/1)
1 2.0 +         \ Integer + Number promotes to Number (3.0)
1/2 1/4 +       \ Rational + Rational = Rational (3/4)
3+4i 1+2i +     \ Complex arithmetic supported
```

### Basic Syntax

```uni
\ Comments start with backslash

42              \ Integers push themselves
3.14            \ Numbers (floats) push themselves
1/2             \ Rationals (exact fractions)
3+4i            \ Complex or Gaussian integers
hello           \ Atoms execute (look up definition)
'hello          \ Quoted atoms push without executing
[1 2 +]         \ Lists are data (quotation/code-as-data)
#[1 2 3]        \ Vectors are dense indexed data
[1 2 +] exec    \ Execute the list: pushes 1, 2, then adds
"text"          \ Strings push themselves

\ List structures (cons cells)
[1 2 3]         \ Proper list: [1 | [2 | [3 | nil]]]
[]              \ Empty list (nil)
[a | b]         \ Improper list (just a pair)
```

## Command Line Usage

```bash
# Interactive REPL
uni                        # Start interactive session

# Execute a Uni file
uni script.uni
uni -f script.uni          # Explicit file flag

# Execute code directly
uni -c "5 3 + ."           # Execute code (prints 8)
uni -e "10 2 /"            # Execute and auto-print result (prints 5)
```

### Interactive REPL Commands

```
stack           \ Display current stack contents
clear           \ Clear the stack
words           \ List all defined words
'word help      \ Show help for a specific word (e.g., 'if help, '+ help)
quit            \ Exit the REPL (or Ctrl-D)
```

### Executable Scripts

Uni supports shebang lines for executable scripts:

```uni
#!/usr/bin/env uni
\ Fibonacci calculator
'fib [
  dup 2 < [1] [
    dup 1 - fib
    swap 2 - fib +
  ] if
] def

10 fib .    \ Calculate and print fibonacci(10)
```

Make it executable and run:
```bash
chmod +x fibonacci.uni
./fibonacci.uni
```

## Built-in Operations

### Arithmetic

Uni provides multiple division operators with different semantics:

```uni
\ Basic arithmetic
5 3 +       \ Addition: 8
10 4 -      \ Subtraction: 6
7 2 *       \ Multiplication: 14

\ Division operators
10 3 /      \ Exact division: 10/3 (Rational)
10 2 /      \ Exact division: 5 (demotes to Integer)
10 3 //     \ Floor division: 3 (rounds down)
10 3 div    \ Truncating division: 3 (rounds toward zero)
-7 2 //     \ Floor: -4
-7 2 div    \ Truncate: -3

\ Modulo
17 5 mod    \ Modulo: 2
-7 2 mod    \ Modulo: -1 (sign of dividend)
```

**Division Operator Summary:**
- `/` - Exact division (Integer/Integer → Rational for precision)
- `//` - Floor division (always rounds down, works with division theorem)
- `div` - Truncating division (rounds toward zero, like C/Java/Rust)
- `mod` - Modulo (remainder after division)

### Comparison

```uni
5 5 =       \ Equality: true
3 7 =       \ Equality: false
5 3 >       \ Greater than: true
5 3 <       \ Less than: false
5 5 >=      \ Greater or equal: true
5 3 <=      \ Less or equal: false
5 3 !=      \ Not equal: true
```

### Math Functions

```uni
\ Basic
-5 abs      \ Absolute value: 5
3 7 max     \ Maximum: 7
3 7 min     \ Minimum: 3
16 sqrt     \ Square root: 4

\ Rounding
3.7 floor   \ Floor: 3
3.2 ceil    \ Ceiling: 4
3.5 round   \ Round: 4

\ Advanced
2 8 pow     \ Power: 256
1 exp       \ Exponential: 2.718...
2.718 log   \ Natural log: 1

\ Trigonometric
0 sin       \ Sine: 0
0 cos       \ Cosine: 1
0 tan       \ Tangent: 0
```

### Bitwise Operations

```uni
12 5 bit-and    \ Bitwise AND: 4
12 5 bit-or     \ Bitwise OR: 13
12 5 bit-xor    \ Bitwise XOR: 9
5 bit-not       \ Bitwise NOT: -6
8 2 shl         \ Shift left: 32
32 2 shr        \ Shift right: 8
```

### Stack Operations

```uni
42 dup      \ Duplicate top: [42 42]
1 2 swap    \ Swap top two: [2 1]
1 2 drop    \ Remove top: [1]
1 2 over    \ Copy second to top: [1 2 1]
1 2 3 rot   \ Rotate top three: [2 3 1]
1 2 nip     \ Remove second: [2]
1 2 tuck    \ Insert copy of top below second: [2 1 2]

\ Conditional stack ops
5 ?dup      \ Dup if non-zero: [5 5]
0 ?dup      \ No dup if zero: [0]
```

### Vector Operations

```uni
\ Creating vectors
1 2 3 3 vector      \ Build #[1 2 3] (count comes last)
"x" 4 make-vector   \ Create #["x" "x" "x" "x"]
[1 2 3] list->vector\ Convert list to vector

\ Accessing vectors
#[10 20 30] 1 vector-ref    \ Get element at index 1: 20
#[10 20 30] 0 42 vector-set!\ Mutate index 0 to 42 (in-place)
#[1 2 3] length             \ Get vector length: 3

\ Converting
#[1 2 3] vector->list       \ Convert vector to list
```

### List Operations

```uni
\ Building lists
1 2 3 3 list    \ Create list [1 2 3]
1 [2 3] cons    \ Prepend: [1 2 3]

\ Accessing lists
[1 2 3] head    \ First element: 1
[1 2 3] tail    \ Rest of list: [2 3]
[1 2 3] length  \ Length: 3

\ Predicates
[] nil?         \ Check if empty: true
null null?      \ Check if null: true
5 truthy?       \ Check if truthy: true
```

### Control Flow

```uni
\ Conditional execution
5 0 > [
  "Positive" .
] [
  "Not positive" .
] if

\ While loops (from prelude)
'counter [0]
[counter @ 5 <] [
  counter @ .
  counter @ 1 + counter !
] while

\ Word definition
'square [dup *] def
5 square .          \ Prints 25
```

### Type Introspection

```uni
42 type         \ Get type: "integer"
3.14 type       \ "number"
1/2 type        \ "rational"
3+4i type       \ "gaussianint" or "complex"
'hello type     \ "atom"
[1 2] type      \ "pair"
```

### Return Stack

For advanced control flow and temporary storage:

```uni
1 2 3           \ Data stack: [1 2 3]
>r              \ Move to return stack: [1 2]
+ r>            \ Do work, bring back: [3 3]
r@              \ Peek return stack (non-destructive)
```

### Getting Help

Uni has built-in documentation for operations:

```uni
words           \ List all available words/operations

'/ help         \ Show help for division operator
'if help        \ Show help for conditional execution
'map help       \ Show help for map function (from prelude)

\ Each operation's help includes:
\ - Brief description
\ - Usage pattern
\ - Example usage
```

Example output:
```
uni> '+ help
Add two values (numbers or strings).
Usage: a b + => result
Example: 5 3 + => 8

uni> 'if help
Conditional execution.
Usage: condition true-branch false-branch if
Example: 5 0 > ["positive"] ["not positive"] if
```

## Example Programs

### Hello World
```uni
#!/usr/bin/env uni
"Hello, World!" .
```

### Calculator
```uni
#!/usr/bin/env uni
\ Simple calculator

'calculate [
  15 4         \ Two numbers
  /            \ Division operator
  "Result: " . .
] def

calculate      \ Prints: Result: 15/4
```

### Factorial (Tail-Recursive)
```uni
'factorial-helper [
  swap                      \ acc n
  dup 1 <= [
    drop                    \ Return accumulator
  ] [
    dup rot * swap 1 -      \ n, n*acc, n-1
    factorial-helper        \ Tail call
  ] if
] def

'factorial [1 swap factorial-helper] def

5 factorial .              \ Prints 120
```

### List Processing
```uni
\ Map function over a list
'map [
  swap                      \ list fn
  over nil? [
    drop drop []            \ Empty list case
  ] [
    over head               \ list fn head
    swap exec               \ list result
    swap tail               \ result tail
    rot                     \ tail result fn
    map                     \ Recursively map over tail
    cons                    \ Cons result onto mapped tail
  ] if
] def

\ Example: square all numbers
[1 2 3 4] ['dup *] map .   \ Prints [1 4 9 16]
```

### Fibonacci Sequence Generator
```uni
'fib-seq [
  \ Generate list of first n fibonacci numbers
  dup 0 <= [
    drop []                 \ Empty for n <= 0
  ] [
    dup 1 = [
      drop [1]              \ Base case: [1]
    ] [
      dup 2 = [
        drop [1 1]          \ Base case: [1 1]
      ] [
        dup 1 - fib-seq     \ Get list of n-1 fibs
        dup tail head       \ Get last element
        over head           \ Get second-to-last
        +                   \ Sum them
        swap                \ result list
        swap cons           \ Append new fib
      ] if
    ] if
  ] if
] def

10 fib-seq .               \ First 10 fibonacci numbers
```

## Installation

### From Source

**Prerequisites:**
- [Rust](https://rustup.rs/) 1.70 or later

```bash
# Clone and build for desktop
git clone https://github.com/edadma/uni.git
cd uni
cargo build --release --no-default-features --features target-linux,std,advanced_math,complex_numbers

# Install system-wide (optional)
sudo cp target/release/uni /usr/local/bin/
```

### Testing Installation

```bash
# Test the installation
uni -c "6 7 * ."           # Should print 42

# Run the test suite
cargo test --no-default-features --features target-linux,std,advanced_math,complex_numbers
```

## Architecture & Implementation

Uni is implemented in **Rust** with extensive educational comments explaining both the language design and Rust concepts.

### Key Features

- **Memory Management**: Reference counting (`Rc<T>`) for automatic memory without GC pauses
- **Atom Interning**: Identical symbols share memory for efficiency
- **Numeric Tower**: Integer → Rational → Number → Complex with automatic promotion
- **Error Handling**: Comprehensive `Result<T, E>` based error propagation
- **Tail-Call Optimization**: Continuation-based evaluator enables infinite recursion
- **Zero-Copy Design**: Efficient tokenization with minimal allocations

### Project Structure

```
src/
├── main.rs              # Entry point and platform dispatch
├── value.rs             # Core data types (Value enum)
├── interpreter.rs       # Stack and dictionary management
├── tokenizer.rs         # Lexical analysis
├── parser.rs            # Syntax analysis and numeric literals
├── evaluator.rs         # Continuation-based execution engine
├── builtins.rs          # Built-in operation registration
├── prelude.rs           # Standard library (Uni code)
├── compat.rs            # Platform compatibility layer
├── hardware/            # Platform-specific implementations
│   ├── mod.rs           # Hardware abstraction layer
│   ├── linux.rs         # Linux/desktop REPL
│   ├── microbit.rs      # BBC micro:bit v2 support
│   └── pico.rs          # Raspberry Pi Pico W support
└── primitives/          # Individual primitive implementations
    ├── plus.rs          # Addition with type promotion
    ├── divide.rs        # Exact division
    ├── floor_div.rs     # Floor division
    ├── trunc_div.rs     # Truncating division
    ├── modulo.rs        # Modulo operation
    └── ...              # 40+ other primitives

flash_microbit           # Script to flash micro:bit
flash_pico               # Script to flash Raspberry Pi Pico
```

## Development

```bash
# Run tests (desktop target)
cargo test --no-default-features --features target-linux,std,advanced_math,complex_numbers

# Run specific test module
cargo test divide::tests --no-default-features --features target-linux,std,advanced_math,complex_numbers

# Run with debug output
RUST_LOG=debug cargo run --no-default-features --features target-linux,std,advanced_math,complex_numbers -- -e "your code"

# Format code
cargo fmt

# Run linter
cargo clippy --no-default-features --features target-linux,std,advanced_math,complex_numbers

# Build optimized release for desktop
cargo build --release --no-default-features --features target-linux,std,advanced_math,complex_numbers

# Build for embedded targets
cargo +nightly build --release --target thumbv7em-none-eabihf --no-default-features --features target-microbit -Z build-std=core,alloc  # micro:bit
cargo +nightly build --release --target thumbv6m-none-eabi --no-default-features --features target-pico -Z build-std=core,alloc        # Pico
```

### Adding New Primitives

Each primitive operation is in its own file with tests:

```rust
// In src/primitives/your_op.rs
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn your_op_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let arg = interp.pop()?;
    // Your logic here
    interp.push(result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_op() {
        let mut interp = Interpreter::new();
        // Test implementation
    }
}
```

Then register in `src/builtins.rs`:

```rust
// Add to imports
use crate::primitives::your_op_builtin;

// Register in register_builtins()
let your_atom = interp.intern_atom("your-op");
interp.dictionary.insert(your_atom, DictEntry {
    value: Value::Builtin(your_op_builtin),
    is_executable: true,
    doc: Some(Rc::<str>::from("Your operation description")),
});
```

## Language Design Notes

### Why Multiple Division Operators?

Uni provides three division operators to support different use cases:

1. **`/` (Exact Division)**: Preserves mathematical precision
   - `1 2 /` → `1/2` (Rational, not 0.5)
   - Best for symbolic math and exact computation

2. **`//` (Floor Division)**: Matches mathematical floor function
   - `-7 2 //` → `-4` (always rounds down)
   - Works with `mod` for the division theorem: `a = (a // b) * b + (a mod b)`

3. **`div` (Truncating Division)**: Matches most programming languages
   - `-7 2 div` → `-3` (rounds toward zero)
   - Intuitive behavior that matches C, Java, Python's `int()`, etc.

### Numeric Type Demotion

Uni automatically demotes to simpler types when possible:
- `6/3` becomes `2` (Integer), not `2/1` (Rational)
- `3.0+0.0i` stays Complex (no demotion from Complex to Number)
- Type safety is preserved: no silent precision loss

## License

Dual licensed under your choice of:

- MIT License
- The Unlicense (public domain)

See [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! This is a learning project with extensive comments explaining Rust concepts. Perfect for:
- Learning Rust through a real project
- Exploring language implementation
- Understanding stack-based languages
- Numeric computing and type systems

---

*Uni: Where code is data, and precision is paramount*
