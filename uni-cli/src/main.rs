#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(target_os = "none")]
extern crate alloc;

// output_adapter is only for sync targets
#[cfg(not(feature = "target-stm32h753zi"))]
mod output_adapter;

#[cfg(not(feature = "target-stm32h753zi"))]
use output_adapter::TerminalOutput;
use uni_core::{Interpreter, RuntimeError, execute_string};
#[cfg(not(target_os = "none"))]
use uni_core::primitives;

// Sync API imports - only for micro:bit, Pico, Pico 2 (not STM32 which uses async)
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
use editline::{LineEditor, Terminal};

#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
use core::cell::RefCell;
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
use alloc::{rc::Rc, boxed::Box, format};
#[cfg(feature = "target-stm32h753zi")]
use alloc::{boxed::Box, format, vec::Vec};

// Wrapper for shared Output access (interpreter needs it)
// Only used by sync embedded targets (not STM32)
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
struct RefCellOutput<T: Terminal> {
    inner: Rc<RefCell<TerminalOutput<T>>>,
}

#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
impl<T: Terminal> uni_core::Output for RefCellOutput<T> {
    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        <TerminalOutput<T> as uni_core::Output>::write(&mut *self.inner.borrow_mut(), data)
    }

    fn flush(&mut self) -> Result<(), ()> {
        <TerminalOutput<T> as uni_core::Output>::flush(&mut *self.inner.borrow_mut())
    }
}

// Wrapper for shared Terminal access (REPL needs it)
// Only used by sync embedded targets (not STM32)
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
struct RefCellTerminal<T: Terminal> {
    inner: Rc<RefCell<TerminalOutput<T>>>,
}

#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
impl<T: Terminal> Terminal for RefCellTerminal<T> {
    fn read_byte(&mut self) -> editline::Result<u8> {
        self.inner.borrow_mut().read_byte()
    }

    fn write(&mut self, data: &[u8]) -> editline::Result<()> {
        self.inner.borrow_mut().write(data)
    }

    fn flush(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().flush()
    }

    fn enter_raw_mode(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().enter_raw_mode()
    }

    fn exit_raw_mode(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().exit_raw_mode()
    }

    fn cursor_left(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().cursor_left()
    }

    fn cursor_right(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().cursor_right()
    }

    fn clear_eol(&mut self) -> editline::Result<()> {
        self.inner.borrow_mut().clear_eol()
    }

    fn parse_key_event(&mut self) -> editline::Result<editline::KeyEvent> {
        self.inner.borrow_mut().parse_key_event()
    }
}

// Platform-specific imports
#[cfg(not(target_os = "none"))]
use editline::terminals::StdioTerminal;
#[cfg(not(target_os = "none"))]
use std::env;

#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
use panic_halt as _;
#[cfg(target_os = "none")]
use alloc_cortex_m::CortexMHeap;

// Micro:bit imports for interrupt handler
#[cfg(feature = "target-microbit")]
use microbit::pac::interrupt;

// Timer interrupt handler for LED display (micro:bit only)
// Note: This must be in the binary crate, not the library
#[cfg(feature = "target-microbit")]
#[microbit::pac::interrupt]
fn TIMER1() {
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = uni_core::hardware::microbit::DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.handle_display_event();
        }
    });
}

// Platform-specific line endings (only for sync embedded targets)
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
const LINE_ENDING: &[u8] = b"\r\n";

// Banner text for REPL (shared by all embedded targets)
#[cfg(target_os = "none")]
const BANNER_LINES: &[&[u8]] = &[
    b"",
    b" _   _       _ ",
    b"| | | |_ __ (_)",
    b"| | | | '_ \\| |",
    b"| |_| | | | | |",
    // Version line is dynamic, added separately
];

#[cfg(target_os = "none")]
const BANNER_INSTRUCTIONS: &[&[u8]] = &[
    b"",
    b"Type 'quit' or press Ctrl-D to exit",
    b"Type 'stack' to see the current stack",
    b"Type 'clear' to clear the stack",
    b"Type 'words' to see defined words",
    b"",
];

// Linux/desktop main function
#[cfg(not(target_os = "none"))]
fn main() {
    // RUST CONCEPT: Command-line argument parsing with std::env::args()
    // args() returns an iterator over command-line arguments
    let args: Vec<String> = env::args().collect();

    // RUST CONCEPT: Pattern matching on argument count and content
    match args.len() {
        // No arguments - run REPL
        1 => run_repl(),

        // One argument - execute file or show help
        2 => {
            if args[1].starts_with('-') {
                // Treat flags without arguments as help request
                eprintln!("Usage:");
                eprintln!("  {} [file.uni]           # Execute Uni file", args[0]);
                eprintln!(
                    "  {} -f [file.uni]        # Execute Uni file (explicit)",
                    args[0]
                );
                eprintln!("  {} -c \"code\"            # Execute code string", args[0]);
                eprintln!(
                    "  {} -e \"code\"            # Execute code and print result",
                    args[0]
                );
                eprintln!("  {}                      # Run interactive REPL", args[0]);
                std::process::exit(1);
            } else {
                execute_file(&args[1]);
            }
        }

        // Two or more arguments - check for flags
        _ => {
            match args[1].as_str() {
                "-c" => {
                    // Execute code without automatic printing
                    execute_code(&args[2], false);
                }
                "-e" => {
                    // Execute code with automatic printing of stack top
                    execute_code(&args[2], true);
                }
                "-f" => {
                    // Explicit file execution flag
                    execute_file(&args[2]);
                }
                _ => {
                    // Show usage and exit
                    eprintln!("Usage:");
                    eprintln!("  {} [file.uni]           # Execute Uni file", args[0]);
                    eprintln!(
                        "  {} -f [file.uni]        # Execute Uni file (explicit)",
                        args[0]
                    );
                    eprintln!("  {} -c \"code\"            # Execute code string", args[0]);
                    eprintln!(
                        "  {} -e \"code\"            # Execute code and print result",
                        args[0]
                    );
                    eprintln!("  {}                      # Run interactive REPL", args[0]);
                    std::process::exit(1);
                }
            }
        }
    }
}

// RUST CONCEPT: File I/O and error handling
// Execute a Uni source file
#[cfg(not(target_os = "none"))]
fn execute_file(filename: &str) {
    use std::fs;

    // RUST CONCEPT: Reading files with proper error handling
    let file_contents = match fs::read_to_string(filename) {
        Ok(contents) => contents,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            std::process::exit(1);
        }
    };

    // RUST CONCEPT: Handling shebang lines
    // Skip the first line if it starts with #! (shebang)
    let code = if file_contents.starts_with("#!") {
        // Find the first newline and skip everything before it
        if let Some(newline_pos) = file_contents.find('\n') {
            &file_contents[newline_pos + 1..]
        } else {
            // File is only a shebang line with no code
            ""
        }
    } else {
        // No shebang, use entire file
        &file_contents
    };

    // RUST CONCEPT: Automatic initialization
    let mut interp = Interpreter::new();

    // Set up output for print primitive
    let terminal = StdioTerminal::new();
    let terminal_output = TerminalOutput::new(terminal);
    interp.set_output(Box::new(terminal_output));

    // Set up time source for datetime operations (std platforms only)
    #[cfg(feature = "std")]
    {
        use uni_core::hardware::linux_time::LinuxTimeSource;
        interp.set_time_source(Box::new(LinuxTimeSource::new()));
    }

    match execute_string(code, &mut interp) {
        Ok(()) => {
            // File executed successfully
            // For files, we don't automatically print anything (unlike -e flag)
            // The file should use '.' if it wants to print output
        }
        Err(e) => {
            eprintln!("Error executing '{}': {:?}", filename, e);
            std::process::exit(1);
        }
    }
}

// RUST CONCEPT: Function extraction for code organization
// Execute a single line of Uni code
// auto_print: if true, automatically prints the top stack value after execution
#[cfg(not(target_os = "none"))]
fn execute_code(code: &str, auto_print: bool) {
    // RUST CONCEPT: Automatic initialization
    // Interpreter::new() now automatically loads builtins and stdlib
    let mut interp = Interpreter::new();

    // Set up output for print primitive
    let terminal = StdioTerminal::new();
    let terminal_output = TerminalOutput::new(terminal);
    interp.set_output(Box::new(terminal_output));

    // Set up time source for datetime operations (std platforms only)
    #[cfg(feature = "std")]
    {
        use uni_core::hardware::linux_time::LinuxTimeSource;
        interp.set_time_source(Box::new(LinuxTimeSource::new()));
    }

    match execute_string(code, &mut interp) {
        Ok(()) => {
            // Success - code executed without errors
            if auto_print {
                // RUST CONCEPT: Conditional execution
                // For -e flag, automatically print the top stack value
                match primitives::print::print_builtin(&mut interp) {
                    Ok(()) => {
                        // Successfully printed the top value
                    }
                    Err(RuntimeError::StackUnderflow) => {
                        // Empty stack is okay - just don't print anything
                    }
                    Err(e) => {
                        eprintln!("Error printing result: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}

// Micro:bit main function
#[cfg(feature = "target-microbit")]
#[cortex_m_rt::entry]
fn mb_main() -> ! {
    // Initialize allocator - micro:bit v2 has 128KB RAM
    // Use ~112KB for heap, leaving 16KB for stack
    const HEAP_SIZE: usize = 114688; // 112 * 1024
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    #[global_allocator]
    static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

    unsafe { ALLOCATOR.init(&raw mut HEAP as *const u8 as usize, HEAP_SIZE) }

    // Just run the REPL - no command-line arguments on micro:bit
    run_repl()
}

// RUST CONCEPT: Function extraction for code organization
// Run the interactive REPL (Read-Eval-Print Loop)
#[cfg(not(target_os = "none"))]
fn run_repl() {
    // RUST CONCEPT: Automatic initialization
    // Interpreter::new() automatically loads builtins and stdlib
    let mut interp = Interpreter::new();

    // Set up time source for datetime operations (std platforms only)
    #[cfg(feature = "std")]
    {
        use uni_core::hardware::linux_time::LinuxTimeSource;
        interp.set_time_source(Box::new(LinuxTimeSource::new()));
    }

    // Use the REPL from uni-core
    uni_core::repl::run_repl(interp, StdioTerminal::new());
}

// Generic REPL loop that works with any Terminal implementation
// Only used by sync embedded targets (not STM32)
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
fn run_repl_loop<T: Terminal + 'static>(editor: &mut LineEditor, terminal: T, interp: &mut Interpreter) {
    // Wrap terminal in Rc<RefCell<>> for shared access between REPL and interpreter
    let shared = Rc::new(RefCell::new(TerminalOutput::new(terminal)));

    // Give the interpreter an output adapter for primitives like '.'
    {
        let output_for_interp = Rc::clone(&shared);
        interp.set_output(Box::new(RefCellOutput {
            inner: output_for_interp,
        }));
    }

    // Create our REPL terminal
    let mut repl_term = RefCellTerminal {
        inner: shared,
    };

    // Print banner
    for line in BANNER_LINES {
        let _ = write_line(&mut repl_term, core::str::from_utf8(line).unwrap());
    }
    let version_line = format!(" \\___/|_| |_|_| v{}", env!("CARGO_PKG_VERSION"));
    let _ = write_line(&mut repl_term, &version_line);
    for line in BANNER_INSTRUCTIONS {
        let _ = write_line(&mut repl_term, core::str::from_utf8(line).unwrap());
    }

    loop {
        // Print prompt
        if write_str(&mut repl_term, "uni> ").is_err() {
            break;
        }

        // Read line
        match editor.read_line(&mut repl_term) {
            Ok(line) => {
                let line = line.trim();

                // Check for empty line - all other inputs are executed as Uni code
                if line.is_empty() {
                    // Empty line, just continue
                    continue;
                }

                // Execute the line as Uni code (includes primitives like quit, stack, clear, words)
                if !execute_repl_line(&mut repl_term, line, interp) {
                    break; // quit was called
                }
            }
            Err(e) => {
                match e {
                    editline::Error::Eof => {
                        let _ = write_line(&mut repl_term, "\nGoodbye!");
                        break;
                    }
                    editline::Error::Interrupted => {
                        let _ = write_line(&mut repl_term, "\nInterrupted. Use 'quit' or Ctrl-D to exit.");
                        continue;
                    }
                    _ => {
                        let _ = write_line(&mut repl_term, "\nGoodbye!");
                        break;
                    }
                }
            }
        }
    }
}

// Helper to write a string without newline
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
fn write_str<T: Terminal>(terminal: &mut T, s: &str) -> editline::Result<()> {
    terminal.write(s.as_bytes())?;
    terminal.flush()
}

// Helper to write a line with platform-appropriate line ending
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
fn write_line<T: Terminal>(terminal: &mut T, s: &str) -> editline::Result<()> {
    terminal.write(s.as_bytes())?;
    terminal.write(LINE_ENDING)?;
    terminal.flush()
}

// Generic helper for REPL line execution
// Returns true if REPL should continue, false if it should exit
#[cfg(all(target_os = "none", not(feature = "target-stm32h753zi")))]
fn execute_repl_line<T: Terminal>(terminal: &mut T, line: &str, interp: &mut Interpreter) -> bool {
    match execute_string(line, interp) {
        Ok(()) => {
            let _ = write_line(terminal, "");  // Blank line before stack display
            if !interp.stack.is_empty()
                && let Some(top) = interp.stack.last() {
                    let msg = format!(" => {} : {}", top, top.type_name());
                    let _ = write_line(terminal, &msg);
                }
            true // Continue REPL
        }
        Err(RuntimeError::QuitRequested) => {
            let _ = write_line(terminal, "Goodbye!");
            false // Exit REPL
        }
        Err(e) => {
            let _ = write_line(terminal, "");  // Blank line before error message
            let msg = format!("Error: {:?}", e);
            let _ = write_line(terminal, &msg);
            true // Continue REPL after error
        }
    }
}

// Micro:bit REPL function
#[cfg(feature = "target-microbit")]
fn run_repl() -> ! {
    use editline::terminals::microbit::{Baudrate, Parity, Uarte, UarteTerminal};
    use microbit::display::nonblocking::Display;

    let board = editline::terminals::microbit::Board::take().unwrap();

    // Extract peripherals we need BEFORE creating terminal
    // This way we keep access to buttons, display, etc.
    let buttons = board.buttons;
    let display_pins = board.display_pins;
    let timer1 = board.TIMER1;

    // Initialize the LED display with TIMER1
    let display = Display::new(timer1, display_pins);

    // Store display in global static for interrupt handler (from uni-core)
    cortex_m::interrupt::free(|cs| {
        *uni_core::hardware::microbit::DISPLAY.borrow(cs).borrow_mut() = Some(display);
    });

    // Enable TIMER1 interrupt
    unsafe {
        microbit::pac::NVIC::unmask(microbit::pac::Interrupt::TIMER1);
    }

    // Manually create UART terminal (instead of using from_board)
    // This is what from_board() does internally
    let serial = Uarte::new(
        board.UARTE0,
        board.uart.into(),
        Parity::EXCLUDED,
        Baudrate::BAUD115200,
    );
    let terminal = UarteTerminal::new(serial);

    let mut editor = LineEditor::new(1024, 20);
    let mut interp = Interpreter::new();

    // Give the interpreter access to hardware peripherals
    interp.buttons = Some(buttons);
    // Note: display is in the global static, not in interpreter

    // Run the shared REPL loop
    run_repl_loop(&mut editor, terminal, &mut interp);

    // REPL exited, enter infinite loop (embedded requirement)
    loop {}
}

// Raspberry Pi Pico W specific imports and setup
#[cfg(feature = "target-pico")]
use rp2040_hal::{
    clocks::init_clocks_and_plls,
    pac,
    usb::UsbBus,
    watchdog::Watchdog,
};

// Raspberry Pi Pico 2 specific imports and setup
#[cfg(feature = "target-pico2")]
use rp235x_hal::{
    clocks::init_clocks_and_plls,
    pac,
    usb::UsbBus,
    watchdog::Watchdog,
};

#[cfg(any(feature = "target-pico", feature = "target-pico2"))]
use usb_device::{
    prelude::*,
    class_prelude::UsbBusAllocator,
};

#[cfg(any(feature = "target-pico", feature = "target-pico2"))]
use usbd_serial::SerialPort;

#[cfg(feature = "target-pico")]
use editline::terminals::rp_pico_usb::UsbCdcTerminal;

#[cfg(feature = "target-pico2")]
use editline::terminals::rp_pico2_usb::UsbCdcTerminal;

// Link boot stage 2 for Pico
#[cfg(feature = "target-pico")]
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// Tell the Boot ROM about our application (RP2350 requires this)
#[cfg(feature = "target-pico2")]
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: rp235x_hal::block::ImageDef = rp235x_hal::block::ImageDef::secure_exe();

// External high-speed crystal on the Pico board is 12MHz
#[cfg(feature = "target-pico")]
const XOSC_CRYSTAL_FREQ: u32 = 12_000_000u32;

// External high-speed crystal on the Pico 2 board is 12MHz
#[cfg(feature = "target-pico2")]
const XOSC_CRYSTAL_FREQ: u32 = 12_000_000u32;

// USB bus allocator (needs static lifetime)
#[cfg(any(feature = "target-pico", feature = "target-pico2"))]
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// Raspberry Pi Pico W main function
#[cfg(feature = "target-pico")]
#[cortex_m_rt::entry]
fn pico_main() -> ! {
    use core::ptr::addr_of_mut;

    // Initialize allocator - RP2040 has 264KB SRAM
    // Use ~127.5KB for heap, leaving room for stack and BSS
    const HEAP_SIZE: usize = 130560; // ~127.5 * 1024
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    #[global_allocator]
    static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

    unsafe { ALLOCATOR.init(&raw mut HEAP as *const u8 as usize, HEAP_SIZE) }

    // Grab singleton objects
    let mut pac_peripherals = pac::Peripherals::take().unwrap();
    let _core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver
    let mut watchdog = Watchdog::new(pac_peripherals.WATCHDOG);

    // Configure the clocks
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac_peripherals.XOSC,
        pac_peripherals.CLOCKS,
        pac_peripherals.PLL_SYS,
        pac_peripherals.PLL_USB,
        &mut pac_peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac_peripherals.USBCTRL_REGS,
        pac_peripherals.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac_peripherals.RESETS,
    ));
    unsafe {
        USB_BUS = Some(usb_bus);
    }

    let usb_bus_ref = unsafe { (*addr_of_mut!(USB_BUS)).as_ref().unwrap() };

    // Set up the USB Communications Class Device driver
    let serial = SerialPort::new(usb_bus_ref);

    // Create a USB device with a fake VID and PID
    let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::new(LangID::EN)
            .manufacturer("Raspberry Pi")
            .product("Uni REPL")
            .serial_number("UNI")])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    // Create terminal and run REPL
    let terminal = UsbCdcTerminal::new(usb_dev, serial);
    run_repl(terminal)
}

// Raspberry Pi Pico 2 main function
#[cfg(feature = "target-pico2")]
#[rp235x_hal::entry]
fn pico2_main() -> ! {
    use core::ptr::addr_of_mut;

    // Initialize allocator - RP2350 has 520KB SRAM
    // Use ~255KB for heap, leaving room for stack and BSS
    const HEAP_SIZE: usize = 261120; // ~255 * 1024
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    #[global_allocator]
    static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

    unsafe { ALLOCATOR.init(&raw mut HEAP as *const u8 as usize, HEAP_SIZE) }

    // Grab singleton objects
    let mut pac_peripherals = pac::Peripherals::take().unwrap();
    let _core = cortex_m::Peripherals::take().unwrap();

    // Set up the watchdog driver
    let mut watchdog = Watchdog::new(pac_peripherals.WATCHDOG);

    // Configure the clocks
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac_peripherals.XOSC,
        pac_peripherals.CLOCKS,
        pac_peripherals.PLL_SYS,
        pac_peripherals.PLL_USB,
        &mut pac_peripherals.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Set up timer for delays
    let timer = rp235x_hal::Timer::new_timer0(pac_peripherals.TIMER0, &mut pac_peripherals.RESETS, &clocks);

    // Set up the USB driver (RP2350 uses pac.USB instead of pac.USBCTRL_REGS)
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac_peripherals.USB,
        pac_peripherals.USB_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac_peripherals.RESETS,
    ));
    unsafe {
        USB_BUS = Some(usb_bus);
    }

    let usb_bus_ref = unsafe { (*addr_of_mut!(USB_BUS)).as_ref().unwrap() };

    // Set up the USB Communications Class Device driver
    let serial = SerialPort::new(usb_bus_ref);

    // Create a USB device with a fake VID and PID
    let usb_dev = UsbDeviceBuilder::new(usb_bus_ref, UsbVidPid(0x16c0, 0x27dd))
        .strings(&[StringDescriptors::new(LangID::EN)
            .manufacturer("Raspberry Pi")
            .product("Uni REPL")
            .serial_number("UNI")])
        .unwrap()
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    // Create terminal and run REPL
    let terminal = UsbCdcTerminal::new(usb_dev, serial);
    run_repl(terminal, timer)
}

// Raspberry Pi Pico W REPL function
#[cfg(feature = "target-pico")]
fn run_repl(mut terminal: UsbCdcTerminal<'static, UsbBus>) -> ! {
    let mut editor = LineEditor::new(1024, 20);
    let mut interp = Interpreter::new();

    // Wait for first byte from terminal (connection signal)
    let _ = terminal.read_byte();

    // Run the shared REPL loop
    run_repl_loop(&mut editor, terminal, &mut interp);

    // REPL exited, enter infinite loop (embedded requirement)
    loop {
        cortex_m::asm::wfi();
    }
}

// Raspberry Pi Pico 2 REPL function
#[cfg(feature = "target-pico2")]
fn run_repl<T: rp235x_hal::timer::TimerDevice>(mut terminal: UsbCdcTerminal<'static, UsbBus>, mut timer: rp235x_hal::Timer<T>) -> ! {
    let mut editor = LineEditor::new(1024, 20);
    let mut interp = Interpreter::new();

    // Wait for terminal connection (DTR signal from picocom/minicom)
    terminal.wait_for_connection(&mut timer);

    // Run the shared REPL loop
    run_repl_loop(&mut editor, terminal, &mut interp);

    // REPL exited, enter infinite loop (embedded requirement)
    loop {
        cortex_m::asm::wfi();
    }
}

// STM32H753ZI specific imports
#[cfg(feature = "target-stm32h753zi")]
use embassy_executor::Spawner;
#[cfg(feature = "target-stm32h753zi")]
use embassy_futures::join::join;
#[cfg(feature = "target-stm32h753zi")]
use embassy_stm32::gpio::{Level, Output, Speed};
#[cfg(feature = "target-stm32h753zi")]
use embassy_stm32::usb::Driver;
#[cfg(feature = "target-stm32h753zi")]
use embassy_stm32::{bind_interrupts, peripherals, usb, Config};
#[cfg(feature = "target-stm32h753zi")]
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
#[cfg(feature = "target-stm32h753zi")]
use embassy_usb::Builder;
#[cfg(feature = "target-stm32h753zi")]
use editline::{AsyncLineEditor, AsyncTerminal, terminals::EmbassyUsbTerminal};
#[cfg(feature = "target-stm32h753zi")]
use {defmt_rtt as _, panic_probe as _};

// Buffering output for async STM32 target
// Uses Rc<RefCell<>> so we can share it between the interpreter and the REPL loop
#[cfg(feature = "target-stm32h753zi")]
use core::cell::RefCell;
#[cfg(feature = "target-stm32h753zi")]
use alloc::rc::Rc;

#[cfg(feature = "target-stm32h753zi")]
struct BufferedOutput {
    buffer: Rc<RefCell<Vec<u8>>>,
}

#[cfg(feature = "target-stm32h753zi")]
impl BufferedOutput {
    fn new(buffer: Rc<RefCell<Vec<u8>>>) -> Self {
        Self { buffer }
    }
}

#[cfg(feature = "target-stm32h753zi")]
impl uni_core::Output for BufferedOutput {
    fn write(&mut self, data: &[u8]) -> Result<(), ()> {
        self.buffer.borrow_mut().extend_from_slice(data);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), ()> {
        // Buffered, so flush is a no-op (we'll flush to terminal after execution)
        Ok(())
    }
}

#[cfg(feature = "target-stm32h753zi")]
bind_interrupts!(struct Irqs {
    OTG_FS => usb::InterruptHandler<peripherals::USB_OTG_FS>;
});

#[cfg(feature = "target-stm32h753zi")]
defmt::timestamp!("{=u64:us}", {
    embassy_time::Instant::now().as_micros()
});

// STM32H753ZI main function
#[cfg(feature = "target-stm32h753zi")]
#[embassy_executor::main]
async fn stm32_main(_spawner: Spawner) {
    // Initialize the allocator
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 262144; // 256KB heap (conservative for now)
        static mut HEAP: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];

        #[global_allocator]
        static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

        #[allow(static_mut_refs)]
        unsafe { ALLOCATOR.init(HEAP.as_ptr() as usize, HEAP_SIZE) }
    }

    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
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

    defmt::info!("STM32H753ZI Uni REPL");

    // Create USB driver
    let mut usb_config = usb::Config::default();
    usb_config.vbus_detection = false;

    let mut ep_out_buffer = [0u8; 256];
    let driver = Driver::new_fs(p.USB_OTG_FS, Irqs, p.PA12, p.PA11, &mut ep_out_buffer, usb_config);

    // Create USB device config
    let mut config_descriptor = [0u8; 256];
    let mut bos_descriptor = [0u8; 256];
    let mut control_buf = [0u8; 64];

    let mut usb_config = embassy_usb::Config::new(0xc0de, 0xcafe);
    usb_config.manufacturer = Some("Uni");
    usb_config.product = Some("STM32H753 Uni REPL");
    usb_config.serial_number = Some("12345678");
    usb_config.max_power = 100;
    usb_config.max_packet_size_0 = 64;

    // Create CDC ACM state
    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut [],
        &mut control_buf,
    );

    // Create CDC ACM class
    let class = CdcAcmClass::new(&mut builder, &mut state, 64);

    // Build USB device
    let mut usb = builder.build();

    defmt::info!("USB device initialized");

    // Turn on green LED to indicate we're ready
    let mut _led = Output::new(p.PB0, Level::High, Speed::Low);

    // Run USB device and REPL concurrently
    let usb_fut = usb.run();

    let repl_fut = async {
        // Create terminal and editor
        let mut terminal = EmbassyUsbTerminal::new(class);
        let mut editor = AsyncLineEditor::new(1024, 20);
        let mut interp = Interpreter::new();

        // Create shared output buffer for the interpreter
        let output_buffer = Rc::new(RefCell::new(Vec::new()));
        interp.set_output(Box::new(BufferedOutput::new(Rc::clone(&output_buffer))));

        defmt::info!("Waiting for terminal connection (DTR)...");
        terminal.wait_connection().await;
        defmt::info!("Terminal connected!");

        // Send banner
        for line in BANNER_LINES {
            let _ = terminal.write(line).await;
            let _ = terminal.write(b"\r\n").await;
        }
        let version_line = format!(" \\___/|_| |_|_| v{}\r\n", env!("CARGO_PKG_VERSION"));
        let _ = terminal.write(version_line.as_bytes()).await;
        for line in BANNER_INSTRUCTIONS {
            let _ = terminal.write(line).await;
            let _ = terminal.write(b"\r\n").await;
        }
        let _ = terminal.flush().await;

        loop {
            // Show prompt
            let _ = terminal.write(b"uni> ").await;
            let _ = terminal.flush().await;

            // Read line with full editing support
            match editor.read_line(&mut terminal).await {
                Ok(line) => {
                    let line = line.trim();
                    defmt::info!("Got command: {}", line);

                    if line.is_empty() {
                        continue;
                    }

                    // Execute the line as Uni code
                    match execute_string(line, &mut interp) {
                        Ok(()) => {
                            // Get and clear the buffered output
                            let output_bytes = {
                                let mut buf = output_buffer.borrow_mut();
                                let bytes = buf.clone();
                                buf.clear();
                                bytes
                            };

                            // Write any buffered output first
                            if !output_bytes.is_empty() {
                                let _ = terminal.write(&output_bytes).await;
                            }

                            // Then show stack top if non-empty
                            let _ = terminal.write(b"\r\n").await;
                            if !interp.stack.is_empty() {
                                if let Some(top) = interp.stack.last() {
                                    let msg = format!(" => {} : {}\r\n", top, top.type_name());
                                    let _ = terminal.write(msg.as_bytes()).await;
                                }
                            }
                        }
                        Err(RuntimeError::QuitRequested) => {
                            let _ = terminal.write(b"Goodbye!\r\n").await;
                            break;
                        }
                        Err(e) => {
                            let _ = terminal.write(b"\r\n").await;
                            let msg = format!("Error: {:?}\r\n", e);
                            let _ = terminal.write(msg.as_bytes()).await;
                        }
                    }
                    let _ = terminal.flush().await;
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

    join(usb_fut, repl_fut).await;
}
