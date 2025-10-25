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
    repl::run_repl().await
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
