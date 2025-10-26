//! Async spawn primitive - spawns a quotation as a background task

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use crate::compat::Box;
use core::future::Future;
use core::pin::Pin;

#[cfg(not(target_os = "none"))]
use std::collections::HashMap;
#[cfg(target_os = "none")]
use alloc::collections::BTreeMap as HashMap;

pub fn spawn(interp: &mut AsyncInterpreter) -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>> {
    Box::pin(async move {
        // Pop quotation from stack
        let quotation = interp.stack.pop()
            .ok_or_else(|| RuntimeError::StackUnderflow)?;

        #[cfg(feature = "target-stm32h753zi")]
        {
            spawn_task_embassy(quotation, interp).await
        }
        #[cfg(not(feature = "target-stm32h753zi"))]
        {
            let _ = quotation; // Silence unused warning
            Err(RuntimeError::TypeError("spawn only supported on STM32 target currently".into()))
        }
    })
}

#[cfg(feature = "target-stm32h753zi")]
async fn spawn_task_embassy(quotation: Value, interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    // Check if spawner is available
    let spawner = interp.spawner.as_ref()
        .ok_or_else(|| RuntimeError::DomainError("No spawner available - cannot spawn tasks".into()))?
        .clone();

    // Validate it's a list/quotation
    match &quotation {
        Value::Pair(_, _) | Value::Nil => {
            #[cfg(feature = "target-stm32h753zi")]
            defmt::info!("Spawning background task with quotation");

            // Clone quotation and dictionary Rc for the spawned task
            let quotation_clone = quotation.clone();
            let dict_rc = interp.dictionary.clone();

            // Spawn the task
            spawner.spawn(background_task(quotation_clone, dict_rc))
                .map_err(|_| {
                    #[cfg(feature = "target-stm32h753zi")]
                    defmt::error!("Failed to spawn task - spawner full");
                    RuntimeError::DomainError("Failed to spawn task - spawner full".into())
                })?;

            #[cfg(feature = "target-stm32h753zi")]
            defmt::info!("Background task spawned successfully");

            Ok(())
        }
        _ => Err(RuntimeError::TypeError("spawn requires a quotation (list)".into())),
    }
}

// Embassy task that executes Uni code in the background
#[cfg(feature = "target-stm32h753zi")]
#[embassy_executor::task]
async fn background_task(
    quotation: Value,
    dictionary: crate::compat::Rc<core::cell::RefCell<HashMap<crate::compat::Rc<str>, crate::interpreter::DictEntry>>>,
) {
    use crate::interpreter::AsyncInterpreter;
    use crate::compat::Box;

    defmt::info!("Background task started");

    // Create new interpreter for this task
    let mut task_interp = AsyncInterpreter::new();

    // Share the dictionary with the main task
    task_interp.dictionary = dictionary;

    // Set up output to use the same channel
    let output = Box::new(UsbOutputForTask::new());
    task_interp.set_async_output(output);
    defmt::info!("Background task: output handler set");

    defmt::info!("Background task executing quotation with {} items in dict", task_interp.dictionary.borrow().len());

    // Execute the quotation by pushing it and calling exec
    // Push quotation to stack
    task_interp.stack.push(quotation.clone());

    // Execute "exec" to actually run the quotation
    use crate::evaluator::execute_string;
    match execute_string("exec", &mut task_interp).await {
        Ok(_) => defmt::info!("Background task completed successfully"),
        Err(e) => defmt::error!("Background task error: {:?}", defmt::Debug2Format(&e)),
    }
}

// Output implementation for spawned tasks
// Note: This uses the UsbOutput type which is defined in the platform-specific code
// For STM32, spawned tasks will write to the same global WRITE_CHANNEL
#[cfg(feature = "target-stm32h753zi")]
struct UsbOutputForTask;

#[cfg(feature = "target-stm32h753zi")]
impl UsbOutputForTask {
    fn new() -> Self {
        Self
    }
}

#[cfg(feature = "target-stm32h753zi")]
impl crate::output::AsyncOutput for UsbOutputForTask {
    fn write<'a>(&'a mut self, data: &'a [u8])
        -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<(), ()>> + 'a>>
    {
        use crate::compat::Box;
        Box::pin(async move {
            if !data.is_empty() {
                defmt::info!("UsbOutputForTask: writing {} bytes", data.len());
                // Write to the shared channel - same as main REPL output
                let mut buf = heapless::Vec::<u8, 256>::new();
                match buf.extend_from_slice(data) {
                    Ok(_) => {
                        defmt::info!("UsbOutputForTask: sending to channel");
                        crate::platform_output::WRITE_CHANNEL.send(buf).await;
                        defmt::info!("UsbOutputForTask: sent successfully");
                    }
                    Err(_) => {
                        defmt::error!("UsbOutputForTask: failed to extend_from_slice (data too large?)");
                    }
                }
            }
            Ok(())
        })
    }

    fn flush<'a>(&'a mut self)
        -> core::pin::Pin<Box<dyn core::future::Future<Output = Result<(), ()>> + 'a>>
    {
        use crate::compat::Box;
        Box::pin(async move {
            Ok(())
        })
    }
}
