mod builtins;
mod evaluator;
mod integration_tests;
mod interpreter;
mod parser;
mod prelude;
mod primitives; // RUST CONCEPT: New modular primitives organization
mod tokenizer;
mod value;

use interpreter::Interpreter;
use editline::{LineEditor, terminals::StdioTerminal};
use std::env;
use std::io::Write;
use value::RuntimeError;

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
            // The file should use 'pr' if it wants to print output
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

// RUST CONCEPT: Function extraction for code organization
// Run the interactive REPL (Read-Eval-Print Loop)
fn run_repl() {
    println!(" _   _       _ ");
    println!("| | | |_ __ (_)");
    println!("| | | | '_ \\| |");
    println!("| |_| | | | | |");
    println!(" \\___/|_| |_|_| v0.0.1");
    println!();
    println!("Type 'quit' or press Ctrl-D to exit");
    println!("Type 'stack' to see the current stack");
    println!("Type 'clear' to clear the stack");
    println!("Type 'words' to see defined words\n");

    // RUST CONCEPT: Result type for error handling in Rust
    // editline provides line editing functionality with history
    // Create a LineEditor with 4KB buffer and 100 history entries
    let mut editor = LineEditor::new(4096, 100);
    let mut terminal = StdioTerminal::new();

    // RUST CONCEPT: Automatic initialization
    // Interpreter::new() automatically loads builtins and stdlib
    let mut interp = Interpreter::new();

    // RUST CONCEPT: Infinite loop with break
    loop {
        // RUST CONCEPT: Printing prompt and flushing stdout
        // editline requires manual prompt printing and flushing
        print!("uni> ");
        if let Err(e) = std::io::stdout().flush() {
            eprintln!("Error flushing stdout: {}", e);
            break;
        }

        // RUST CONCEPT: Match expression for comprehensive error handling
        match editor.read_line(&mut terminal) {
            Ok(line) => {
                // RUST CONCEPT: Detecting EOF
                // editline may return an empty line on EOF (Ctrl-D)
                // Check if the line is empty AND if we're at EOF
                if line.is_empty() {
                    // Empty line could be just Enter key or Ctrl-D
                    // We'll treat completely empty (no whitespace) as potential EOF
                    // But for now, just continue - user can type 'quit'
                }

                // RUST CONCEPT: String trimming to remove whitespace
                let line = line.trim();

                // Check for special REPL commands
                match line {
                    "quit" => {
                        println!("Goodbye!");
                        break;
                    }
                    "stack" => {
                        // Display the current stack
                        print_stack(&interp);
                    }
                    "clear" => {
                        // Clear the stack
                        interp.stack.clear();
                        println!("Stack cleared");
                    }
                    "words" => {
                        // Display all defined words
                        print_words(&interp);
                    }
                    "" => {
                        // Empty line, just continue
                    }
                    _ => {
                        // Execute the line as Uni code
                        execute_repl_line(line, &mut interp);
                    }
                }
            }
            Err(e) => {
                // RUST CONCEPT: Handling EOF (Ctrl-D) or read errors
                // editline returns different error kinds for different conditions
                match e.kind() {
                    std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::Other => {
                        // EOF (Ctrl-D) - exit gracefully
                        // editline might return Other for EOF
                        println!("\nGoodbye!");
                        break;
                    }
                    std::io::ErrorKind::Interrupted => {
                        // Ctrl-C was pressed - ask user to use quit or Ctrl-D
                        println!("\nInterrupted. Use 'quit' or Ctrl-D to exit.");
                        continue;
                    }
                    _ => {
                        // Other errors - could be EOF, so exit gracefully
                        println!("\nGoodbye!");
                        break;
                    }
                }
            }
        }
    }
}

// RUST CONCEPT: Helper function for REPL line execution
fn execute_repl_line(line: &str, interp: &mut Interpreter) {
    use evaluator::execute_string;

    // RUST CONCEPT: Match for error handling
    match execute_string(line, interp) {
        Ok(()) => {
            // Execution succeeded, show top of stack if non-empty
            if !interp.stack.is_empty() {
                // RUST CONCEPT: Getting the last element without removing it
                if let Some(top) = interp.stack.last() {
                    println!(" => {} : {}", top, top.type_name());
                }
            }
        }
        Err(e) => {
            // RUST CONCEPT: Error formatting with Display trait
            eprintln!("Error: {:?}", e);
        }
    }
}

// RUST CONCEPT: Helper function to display the stack
fn print_stack(interp: &Interpreter) {
    if interp.stack.is_empty() {
        println!("Stack is empty");
    } else {
        println!("Stack ({} items):", interp.stack.len());
        // RUST CONCEPT: Iterating in reverse to show top first
        for (i, value) in interp.stack.iter().rev().enumerate() {
            if i >= 10 {
                println!("  ... and {} more", interp.stack.len() - 10);
                break;
            }
            println!("  {}: {}", i, value);
        }
    }
}

// RUST CONCEPT: Helper function to display defined words
fn print_words(interp: &Interpreter) {
    // RUST CONCEPT: Collecting and sorting for display
    let mut words: Vec<_> = interp.dictionary.keys().map(|k| k.as_ref()).collect();

    // Add special forms that are handled in the evaluator
    words.push("exec");
    words.push("if");

    words.sort();

    println!("Defined words ({}):", words.len());
    // RUST CONCEPT: Chunking for columnar display
    for chunk in words.chunks(5) {
        for word in chunk {
            print!("{:15} ", word);
        }
        println!();
    }
}
