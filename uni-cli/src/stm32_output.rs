//! STM32 output for Uni REPL using async channel
//!
//! This module provides an AsyncOutput implementation that sends output
//! via an async channel shared with uni-core. This allows both the main REPL
//! and spawned background tasks to output to USB.

#[cfg(target_os = "none")]
use alloc::boxed::Box;

use uni_core::output::AsyncOutput;
use core::future::Future;
use core::pin::Pin;

// Re-export the channel from uni-core for convenience
pub use uni_core::platform_output::WRITE_CHANNEL;

pub struct UsbOutput;

impl UsbOutput {
    pub fn new() -> Self {
        Self
    }
}

impl AsyncOutput for UsbOutput {
    fn write<'a>(&'a mut self, data: &'a [u8])
        -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>
    {
        Box::pin(async move {
            if !data.is_empty() {
                // Write to the shared channel - same as spawned tasks
                let mut buf = heapless::Vec::<u8, 256>::new();
                if buf.extend_from_slice(data).is_ok() {
                    WRITE_CHANNEL.send(buf).await;
                }
            }
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
