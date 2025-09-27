// RUST CONCEPT: Uni Standard Library Module
// This module contains Uni's standard library definitions
// Following the Forth tradition, we define higher-level operations in terms of primitives

use crate::evaluator::execute_string;
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: Standard library initialization
// This function loads all standard library definitions into the interpreter
pub fn load_stdlib(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Define stdlib words using actual Uni code
    // This is much more natural than building def commands from string pairs
    // Each line is real Uni code that defines a word
    // RUST CONCEPT: Multi-line string for multiple definitions
    // Each definition is actual Uni code that uses def naturally
    let stdlib_code = r#"
        'swap [1 roll] def
        'dup [0 pick] def
        'over [1 pick] def
        'rot [2 roll] def
        'nip [swap drop] def
        'tuck [swap over] def
        'nil? [[] =] def

        \\ List processing primitives
        'length [
            dup nil?
            [drop 0]
            [tail length 1 +]
            if
        ] def

        'null? [null =] def

        \\ Conditional duplication from Forth
        '?dup [
            dup truthy? [dup] [] if
        ] def

        \\ Short-circuiting logical operations
        'and [
            swap                          \\ Move first quotation to top
            exec                          \\ Execute first quotation
            dup                           \\ Always duplicate the result
            [
                drop                      \\ Drop the duplicate, keep original
                exec                      \\ Execute second quotation
            ]
            [
                swap drop                 \\ If falsy, drop second quotation, keep falsy result
            ]
            if
        ] def

        'or [
            swap                          \\ Move first quotation to top
            exec                          \\ Execute first quotation
            dup                           \\ Always duplicate the result
            [
                swap drop                 \\ If truthy, drop second quotation, keep result
            ]
            [
                drop                      \\ Drop the duplicate
                exec                      \\ If falsy, execute second quotation
            ]
            if
        ] def

        \\ Control flow primitives
        'while [
            >r >r                         \\ move body and condition to return stack
            r@ exec                       \\ execute condition (copy from R-stack)
            [
                r> r> dup rot swap >r >r  \\ get body and move body and condition back to return stack
                exec                      \\ execute body
                r> r> while               \\ recursive call
            ]
            [ r> r> drop drop ]
            if
        ] def
    "#;

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

    #[test]
    fn test_stdlib_length() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: [1 2 3 4 5] length should give us 5
        execute_string("[1 2 3 4 5] length", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test: [] length should give us 0
        execute_string("[] length", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_stdlib_null_predicate() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: null null? should give us true
        execute_string("null null?", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test: 42 null? should give us false
        execute_string("42 null?", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Test: [] null? should give us false (empty list is not null)
        execute_string("[] null?", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_stdlib_while_loop_counter() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: while loop that counts from 1 and leaves 5 on the stack
        // Start with 1, condition checks if counter < 5, body increments counter
        // Final result should leave 5 on the stack when condition becomes false
        let code = r#"
            1
            [ dup 5 < ]
            [ 1 + ]
            while
        "#;

        execute_string(code, &mut interp).unwrap();

        // Should have 5 on stack (started at 1, incremented while < 5)
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_stdlib_while_sum_accumulator() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: while loop that sums numbers 1+2+3+4+5 = 15
        // Stack: [counter sum]
        let code = r#"
            1 0
            [ over 5 <= ]
            [ over + swap 1 + swap ]
            while
            nip
        "#;

        execute_string(code, &mut interp).unwrap();

        // Should have 15 on stack (sum of 1+2+3+4+5)
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 15.0));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_stdlib_while_empty_body() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: while loop with false condition - body should never execute
        let code = r#"
            42
            [ 0 ]
            [ 99 ]
            while
        "#;

        execute_string(code, &mut interp).unwrap();

        // Should still have 42 on stack (body never executed)
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_stdlib_question_dup() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: truthy value should be duplicated
        execute_string("42 ?dup", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 42.0));
        assert!(matches!(second, Value::Number(n) if n == 42.0));

        // Test: falsy value (0) should not be duplicated
        execute_string("0 ?dup", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
        // Should be empty now - no duplication occurred
        assert!(interp.pop().is_err());

        // Test: false should not be duplicated
        execute_string("false ?dup", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_stdlib_and_short_circuit() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: true and true -> returns second value
        execute_string("[5] [10] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 10.0));

        // Test: false and anything -> returns false, doesn't execute second
        execute_string("[0] [99] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));

        // Test: short-circuiting - second quotation should not execute
        // This pushes a marker, then does false and [marker-remover]
        // If short-circuiting works, marker should still be there
        execute_string("999 [0] [drop] and", &mut interp).unwrap();

        let and_result = interp.pop().unwrap();
        let marker = interp.pop().unwrap();
        assert!(matches!(and_result, Value::Number(n) if n == 0.0));
        assert!(matches!(marker, Value::Number(n) if n == 999.0)); // Marker should still be there
    }

    #[test]
    fn test_stdlib_or_short_circuit() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: false or true -> returns second value
        execute_string("[0] [42] or", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test: true or anything -> returns first value, doesn't execute second
        execute_string("[5] [99] or", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test: short-circuiting - second quotation should not execute
        execute_string("888 [7] [drop] or", &mut interp).unwrap();

        let or_result = interp.pop().unwrap();
        let marker = interp.pop().unwrap();
        assert!(matches!(or_result, Value::Number(n) if n == 7.0));
        assert!(matches!(marker, Value::Number(n) if n == 888.0)); // Marker should still be there
    }

    #[test]
    fn test_stdlib_and_or_chaining() {
        let mut interp = setup_interpreter_with_stdlib();

        // Test: chaining and operations - all true
        execute_string("[1] [2] and [3] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test: chaining or operations - first true
        execute_string("[1] [2] or [3] or", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));

        // Test: mixed and/or
        execute_string("[0] [5] or [10] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }
}
