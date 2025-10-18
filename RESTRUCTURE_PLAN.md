# Uni Multi-Target Restructure Plan

## Overview

Restructure Uni to support multiple embedded and desktop targets while maintaining orthogonal feature selection for memory control.

## Design Principles

1. **Target selection is independent from feature selection**
   - Targets define *where* the code runs (Linux, micro:bit, RP Pico W)
   - Features define *what* capabilities are included (math, complex numbers, etc.)

2. **Features work on ALL targets**
   - `advanced_math`, `complex_numbers`, `datetime` available everywhere
   - Each target can choose which features to enable based on memory constraints

3. **Each target has board-specific primitives**
   - Linux: file I/O, potentially network operations
   - micro:bit: LED matrix (5×5), buttons (A, B)
   - RP Pico W: GPIO, WiFi, potentially displays/sensors

4. **Memory footprint is controllable per-target**
   - Minimal build: just core interpreter
   - Add features as memory allows
   - Each target decides its own feature set

## Target Features

### Target Selection (mutually exclusive)

- `target-linux` (default) - Desktop/server platform
  - Full std library
  - File I/O primitives
  - REPL over stdio
  - 128KB+ RAM, megabytes of disk

- `target-microbit` - micro:bit v2
  - ARM Cortex-M4 @ 64MHz
  - 128KB RAM, 512KB flash
  - 5×5 LED matrix display
  - 2 buttons (A, B)
  - no_std environment
  - REPL over UART (115200 baud)

- `target-pico` - Raspberry Pi Pico W
  - ARM Cortex-M0+ @ 133MHz
  - 264KB RAM, 2MB flash
  - WiFi (CYW43439)
  - 26 GPIO pins
  - no_std environment
  - REPL over UART or USB

### Orthogonal Features (combinable with any target)

- `std` - Standard library support
  - Auto-enabled with `target-linux`
  - Provides: HashMap, Vec, String, etc. from std
  - Disabled on embedded: uses alloc and BTreeMap

- `advanced_math` - Mathematical functions
  - Trig: sin, cos, tan, asin, acos, atan, atan2
  - Exponential: exp, log, log10, log2
  - Rounding: floor, ceil, round
  - Advanced division: divmod
  - Uses libm for no_std targets

- `complex_numbers` - Complex number support
  - Floating-point complex (Complex64)
  - Gaussian integers (a+bi where a,b are integers)
  - Complex arithmetic operations
  - Requires num-complex crate

- `datetime` - Date and time operations
  - Requires chrono crate
  - Larger memory footprint
  - May not fit on smallest targets

## Example Feature Combinations

```bash
# Full-featured desktop build
cargo build --features target-linux,advanced_math,complex_numbers,datetime

# Minimal micro:bit (save flash for application code)
cargo build --target thumbv7em-none-eabihf \
  --no-default-features --features target-microbit

# micro:bit with math (fits in 512KB flash)
cargo build --target thumbv7em-none-eabihf \
  --no-default-features --features target-microbit,advanced_math

# RP Pico W with all numeric features (has 2MB flash)
cargo build --target thumbv6m-none-eabi \
  --no-default-features --features target-pico,advanced_math,complex_numbers
```

## Architecture Changes

### 1. New Directory Structure

```
src/
├── hardware/              # NEW: Hardware abstraction per target
│   ├── mod.rs            # Platform selection based on features
│   ├── linux.rs          # Linux-specific primitives (file I/O)
│   ├── microbit.rs       # micro:bit primitives (LED, buttons)
│   └── pico.rs           # RP Pico W primitives (GPIO, WiFi)
├── primitives/           # Existing: Core language primitives
│   ├── hardware.rs       # DELETE: Move to hardware/
│   └── ...               # Other primitives unchanged
├── main.rs               # UPDATE: Feature-gated platform init
├── builtins.rs           # UPDATE: Target-specific registration
├── interpreter.rs        # UPDATE: Feature-gated hardware fields
└── ...                   # Other files mostly unchanged
```

### 2. Cargo.toml Restructure

```toml
[package]
name = "uni"
version = "0.0.1"
edition = "2024"

[[bin]]
name = "uni"
path = "src/main.rs"

[dependencies]
# Core dependencies (all targets)
num-bigint = { version = "0.4", default-features = false }
num-rational = { version = "0.4", default-features = false, features = ["num-bigint"] }
num-traits = { version = "0.2", default-features = false, features = ["libm"] }

# Optional feature dependencies
num-complex = { version = "0.4", default-features = false, optional = true }
chrono = { version = "0.4", optional = true }

# editline - feature-gated for different targets
editline = { version = "0.0.16", optional = true }

[features]
# Default: full-featured Linux build
default = ["target-linux", "std", "advanced_math", "complex_numbers", "datetime"]

# Target selection (mutually exclusive - pick ONE)
target-linux = ["std", "editline/std"]
target-microbit = []
target-pico = []

# Orthogonal features (work on all targets)
std = []  # Enables std library (HashMap, etc.)
advanced_math = []
complex_numbers = ["num-complex"]
datetime = ["chrono"]

# Linux-specific dependencies
[target.'cfg(all(target_os = "linux", feature = "target-linux"))'.dependencies]
editline = { version = "0.0.16", features = ["std"] }

# micro:bit-specific dependencies
[target.'cfg(feature = "target-microbit")'.dependencies]
editline = { version = "0.0.16", features = ["microbit"], default-features = false }
microbit = { package = "microbit-v2", version = "0.15" }
embedded-hal = "1.0"
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
alloc-cortex-m = "0.4"

# RP Pico W-specific dependencies (future)
[target.'cfg(feature = "target-pico")'.dependencies]
# rp-pico = "0.8"
# embassy-executor = { version = "0.5", features = ["arch-cortex-m"] }
# embassy-rp = { version = "0.1", features = ["time-driver"] }
# cyw43 = "0.1"  # WiFi driver
# ... (commented out initially)
```

### 3. Hardware Module Structure

#### src/hardware/mod.rs

```rust
//! Hardware abstraction layer for different targets
//!
//! Each target provides its own primitives and hardware access.

// Re-export the active target's hardware interface
#[cfg(feature = "target-linux")]
pub mod linux;
#[cfg(feature = "target-linux")]
pub use linux::*;

#[cfg(feature = "target-microbit")]
pub mod microbit;
#[cfg(feature = "target-microbit")]
pub use microbit::*;

#[cfg(feature = "target-pico")]
pub mod pico;
#[cfg(feature = "target-pico")]
pub use pico::*;

// Ensure exactly one target is selected at compile time
#[cfg(not(any(
    feature = "target-linux",
    feature = "target-microbit",
    feature = "target-pico"
)))]
compile_error!("Must select exactly one target: target-linux, target-microbit, or target-pico");

#[cfg(all(feature = "target-linux", feature = "target-microbit"))]
compile_error!("Cannot select multiple targets");

#[cfg(all(feature = "target-linux", feature = "target-pico"))]
compile_error!("Cannot select multiple targets");

#[cfg(all(feature = "target-microbit", feature = "target-pico"))]
compile_error!("Cannot select multiple targets");
```

#### src/hardware/linux.rs

```rust
//! Linux-specific hardware primitives
//!
//! Provides file I/O and other OS-level operations

use crate::interpreter::Interpreter;

pub fn register_linux_primitives(interp: &mut Interpreter) {
    // Future: file I/O primitives
    // - open, close, read, write
    // - readdir, stat
    // - potentially network operations

    // For now, Linux uses the core primitives only
    // File I/O can be added later as needed
}
```

#### src/hardware/microbit.rs

```rust
//! micro:bit v2 hardware primitives
//!
//! Provides LED matrix and button access

use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// Button hardware type (stored in interpreter)
#[cfg(feature = "target-microbit")]
pub type HardwareButtons = microbit::board::Buttons;

pub fn register_microbit_primitives(interp: &mut Interpreter) {
    use crate::interpreter::DictEntry;

    // Register micro:bit-specific primitives
    let led_on = interp.intern_atom("led-on");
    interp.dictionary.insert(led_on.clone(), DictEntry {
        value: Value::Builtin(led_on_builtin),
        is_executable: true,
        doc: Some("( x y brightness -- ) Set LED at position (x, y) to brightness (0-9)".into()),
    });

    let led_off = interp.intern_atom("led-off");
    interp.dictionary.insert(led_off.clone(), DictEntry {
        value: Value::Builtin(led_off_builtin),
        is_executable: true,
        doc: Some("( x y -- ) Turn off LED at position (x, y)".into()),
    });

    // ... register other micro:bit primitives
}

// Implementation of micro:bit primitives
// (Move existing code from src/primitives/hardware.rs here)

pub fn led_on_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // ... existing implementation
}

pub fn led_off_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // ... existing implementation
}

// ... other micro:bit primitive implementations
```

#### src/hardware/pico.rs

```rust
//! Raspberry Pi Pico W hardware primitives
//!
//! Provides GPIO and WiFi access (STUB - to be implemented)

use crate::interpreter::Interpreter;

pub fn register_pico_primitives(interp: &mut Interpreter) {
    // TODO: Implement Pico-specific primitives
    // - gpio-set, gpio-read (digital I/O)
    // - gpio-pwm (analog output)
    // - wifi-connect, wifi-disconnect
    // - wifi-scan, wifi-status
    // - potentially I2C, SPI primitives for sensors/displays

    // For now, just a stub
    let _ = interp;
}

// Stub implementations (to be filled in)
// pub fn gpio_set_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> { ... }
// pub fn gpio_read_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> { ... }
```

### 4. Changes to Existing Files

#### src/builtins.rs

```rust
pub fn register_builtins(interp: &mut Interpreter) {
    // Always register core primitives
    register_core_builtins(interp);

    // Register feature-gated primitives (available on all targets)
    #[cfg(feature = "advanced_math")]
    register_math_builtins(interp);

    #[cfg(feature = "complex_numbers")]
    register_complex_builtins(interp);

    #[cfg(feature = "datetime")]
    register_datetime_builtins(interp);

    // Register target-specific primitives
    #[cfg(feature = "target-linux")]
    crate::hardware::linux::register_linux_primitives(interp);

    #[cfg(feature = "target-microbit")]
    crate::hardware::microbit::register_microbit_primitives(interp);

    #[cfg(feature = "target-pico")]
    crate::hardware::pico::register_pico_primitives(interp);
}

// Split existing register_builtins into:
fn register_core_builtins(interp: &mut Interpreter) {
    // Core primitives always available:
    // - Stack operations: dup, drop, swap, etc.
    // - Arithmetic: +, -, *, /, etc.
    // - Comparison: =, <, >, etc.
    // - Control flow: if, while, etc.
    // - List operations: head, tail, cons, etc.
    // - I/O: ., print, etc.
}

#[cfg(feature = "advanced_math")]
fn register_math_builtins(interp: &mut Interpreter) {
    // Math functions:
    // - sin, cos, tan, asin, acos, atan, atan2
    // - exp, log, log10, log2
    // - floor, ceil, round
    // - abs, sqrt, pow
}

#[cfg(feature = "complex_numbers")]
fn register_complex_builtins(interp: &mut Interpreter) {
    // Complex number primitives already integrated
    // into arithmetic operations via feature guards
}

#[cfg(feature = "datetime")]
fn register_datetime_builtins(interp: &mut Interpreter) {
    // Date/time operations:
    // - now, parse-date, format-date
    // - duration operations
}
```

#### src/interpreter.rs

```rust
pub struct Interpreter {
    pub stack: Vec<Value>,
    pub return_stack: Vec<Value>,
    pub dictionary: HashMap<Rc<str>, DictEntry>,
    pub atoms: HashMap<String, Rc<str>>,
    pub current_pos: Option<SourcePos>,
    pending_doc_target: Option<Rc<str>>,
    terminal: Option<Box<dyn Terminal>>,

    // Hardware peripherals (target-specific)
    #[cfg(feature = "target-microbit")]
    pub buttons: Option<microbit::board::Buttons>,
    #[cfg(feature = "target-microbit")]
    pub display_buffer: [[u8; 5]; 5],

    // Future: Pico-specific fields
    // #[cfg(feature = "target-pico")]
    // pub gpio: Option<PicoGpio>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            stack: Vec::new(),
            return_stack: Vec::new(),
            dictionary: HashMap::new(),
            atoms: HashMap::new(),
            current_pos: None,
            pending_doc_target: None,
            terminal: None,

            #[cfg(feature = "target-microbit")]
            buttons: None,
            #[cfg(feature = "target-microbit")]
            display_buffer: [[0u8; 5]; 5],

            // #[cfg(feature = "target-pico")]
            // gpio: None,
        };

        // Load builtins and prelude
        crate::builtins::register_builtins(&mut interpreter);
        if let Err(_e) = crate::prelude::load_prelude(&mut interpreter) {
            // Error loading prelude - continue anyway
        }

        interpreter
    }

    // ... rest unchanged
}
```

#### src/main.rs

Key changes:
1. Replace `#[cfg(target_os = "none")]` with specific target features
2. Add feature guards for platform-specific initialization
3. Keep REPL code generic where possible

```rust
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(any(feature = "target-microbit", feature = "target-pico"), no_main)]

// Platform-agnostic imports
mod compat;
mod builtins;
mod evaluator;
mod integration_tests;
mod interpreter;
mod parser;
mod prelude;
mod primitives;
mod tokenizer;
mod value;
mod hardware;  // NEW: Hardware abstraction

// ... (rest of file with feature guards updated)

// Linux main
#[cfg(feature = "target-linux")]
fn main() {
    // ... existing Linux main code
}

// micro:bit main
#[cfg(feature = "target-microbit")]
#[entry]
fn mb_main() -> ! {
    // ... existing micro:bit init code
}

// Pico main (future)
#[cfg(feature = "target-pico")]
#[entry]
fn pico_main() -> ! {
    // TODO: Initialize Pico hardware
    // - Setup heap allocator
    // - Initialize UART for REPL
    // - Setup GPIO, WiFi, etc.
    // run_repl()
    loop {}
}
```

#### src/prelude.rs

Remove target-specific definitions:

```rust
pub fn load_prelude(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let prelude_code = r#"
        \\ Stack manipulation words
        'swap [1 roll] def
        // ... (existing prelude)
    "#;

    execute_string(prelude_code, interp)?;

    // REMOVE: Platform-specific prelude code
    // Hardware-specific words should be discoverable via 'words' primitive
    // No need for convenience wrappers in prelude

    Ok(())
}
```

### 5. Build Scripts

#### linux (existing - update if needed)

```bash
#!/bin/bash
cargo build --release --features target-linux,advanced_math,complex_numbers
```

#### mb (update)

```bash
#!/bin/bash
# Minimal micro:bit build
cargo +nightly run --release --target thumbv7em-none-eabihf \
  --no-default-features --features target-microbit \
  -Z build-std=core,alloc
```

#### mb-full (new - with all features that fit)

```bash
#!/bin/bash
# micro:bit with math support
cargo +nightly run --release --target thumbv7em-none-eabihf \
  --no-default-features --features target-microbit,advanced_math \
  -Z build-std=core,alloc
```

#### pico (new - stub)

```bash
#!/bin/bash
# RP Pico W build (future)
cargo +nightly build --release --target thumbv6m-none-eabi \
  --no-default-features --features target-pico,advanced_math \
  -Z build-std=core,alloc

# TODO: Flash to Pico using picotool or elf2uf2
```

## Implementation Steps

### Phase 1: Structure Setup (No Breaking Changes)

1. Create `src/hardware/` directory
2. Create `src/hardware/mod.rs` with compile-time target checks
3. Create stub files: `linux.rs`, `microbit.rs`, `pico.rs`
4. Add hardware module to `src/main.rs` imports
5. Test: Existing builds should still work

### Phase 2: Move micro:bit Code

6. Copy `src/primitives/hardware.rs` → `src/hardware/microbit.rs`
7. Update imports in `microbit.rs`
8. Add `register_microbit_primitives()` function
9. Test: micro:bit build should work

### Phase 3: Update Cargo.toml

10. Add target-* features to `[features]`
11. Add compile-time mutual exclusion checks
12. Move microbit dependencies to feature-gated section
13. Test: Build with different feature combinations

### Phase 4: Update Source Files

14. Replace `#[cfg(target_os = "none")]` with `#[cfg(feature = "target-microbit")]`
15. Update `src/builtins.rs` to call target-specific registration
16. Update `src/interpreter.rs` with feature-gated fields
17. Update `src/main.rs` platform initialization
18. Test: All three paths (Linux, micro:bit, Pico stub)

### Phase 5: Clean Up

19. Delete `src/primitives/hardware.rs` (now in `src/hardware/microbit.rs`)
20. Update `src/prelude.rs` to remove platform-specific code
21. Update build scripts with new feature flags
22. Test: Full matrix (3 targets × feature combinations)

### Phase 6: Documentation

23. Update README.md with multi-target instructions
24. Update CLAUDE.md with new architecture
25. Add examples for each target
26. Document feature combinations and memory usage

## Testing Matrix

| Target | Features | Expected Result |
|--------|----------|-----------------|
| Linux | default | Full REPL, all features |
| Linux | target-linux only | Basic REPL, no math |
| micro:bit | target-microbit | Minimal, LED/buttons work |
| micro:bit | target-microbit,advanced_math | Math works, fits in flash |
| Pico (stub) | target-pico | Compiles, doesn't run yet |

## Migration Notes

### For Existing Code

- Replace `#[cfg(target_os = "none")]` with specific target features
- Move hardware-specific code to `src/hardware/`
- Use `#[cfg(not(feature = "std"))]` for no_std adjustments
- Comment out any code that doesn't compile initially

### For Users

- Old: `bash mb` still works (update script)
- New: Can customize features per build
- Can mix and match features on any target
- Each target has its own primitives visible via `words`

## Future Expansion

### Adding New Targets

To add a new embedded target (e.g., ESP32):

1. Add `target-esp32` feature to Cargo.toml
2. Add ESP32 dependencies with feature guard
3. Create `src/hardware/esp32.rs`
4. Implement target-specific primitives
5. Add platform init code to `src/main.rs`
6. Create build script `esp32`

### Adding New Features

To add a new optional feature (e.g., JSON support):

1. Add `json` feature to Cargo.toml
2. Add dependencies: `serde = { version = "1.0", optional = true }`
3. Create `src/primitives/json.rs` with `#[cfg(feature = "json")]`
4. Register in `src/builtins.rs` with feature guard
5. Works on all targets (memory permitting)

## Benefits

1. **Clean separation**: Hardware code isolated per platform
2. **Feature orthogonality**: Math, complex work everywhere
3. **Memory control**: Choose features per target
4. **Easy expansion**: New targets and features simple to add
5. **No platform pollution**: Each target only sees its primitives
6. **Compile-time safety**: Can't select conflicting targets
7. **Backward compatible**: Existing builds keep working
