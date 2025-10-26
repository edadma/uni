//! Uni CLI - Command-line async REPL and interpreter
//!
//! This is a thin wrapper around uni-core that builds the executable.
//! Users can create their own enhanced executables by using uni-core directly
//! and adding custom primitives.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(target_os = "none")]
extern crate alloc;

// Global allocator for embedded targets
#[cfg(target_os = "none")]
use alloc_cortex_m::CortexMHeap;

#[cfg(target_os = "none")]
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

// Embedded runtime support (defmt logging and panic handler)
#[cfg(feature = "target-stm32h753zi")]
use {defmt_rtt as _, panic_probe as _};

// defmt timestamp
#[cfg(feature = "target-stm32h753zi")]
defmt::timestamp!("{=u64:us}", {
    embassy_time::Instant::now().as_micros()
});

// Linux/std modules
#[cfg(not(target_os = "none"))]
mod stdout_output;
#[cfg(not(target_os = "none"))]
mod repl;

// STM32 modules
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
mod stm32_output;

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

// STM32H753ZI USB interrupt bindings
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
embassy_stm32::bind_interrupts!(struct Irqs {
    OTG_FS => embassy_stm32::usb::InterruptHandler<embassy_stm32::peripherals::USB_OTG_FS>;
});

// STM32H753ZI async entry point
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
#[embassy_executor::main]
async fn stm32_main(spawner: embassy_executor::Spawner) {
    use embassy_stm32::{usb, Config};
    use embassy_stm32::rcc::*;
    use embassy_stm32::usb::Driver;
    use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
    use embassy_usb::Builder;

    // Initialize the allocator
    const HEAP_SIZE: usize = 128 * 1024; // 128KB heap
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
    unsafe { ALLOCATOR.init(&raw mut HEAP as *const u8 as usize, HEAP_SIZE) }

    // Configure clocks for USB
    let mut config = Config::default();
    {
        config.rcc.hse = Some(Hse {
            freq: embassy_stm32::time::Hertz(8_000_000),
            mode: HseMode::Bypass,
        });
        config.rcc.hsi48 = Some(Hsi48Config { sync_from_usb: true });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL240,
            divp: Some(PllDiv::DIV2),
            divq: Some(PllDiv::DIV8),
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV2;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV2;
        config.rcc.apb3_pre = APBPrescaler::DIV2;
        config.rcc.apb4_pre = APBPrescaler::DIV2;
        config.rcc.voltage_scale = VoltageScale::Scale1;
        config.rcc.mux.usbsel = mux::Usbsel::HSI48;
    }

    let p = embassy_stm32::init(config);

    defmt::info!("Uni REPL for STM32H753ZI");
    defmt::info!("Heap initialized: {} bytes", HEAP_SIZE);

    // Create USB driver
    let mut usb_config = usb::Config::default();
    usb_config.vbus_detection = false;

    use core::ptr::addr_of_mut;

    static mut EP_OUT_BUFFER: [u8; 256] = [0u8; 256];
    let driver = Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, unsafe { &mut *addr_of_mut!(EP_OUT_BUFFER) }, usb_config);

    // Create USB device config
    static mut CONFIG_DESCRIPTOR: [u8; 256] = [0u8; 256];
    static mut BOS_DESCRIPTOR: [u8; 256] = [0u8; 256];
    static mut CONTROL_BUF: [u8; 64] = [0u8; 64];

    let mut device_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    device_config.manufacturer = Some("Uni Language");
    device_config.product = Some("Uni REPL");
    device_config.serial_number = Some("12345678");
    device_config.max_power = 100;
    device_config.max_packet_size_0 = 64;

    // Create CDC ACM state
    static mut STATE: State = State::new();

    let mut builder = Builder::new(
        driver,
        device_config,
        unsafe { &mut *addr_of_mut!(CONFIG_DESCRIPTOR) },
        unsafe { &mut *addr_of_mut!(BOS_DESCRIPTOR) },
        &mut [],
        unsafe { &mut *addr_of_mut!(CONTROL_BUF) },
    );

    // Create CDC ACM class
    let class = CdcAcmClass::new(&mut builder, unsafe { &mut *addr_of_mut!(STATE) }, 64);

    // Build USB device
    let mut usb = builder.build();

    defmt::info!("USB device initialized");
    defmt::info!("Connect via USB serial (tio /dev/ttyACM0)");

    // REPL using editline
    let repl_fut = async {
        use uni_core::interpreter::AsyncInterpreter;
        use uni_core::evaluator::execute_string;
        use alloc::boxed::Box;
        use editline::{AsyncLineEditor, AsyncTerminal, terminals::EmbassyUsbTerminal};
        use embassy_futures::select::{select, Either};
        use core::pin::pin;

        // Create terminal and editor
        let mut terminal = EmbassyUsbTerminal::new(class);
        let mut editor = AsyncLineEditor::new(256, 10);

        defmt::info!("Waiting for terminal connection (DTR)...");
        terminal.wait_connection().await;
        defmt::info!("Terminal connected!");

        // Initialize interpreter
        let mut interp = AsyncInterpreter::new();

        // Set up channel-based output
        let output = Box::new(stm32_output::UsbOutput::new());
        interp.set_async_output(output);

        // Inject spawner for async task spawning
        interp.set_spawner(spawner);

        // Load prelude
        match interp.load_prelude().await {
            Ok(_) => defmt::info!("Prelude loaded"),
            Err(_) => defmt::warn!("Failed to load prelude"),
        }

        // Send welcome message
        let _ = terminal.write(b"\r\nUni REPL v0.1.0 on STM32H753ZI\r\n").await;
        let _ = terminal.write(b"Type expressions to evaluate\r\n\r\n").await;
        let _ = terminal.flush().await;

        loop {
            // Show prompt
            let _ = terminal.write(b"> ").await;
            let _ = terminal.flush().await;

            // Read line with full editing support (backspace, etc.)
            match editor.read_line(&mut terminal).await {
                Ok(line) => {
                    defmt::info!("Got input: {}", line.as_str());

                    if !line.trim().is_empty() {
                        // Execute code while draining output in real-time
                        let exec_result = {
                            let exec_fut = pin!(execute_string(line.as_str(), &mut interp));
                            let mut exec_fut = exec_fut;

                            // Continuously drain output until execution completes
                            loop {
                                match select(&mut exec_fut, stm32_output::WRITE_CHANNEL.receive()).await {
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
                        while let Ok(data) = stm32_output::WRITE_CHANNEL.try_receive() {
                            let _ = terminal.write(&data).await;
                        }
                        let _ = terminal.flush().await;

                        // Handle execution result
                        match exec_result {
                            Ok(_) => {
                                // Print stack top
                                if let Some(value) = interp.stack.last() {
                                    let val_str = alloc::format!("{}\r\n", value);
                                    let _ = terminal.write(val_str.as_bytes()).await;
                                    let _ = terminal.flush().await;
                                }
                            }
                            Err(e) => {
                                let err_str = alloc::format!("Error: {}\r\n", e);
                                let _ = terminal.write(err_str.as_bytes()).await;
                                let _ = terminal.flush().await;
                            }
                        }
                    }
                }
                Err(_e) => {
                    defmt::error!("Error reading line");
                    break;
                }
            }

            // Check if still connected
            if !terminal.dtr() {
                defmt::info!("Terminal disconnected");
                break;
            }
        }

        defmt::info!("REPL exited");
    };

    // Run USB device and REPL concurrently
    embassy_futures::join::join(usb.run(), repl_fut).await;
}
