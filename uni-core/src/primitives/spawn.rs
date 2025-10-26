//! Async spawn primitive - spawns a quotation as a background task

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use crate::compat::{Box, Rc, Vec};
use core::future::Future;
use core::pin::Pin;

pub fn spawn(interp: &mut AsyncInterpreter) -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>> {
    Box::pin(async move {
        // Pop quotation from stack
        let quotation = interp.stack.pop()
            .ok_or_else(|| RuntimeError::StackUnderflow)?;

        match quotation {
            Value::List(code) => {
                #[cfg(feature = "target-stm32h753zi")]
                {
                    spawn_task_embassy(code, interp).await
                }
                #[cfg(not(feature = "target-stm32h753zi"))]
                {
                    Err(RuntimeError::Custom("spawn only supported on STM32 target currently".into()))
                }
            }
            _ => Err(RuntimeError::Custom("spawn requires a quotation".into())),
        }
    })
}

#[cfg(feature = "target-stm32h753zi")]
async fn spawn_task_embassy(code: Rc<Vec<Value>>, interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    use crate::evaluator::eval_list;
    use crate::interpreter::{AsyncInterpreter as Interp, DictEntry};
    use alloc::boxed::Box as AllocBox;
    use alloc::vec::Vec as AllocVec;

    // Check if spawner is available
    let spawner = interp.spawner.as_ref()
        .ok_or_else(|| RuntimeError::Custom("No spawner available - cannot spawn tasks".into()))?
        .clone();

    // Clone code and dictionary for the spawned task
    let code_clone: AllocVec<Value> = (*code).clone();
    let dict_clone = interp.dictionary.clone();

    // Create channel for output if async_output is available
    let has_output = interp.async_output.is_some();

    // Spawn the task
    spawner.spawn(background_task(code_clone, dict_clone, has_output))
        .map_err(|_| RuntimeError::Custom("Failed to spawn task - spawner full".into()))?;

    Ok(())
}

// Embassy task that executes Uni code in the background
#[cfg(feature = "target-stm32h753zi")]
#[embassy_executor::task]
async fn background_task(
    code: alloc::vec::Vec<Value>,
    dictionary: alloc::collections::BTreeMap<crate::compat::Rc<str>, crate::interpreter::DictEntry>,
    has_output: bool,
) {
    use crate::evaluator::eval_list;
    use crate::interpreter::AsyncInterpreter;
    use alloc::boxed::Box;

    // Create new interpreter for this task
    let mut task_interp = AsyncInterpreter::new();

    // Copy the dictionary so it has access to defined words
    task_interp.dictionary = dictionary;

    // Set up output if parent had output
    // Note: Output setup is handled by uni-cli for STM32
    // The spawned task won't have direct access to output,
    // but it can use the write primitives which will use the global channel
    // For now, we skip setting up output to keep things simple
    let _ = has_output; // Silence unused warning

    // Execute the code
    match eval_list(&code, &mut task_interp).await {
        Ok(_) => {
            // Task completed successfully
        }
        Err(e) => {
            // Log error via defmt
            defmt::error!("Spawned task error: {}", defmt::Debug2Format(&e));
        }
    }
}
