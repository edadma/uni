// This module implements the async AST-walking evaluator for Uni
//
// UNI EXECUTION MODEL (detailed):
// 1. Numbers, strings, lists: Push themselves onto the stack (they are data)
// 2. Atoms: Look up in dictionary and execute the definition
// 3. Quoted atoms: Already parsed as (quote atom), quote builtin handles them
// 4. Lists are data by default, use 'exec' builtin to execute them
//
// ASYNC CONCEPT: This evaluator is fully async to support async I/O primitives
// The continuation-based architecture remains identical to the sync version
// We just add .await at primitive execution points and recursive calls
//
// RUST LEARNING NOTES:
// - This demonstrates async recursive function calls and pattern matching
// - We use async fn and .await for asynchronous operations
// - Mutable references (&mut) allow us to modify the interpreter state
// - The ? operator propagates errors up the call stack automatically

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use crate::compat::{Rc, Vec, ToString};

#[cfg(not(target_os = "none"))]
use std::collections::HashMap;

#[cfg(target_os = "none")]
use alloc::collections::BTreeMap as HashMap;

// RUST CONCEPT: Continuation-based evaluation for tail-call optimization
// Instead of using recursion, we use an explicit continuation stack
// This enables proper tail-call optimization and prevents stack overflow
#[derive(Debug, Clone)]
enum Continuation {
    // Execute a single value (push data or execute atom)
    Value(Value),

    // Execute a list of values sequentially with index tracking
    // When index == items.len()-1, we can do tail-call optimization
    List {
        items: Vec<Value>,
        index: usize,
    },

    // Execute an if statement (condition already evaluated)
    If {
        condition_result: bool,
        true_branch: Value,
        false_branch: Value,
    },

    // Execute an exec'd expression
    Exec(Value),

    // Execute a defined word's body
    Definition(Value),

    // Pop a local frame when this continuation is reached
    // Used to clean up local variables after quotation/definition execution
    PopLocalFrame,
}

// ASYNC CONCEPT: Continuation-based execution loop (now async!)
// This replaces recursion with an explicit continuation stack for tail-call optimization
// The logic is IDENTICAL to the sync version, we just add .await when executing values
pub async fn execute_with_continuations(
    initial_value: &Value,
    interp: &mut AsyncInterpreter,
) -> Result<(), RuntimeError> {
    let mut continuation_stack: Vec<Continuation> = Vec::new();
    continuation_stack.push(Continuation::Value(initial_value.clone()));

    while let Some(continuation) = continuation_stack.pop() {
        match continuation {
            Continuation::Value(value) => {
                // ASYNC CONCEPT: Await the async execution
                execute_value_direct(&value, interp, &mut continuation_stack).await?;
            }

            Continuation::List { items, index } => {
                if index >= items.len() {
                    continue; // Empty list or finished
                }

                let item = &items[index];
                let is_tail_call = index == items.len() - 1;

                if is_tail_call {
                    // TAIL-CALL OPTIMIZATION: Don't push continuation for last item
                    // This reuses the current "stack frame" enabling proper TCO
                    continuation_stack.push(Continuation::Value(item.clone()));
                } else {
                    // Push continuation for next item, then execute current
                    continuation_stack.push(Continuation::List {
                        items: items.clone(),
                        index: index + 1,
                    });
                    continuation_stack.push(Continuation::Value(item.clone()));
                }
            }

            Continuation::If {
                condition_result,
                true_branch,
                false_branch,
            } => {
                let branch = if condition_result {
                    true_branch
                } else {
                    false_branch
                };
                // TAIL-CALL OPTIMIZATION: Execute branch directly without adding continuation
                match &branch {
                    Value::Pair(_, _) | Value::Nil => {
                        let items = list_to_vec(&branch)?;
                        continuation_stack.push(Continuation::List { items, index: 0 });
                    }
                    _ => {
                        continuation_stack.push(Continuation::Value(branch));
                    }
                }
            }

            Continuation::Exec(value) => {
                // Convert list to continuation or execute single value directly
                match &value {
                    Value::Pair(_, _) => {
                        // Push local frame for quotation execution
                        interp.local_frames.push(HashMap::new());
                        // Schedule frame cleanup after execution
                        continuation_stack.push(Continuation::PopLocalFrame);
                        // Execute the list
                        let items = list_to_vec(&value)?;
                        continuation_stack.push(Continuation::List { items, index: 0 });
                    }
                    Value::Nil => {
                        // Empty list - do nothing (no frame needed)
                    }
                    _ => {
                        // Single value - execute directly (tail-call optimized, no frame needed)
                        continuation_stack.push(Continuation::Value(value));
                    }
                }
            }

            Continuation::Definition(definition) => {
                match &definition {
                    Value::Pair(_, _) | Value::Nil => {
                        // Push local frame for definition execution
                        interp.local_frames.push(HashMap::new());
                        // Schedule frame cleanup after execution
                        continuation_stack.push(Continuation::PopLocalFrame);
                        // Execute list as code (tail-call optimized)
                        let items = list_to_vec(&definition)?;
                        continuation_stack.push(Continuation::List { items, index: 0 });
                    }
                    _ => {
                        // Execute single value directly (tail-call optimized, no frame needed for single values)
                        continuation_stack.push(Continuation::Value(definition));
                    }
                }
            }

            Continuation::PopLocalFrame => {
                // Pop the local frame to clean up local variables
                if interp.local_frames.is_empty() {
                    return Err(RuntimeError::TypeError(
                        "PopLocalFrame: no local frame to pop (internal error)".to_string(),
                    ));
                }
                interp.local_frames.pop();
            }
        }
    }

    Ok(())
}

// ASYNC CONCEPT: Helper function to execute a value directly and manage continuations
// This is where atoms are looked up and special forms are handled
// Now async to support async builtins!
async fn execute_value_direct(
    value: &Value,
    interp: &mut AsyncInterpreter,
    continuation_stack: &mut Vec<Continuation>,
) -> Result<(), RuntimeError> {
    match value {
        // RUST CONCEPT: Simple values just push themselves onto the stack
        Value::Number(n) => {
            interp.push(Value::Number(*n));
            Ok(())
        }
        // RUST CONCEPT: New number types also push themselves
        Value::Int32(i) => {
            interp.push(Value::Int32(*i));
            Ok(())
        }
        Value::Integer(i) => {
            interp.push(Value::Integer(i.clone()));
            Ok(())
        }
        Value::Rational(r) => {
            interp.push(Value::Rational(r.clone()));
            Ok(())
        }
        #[cfg(feature = "complex_numbers")]
        Value::Complex(c) => {
            interp.push(Value::Complex(*c));
            Ok(())
        }
        #[cfg(feature = "complex_numbers")]
        Value::GaussianInt(re, im) => {
            interp.push(Value::GaussianInt(re.clone(), im.clone()));
            Ok(())
        }
        Value::String(s) => {
            interp.push(Value::String(s.clone()));
            Ok(())
        }
        Value::Boolean(b) => {
            interp.push(Value::Boolean(*b));
            Ok(())
        }
        Value::Null => {
            interp.push(Value::Null);
            Ok(())
        }
        Value::Array(array) => {
            interp.push(Value::Array(array.clone()));
            Ok(())
        }
        Value::Variable(var) => {
            interp.push(Value::Variable(var.clone()));
            Ok(())
        }
        Value::Pair(_, _) | Value::Nil => {
            interp.push(value.clone());
            Ok(())
        }
        Value::QuotedAtom(atom_name) => {
            interp.push(Value::Atom(atom_name.clone()));
            Ok(())
        }
        // ASYNC CONCEPT: Async builtins must be awaited!
        Value::AsyncBuiltin(func) => func(interp).await,
        // ASYNC CONCEPT: Atoms need async lookup since they might execute async builtins
        Value::Atom(atom_name) => {
            execute_atom_with_continuations(atom_name, interp, continuation_stack).await
        }
        // RUST CONCEPT: Records and record types push themselves
        Value::Record { .. } | Value::RecordType { .. } => {
            interp.push(value.clone());
            Ok(())
        }
        // RUST CONCEPT: I32 buffers push themselves (they are data)
        Value::I32Buffer(_) => {
            interp.push(value.clone());
            Ok(())
        }
        // RUST CONCEPT: F32 buffers push themselves (they are data)
        Value::F32Buffer(_) => {
            interp.push(value.clone());
            Ok(())
        }
    }
}

// RUST CONCEPT: Convert list structure to vector for sequential processing
fn list_to_vec(list: &Value) -> Result<Vec<Value>, RuntimeError> {
    let mut current = list.clone();
    let mut items = Vec::new();

    loop {
        match current {
            Value::Pair(car, cdr) => {
                items.push((*car).clone());
                current = (*cdr).clone();
            }
            Value::Nil => break,
            _ => {
                // Single value (improper list) - just return it as single item
                items.push(current);
                break;
            }
        }
    }

    Ok(items)
}

// ASYNC CONCEPT: Atom execution with continuation support (now async!)
async fn execute_atom_with_continuations(
    atom_name: &Rc<str>,
    interp: &mut AsyncInterpreter,
    continuation_stack: &mut Vec<Continuation>,
) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Special handling for exec, if, and quit
    if &**atom_name == "exec" {
        let value = interp.pop()?;
        continuation_stack.push(Continuation::Exec(value));
        return Ok(());
    }

    if &**atom_name == "if" {
        let false_branch = interp.pop()?;
        let true_branch = interp.pop()?;
        let condition = interp.pop()?;

        let condition_result = interp.is_truthy(&condition);
        continuation_stack.push(Continuation::If {
            condition_result,
            true_branch,
            false_branch,
        });
        return Ok(());
    }

    if &**atom_name == "quit" {
        // Return special error to signal clean exit
        return Err(RuntimeError::QuitRequested);
    }

    // RUST CONCEPT: Local frame lookup first, then dictionary lookup
    // Check local frames from top (most recent) to bottom (oldest)
    for frame in interp.local_frames.iter().rev() {
        if let Some(value) = frame.get(atom_name) {
            // Found in local frame - push the value directly (it's a constant)
            interp.push(value.clone());
            return Ok(());
        }
    }

    // Not in local frames - try dictionary lookup
    match interp.dictionary.get(atom_name) {
        Some(entry) => {
            let entry_copy = entry.clone();

            if entry_copy.is_executable {
                // Push definition execution continuation
                continuation_stack.push(Continuation::Definition(entry_copy.value));
            } else {
                // Non-executable entry - just push as constant
                interp.push(entry_copy.value);
            }
            Ok(())
        }
        None => Err(RuntimeError::UndefinedWord((**atom_name).to_string())),
    }
}

// ASYNC CONCEPT: Public async functions that other modules can use
// This is the main entry point for executing Uni values
// Now uses async continuation-based execution for tail-call optimization
pub async fn execute(value: &Value, interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    execute_with_continuations(value, interp).await
}

// ASYNC CONCEPT: Top-level async execution function
// This parses and executes a string of Uni code asynchronously
pub async fn execute_string(code: &str, interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Module imports and error conversion
    // We import parse from our parser module
    use crate::parser::parse;

    // RUST CONCEPT: Error propagation with ?
    // parse() returns Result<Vec<Value>, ParseError>
    // The ? converts ParseError to RuntimeError using our From implementation
    let values = parse(code, interp)?;

    // ASYNC CONCEPT: Execute each value asynchronously
    // We execute each top-level value in sequence
    // If any execution fails, the ? will return that error immediately
    for value in values {
        execute(&value, interp).await?;
    }

    Ok(())
}

// RUST CONCEPT: Conditional compilation for tests
#[cfg(test)]
mod tests {
    use super::*;

    // RUST CONCEPT: Test helper function
    // DRY principle - Don't Repeat Yourself
    async fn setup_interpreter() -> AsyncInterpreter {
        // RUST CONCEPT: Async interpreter setup
        // Create interpreter and load prelude for tests
        let mut interp = AsyncInterpreter::new();
        interp.load_prelude().await.expect("Failed to load prelude in tests");
        interp
    }

    // ASYNC CONCEPT: Async tests using tokio::test
    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_numbers() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing basic functionality
        let number = Value::Number(42.0);
        execute(&number, &mut interp).await.unwrap();

        // RUST CONCEPT: Verifying state changes
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_strings() {
        let mut interp = setup_interpreter().await;

        let string_val: Rc<str> = "hello world".into();
        let string = Value::String(string_val);
        execute(&string, &mut interp).await.unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello world"));
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_lists_as_data() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing that lists don't execute, just push themselves
        let plus_atom = interp.intern_atom("+");
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Atom(plus_atom),
        ]);

        execute(&list, &mut interp).await.unwrap();

        // Should have pushed the list as data, not executed it
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Pair(_, _)));

        // Stack should be empty (list didn't execute and push 3)
        assert!(interp.pop().is_err());
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_builtin_functions() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing builtin execution
        // Set up stack for addition: 3 + 5
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));

        // Get the + builtin from dictionary and execute it
        let plus_atom = interp.intern_atom("+");
        execute(&Value::Atom(plus_atom), &mut interp).await.unwrap();

        // Should have popped 3 and 5, pushed 8
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_undefined_atom() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing error cases
        let undefined_atom = interp.intern_atom("nonexistent");
        let result = execute(&Value::Atom(undefined_atom), &mut interp).await;

        assert!(result.is_err());
        assert!(
            matches!(result.unwrap_err(), RuntimeError::UndefinedWord(s) if s == "nonexistent")
        );
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_string_simple() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing integration between parser and evaluator
        execute_string("42 3.14", &mut interp).await.unwrap();

        // Should have two values on stack
        let result1 = interp.pop().unwrap();
        let result2 = interp.pop().unwrap();

        assert!(matches!(result1, Value::Number(n) if n == 3.14));
        assert!(matches!(result2, Value::Int32(42))); // Small integers use Int32
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_string_with_builtin() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing complete execution flow
        execute_string("5 3 +", &mut interp).await.unwrap();

        // Should have executed: push 5, push 3, execute +
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(8))); // Small integers use Int32

        // Stack should be empty
        assert!(interp.pop().is_err());
    }

    // RUST CONCEPT: Tail-call optimization tests
    // These tests demonstrate that the continuation-based evaluator
    // provides proper tail-call optimization for recursive functions

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_tail_recursive_factorial() {
        let mut interp = setup_interpreter().await;

        // Define simple tail-recursive countdown
        execute_string(
            "'count-down [dup 1 <= [drop 999] [1 - count-down] if] def",
            &mut interp,
        )
        .await
        .unwrap();

        // Test with small value
        execute_string("5 count-down", &mut interp).await.unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(999))); // Small integers use Int32
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_deep_tail_recursion() {
        let mut interp = setup_interpreter().await;

        // Define a tail-recursive countdown that would overflow regular recursion
        execute_string(
            "'countdown [dup 0 <= [drop 42] [1 - countdown] if] def",
            &mut interp,
        )
        .await
        .unwrap();

        // Test with moderately deep recursion (this would cause stack overflow without TCO)
        execute_string("1000 countdown", &mut interp).await.unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(42))); // Small integers use Int32
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_mutually_tail_recursive_functions() {
        let mut interp = setup_interpreter().await;

        // Define mutually recursive even/odd functions using available primitives
        execute_string("'even [dup 0 = [drop true] [1 - odd] if] def", &mut interp).await.unwrap();
        execute_string("'odd [dup 0 = [drop false] [1 - even] if] def", &mut interp).await.unwrap();

        // Test even/odd with moderate numbers
        execute_string("10 even", &mut interp).await.unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        execute_string("11 odd", &mut interp).await.unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_tail_call_in_if_branches() {
        let mut interp = setup_interpreter().await;

        // Test that both branches of if are tail-call optimized
        execute_string(
            "'branch-test [dup 10 > [drop 99] [1 + branch-test] if] def",
            &mut interp,
        )
        .await
        .unwrap();

        execute_string("5 branch-test", &mut interp).await.unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(99))); // Small integers use Int32
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_string_with_list() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing that lists remain as data
        execute_string("[1 2 +] 42", &mut interp).await.unwrap();

        // Should have list and number on stack
        let number = interp.pop().unwrap();
        let list = interp.pop().unwrap();

        assert!(matches!(number, Value::Int32(42))); // Small integers use Int32
        assert!(matches!(list, Value::Pair(_, _)));
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_quoted_atoms() {
        let mut interp = setup_interpreter().await;

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
                Rc::new(Value::Nil),
            )),
        );

        // This should push the quoted structure as data (since it's a list)
        execute(&quoted_hello, &mut interp).await.unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Pair(_, _)));
    }

    #[cfg(not(target_os = "none"))]
    #[tokio::test]
    async fn test_execute_string_parse_errors() {
        let mut interp = setup_interpreter().await;

        // RUST CONCEPT: Testing error propagation from parser
        let result = execute_string("[1 2", &mut interp).await; // Unclosed bracket
        assert!(result.is_err());

        let result = execute_string("'[1 2]", &mut interp).await; // Invalid quote
        assert!(result.is_err());
    }
}
