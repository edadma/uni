// RUST CONCEPT: Uni Standard Library Module
// This module contains Uni's standard library definitions
// Following the Forth tradition, we define higher-level operations in terms of primitives

use crate::interpreter::Interpreter;
use crate::evaluator::execute_string;
use crate::value::RuntimeError;

// RUST CONCEPT: Standard library initialization
// This function loads all standard library definitions into the interpreter
pub fn load_stdlib(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Array of (name, definition) pairs
    // Each entry defines a word in terms of primitives or other words
    let stdlib_definitions = [
        // RUST CONCEPT: Stack manipulation operations built on primitives
        // These follow ANS Forth semantics using roll and pick

        // dup: ( a -- a a ) - Duplicate top of stack
        // Equivalent to: 0 pick
        ("dup", "0 pick"),

        // swap: ( a b -- b a ) - Swap top two items
        // Equivalent to: 1 roll
        ("swap", "1 roll"),

        // over: ( a b -- a b a ) - Copy second item to top
        // Equivalent to: 1 pick
        ("over", "1 pick"),

        // rot: ( a b c -- b c a ) - Rotate top three items
        // Equivalent to: 2 roll
        ("rot", "2 roll"),

        // RUST CONCEPT: Additional useful stack operations
        // These can be built from the primitives above

        // -rot: ( a b c -- c a b ) - Reverse rotate top three
        // Equivalent to: rot rot (rotate twice more = reverse rotate)
        ("-rot", "rot rot"),

        // nip: ( a b -- b ) - Remove second item
        // Equivalent to: swap drop
        ("nip", "swap drop"),

        // tuck: ( a b -- b a b ) - Copy top item under second
        // Equivalent to: swap over
        ("tuck", "swap over"),

        // 2drop: ( a b -- ) - Remove top two items
        // Equivalent to: drop drop
        ("2drop", "drop drop"),

        // 2dup: ( a b -- a b a b ) - Duplicate top two items
        // Equivalent to: over over
        ("2dup", "over over"),

        // 2swap: ( a b c d -- c d a b ) - Swap top two pairs
        // Equivalent to: rot >r rot r>  (but we don't have return stack yet)
        // For now: 3 roll 3 roll
        ("2swap", "3 roll 3 roll"),
    ];

    // RUST CONCEPT: Iterating over definitions and loading them
    for (name, definition) in stdlib_definitions.iter() {
        // RUST CONCEPT: String formatting for def command
        // We build a def command: 'name definition def
        let def_command = format!("'{} {} def", name, definition);

        // RUST CONCEPT: Error propagation with ?
        // If any definition fails, we propagate the error up
        execute_string(&def_command, interp)?;
    }

    Ok(())
}

// RUST CONCEPT: Testing the standard library
#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::register_builtins;
    use crate::value::Value;

    // RUST CONCEPT: Test helper function
    fn setup_interpreter_with_stdlib() -> Interpreter {
        let mut interp = Interpreter::new();
        register_builtins(&mut interp);
        load_stdlib(&mut interp).unwrap();
        interp
    }

    #[test]
    fn test_stdlib_dup() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: 42 dup should give us 42 42
        execute_string("42 dup", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 42.0));
        assert!(matches!(second, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_stdlib_swap() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: 1 2 swap should give us 2 1
        execute_string("1 2 swap", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 1.0));
        assert!(matches!(second, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_stdlib_over() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: 1 2 over should give us 1 2 1
        execute_string("1 2 over", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 1.0));
        assert!(matches!(second, Value::Number(n) if n == 2.0));
        assert!(matches!(third, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_stdlib_rot() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: 1 2 3 rot should give us 2 3 1
        execute_string("1 2 3 rot", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 1.0));
        assert!(matches!(second, Value::Number(n) if n == 3.0));
        assert!(matches!(third, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_stdlib_complex_operations() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test more complex operation: nip (remove second item)
        // 1 2 3 nip should give us 1 3
        execute_string("1 2 3 nip", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 3.0));
        assert!(matches!(second, Value::Number(n) if n == 1.0));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }
}