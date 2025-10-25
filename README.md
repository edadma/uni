# Uni (Async)

![Version](https://img.shields.io/badge/version-0.1.0-blue)
![License](https://img.shields.io/badge/license-MIT%20OR%20Unlicense-green)

A fully asynchronous stack-based programming language designed for embedded systems with async runtimes like Embassy.

**Status:** 🚧 Work in Progress - Async rewrite of the Uni language

## Overview

Uni Async is a complete rewrite of the Uni programming language with async/await at its core. The original synchronous version is preserved in `uni-old/` for reference.

### Why Async?

The synchronous Uni worked well on Linux but had limitations on embedded systems:
- Output had to be buffered before sending to async terminals
- No way to implement non-blocking delays
- Couldn't integrate with async hardware (DMA, interrupts)
- No support for concurrent operations

**Uni Async solves all of these** by making I/O primitives truly async while preserving the elegant continuation-based evaluator.

## Key Features

- **Async I/O**: All I/O primitives are non-blocking
- **Continuation-based**: Preserves tail-call optimization
- **Homoiconic**: Code and data have identical representation
- **Stack-based**: Clean, simple execution model
- **Multiple numeric types**: Int32, BigInt, Rational, Complex
- **no_std compatible**: Runs on bare metal

## Project Structure

This workspace contains two crates following the library/binary pattern:

- **uni-core**: Core async interpreter library with optional REPL feature
- **uni-cli**: Thin binary wrapper that builds the executable

Users can use `uni-core` directly to build custom interpreters with their own primitives.

## Quick Start

### Linux REPL

```bash
cargo run
```

### STM32H753ZI (Embassy)

```bash
cargo build --no-default-features --features target-stm32h753zi --target thumbv7em-none-eabihf --release
```

## Language Examples

### Basic Arithmetic (works synchronously)

```forth
5 3 +        # => 8
10 dup *     # => 100
```

### Async Delays (non-blocking!)

```forth
100 delay    # Yields control for 100ms
"Done!" .    # Prints after delay completes
```

### Concurrent Tasks (future)

```forth
[slow-blink] spawn    # Run in background
[fast-blink] spawn    # Run concurrently
```

## Architecture

The continuation-based evaluator is **identical** between sync and async versions:

```rust
enum Continuation {
    Value(Value),
    List { items: Vec<Value>, index: usize },
    If { condition_result: bool, true_branch: Value, false_branch: Value },
    Exec(Value),
    Definition(Value),
    PopLocalFrame,
}
```

This is already a state machine - perfect for async! We just add `.await` at primitive execution points.

## Design Document

See [uni-async-design.md](/home/ed/RustroverProjects/uni-async-design.md) for the complete architectural design.

## Development Roadmap

### Phase 1: Core Async Evaluator (Week 1)
- [ ] Port value.rs, interpreter.rs from uni-old
- [ ] Convert evaluator to async
- [ ] Implement basic async primitives (print, cr, words)
- [ ] Basic tests

### Phase 2: Async Primitives (Week 2)
- [ ] Implement delay primitive
- [ ] Async I/O primitives (UART, GPIO)
- [ ] Async prelude definitions
- [ ] Example programs

### Phase 3: Embassy Integration (Week 3)
- [ ] STM32H753ZI full support
- [ ] Hardware peripheral integration
- [ ] Build/flash scripts
- [ ] Documentation

## Comparison: Sync vs Async

| Feature | Sync Uni (`uni-old/`) | Async Uni |
|---------|----------------------|-----------|
| I/O Model | Blocking | Non-blocking |
| Output on STM32 | Buffered | Streaming |
| Delays | Blocking entire runtime | Yields control |
| Concurrent tasks | Not supported | Planned |
| Hardware integration | Limited | Full async support |
| Performance | Fast | Fast + cooperative |

## Installation

### From Source

```bash
cargo install --path uni-cli
```

### Using the Library

```toml
[dependencies]
uni-core = "0.1.0"
```

## Documentation

- [Design Document](uni-async-design.md) - Complete architectural design
- [uni-core README](uni-core/README.md) - Library documentation
- [uni-cli README](uni-cli/README.md) - CLI usage

## Contributing

This is a personal project, but feedback and ideas are welcome!

## License

Dual-licensed under MIT OR Unlicense.

## Credits

Original Uni language by Ed A. Maxedon.
Async rewrite designed with assistance from Claude Code.
