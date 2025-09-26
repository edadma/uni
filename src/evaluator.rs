// This module implements the AST-walking evaluator for Uni
//
// UNI EXECUTION MODEL (detailed):
// 1. Numbers, strings, lists: Push themselves onto the stack (they are data)
// 2. Atoms: Look up in dictionary and execute the definition
// 3. Quoted atoms: Already parsed as (quote atom), quote builtin handles them
// 4. Lists are data by default, use 'eval' builtin to execute them
//
// RUST LEARNING NOTES:
// - This demonstrates recursive function calls and pattern matching
// - We use Result<(), RuntimeError> to handle execution errors
// - Mutable references (&mut) allow us to modify the interpreter state
// - The ? operator propagates errors up the call stack automatically

use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;
use std::rc::Rc;

// RUST CONCEPT: Public functions that other modules can use
// This is the main entry point for executing Uni values
pub fn execute(value: &Value, interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Pattern matching on enum variants
    // Each Value type has different execution behavior
    match value {
        // RUST CONCEPT: Dereferencing and cloning
        // Numbers are Copy, so we can dereference and create a new Value
        Value::Number(n) => {
            interp.push(Value::Number(*n));
            Ok(())
        },

        // RUST CONCEPT: Cloning reference-counted data
        // Strings are Rc<str>, so .clone() just increments the reference count (cheap!)
        Value::String(s) => {
            interp.push(Value::String(s.clone()));
            Ok(())
        },

        // RUST CONCEPT: Boolean and Null values are data
        // They push themselves onto the stack like numbers and strings
        Value::Boolean(b) => {
            interp.push(Value::Boolean(*b));
            Ok(())
        },

        Value::Null => {
            interp.push(Value::Null);
            Ok(())
        },

        // RUST CONCEPT: Pattern matching with complex data
        // Lists (Pairs and Nil) are data - they push themselves onto the stack
        Value::Pair(_, _) | Value::Nil => {
            // RUST CONCEPT: Cloning entire Value structures
            // This clones the whole list structure, but since we use Rc everywhere,
            // it's just incrementing reference counts - no deep copying!
            interp.push(value.clone());
            Ok(())
        },

        // RUST CONCEPT: Dictionary lookups and execution
        // Atoms are the only things that execute by default
        Value::Atom(atom_name) => {
            execute_atom(atom_name, interp)
        },

        // RUST CONCEPT: Quoted atoms push themselves without executing
        // This is the key difference from regular atoms
        Value::QuotedAtom(atom_name) => {
            // Convert quoted atom back to regular atom and push it
            interp.push(Value::Atom(atom_name.clone()));
            Ok(())
        },

        // RUST CONCEPT: Function pointers and direct calls
        // Builtins are function pointers - call them directly
        Value::Builtin(func) => {
            func(interp)
        },
    }
}

// RUST CONCEPT: Helper function to execute a list as code
// This is used by execute_atom for executable definitions, eval, and if
pub fn execute_list(list: &Value, interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Converting list to vector of values
    let mut current = list.clone();
    let mut items = Vec::new();

    // RUST CONCEPT: Traversing cons cell lists
    loop {
        match current {
            Value::Pair(car, cdr) => {
                items.push((*car).clone());
                current = (*cdr).clone();
            },
            Value::Nil => break,
            _ => {
                // RUST CONCEPT: Single values can be "executed" too (for eval)
                // If it's not a proper list, just execute the single value
                return execute(&current, interp);
            }
        }
    }

    // RUST CONCEPT: Execute each element in sequence
    for item in items {
        execute(&item, interp)?;
    }

    Ok(())
}

// RUST CONCEPT: Helper functions for code organization
// This handles the specific logic for executing atoms (looking them up in dictionary)
fn execute_atom(atom_name: &Rc<str>, interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Special handling for eval and if
    // These are now special forms, not primitives
    if &**atom_name == "eval" {
        // Pop value from stack and execute it
        let value = interp.pop()?;
        // For lists, execute_list handles both proper lists and single values
        // For non-lists, execute handles them directly
        return execute_list(&value, interp);
    }

    if &**atom_name == "if" {
        // if ( condition true-branch false-branch -- ... )
        // Pop in reverse order: false-branch, true-branch, condition
        let false_branch = interp.pop()?;
        let true_branch = interp.pop()?;
        let condition = interp.pop()?;

        // Use the interpreter's is_truthy method for consistency
        let is_true = interp.is_truthy(&condition);
        let branch_to_execute = if is_true { true_branch } else { false_branch };

        // Execute the chosen branch
        return execute_list(&branch_to_execute, interp);
    }

    // RUST CONCEPT: HashMap lookups return Option<T>
    // We use match to handle both found and not-found cases
    match interp.dictionary.get(atom_name) {
        // RUST CONCEPT: Dictionary entries with metadata
        Some(entry) => {
            // RUST CONCEPT: Cloning to avoid borrow checker issues
            let entry_copy = entry.clone();

            // RUST CONCEPT: Execution behavior based on metadata
            if entry_copy.is_executable {
                // Entry was created with def - execute lists, others execute normally
                match &entry_copy.value {
                    Value::Pair(_, _) | Value::Nil => {
                        // Execute the list as code
                        execute_list(&entry_copy.value, interp)
                    },
                    _ => {
                        // Execute builtins, atoms, etc normally
                        execute(&entry_copy.value, interp)
                    }
                }
            } else {
                // Entry was created with val - always push as constant
                interp.push(entry_copy.value);
                Ok(())
            }
        },

        // RUST CONCEPT: Custom error types
        // If the atom isn't in the dictionary, it's an undefined word error
        None => {
            // RUST CONCEPT: Converting Rc<str> to String for error messages
            // &**atom_name dereferences the Rc, then the str, then takes a reference
            // .to_string() converts &str to owned String
            Err(RuntimeError::UndefinedWord((&**atom_name).to_string()))
        }
    }
}

// RUST CONCEPT: Top-level execution function
// This parses and executes a string of Uni code
pub fn execute_string(code: &str, interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Module imports and error conversion
    // We import parse from our parser module
    use crate::parser::parse;

    // RUST CONCEPT: Error propagation with ?
    // parse() returns Result<Vec<Value>, ParseError>
    // The ? converts ParseError to RuntimeError using our From implementation
    let values = parse(code, interp)?;

    // RUST CONCEPT: Iterators and error handling in loops
    // We execute each top-level value in sequence
    // If any execution fails, the ? will return that error immediately
    for value in values {
        execute(&value, interp)?;
    }

    Ok(())
}

// RUST CONCEPT: Conditional compilation for tests
#[cfg(test)]
mod tests {
    use super::*;
    // use crate::builtins::register_builtins;  // No longer needed

    // RUST CONCEPT: Test helper function
    // DRY principle - Don't Repeat Yourself
    fn setup_interpreter() -> Interpreter {
        // RUST CONCEPT: Automatic initialization
        // Interpreter::new() now automatically loads builtins and stdlib
        Interpreter::new()
    }

    #[test]
    fn test_execute_numbers() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing basic functionality
        let number = Value::Number(42.0);
        execute(&number, &mut interp).unwrap();

        // RUST CONCEPT: Verifying state changes
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_execute_strings() {
        let mut interp = setup_interpreter();

        let string_val: Rc<str> = "hello world".into();
        let string = Value::String(string_val);
        execute(&string, &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello world"));
    }

    #[test]
    fn test_execute_lists_as_data() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing that lists don't execute, just push themselves
        let plus_atom = interp.intern_atom("+");
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Atom(plus_atom),
        ]);

        execute(&list, &mut interp).unwrap();

        // Should have pushed the list as data, not executed it
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Pair(_, _)));

        // Stack should be empty (list didn't execute and push 3)
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_execute_builtin_functions() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing builtin execution
        // Set up stack for addition: 3 + 5
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));

        // Get the + builtin from dictionary and execute it
        let plus_atom = interp.intern_atom("+");
        execute(&Value::Atom(plus_atom), &mut interp).unwrap();

        // Should have popped 3 and 5, pushed 8
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_execute_undefined_atom() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing error cases
        let undefined_atom = interp.intern_atom("nonexistent");
        let result = execute(&Value::Atom(undefined_atom), &mut interp);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RuntimeError::UndefinedWord(s) if s == "nonexistent"));
    }

    #[test]
    fn test_execute_string_simple() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing integration between parser and evaluator
        execute_string("42 3.14", &mut interp).unwrap();

        // Should have two values on stack
        let result1 = interp.pop().unwrap();
        let result2 = interp.pop().unwrap();

        assert!(matches!(result1, Value::Number(n) if n == 3.14));
        assert!(matches!(result2, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_execute_string_with_builtin() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing complete execution flow
        execute_string("5 3 +", &mut interp).unwrap();

        // Should have executed: push 5, push 3, execute +
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));

        // Stack should be empty
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_execute_string_with_list() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing that lists remain as data
        execute_string("[1 2 +] 42", &mut interp).unwrap();

        // Should have list and number on stack
        let number = interp.pop().unwrap();
        let list = interp.pop().unwrap();

        assert!(matches!(number, Value::Number(n) if n == 42.0));
        assert!(matches!(list, Value::Pair(_, _)));
    }

    #[test]
    fn test_execute_quoted_atoms() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing quote behavior
        // 'hello should parse as (quote hello) and quote should be a builtin that
        // pushes the atom without executing it

        // First we need to add a quote builtin for this test to work
        // For now, let's test that quoted structure gets created correctly
        let hello_atom = interp.intern_atom("hello");
        let quote_atom = interp.intern_atom("quote");

        let quoted_hello = Value::Pair(
            Rc::new(Value::Atom(quote_atom)),
            Rc::new(Value::Pair(
                Rc::new(Value::Atom(hello_atom)),
                Rc::new(Value::Nil)
            ))
        );

        // This should push the quoted structure as data (since it's a list)
        execute(&quoted_hello, &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Pair(_, _)));
    }

    #[test]
    fn test_execute_string_parse_errors() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing error propagation from parser
        let result = execute_string("[1 2", &mut interp);  // Unclosed bracket
        assert!(result.is_err());

        let result = execute_string("'[1 2]", &mut interp);  // Invalid quote
        assert!(result.is_err());
    }
}