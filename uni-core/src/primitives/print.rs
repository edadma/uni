// ASYNC CONCEPT: Print builtin - pops and displays the top stack value
// Usage: 42 .  (prints "42" and removes it from stack)

use crate::compat::{Box, format};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use core::future::Future;
use core::pin::Pin;

pub fn print_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        #[cfg(feature = "target-stm32h753zi")]
        defmt::info!("print_builtin called");

        let value = interp.pop()?;

        #[cfg(feature = "target-stm32h753zi")]
        defmt::info!("print_builtin: popped value, formatting");

        // User-friendly printing - strings without quotes for readability
        let output = match &value {
            Value::String(s) => format!("{}", s),
            _ => format!("{}", value),
        };

        #[cfg(feature = "target-stm32h753zi")]
        defmt::info!("print_builtin: calling write_str_async with output len={}", output.len());

        // ASYNC CONCEPT: Await the async write operation
        interp.write_str_async(&output).await.map_err(|_| {
            RuntimeError::TypeError("Failed to write to output".into())
        })?;

        #[cfg(feature = "target-stm32h753zi")]
        defmt::info!("print_builtin: write completed");

        Ok(())
    })
}
