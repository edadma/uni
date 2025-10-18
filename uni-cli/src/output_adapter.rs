/// Output adapter that wraps editline::Terminal to implement uni_core::Output.
///
/// This allows the CLI to use editline for I/O while keeping uni-core independent
/// of any specific I/O library.

use uni_core::Output;
use editline::Terminal;

/// Wraps any editline Terminal to implement the uni_core Output trait.
///
/// This adapter converts between editline's error handling (editline::Error/Result)
/// and uni_core's simpler error model (Result<(), ()>).
pub struct TerminalOutput<T: Terminal> {
    terminal: T,
}

impl<T: Terminal> TerminalOutput<T> {
    pub fn new(terminal: T) -> Self {
        Self { terminal }
    }

    /// Get a mutable reference to the underlying terminal.
    /// Useful for operations that need full Terminal functionality.
    #[allow(dead_code)]
    pub fn terminal_mut(&mut self) -> &mut T {
        &mut self.terminal
    }
}

impl<T: Terminal> Output for TerminalOutput<T> {
    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        self.terminal.write(data).map_err(|_| ())
    }

    fn flush(&mut self) -> Result<(), ()> {
        self.terminal.flush().map_err(|_| ())
    }
}

/// Also implement Terminal for TerminalOutput so it can be used as a Terminal
/// in the REPL (for reading input).
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
