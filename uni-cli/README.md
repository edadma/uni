# uni-cli

![Version](https://img.shields.io/badge/version-0.0.2-blue)
![License](https://img.shields.io/badge/license-MIT%20OR%20Unlicense-green)

Command-line REPL and interpreter for the Uni programming language - a homoiconic stack-based language that unifies code and data.

## Installation

```bash
cargo install uni-cli
```

This installs the `uni` command.

## Usage

### Interactive REPL

```bash
uni
```

```
 _   _       _
| | | |_ __ (_)
| | | | '_ \| |
| |_| | | | | |
 \___/|_| |_|_| v0.0.1

Type 'quit' or press Ctrl-D to exit
uni> 5 3 +
 => 8 : int32
uni> 'square [dup *] def
uni> 7 square
 => 49 : int32
```

### Execute Code

```bash
# Execute and print result
uni -e "5 3 +"
# Output: 8

# Execute code without printing
uni -c "5 3 + ."
# Output: 8
```

### Run a File

```bash
uni script.uni
```

## Language Quick Reference

### Basic Arithmetic
```
5 3 +        # Addition: 8
10 4 -       # Subtraction: 6
7 6 *        # Multiplication: 42
20 4 /       # Division: 5
```

### Stack Operations
```
dup          # Duplicate top item
swap         # Swap top two items
drop         # Remove top item
```

### Lists
```
[1 2 3]           # Create list
[1 2 3] head      # Get first element: 1
[1 2 3] tail      # Get rest: [2 3]
[1 2 3] length    # Length: 3
```

### Functions
```
'square [dup *] def    # Define function
5 square               # Use it: 25
```

### Control Flow
```
5 0 > [true] [false] if    # Conditional
```

## Embedded Targets

The CLI can also be built for embedded systems:

- **micro:bit v2** - Interactive REPL over USB serial
- **Raspberry Pi Pico W** - Full interpreter on ARM Cortex-M0+

For embedded builds and more documentation, see the main repository:

**GitHub:** https://github.com/edadma/uni

## Library Usage

To embed Uni in your own application, use the `uni-core` library crate:

```toml
[dependencies]
uni-core = "0.0.1"
```

See the `uni-core` crate documentation for API details.

## License

Dual-licensed under MIT OR Unlicense.
