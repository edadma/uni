mod value;
mod interpreter;
mod builtins;
mod primitives;  // RUST CONCEPT: New modular primitives organization
mod tokenizer;
mod parser;
mod evaluator;
mod stdlib;

use interpreter::Interpreter;
use value::{Value, RuntimeError};
use std::env;

fn main() {
    // RUST CONCEPT: Command-line argument parsing with std::env::args()
    // args() returns an iterator over command-line arguments
    let args: Vec<String> = env::args().collect();

    // RUST CONCEPT: Pattern matching on argument count and content
    match args.len() {
        // No arguments - run demo
        1 => run_demo(),

        // One argument - execute file or show help
        2 => {
            if args[1].starts_with('-') {
                // Treat flags without arguments as help request
                eprintln!("Usage:");
                eprintln!("  {} [file.uni]           # Execute Uni file", args[0]);
                eprintln!("  {} -f [file.uni]        # Execute Uni file (explicit)", args[0]);
                eprintln!("  {} -c \"code\"            # Execute code string", args[0]);
                eprintln!("  {} -e \"code\"            # Execute code and print result", args[0]);
                eprintln!("  {}                      # Run interactive demo", args[0]);
                std::process::exit(1);
            } else {
                execute_file(&args[1]);
            }
        },

        // Two or more arguments - check for flags
        _ => {
            match args[1].as_str() {
                "-c" => {
                    // Execute code without automatic printing
                    execute_code(&args[2], false);
                },
                "-e" => {
                    // Execute code with automatic printing of stack top
                    execute_code(&args[2], true);
                },
                "-f" => {
                    // Explicit file execution flag
                    execute_file(&args[2]);
                },
                _ => {
                    // Show usage and exit
                    eprintln!("Usage:");
                    eprintln!("  {} [file.uni]           # Execute Uni file", args[0]);
                    eprintln!("  {} -f [file.uni]        # Execute Uni file (explicit)", args[0]);
                    eprintln!("  {} -c \"code\"            # Execute code string", args[0]);
                    eprintln!("  {} -e \"code\"            # Execute code and print result", args[0]);
                    eprintln!("  {}                      # Run interactive demo", args[0]);
                    std::process::exit(1);
                }
            }
        }
    }
}

// RUST CONCEPT: File I/O and error handling
// Execute a Uni source file
fn execute_file(filename: &str) {
    use std::fs;
    use evaluator::execute_string;

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
        },
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
                    },
                    Err(RuntimeError::StackUnderflow) => {
                        // Empty stack is okay - just don't print anything
                    },
                    Err(e) => {
                        eprintln!("Error printing result: {:?}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Err(e) => {
            eprintln!("Error: {:?}", e);
            std::process::exit(1);
        }
    }
}

// RUST CONCEPT: Function extraction for code organization
// Run the interactive demo (original main() code)
fn run_demo() {
    println!("Uni interpreter starting...");

    // RUST CONCEPT: Automatic initialization
    // Interpreter::new() now automatically loads builtins and stdlib
    let mut interp = Interpreter::new();

    interp.push(Value::Number(42.0));
    println!("Pushed number: 42.0");

    let hello_atom = interp.intern_atom("hello");
    interp.push(Value::Atom(hello_atom));
    println!("Pushed atom: hello");

    let list = interp.make_list(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0)
    ]);
    interp.push(list);
    println!("Pushed list: [1 2 3] (as cons cells)");

    println!("Defined builtin: +");

    interp.push(Value::Number(5.0));
    interp.push(Value::Number(3.0));

    let plus_atom = interp.intern_atom("+");
    if let Some(entry) = interp.dictionary.get(&plus_atom) {
        if let Value::Builtin(func) = &entry.value {
            match func(&mut interp) {
                Ok(()) => println!("Successfully called + builtin"),
                Err(e) => println!("Error calling +: {:?}", e),
            }
        }
    }

    match interp.pop() {
        Ok(Value::Number(n)) => println!("Result: {}", n),
        Ok(other) => println!("Got non-number: {:?}", other),
        Err(e) => println!("Error: {:?}", e),
    }

    match interp.pop() {
        Ok(val) => println!("Unexpected value: {:?}", val),
        Err(RuntimeError::StackUnderflow) => println!("Caught stack underflow as expected"),
        Err(e) => println!("Unexpected error: {:?}", e),
    }

    interp.push(Value::Nil);
    println!("Pushed empty list (Nil)");

    let not_number_atom = interp.intern_atom("not-a-number");
    interp.push(Value::Atom(not_number_atom));
    match interp.pop_number() {
        Ok(n) => println!("Unexpected number: {}", n),
        Err(RuntimeError::TypeError(msg)) => println!("Caught type error: {}", msg),
        Err(e) => println!("Unexpected error: {:?}", e),
    }

    // Test string handling
    use std::rc::Rc;
    let string_val: Rc<str> = "Hello, Uni!".into();
    interp.push(Value::String(string_val));
    println!("Pushed string: \"Hello, Uni!\"");

    match interp.pop_string() {
        Ok(s) => println!("Retrieved string: \"{}\"", s),
        Err(e) => println!("Error retrieving string: {:?}", e),
    }

    // Test boolean handling
    interp.push(Value::Boolean(true));
    println!("Pushed boolean: true");

    match interp.pop_boolean() {
        Ok(b) => println!("Retrieved boolean: {}", b),
        Err(e) => println!("Error retrieving boolean: {:?}", e),
    }

    interp.push(Value::Boolean(false));
    println!("Pushed boolean: false");

    // Test null handling
    interp.push(Value::Null);
    println!("Pushed null value");

    match interp.pop() {
        Ok(Value::Null) => println!("Retrieved null value"),
        Ok(other) => println!("Got unexpected value: {:?}", other),
        Err(e) => println!("Error retrieving value: {:?}", e),
    }

    // Test parser functionality
    use parser::parse;
    println!("\n--- Parser Demo ---");

    // Parse some Uni code
    let code = "[1 2 +] 'hello \"world\" [a . b]";
    println!("Parsing: {}", code);

    match parse(code, &mut interp) {
        Ok(values) => {
            println!("Parsed {} values:", values.len());
            for (i, value) in values.iter().enumerate() {
                println!("  {}: {:?}", i, value);
            }
        },
        Err(e) => println!("Parse error: {:?}", e),
    }

    // Test execution functionality
    use evaluator::execute_string;
    println!("\n--- Execution Demo ---");

    // Demo 1: Execute simple arithmetic
    println!("Executing: 5 3 +");
    match execute_string("5 3 +", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(Value::Number(n)) => println!("Result: {}", n),
                Ok(other) => println!("Got non-number: {:?}", other),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Execution error: {:?}", e),
    }

    // Demo 2: Execute quoted atoms (should just push the atom)
    println!("\nExecuting: 'hello");
    match execute_string("'hello", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(Value::Atom(atom)) => println!("Result: atom '{}'", atom),
                Ok(other) => println!("Got unexpected: {:?}", other),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Execution error: {:?}", e),
    }

    // Demo 3: Execute list as data, then eval it
    println!("\nExecuting: [10 2 /] eval");
    match execute_string("[10 2 /] eval", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(Value::Number(n)) => println!("Result: {}", n),
                Ok(other) => println!("Got non-number: {:?}", other),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Execution error: {:?}", e),
    }

    // Demo 4: Define and use constants with val
    println!("\nDefining constant with val: 'pi 3.14159 val");
    match execute_string("'pi 3.14159 val", &mut interp) {
        Ok(()) => println!("Defined pi as constant"),
        Err(e) => println!("Error defining pi: {:?}", e),
    }

    // Use the constant - it executes by pushing its value
    println!("Using constant: pi");
    match execute_string("pi", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(Value::Number(n)) => println!("pi = {}", n),
                Ok(other) => println!("Got unexpected: {:?}", other),
                Err(e) => println!("Error popping pi: {:?}", e),
            }
        },
        Err(e) => println!("Error executing pi: {:?}", e),
    }

    // Demo 5: Define procedures with def
    println!("\nDefining procedure with def: 'square [dup *] def");
    match execute_string("'square [dup *] def", &mut interp) {
        Ok(()) => println!("Defined square procedure"),
        Err(e) => println!("Error defining square: {:?}", e),
    }

    // Use the procedure - lists are data by default, so we need eval
    println!("Using procedure: 7 square eval");
    match execute_string("7 square eval", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(Value::Number(n)) => println!("7 squared = {}", n),
                Ok(other) => println!("Got unexpected: {:?}", other),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Error executing square: {:?}", e),
    }

    // Demo 6: Show the difference - square pushes the list, eval executes it
    println!("\nJust calling square (without eval): 9 square");
    match execute_string("9 square", &mut interp) {
        Ok(()) => {
            // Should have 9 and the list [dup *] on stack
            if let Ok(list_val) = interp.pop() {
                if let Ok(num_val) = interp.pop() {
                    println!("Got number: {:?} and procedure: {:?}", num_val, list_val);
                } else {
                    println!("Got procedure: {:?}", list_val);
                }
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    // Demo 7: Show that def works for constants too (like Scheme's define)
    println!("\nUsing def for constants: 'answer 42 def");
    match execute_string("'answer 42 def", &mut interp) {
        Ok(()) => println!("Defined answer as constant with def"),
        Err(e) => println!("Error: {:?}", e),
    }

    println!("Using def-defined constant: answer");
    match execute_string("answer", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(Value::Number(n)) => println!("answer = {}", n),
                Ok(other) => println!("Got: {:?}", other),
                Err(e) => println!("Error: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    // Demo 8: List construction with cons
    println!("\n--- List Construction Demo ---");
    println!("Using cons primitive: 1 [2 3] cons");
    match execute_string("1 [2 3] cons", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(list) => println!("Result: {}", list),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    println!("Building list step by step: 'hello [] cons 42 swap cons");
    match execute_string("'hello [] cons 42 swap cons", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(list) => println!("Result: {}", list),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    // Demo 9: List construction with list builtin
    println!("\nUsing list primitive: 1 2 3 3 list");
    match execute_string("1 2 3 3 list", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(list) => println!("Result: {}", list),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    println!("Creating empty list: 0 list");
    match execute_string("0 list", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(list) => println!("Result: {}", list),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    println!("Mixed types in list: 'hello 42 true 3 list");
    match execute_string("'hello 42 true 3 list", &mut interp) {
        Ok(()) => {
            match interp.pop() {
                Ok(list) => println!("Result: {}", list),
                Err(e) => println!("Error popping result: {:?}", e),
            }
        },
        Err(e) => println!("Error: {:?}", e),
    }

    println!("\nUni interpreter with cons and list primitives demo complete!");
}