//! Async output trait for Uni interpreter
//!
//! This module defines the AsyncOutput trait that allows the interpreter
//! to write to async I/O destinations like async terminals, UART, etc.
//!
//! Unlike the sync version, all I/O operations are truly async and non-blocking.

use crate::compat::Box;
use core::future::Future;
use core::pin::Pin;

/// AsyncOutput trait for async I/O operations
///
/// This is the async version of the Output trait. All write operations
/// are async and return futures that can be awaited.
///
/// The interpreter uses this trait for all output operations like print, cr, words, etc.
pub trait AsyncOutput {
    /// Write bytes to the output asynchronously
    ///
    /// Returns a future that completes when the write is done.
    /// The future returns Ok(()) on success, Err(()) on failure.
    fn write<'a>(&'a mut self, data: &'a [u8])
        -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>;

    /// Flush any buffered output asynchronously
    ///
    /// Returns a future that completes when the flush is done.
    /// The future returns Ok(()) on success, Err(()) on failure.
    fn flush<'a>(&'a mut self)
        -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>;
}

// Helper macro to box async functions
// This makes it easier to implement AsyncOutput
#[macro_export]
macro_rules! box_async {
    ($future:expr) => {
        Box::pin($future)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(target_os = "none"))]
    use std::vec::Vec;
    #[cfg(target_os = "none")]
    use alloc::vec::Vec;

    // Mock async output for testing
    struct MockAsyncOutput {
        buffer: Vec<u8>,
    }

    impl MockAsyncOutput {
        fn new() -> Self {
            Self {
                buffer: Vec::new(),
            }
        }

        fn get_output(&self) -> &[u8] {
            &self.buffer
        }
    }

    impl AsyncOutput for MockAsyncOutput {
        fn write<'a>(&'a mut self, data: &'a [u8])
            -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>
        {
            Box::pin(async move {
                self.buffer.extend_from_slice(data);
                Ok(())
            })
        }

        fn flush<'a>(&'a mut self)
            -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>
        {
            Box::pin(async move {
                Ok(())
            })
        }
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_mock_output() {
        let mut output = MockAsyncOutput::new();

        output.write(b"Hello").await.unwrap();
        output.write(b" ").await.unwrap();
        output.write(b"World").await.unwrap();
        output.flush().await.unwrap();

        assert_eq!(output.get_output(), b"Hello World");
    }
}
