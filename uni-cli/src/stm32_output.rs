//! STM32 output for Uni REPL using async channel
//!
//! This module provides an AsyncOutput implementation that sends output
//! via an async channel. A separate task drains this channel and writes to USB.

#[cfg(target_os = "none")]
use alloc::boxed::Box;
#[cfg(target_os = "none")]
use alloc::vec::Vec;

use uni_core::output::AsyncOutput;
use core::future::Future;
use core::pin::Pin;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;

// Static channel for USB write requests
// Large capacity (200 messages) to handle big outputs like `words`
pub static WRITE_CHANNEL: Channel<CriticalSectionRawMutex, Vec<u8>, 200> = Channel::new();

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
                let vec = Vec::from(data);
                // Send to channel - will be drained by output task
                WRITE_CHANNEL.send(vec).await;
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
