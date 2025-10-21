// RUST CONCEPT: REPL (Read-Eval-Print Loop) module
// This module provides a generic REPL that works with any editline Terminal implementation

use crate::compat::{Rc, Box, format};
use crate::interpreter::Interpreter;
use crate::evaluator::execute_string;
use crate::value::RuntimeError;
use crate::output::Output;
use editline::{LineEditor, Terminal};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// Platform-specific line endings
#[cfg(not(target_os = "none"))]
const LINE_ENDING: &[u8] = b"\n";
#[cfg(target_os = "none")]
const LINE_ENDING: &[u8] = b"\r\n";

// Wrapper for shared Output access (interpreter needs it)
struct RefCellOutput<T: Terminal> {
    inner: Rc<RefCell<TerminalOutput<T>>>,
}

impl<T: Terminal> Output for RefCellOutput<T> {
    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        <TerminalOutput<T> as Output>::write(&mut *self.inner.borrow_mut(), data)
    }

    fn flush(&mut self) -> Result<(), ()> {
        <TerminalOutput<T> as Output>::flush(&mut *self.inner.borrow_mut())
    }
}

// Wrapper for shared Terminal access (REPL needs it)
struct RefCellTerminal<T: Terminal> {
    inner: Rc<RefCell<TerminalOutput<T>>>,
}

impl<T: Terminal> Terminal for RefCellTerminal<T> {
    fn read_byte(&mut self) -> editline::Result<u8> {
        self.inner.borrow_mut().read_byte()
    }

    fn write(&mut self, data: &[u8]) -> editline::Result<()> {
        Terminal::write(&mut *self.inner.borrow_mut(), data)
    }

    fn flush(&mut self) -> editline::Result<()> {
        Terminal::flush(&mut *self.inner.borrow_mut())
    }

    fn enter_raw_mode(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().enter_raw_mode()
    }

    fn exit_raw_mode(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().exit_raw_mode()
    }

    fn cursor_left(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().cursor_left()
    }

    fn cursor_right(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().cursor_right()
    }

    fn clear_eol(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().clear_eol()
    }

    fn parse_key_event(&mut self) -> editline::Result<editline::KeyEvent> {
        self.inner.borrow_mut().parse_key_event()
    }
}

// Terminal output wrapper that implements both Terminal and Output
pub struct TerminalOutput<T: Terminal> {
    terminal: T,
}

impl<T: Terminal> TerminalOutput<T> {
    pub fn new(terminal: T) -> Self {
        Self { terminal }
    }
}

impl<T: Terminal> Terminal for TerminalOutput<T> {
    fn read_byte(&mut self) -> editline::Result<u8> {
        self.terminal.read_byte()
    }

    fn write(&mut self, data: &[u8]) -> editline::Result<()> {
        self.terminal.write(data)
    }

    fn flush(&mut self) -> editline::Result<()> {
        self.terminal.flush()
    }

    fn enter_raw_mode(&mut self) -> editline::Result<()> {
        self.terminal.enter_raw_mode()
    }

    fn exit_raw_mode(&mut self) -> editline::Result<()> {
        self.terminal.exit_raw_mode()
    }

    fn cursor_left(&mut self) -> editline::Result<()> {
        self.terminal.cursor_left()
    }

    fn cursor_right(&mut self) -> editline::Result<()> {
        self.terminal.cursor_right()
    }

    fn clear_eol(&mut self) -> editline::Result<()> {
        self.terminal.clear_eol()
    }

    fn parse_key_event(&mut self) -> editline::Result<editline::KeyEvent> {
        self.terminal.parse_key_event()
    }
}

impl<T: Terminal> Output for TerminalOutput<T> {
    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        Terminal::write(self, data).map_err(|_| ())
    }

    fn flush(&mut self) -> Result<(), ()> {
        Terminal::flush(self).map_err(|_| ())
    }
}

// Helper to write a string without newline
fn write_str<T: Terminal>(terminal: &mut T, s: &str) -> editline::Result<()> {
    terminal.write(s.as_bytes())?;
    terminal.flush()
}

// Helper to write a line with platform-appropriate line ending
fn write_line<T: Terminal>(terminal: &mut T, s: &str) -> editline::Result<()> {
    terminal.write(s.as_bytes())?;
    terminal.write(LINE_ENDING)?;
    terminal.flush()
}

// Generic helper for REPL line execution
// Returns true if REPL should continue, false if it should exit
fn execute_repl_line<T: Terminal>(terminal: &mut T, line: &str, interp: &mut Interpreter) -> bool {
    match execute_string(line, interp) {
        Ok(()) => {
            let _ = write_line(terminal, "");  // Blank line before stack display
            if !interp.stack.is_empty()
                && let Some(top) = interp.stack.last() {
                    let msg = format!(" => {} : {}", top, top.type_name());
                    let _ = write_line(terminal, &msg);
                }
            true // Continue REPL
        }
        Err(RuntimeError::QuitRequested) => {
            let _ = write_line(terminal, "Goodbye!");
            false // Exit REPL
        }
        Err(e) => {
            let _ = write_line(terminal, "");  // Blank line before error message
            let msg = format!("Error: {:?}", e);
            let _ = write_line(terminal, &msg);
            true // Continue REPL after error
        }
    }
}

/// Run a REPL (Read-Eval-Print Loop) with the given interpreter and terminal
///
/// This is the main entry point for creating a custom REPL. You can:
/// 1. Create your own Interpreter
/// 2. Add custom primitives and prelude code
/// 3. Call this function to start an interactive session
///
/// # Example
///
/// ```no_run
/// use uni_core::{Interpreter, repl::run_repl};
/// use editline::terminals::StdioTerminal;
///
/// let mut interp = Interpreter::new();
/// // Add your custom primitives here
/// run_repl(interp, StdioTerminal::new());
/// ```
pub fn run_repl<T: Terminal + 'static>(mut interp: Interpreter, terminal: T) {
    let mut editor = LineEditor::new(4096, 100);
    run_repl_with_editor(&mut editor, terminal, &mut interp);
}

/// Run a REPL with a custom LineEditor configuration
///
/// This allows you to control the buffer size and history size.
pub fn run_repl_with_editor<T: Terminal + 'static>(
    editor: &mut LineEditor,
    terminal: T,
    interp: &mut Interpreter
) {
    // Wrap terminal in Rc<RefCell<>> for shared access between REPL and interpreter
    let shared = Rc::new(RefCell::new(TerminalOutput::new(terminal)));

    // Give the interpreter an output adapter for primitives like '.'
    {
        let output_for_interp = Rc::clone(&shared);
        interp.set_output(Box::new(RefCellOutput {
            inner: output_for_interp,
        }));
    }

    // Create our REPL terminal
    let mut repl_term = RefCellTerminal {
        inner: shared,
    };

    // Print banner with blank line first
    let _ = write_line(&mut repl_term, "");
    let _ = write_line(&mut repl_term, " _   _       _ ");
    let _ = write_line(&mut repl_term, "| | | |_ __ (_)");
    let _ = write_line(&mut repl_term, "| | | | '_ \\| |");
    let _ = write_line(&mut repl_term, "| |_| | | | | |");
    let version_line = format!(" \\___/|_| |_|_| v{}", env!("CARGO_PKG_VERSION"));
    let _ = write_line(&mut repl_term, &version_line);
    let _ = write_line(&mut repl_term, "");
    let _ = write_line(&mut repl_term, "Type 'quit' or press Ctrl-D to exit");
    let _ = write_line(&mut repl_term, "Type 'stack' to see the current stack");
    let _ = write_line(&mut repl_term, "Type 'clear' to clear the stack");
    let _ = write_line(&mut repl_term, "Type 'words' to see defined words");
    let _ = write_line(&mut repl_term, "");

    loop {
        // Print prompt
        if write_str(&mut repl_term, "uni> ").is_err() {
            break;
        }

        // Read line
        match editor.read_line(&mut repl_term) {
            Ok(line) => {
                let line = line.trim();

                // Check for empty line - all other inputs are executed as Uni code
                if line.is_empty() {
                    // Empty line, just continue
                    continue;
                }

                // Execute the line as Uni code (includes primitives like quit, stack, clear, words)
                if !execute_repl_line(&mut repl_term, line, interp) {
                    break; // quit was called
                }
            }
            Err(e) => {
                match e {
                    editline::Error::Eof => {
                        let _ = write_line(&mut repl_term, "\nGoodbye!");
                        break;
                    }
                    editline::Error::Interrupted => {
                        let _ = write_line(&mut repl_term, "\nInterrupted. Use 'quit' or Ctrl-D to exit.");
                        continue;
                    }
                    _ => {
                        let _ = write_line(&mut repl_term, "\nGoodbye!");
                        break;
                    }
                }
            }
        }
    }
}
