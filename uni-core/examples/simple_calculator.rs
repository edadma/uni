//! Simple calculator using uni-core as a library
//!
//! This example shows how to embed the Uni interpreter in your own application
//! to evaluate mathematical expressions.

use uni_core::{Interpreter, execute_string};

fn main() {
    // Create a new interpreter instance
    let mut interp = Interpreter::new();

    // Define some calculations to perform
    let expressions = vec![
        ("5 3 +", "Addition"),
        ("10 4 -", "Subtraction"),
        ("7 6 *", "Multiplication"),
        ("20 4 /", "Division"),
        ("3 dup *", "3 squared (3 * 3)"),
        ("[1 2 3 4 5] length", "List length"),
    ];

    println!("Uni Calculator Example\n");

    for (expr, description) in expressions {
        // Execute the expression
        match execute_string(expr, &mut interp) {
            Ok(()) => {
                // Get the result from the stack
                if let Some(result) = interp.stack.last() {
                    println!("{}: {} = {}", description, expr, result);
                }
            }
            Err(e) => {
                eprintln!("Error evaluating '{}': {:?}", expr, e);
            }
        }

        // Clear the stack for next calculation
        interp.stack.clear();
    }

    println!("\nDefining and using a custom function:");

    // Define a square function and use it
    match execute_string("'square [dup *] def  5 square", &mut interp) {
        Ok(()) => {
            if let Some(result) = interp.stack.last() {
                println!("5 squared = {}", result);
            }
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}
