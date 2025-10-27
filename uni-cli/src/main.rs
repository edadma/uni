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
mod repl;

// STM32 modules
#[cfg(all(target_os = "none", feature = "target-stm32h753zi"))]
mod stm32_output;

// Linux entry point
#[cfg(not(target_os = "none"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::env;
    use tokio::task::LocalSet;

    let args: Vec<String> = env::args().collect();

    // Create LocalSet for spawn support (allows !Send types like Rc<>)
    let local = LocalSet::new();

    local.run_until(async move {
        main_async(args).await
    }).await
}

#[cfg(not(target_os = "none"))]
async fn main_async(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, IsTerminal, Read};
    use std::fs;

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

    let mut interp = AsyncInterpreter::new();

    // Set up stdout output handler
    let output = Box::new(uni_core::StdoutOutput::new());
    interp.set_async_output(output);

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

    let mut interp = AsyncInterpreter::new();

    // Set up stdout output handler
    let output = Box::new(uni_core::StdoutOutput::new());
    interp.set_async_output(output);

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

    // Initialize RTC with LSE (Low Speed External) clock for battery backup
    use embassy_stm32::rtc::{Rtc, RtcConfig};
    use alloc::sync::Arc;
    use core::cell::RefCell;

    let rtc_config = RtcConfig::default();
    let rtc = Rtc::new(p.RTC, rtc_config);
    let rtc_arc = Arc::new(RefCell::new(rtc));

    defmt::info!("RTC initialized");

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

    // REPL using uni-core
    let repl_fut = async {
        use uni_core::interpreter::AsyncInterpreter;
        use alloc::boxed::Box;
        use editline::{AsyncTerminal, terminals::EmbassyUsbTerminal};

        // Create terminal
        let mut terminal = EmbassyUsbTerminal::new(class);

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

        // Inject RTC into Platform
        use uni_core::platform::{Platform, Stm32Platform};
        interp.platform = Platform::Stm32(Stm32Platform {
            rtc: Some(rtc_arc.clone()),
        });

        // Load prelude
        match interp.load_prelude().await {
            Ok(_) => defmt::info!("Prelude loaded"),
            Err(_) => defmt::warn!("Failed to load prelude"),
        }

        // Send welcome message
        let _ = terminal.write(b"\r\nUni REPL v0.1.0 on STM32H753ZI\r\n").await;
        let _ = terminal.write(b"Type expressions to evaluate\r\n\r\n").await;
        let _ = terminal.flush().await;

        // Run the REPL from uni-core
        let _ = uni_core::repl::run_repl_with_async_output(
            &mut terminal,
            &mut interp,
            || async {
                Some(stm32_output::WRITE_CHANNEL.receive().await)
            }
        ).await;

        defmt::info!("REPL exited");
    };

    // Run USB device and REPL concurrently
    embassy_futures::join::join(usb.run(), repl_fut).await;
}
