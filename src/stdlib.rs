// RUST CONCEPT: Uni Standard Library Module
// This module contains Uni's standard library definitions
// Following the Forth tradition, we define higher-level operations in terms of primitives

use crate::interpreter::Interpreter;
use crate::evaluator::execute_string;
use crate::value::RuntimeError;

// RUST CONCEPT: Standard library initialization
// This function loads all standard library definitions into the interpreter
pub fn load_stdlib(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Define stdlib words using actual Uni code
    // This is much more natural than building def commands from string pairs
    // Each line is real Uni code that defines a word
    // RUST CONCEPT: Multi-line string for multiple definitions
    // Each definition is actual Uni code that uses def naturally
    let stdlib_code = "
        'swap [1 roll] def
        'dup [0 pick] def
        'over [1 pick] def
        'rot [2 roll] def
        'nip [swap drop] def
        'tuck [swap over] def
    ";

    // RUST CONCEPT: Execute the stdlib code directly
    // This uses the normal execution path - no special handling needed
    execute_string(stdlib_code, interp)?;

    Ok(())
}

// RUST CONCEPT: Testing the standard library
#[cfg(test)]
mod tests {
    use super::*;
    // use crate::builtins::register_builtins;  // No longer needed
    use crate::value::Value;

    // RUST CONCEPT: Test helper function
    fn setup_interpreter_with_stdlib() -> Interpreter {
        // RUST CONCEPT: Manual stdlib loading for debugging
        // Interpreter::new() now automatically loads builtins only
        let mut interp = Interpreter::new();
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
    fn test_stdlib_nip() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: 1 2 3 nip should give us 1 3 (removes second item)
        execute_string("1 2 3 nip", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 3.0));
        assert!(matches!(second, Value::Number(n) if n == 1.0));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_stdlib_tuck() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: 5 6 tuck should give us 6 5 6 (insert copy of top below second)
        execute_string("5 6 tuck", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Number(n) if n == 6.0));
        assert!(matches!(second, Value::Number(n) if n == 5.0));
        assert!(matches!(third, Value::Number(n) if n == 6.0));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    // #[test]
    // fn test_stdlib_complex_operations() {
    //     let mut interp = setup_interpreter_with_stdlib();
    //
    //     // Test more complex operation: nip (remove second item)
    //     // 1 2 3 nip should give us 1 3
    //     execute_string("1 2 3 nip", &mut interp).unwrap();
    //
    //     let top = interp.pop().unwrap();
    //     let second = interp.pop().unwrap();
    //
    //     assert!(matches!(top, Value::Number(n) if n == 3.0));
    //     assert!(matches!(second, Value::Number(n) if n == 1.0));
    //
    //     // Stack should be empty now
    //     assert!(interp.pop().is_err());
    // }
}