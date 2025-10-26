//! Async delay primitive - waits for N milliseconds while letting other tasks run

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use core::future::Future;
use core::pin::Pin;
use crate::compat::Box;

pub fn delay(interp: &mut AsyncInterpreter) -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>> {
    Box::pin(async move {
        // Pop milliseconds from stack
        let ms_value = interp.stack.pop()
            .ok_or_else(|| RuntimeError::StackUnderflow)?;

        let ms = match ms_value {
            Value::Int32(n) => {
                if n < 0 {
                    return Err(RuntimeError::DomainError("delay requires non-negative milliseconds".into()));
                }
                n as u64
            }
            Value::Number(f) => {
                if f < 0.0 {
                    return Err(RuntimeError::DomainError("delay requires non-negative milliseconds".into()));
                }
                f as u64
            }
            Value::Integer(ref i) => {
                use num_traits::ToPrimitive;
                let n = i.to_i64().ok_or_else(|| RuntimeError::DomainError("delay value too large".into()))?;
                if n < 0 {
                    return Err(RuntimeError::DomainError("delay requires non-negative milliseconds".into()));
                }
                n as u64
            }
            _ => return Err(RuntimeError::TypeError("delay requires a number".into())),
        };

        // Platform-specific delay
        #[cfg(feature = "target-stm32h753zi")]
        {
            embassy_time::Timer::after_millis(ms).await;
        }

        #[cfg(not(feature = "target-stm32h753zi"))]
        {
            // For std targets, use tokio
            #[cfg(feature = "std")]
            {
                tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
            }
            #[cfg(not(feature = "std"))]
            {
                return Err(RuntimeError::TypeError("delay not supported on this platform".into()));
            }
        }

        Ok(())
    })
}
