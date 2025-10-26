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
        let value = interp.pop()?;

        // User-friendly printing - strings without quotes for readability
        let output = match &value {
            Value::String(s) => format!("{}", s),
            _ => format!("{}", value),
        };

        // ASYNC CONCEPT: Await the async write operation
        interp.write_str_async(&output).await.map_err(|_| {
            RuntimeError::TypeError("Failed to write to output".into())
        })?;

        Ok(())
    })
}
