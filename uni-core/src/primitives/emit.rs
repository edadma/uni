// ASYNC CONCEPT: Emit builtin - outputs a character from unicode codepoint
// Stack effect: ( codepoint -- )
// Example: 65 emit  => prints 'A'
//          10 emit  => prints newline

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;
use core::future::Future;
use core::pin::Pin;

pub fn emit_builtin(interp: &mut AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>
{
    Box::pin(async move {
        let val = interp.pop()?;

        let codepoint = match val {
            crate::value::Value::Int32(i) => i,
            crate::value::Value::Integer(i) => {
                use num_traits::ToPrimitive;
                i.to_i32().ok_or_else(|| RuntimeError::DomainError(
                    "codepoint too large for i32".to_string()
                ))?
            }
            _ => {
                return Err(RuntimeError::TypeError(
                    "emit expects an integer codepoint".to_string()
                ));
            }
        };

        // Convert i32 to u32 for char::from_u32
        if codepoint < 0 {
            return Err(RuntimeError::DomainError(
                "emit requires non-negative codepoint".to_string()
            ));
        }

        let c = char::from_u32(codepoint as u32)
            .ok_or_else(|| RuntimeError::DomainError(
                "invalid unicode codepoint".to_string()
            ))?;

        let s = c.to_string();
        interp.write_str_async(&s).await.map_err(|_| {
            RuntimeError::TypeError("Failed to write character".into())
        })?;

        Ok(())
    })
}
