//! Async spawn primitive - spawns a quotation as a background task

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;
use crate::compat::Box;
use core::future::Future;
use core::pin::Pin;

// Import Value for both targets
use crate::value::Value;

// Import HashMap for type signatures (BTreeMap for STM32, HashMap for Linux)
#[cfg(feature = "target-stm32h753zi")]
use alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "target-stm32h753zi"))]
use std::collections::HashMap;

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
            spawn_task_tokio(quotation, interp).await
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
    dictionary: crate::compat::Arc<core::cell::RefCell<HashMap<crate::compat::Rc<str>, crate::interpreter::DictEntry>>>,
) {
    use crate::interpreter::AsyncInterpreter;
    use crate::compat::Box;

    defmt::info!("Background task started");

    // Create new interpreter for this task
    let mut task_interp = AsyncInterpreter::new();

    // Share the dictionary with the main task
    task_interp.dictionary = dictionary;

    // Set up output to use the same USB channel
    let output = Box::new(UsbOutputForTask::new());
    task_interp.set_async_output(output);
    defmt::info!("Background task: output handler set");

    #[cfg(target_os = "none")]
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

// Output implementation for spawned tasks on STM32
// Writes to the same global WRITE_CHANNEL as the main REPL
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

// Tokio spawn implementation for Linux/std targets
#[cfg(not(feature = "target-stm32h753zi"))]
async fn spawn_task_tokio(quotation: Value, interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    // Validate it's a list/quotation
    match &quotation {
        Value::Pair(_, _) | Value::Nil => {
            // Clone quotation and dictionary for the spawned task
            let quotation_clone = quotation.clone();
            let dict_clone = interp.dictionary.clone();

            // Check if we have async output
            let has_output = interp.has_async_output();

            // Spawn the task using tokio::task::spawn_local
            // This allows us to use !Send types like Rc<>
            tokio::task::spawn_local(background_task_tokio(quotation_clone, dict_clone, has_output));

            Ok(())
        }
        _ => Err(RuntimeError::TypeError("spawn requires a quotation (list)".into())),
    }
}

// Tokio background task for executing Uni code
#[cfg(not(feature = "target-stm32h753zi"))]
async fn background_task_tokio(
    quotation: Value,
    dictionary: std::sync::Arc<std::sync::Mutex<HashMap<std::rc::Rc<str>, crate::interpreter::DictEntry>>>,
    has_output: bool,
) {
    use crate::interpreter::AsyncInterpreter;

    // Create new interpreter for this task
    let mut task_interp = AsyncInterpreter::new();

    // Share the dictionary with the main task
    task_interp.dictionary = dictionary;

    // Set up output if needed
    if has_output {
        let output = Box::new(crate::stdout_output::StdoutOutput::new()) as Box<dyn crate::output::AsyncOutput>;
        task_interp.set_async_output(output);
    }

    // Execute the quotation by pushing it and calling exec
    task_interp.stack.push(quotation.clone());

    // Execute "exec" to actually run the quotation
    use crate::evaluator::execute_string;
    if let Err(e) = execute_string("exec", &mut task_interp).await {
        eprintln!("Background task error: {}", e);
    }
}
