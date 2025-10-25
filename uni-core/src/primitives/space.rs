// Space primitive - outputs a single space character

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;
use core::future::Future;
use core::pin::Pin;

pub fn space_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        interp.write_str_async(" ").await.map_err(|_| {
            RuntimeError::TypeError("Failed to write space".into())
        })?;
        Ok(())
    })
}
