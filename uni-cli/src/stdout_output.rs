// Async stdout wrapper implementing AsyncOutput

use core::future::Future;
use core::pin::Pin;
use std::io::{self, Write};
use uni_core::AsyncOutput;

pub struct StdoutOutput;

impl StdoutOutput {
    pub fn new() -> Self {
        StdoutOutput
    }
}

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
