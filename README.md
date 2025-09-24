# Uni Programming Language

A homoiconic stack-based programming language that unifies code and data, featuring immediate execution and powerful list-based data structures.

## Quick Start

```bash
# Clone the repository
git clone https://github.com/edadma/uni.git
cd uni

# Build and run
cargo build --release
./target/release/uni hello.uni
```

## Language Overview

**Uni** is designed around three core principles:

- **Homoiconic**: Code and data have identical representation - no special syntax that can't be represented as data
- **Stack-based**: Operations manipulate a central computation stack
- **Immediate execution**: Atoms execute when encountered unless explicitly quoted

### Data Types

Uni has just four fundamental types that compose to create everything:

| Type | Example | Description |
|------|---------|-------------|
| **Numbers** | `42`, `3.14` | 64-bit floating point |
| **Atoms** | `hello`, `+`, `print` | Interned symbols that execute when encountered |
| **Strings** | `"Hello, World!"` | Reference-counted UTF-8 text |
| **Lists** | `[1 2 3]`, `[a . b]` | Cons cells (pairs + nil) for list structures |

### Basic Syntax

```uni
\ Comments start with backslash

42              \ Numbers push themselves onto stack
hello           \ Atoms execute (look up definition)
'hello          \ Quoted atoms push without executing
[1 2 +]         \ Lists are data (quotation/code-as-data)
[1 2 +] eval    \ Execute the list: pushes 1, 2, then adds
"text"          \ Strings push themselves onto stack

\ List structures (cons cells)
[1 2 3]         \ Proper list: [1 . [2 . [3 . nil]]]
[]              \ Empty list (nil)
[a . b]         \ Improper list (just a pair)
```

## Command Line Usage

```bash
# Execute a Uni file
uni script.uni
uni -f script.uni          # Explicit file flag

# Execute code directly
uni -c "5 3 + pr"          # Execute code (prints 8)
uni -e "10 2 /"            # Execute and auto-print result (prints 5)

# Help and demo
uni --help                 # Show usage information
uni                        # Run interactive demo
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

10 fib pr   \ Calculate and print fibonacci(10)
```

Make it executable and run:
```bash
chmod +x fibonacci.uni
./fibonacci.uni
```

## Built-in Operations

### Arithmetic
```uni
5 3 +       \ Addition: 8
10 4 -      \ Subtraction: 6
7 2 *       \ Multiplication: 14
15 3 /      \ Division: 5
17 5 mod    \ Modulo: 2
```

### Comparison
```uni
5 5 =       \ Equality: 1 (true)
3 7 =       \ Equality: 0 (false)
```

### Stack Operations
```uni
42 dup      \ Duplicate top: [42 42]
1 2 swap    \ Swap top two: [2 1]
1 2 drop    \ Remove top: [1]
1 2 3 over  \ Copy second to top: [1 2 3 2]
```

### Control Flow
```uni
\ Conditional execution
5 0 > [
  "Positive" pr
] [
  "Not positive" pr
] if

\ Word definition
'square [dup *] def
5 square pr         \ Prints 25
```

### Standard Library

The standard library provides common stack manipulation words:

```uni
'swap [1 roll] def     \ Swap top two items
'dup [0 pick] def      \ Duplicate top item
'over [1 pick] def     \ Copy second item to top
'rot [2 roll] def      \ Rotate top three items
'nip [swap drop] def   \ Remove second item
'tuck [swap over] def  \ Insert copy of top below second
```

## Example Programs

### Hello World
```uni
#!/usr/bin/env uni
"Hello, World!" pr
```

### Calculator
```uni
#!/usr/bin/env uni
\ Simple calculator demo

'calculate [
  "Enter two numbers and operator (+, -, *, /, mod):"

  \ For demo, we'll use predefined values
  15 4    \ Two numbers
  '+ eval \ Operator (could be +, -, *, /, mod)

  "Result: " pr pr
] def

calculate
```

### List Processing
```uni
\ Sum all numbers in a list
'sum [
  0 swap                    \ Initialize accumulator
  [
    dup nil? [drop] [       \ If empty, we're done
      dup head swap tail    \ Get head and tail
      rot + swap sum        \ Add head to accumulator, recurse on tail
    ] if
  ] eval
] def

[1 2 3 4 5] sum pr         \ Prints 15
```

### Factorial
```uni
'factorial [
  dup 1 <= [
    drop 1                  \ Base case: factorial(0) = factorial(1) = 1
  ] [
    dup 1 - factorial *    \ Recursive case: n * factorial(n-1)
  ] if
] def

5 factorial pr             \ Prints 120
```

## Installation

### From Source

**Prerequisites:**
- [Rust](https://rustup.rs/) 1.70 or later

```bash
# Clone and build
git clone https://github.com/edadma/uni.git
cd uni
cargo build --release

# Install system-wide (optional)
sudo cp target/release/uni /usr/local/bin/
```

### Testing Installation

```bash
# Test the installation
uni -e "6 7 * pr"          # Should print 42

# Run the test suite
cargo test
```

## Architecture & Implementation

Uni is implemented in **Rust** with extensive educational comments explaining both the language design and Rust concepts. Key architectural decisions:

- **Memory Management**: Reference counting (`Rc<T>`) for automatic memory management without GC pauses
- **Atom Interning**: Identical symbols share memory for efficiency
- **Error Handling**: Comprehensive `Result<T, E>` based error propagation
- **Zero-Copy Parsing**: Efficient tokenization and parsing with minimal allocations

### Project Structure

```
src/
├── main.rs          # CLI interface and file execution
├── value.rs         # Core data types (Value enum)
├── interpreter.rs   # Stack and dictionary management
├── tokenizer.rs     # Lexical analysis
├── parser.rs        # Syntax analysis
├── evaluator.rs     # Execution engine
├── builtins.rs      # Built-in operations
└── stdlib.rs        # Standard library definitions
```

## Development

```bash
# Run tests
cargo test

# Run with debug output
cargo run -- -e "your code here"

# Format code
cargo fmt

# Run linter
cargo clippy

# Build optimized release
cargo build --release
```

### Adding New Builtins

```rust
// In src/builtins.rs
pub fn your_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let arg = interp.pop_number()?;
    // Your logic here
    interp.push(Value::Number(result));
    Ok(())
}

// Register in register_builtins()
let your_atom = interp.intern_atom("your-word");
interp.dictionary.insert(your_atom, DictEntry {
    value: Value::Builtin(your_builtin),
    is_executable: true,
});
```

## License

MIT License - see [LICENSE](LICENSE) for details.

---

*Uni: Where code meets data in perfect harmony*