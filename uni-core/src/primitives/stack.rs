// Stack manipulation primitives
// Note: swap, dup, over, rot are defined in the prelude using pick and roll

use crate::compat::{format, Vec};
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;
use core::future::Future;
use core::pin::Pin;

// Drop: ( a -- )
pub fn drop_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    interp.pop()?;
    Ok(())
}

// Stack: ( -- ) Display the current stack contents (non-destructive)
pub fn stack_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        stack_impl(interp).await
    })
}

async fn stack_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    if interp.stack.is_empty() {
        let _ = interp.writeln_async("Stack is empty").await;
    } else {
        // Collect all lines first to avoid borrow checker issues
        let mut lines = Vec::new();

        let msg = format!("Stack ({} items):", interp.stack.len());
        lines.push(msg);

        // Platform-specific limits: show fewer items on embedded systems
        let limit = if cfg!(target_os = "none") { 5 } else { 10 };

        for (i, value) in interp.stack.iter().rev().enumerate() {
            if i >= limit {
                let msg = format!("  ... and {} more", interp.stack.len() - limit);
                lines.push(msg);
                break;
            }
            let msg = format!("  {}: {}", i, value);
            lines.push(msg);
        }

        // Write all lines
        for line in lines {
            let _ = interp.writeln_async(&line).await;
        }
    }

    Ok(())
}
