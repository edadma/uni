# Uni CLI

Command-line async REPL and interpreter for the Uni programming language.

## Features

- **Async I/O**: Fully async interpreter with non-blocking I/O
- **Line Editing**: Full editline support with arrow keys, history, and editing
- **Interactive REPL**: Read-Eval-Print Loop with command history
- **Cross-platform**: Works on Linux, macOS, and Windows (std), plus STM32H753ZI (embedded)

## Usage

### Interactive REPL

Run the REPL interactively in your terminal:

```bash
cargo run --release
```

Or use the compiled binary:

```bash
./target/release/uni
```

### Features

The REPL supports full line editing with:
- **Arrow keys**: Navigate left/right in the current line
- **Up/Down**: Navigate through command history (50 entries)
- **Ctrl-C**: Clear current line (continue REPL)
- **Ctrl-D**: Exit REPL
- **Home/End**: Jump to start/end of line
- **Ctrl-Left/Right**: Word-wise navigation

### Basic Examples

```
uni> 5 3 +
Stack: 8

uni> . cr
8

uni> 'double [dup +] def
uni> 10 double
Stack: 20

uni> words
Defined words (23):
+        -        .        <        <=
=        >        >=       car      cdr
cons     cr       def      drop     dup
exec     if       over     quit     rot
swap     val      words

uni> quit
Goodbye!
```

## Building

### Linux/macOS (default)

```bash
cargo build --release
```

### STM32H753ZI

```bash
cargo build --release --no-default-features --features target-stm32h753zi
```

## Dependencies

- `uni-core`: Async interpreter library
- `editline`: Line editing with history
- `tokio`: Async runtime (for std builds)

For embedded builds, uses Embassy instead of tokio.
