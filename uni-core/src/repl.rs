//! Async REPL implementation for Uni
//!
//! This module provides a reusable async REPL that works with any AsyncTerminal
//! implementation from the editline crate. Users can easily create custom REPLs
//! by providing their terminal implementation.

#[cfg(feature = "repl")]
use crate::interpreter::AsyncInterpreter;
#[cfg(feature = "repl")]
use crate::evaluator::execute_string;

/// Run an async REPL loop with the given terminal and interpreter.
///
/// This is the basic REPL without async output support. Use this for simple
/// terminals or when spawned tasks don't need to interrupt the prompt.
///
/// # Arguments
///
/// * `terminal` - An async terminal implementing editline's AsyncTerminal trait
/// * `interpreter` - The async interpreter to execute code with
///
/// # Example
///
/// ```ignore
/// use uni_core::repl::run_repl;
/// use uni_core::interpreter::AsyncInterpreter;
/// use editline::terminals::EmbassyUsbTerminal;
///
/// let mut interp = AsyncInterpreter::new();
/// let mut terminal = EmbassyUsbTerminal::new(usb_class);
///
/// run_repl(&mut terminal, &mut interp).await?;
/// ```
#[cfg(feature = "repl")]
pub async fn run_repl<T>(
    terminal: &mut T,
    interpreter: &mut AsyncInterpreter,
) -> Result<(), ()>
where
    T: editline::AsyncTerminal,
{
    use editline::AsyncLineEditor;

    let mut editor = AsyncLineEditor::new(256, 10);

    loop {
        terminal.write(b"> ").await.map_err(|_| ())?;
        terminal.flush().await.map_err(|_| ())?;

        match editor.read_line(terminal).await {
            Ok(line) => {
                if !line.trim().is_empty() {
                    match execute_string(line.as_str(), interpreter).await {
                        Ok(_) => {
                            // Print blank line, then stack top
                            if let Some(value) = interpreter.stack.last() {
                                #[cfg(target_os = "none")]
                                {
                                    use crate::compat::format;
                                    let val_str = format!("\r\n{}\r\n", value);
                                    let _ = terminal.write(val_str.as_bytes()).await;
                                }
                                #[cfg(not(target_os = "none"))]
                                {
                                    let val_str = format!("\n{}\n", value);
                                    let _ = terminal.write(val_str.as_bytes()).await;
                                }
                                let _ = terminal.flush().await;
                            }
                        }
                        Err(e) => {
                            #[cfg(target_os = "none")]
                            {
                                use crate::compat::format;
                                let err_str = format!("Error: {}\r\n", e);
                                let _ = terminal.write(err_str.as_bytes()).await;
                            }
                            #[cfg(not(target_os = "none"))]
                            {
                                let err_str = format!("Error: {}\n", e);
                                let _ = terminal.write(err_str.as_bytes()).await;
                            }
                            let _ = terminal.flush().await;
                        }
                    }

                    // Add blank line before next prompt
                    #[cfg(target_os = "none")]
                    let _ = terminal.write(b"\r\n").await;
                    #[cfg(not(target_os = "none"))]
                    let _ = terminal.write(b"\n").await;
                }
            }
            Err(_e) => {
                break;
            }
        }

        // Check if still connected (for terminals that support DTR)
        if !terminal.dtr() {
            break;
        }
    }

    Ok(())
}

/// Run an async REPL with support for real-time output from spawned tasks.
///
/// This version uses editline's `read_line_with_async_output()` to allow
/// background tasks to interrupt the prompt with output.
///
/// # Arguments
///
/// * `terminal` - An async terminal implementing editline's AsyncTerminal trait
/// * `interpreter` - The async interpreter to execute code with
/// * `output_channel` - A closure that returns a future yielding async output
///
/// # Example
///
/// ```ignore
/// use uni_core::repl::run_repl_with_async_output;
/// use uni_core::platform_output::WRITE_CHANNEL;
///
/// run_repl_with_async_output(&mut terminal, &mut interp, || async {
///     Some(WRITE_CHANNEL.receive().await)
/// }).await?;
/// ```
#[cfg(all(feature = "repl", feature = "target-stm32h753zi"))]
pub async fn run_repl_with_async_output<T, F, Fut>(
    terminal: &mut T,
    interpreter: &mut AsyncInterpreter,
    mut output_fut: F,
) -> Result<(), ()>
where
    T: editline::AsyncTerminal,
    F: FnMut() -> Fut,
    Fut: core::future::Future<Output = Option<heapless::Vec<u8, 256>>>,
{
    use editline::AsyncLineEditor;
    use embassy_futures::select::{select, Either};
    use core::pin::pin;

    let mut editor = AsyncLineEditor::new(256, 10);

    loop {
        // Drain any pending output from background tasks before showing prompt
        while let Ok(data) = crate::platform_output::WRITE_CHANNEL.try_receive() {
            let _ = terminal.write(&data).await;
        }
        let _ = terminal.flush().await;

        // Show prompt
        terminal.write(b"> ").await.map_err(|_| ())?;
        terminal.flush().await.map_err(|_| ())?;

        // Read line with async output support
        match editor.read_line_with_async_output(terminal, &mut output_fut).await {
            Ok(line) => {
                if !line.trim().is_empty() {
                    // Execute code while draining output in real-time
                    let exec_result = {
                        let exec_fut = pin!(execute_string(line.as_str(), interpreter));
                        let mut exec_fut = exec_fut;

                        // Continuously drain output until execution completes
                        loop {
                            match select(&mut exec_fut, crate::platform_output::WRITE_CHANNEL.receive()).await {
                                Either::First(result) => {
                                    // Execution completed
                                    break result;
                                }
                                Either::Second(data) => {
                                    // Output available - write it immediately
                                    let _ = terminal.write(&data).await;
                                    let _ = terminal.flush().await;
                                }
                            }
                        }
                    };

                    // Drain any remaining output
                    while let Ok(data) = crate::platform_output::WRITE_CHANNEL.try_receive() {
                        let _ = terminal.write(&data).await;
                    }
                    let _ = terminal.flush().await;

                    // Handle execution result
                    match exec_result {
                        Ok(_) => {
                            // Print blank line, then stack top
                            if let Some(value) = interpreter.stack.last() {
                                use crate::compat::format;
                                let val_str = format!("\r\n{}\r\n", value);
                                let _ = terminal.write(val_str.as_bytes()).await;
                                let _ = terminal.flush().await;
                            }
                        }
                        Err(e) => {
                            use crate::compat::format;
                            let err_str = format!("Error: {}\r\n", e);
                            let _ = terminal.write(err_str.as_bytes()).await;
                            let _ = terminal.flush().await;
                        }
                    }

                    // Add blank line before next prompt
                    let _ = terminal.write(b"\r\n").await;
                }
            }
            Err(_e) => {
                break;
            }
        }

        // Check if still connected (for terminals that support DTR)
        if !terminal.dtr() {
            break;
        }
    }

    Ok(())
}
