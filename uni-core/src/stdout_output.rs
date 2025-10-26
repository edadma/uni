//! Async stdout wrapper implementing AsyncOutput
//!
//! This module provides a simple stdout output implementation for std platforms.

#[cfg(feature = "std")]
use core::future::Future;
#[cfg(feature = "std")]
use core::pin::Pin;
#[cfg(feature = "std")]
use std::io::{self, Write};
#[cfg(feature = "std")]
use crate::output::AsyncOutput;
#[cfg(feature = "std")]
use crate::compat::Box;

#[cfg(feature = "std")]
pub struct StdoutOutput;

#[cfg(feature = "std")]
impl StdoutOutput {
    pub fn new() -> Self {
        StdoutOutput
    }
}

#[cfg(feature = "std")]
impl AsyncOutput for StdoutOutput {
    fn write<'a>(&'a mut self, data: &'a [u8])
        -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>
    {
        Box::pin(async move {
            io::stdout().write_all(data).map_err(|_| ())?;
            Ok(())
        })
    }

    fn flush<'a>(&'a mut self)
        -> Pin<Box<dyn Future<Output = Result<(), ()>> + 'a>>
    {
        Box::pin(async move {
            io::stdout().flush().map_err(|_| ())?;
            Ok(())
        })
    }
}
