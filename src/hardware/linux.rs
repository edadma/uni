//! Linux-specific hardware primitives
//!
//! Provides file I/O and other OS-level operations for desktop targets

use crate::interpreter::Interpreter;

/// Register Linux-specific primitives with the interpreter
///
/// Currently this is a placeholder - Linux uses the core primitives only.
/// Future additions could include:
/// - File I/O primitives (open, close, read, write)
/// - Directory operations (readdir, stat)
/// - Network operations (socket, connect, etc.)
pub fn register_linux_primitives(_interp: &mut Interpreter) {
    // Placeholder for future Linux-specific primitives
    // For now, Linux relies on the core primitives and standard library features
}
