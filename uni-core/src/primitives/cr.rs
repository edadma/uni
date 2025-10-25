// ASYNC CONCEPT: CR (carriage return) builtin - outputs a newline
// Stack effect: ( -- )

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;
use core::future::Future;
use core::pin::Pin;

pub fn cr_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        #[cfg(not(target_os = "none"))]
        let newline = "\n";
        #[cfg(target_os = "none")]
        let newline = "\r\n";

        interp.write_str_async(newline).await.map_err(|_| {
            RuntimeError::TypeError("Failed to write newline".into())
        })?;

        Ok(())
    })
}
