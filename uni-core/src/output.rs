/// Minimal output abstraction for the Uni interpreter.
///
/// The interpreter uses this trait to write output from primitives like `.` (print).
/// This allows embedding applications to provide custom output implementations
/// (files, network, displays, etc.) without depending on specific I/O libraries.
///
/// # Examples
///
/// ```
/// use uni_core::Output;
///
/// struct ConsoleOutput;
///
/// impl Output for ConsoleOutput {
///     fn write(&mut self, data: &[u8]) -> Result<(), ()> {
///         print!("{}", String::from_utf8_lossy(data));
///         Ok(())
///     }
///
///     fn flush(&mut self) -> Result<(), ()> {
///         use std::io::Write;
///         std::io::stdout().flush().map_err(|_| ())
///     }
/// }
/// ```
pub trait Output {
    /// Write bytes to the output.
    ///
    /// This method should write the provided bytes to whatever output destination
    /// is appropriate for the implementation (stdout, file, network, display, etc.).
    fn write(&mut self, data: &[u8]) -> Result<(), ()>;

    /// Flush any buffered output.
    ///
    /// This method should ensure that any buffered data is written to the output
    /// destination immediately. Implementations may choose to do nothing if the
    /// output is unbuffered.
    fn flush(&mut self) -> Result<(), ()>;
}
