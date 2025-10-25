//! Uni CLI - Command-line async REPL and interpreter
//!
//! This is a thin wrapper around uni-core that builds the executable.
//! Users can create their own enhanced executables by using uni-core directly
//! and adding custom primitives.

#![cfg_attr(target_os = "none", no_std)]

#[cfg(target_os = "none")]
extern crate alloc;

// Linux/std modules
#[cfg(not(target_os = "none"))]
mod stdout_output;
#[cfg(not(target_os = "none"))]
mod repl;

// Linux entry point
#[cfg(not(target_os = "none"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use std::io::{self, IsTerminal, Read};
    use std::fs;

    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    if args.len() > 1 {
        match args[1].as_str() {
            "-e" => {
                // Evaluate mode: execute code and print top of stack
                if args.len() < 3 {
                    eprintln!("Usage: {} -e <code>", args[0]);
                    std::process::exit(1);
                }
                let code = &args[2];
                execute_and_print(code).await?;
            }
            "-c" => {
                // Command mode: execute code without printing
                if args.len() < 3 {
                    eprintln!("Usage: {} -c <code>", args[0]);
                    std::process::exit(1);
                }
                let code = &args[2];
                execute_code(code).await?;
            }
            _ => {
                // File mode: execute code from file
                let file_path = &args[1];
                let code = fs::read_to_string(file_path)
                    .map_err(|e| format!("Failed to read file '{}': {}", file_path, e))?;
                execute_code(&code).await?;
            }
        }
    } else {
        // Check if stdin is piped
        if !io::stdin().is_terminal() {
            // Read from stdin and execute
            let mut code = String::new();
            io::stdin().read_to_string(&mut code)?;
            execute_code(&code).await?;
        } else {
            // No arguments and stdin is terminal - run REPL
            repl::run_repl().await?;
        }
    }

    Ok(())
}

#[cfg(not(target_os = "none"))]
async fn execute_code(code: &str) -> Result<(), Box<dyn std::error::Error>> {
    use uni_core::evaluator::execute_string;
    use uni_core::interpreter::AsyncInterpreter;
    use uni_core::hardware::linux::LinuxTimeSource;

    let mut interp = AsyncInterpreter::new();

    // Set up stdout output handler
    let output = Box::new(stdout_output::StdoutOutput::new());
    interp.set_async_output(output);

    interp.set_time_source(Box::new(LinuxTimeSource::new()));

    // Load prelude
    interp.load_prelude().await
        .map_err(|e| format!("Failed to load prelude: {}", e))?;

    execute_string(code, &mut interp).await
        .map_err(|e| format!("Error: {}", e))?;

    Ok(())
}

#[cfg(not(target_os = "none"))]
async fn execute_and_print(code: &str) -> Result<(), Box<dyn std::error::Error>> {
    use uni_core::evaluator::execute_string;
    use uni_core::interpreter::AsyncInterpreter;
    use uni_core::hardware::linux::LinuxTimeSource;

    let mut interp = AsyncInterpreter::new();

    // Set up stdout output handler
    let output = Box::new(stdout_output::StdoutOutput::new());
    interp.set_async_output(output);

    interp.set_time_source(Box::new(LinuxTimeSource::new()));

    // Load prelude
    interp.load_prelude().await
        .map_err(|e| format!("Failed to load prelude: {}", e))?;

    execute_string(code, &mut interp).await
        .map_err(|e| format!("Error: {}", e))?;

    // Print the top value on the stack
    if let Some(value) = interp.stack.last() {
        println!("{}", value);
    }

    Ok(())
}

// STM32H753ZI async entry point (will be added later)
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
#[embassy_executor::main]
async fn stm32_main(_spawner: embassy_executor::Spawner) {
    // TODO: Implement STM32 async REPL
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
