#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

#[cfg(target_os = "none")]
extern crate alloc;

mod compat;
mod builtins;
mod evaluator;
mod hardware; // NEW: Hardware abstraction for different targets
mod integration_tests;
mod interpreter;
mod parser;
mod prelude;
mod primitives; // RUST CONCEPT: New modular primitives organization
mod tokenizer;
mod value;

use compat::{format, Rc, Box};
use interpreter::Interpreter;
use editline::{LineEditor, Terminal};

#[cfg(not(target_os = "none"))]
use value::RuntimeError;

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// Wrapper to share a terminal reference with the interpreter
// In C this would just be a pointer, but Rust requires Rc<RefCell<>> for shared mutable access
struct SharedTerminal<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for SharedTerminal<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Terminal> Terminal for SharedTerminal<T> {
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

#[cfg(target_os = "none")]
use cortex_m_rt::entry;
#[cfg(feature = "target-microbit")]
use microbit::pac::interrupt;
#[cfg(target_os = "none")]
use panic_halt as _;
#[cfg(target_os = "none")]
use alloc_cortex_m::CortexMHeap;

// Global display for interrupt handler (micro:bit only)
#[cfg(feature = "target-microbit")]
use cortex_m::interrupt::Mutex;
#[cfg(feature = "target-microbit")]
pub static DISPLAY: Mutex<RefCell<Option<microbit::display::nonblocking::Display<microbit::pac::TIMER1>>>> =
    Mutex::new(RefCell::new(None));

// Timer interrupt handler for LED display
#[cfg(feature = "target-microbit")]
#[microbit::pac::interrupt]
fn TIMER1() {
    cortex_m::interrupt::free(|cs| {
        if let Some(display) = DISPLAY.borrow(cs).borrow_mut().as_mut() {
            display.handle_display_event();
        }
    });
}

// Platform-specific line endings
#[cfg(not(target_os = "none"))]
const LINE_ENDING: &[u8] = b"\n";
#[cfg(target_os = "none")]
const LINE_ENDING: &[u8] = b"\r\n";

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
    use evaluator::execute_string;
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

    use evaluator::execute_string;
    use primitives::print_builtin;

    match execute_string(code, &mut interp) {
        Ok(()) => {
            // Success - code executed without errors
            if auto_print {
                // RUST CONCEPT: Conditional execution
                // For -e flag, automatically print the top stack value
                match print_builtin(&mut interp) {
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
#[entry]
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
    // RUST CONCEPT: Result type for error handling in Rust
    // editline provides line editing functionality with history
    // Create a LineEditor with 4KB buffer and 100 history entries
    let mut editor = LineEditor::new(4096, 100);
    let terminal = StdioTerminal::new();

    // RUST CONCEPT: Automatic initialization
    // Interpreter::new() automatically loads builtins and stdlib
    let mut interp = Interpreter::new();

    run_repl_loop(&mut editor, terminal, &mut interp);
}

// Generic REPL loop that works with any Terminal implementation
fn run_repl_loop<T: Terminal + 'static>(editor: &mut LineEditor, terminal: T, interp: &mut Interpreter) {
    // Wrap terminal in Rc<RefCell<>> for shared access
    let shared = Rc::new(RefCell::new(terminal));

    // Give a clone to the interpreter so primitives like '.' can write output
    let term_for_interp = SharedTerminal {
        inner: shared.clone(),
    };
    interp.set_terminal(Box::new(term_for_interp));

    // Create our own SharedTerminal for REPL operations
    let mut repl_term = SharedTerminal {
        inner: shared,
    };

    // Print banner with blank line first
    let _ = write_line(&mut repl_term, "");
    let _ = write_line(&mut repl_term, " _   _       _ ");
    let _ = write_line(&mut repl_term, "| | | |_ __ (_)");
    let _ = write_line(&mut repl_term, "| | | | '_ \\| |");
    let _ = write_line(&mut repl_term, "| |_| | | | | |");
    let _ = write_line(&mut repl_term, " \\___/|_| |_|_| v0.0.1");
    let _ = write_line(&mut repl_term, "");
    let _ = write_line(&mut repl_term, "Type 'quit' or press Ctrl-D to exit");
    let _ = write_line(&mut repl_term, "Type 'stack' to see the current stack");
    let _ = write_line(&mut repl_term, "Type 'clear' to clear the stack");
    let _ = write_line(&mut repl_term, "Type 'words' to see defined words");
    let _ = write_line(&mut repl_term, "");

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
fn write_str<T: Terminal>(terminal: &mut T, s: &str) -> editline::Result<()> {
    terminal.write(s.as_bytes())?;
    terminal.flush()
}

// Helper to write a line with platform-appropriate line ending
fn write_line<T: Terminal>(terminal: &mut T, s: &str) -> editline::Result<()> {
    terminal.write(s.as_bytes())?;
    terminal.write(LINE_ENDING)?;
    terminal.flush()
}

// Generic helper for REPL line execution
// Returns true if REPL should continue, false if it should exit
fn execute_repl_line<T: Terminal>(terminal: &mut T, line: &str, interp: &mut Interpreter) -> bool {
    use evaluator::execute_string;
    use value::RuntimeError;

    match execute_string(line, interp) {
        Ok(()) => {
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

    // Store display in global static for interrupt handler
    cortex_m::interrupt::free(|cs| {
        *DISPLAY.borrow(cs).borrow_mut() = Some(display);
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

#[cfg(feature = "target-pico")]
use usb_device::{
    prelude::*,
    class_prelude::UsbBusAllocator,
};

#[cfg(feature = "target-pico")]
use usbd_serial::SerialPort;

#[cfg(feature = "target-pico")]
use editline::terminals::rp_pico_usb::UsbCdcTerminal;

// Link boot stage 2 for Pico
#[cfg(feature = "target-pico")]
#[unsafe(link_section = ".boot2")]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_GENERIC_03H;

// External high-speed crystal on the Pico board is 12MHz
#[cfg(feature = "target-pico")]
const XOSC_CRYSTAL_FREQ: u32 = 12_000_000u32;

// USB bus allocator (needs static lifetime)
#[cfg(feature = "target-pico")]
static mut USB_BUS: Option<UsbBusAllocator<UsbBus>> = None;

// Raspberry Pi Pico W main function
#[cfg(feature = "target-pico")]
#[entry]
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
