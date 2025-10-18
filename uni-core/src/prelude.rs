// RUST CONCEPT: Uni Prelude Module
// This module contains Uni's prelude definitions - the standard words loaded at startup
// Following the Forth tradition, we define higher-level operations in terms of primitives

use crate::evaluator::execute_string;
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: Prelude initialization
// This function loads all prelude definitions into the interpreter
pub fn load_prelude(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Define prelude words using actual Uni code
    // This is much more natural than building def commands from string pairs
    // Each line is real Uni code that defines a word
    // RUST CONCEPT: Multi-line string for multiple definitions
    // Each definition is actual Uni code that uses def naturally
    let prelude_code = r#"
        \\ Stack manipulation words
        'swap [1 roll] def
        "( a b -- b a ) Swap top two stack items" doc

        'dup [0 pick] def
        "( a -- a a ) Duplicate top stack item" doc

        'over [1 pick] def
        "( a b -- a b a ) Copy second stack item to top" doc

        'rot [2 roll] def
        "( a b c -- b c a ) Rotate third item to top" doc

        'nip [swap drop] def
        "( a b -- b ) Remove second stack item" doc

        'tuck [swap over] def
        "( a b -- b a b ) Copy top below second item" doc

        'nil? [[] =] def
        "( x -- bool ) Test if value is empty list" doc

        \\ List processing primitives
        'length [
            dup nil?
            [drop 0]
            [tail length 1 +]
            if
        ] def
        "( list -- n ) Calculate list length recursively" doc

        'null? [null =] def
        "( x -- bool ) Test if value is null" doc

        'record? [type "record" =] def
        "( x -- bool ) Test if value is any record type" doc

        \\ Conditional duplication from Forth
        '?dup [
            dup truthy? [dup] [] if
        ] def
        "( x -- x x | x ) Duplicate if truthy, otherwise leave unchanged" doc

        \\ Variable operations (Forth-style)
        '1+ [1 +] def
        "( n -- n+1 ) Increment by 1" doc

        '1- [1 -] def
        "( n -- n-1 ) Decrement by 1" doc

        '+! [dup @ rot + swap !] def
        "( n var -- ) Add n to variable" doc

        'on [true swap !] def
        "( var -- ) Store true to variable" doc

        'off [false swap !] def
        "( var -- ) Store false to variable" doc

        \\ List iteration
        'each [
            >r                      \\ Move fn to return stack: list | fn
            dup nil?                \\ Check if list is empty: list bool | fn
            [
                drop r> drop        \\ Empty list: clean up list and fn
            ]
            [
                dup head            \\ list -> list head | fn
                r@                  \\ Get fn: list head fn | fn
                exec                \\ Execute fn: list ... | fn (fn consumes head, may leave results)
                tail                \\ Get tail: ... tail | fn
                r> each             \\ Recurse: ... tail fn
            ]
            if
        ] def
        "( list [fn] -- ) Execute fn on each element of list (fn consumes argument, may leave results)" doc

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
        "( [cond1] [cond2] -- result ) Short-circuit AND: executes cond2 only if cond1 is truthy" doc

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
        "( [cond1] [cond2] -- result ) Short-circuit OR: executes cond2 only if cond1 is falsy" doc

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
        "( [condition] [body] -- ) Loop: executes body while condition returns truthy" doc
    "#;

    // RUST CONCEPT: Execute the prelude code directly
    // This uses the normal execution path - no special handling needed
    execute_string(prelude_code, interp)?;

    // RUST CONCEPT: Conditional compilation for feature-specific prelude
    // Complex number constants (only when complex_numbers feature is enabled)
    #[cfg(feature = "complex_numbers")]
    {
        let complex_prelude = r#"
            \\ Mathematical constants (complex numbers)
            'i 0+1i def
            "Imaginary unit constant (0+1i)" doc
        "#;
        execute_string(complex_prelude, interp)?;
    }

    // RUST CONCEPT: Conditional compilation for platform-specific prelude
    // Hardware convenience wrappers for micro:bit
    #[cfg(target_os = "none")]
    {
        let hardware_prelude = r#"
            \\ Hardware convenience wrappers (micro:bit only)
            'button-a? [0 button-read] def
            "( -- bool ) Read button A state (true = pressed)" doc

            'button-b? [1 button-read] def
            "( -- bool ) Read button B state (true = pressed)" doc
        "#;
        execute_string(hardware_prelude, interp)?;
    }

    Ok(())
}

// RUST CONCEPT: Testing the prelude
#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    // RUST CONCEPT: Test helper function
    fn setup_interpreter_with_prelude() -> Interpreter {
        // RUST CONCEPT: Manual prelude loading for testing
        // Interpreter::new() now automatically loads builtins and prelude
        let mut interp = Interpreter::new();
        load_prelude(&mut interp).unwrap();
        interp
    }

    #[test]
    fn test_prelude_dup() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: 42 dup should give us 42 42
        execute_string("42 dup", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Int32(42)));
        assert!(matches!(second, Value::Int32(42)));
    }

    #[test]
    fn test_prelude_swap() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: 1 2 swap should give us 2 1
        execute_string("1 2 swap", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Int32(1)));
        assert!(matches!(second, Value::Int32(2)));
    }

    #[test]
    fn test_prelude_over() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: 1 2 over should give us 1 2 1
        execute_string("1 2 over", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Int32(1)));
        assert!(matches!(second, Value::Int32(2)));
        assert!(matches!(third, Value::Int32(1)));
    }

    #[test]
    fn test_prelude_rot() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: 1 2 3 rot should give us 2 3 1
        execute_string("1 2 3 rot", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Int32(1)));
        assert!(matches!(second, Value::Int32(3)));
        assert!(matches!(third, Value::Int32(2)));
    }

    #[test]
    fn test_prelude_nip() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: 1 2 3 nip should give us 1 3 (removes second item)
        execute_string("1 2 3 nip", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();

        assert!(matches!(top, Value::Int32(3)));
        assert!(matches!(second, Value::Int32(1)));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_tuck() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: 5 6 tuck should give us 6 5 6 (insert copy of top below second)
        execute_string("5 6 tuck", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        let third = interp.pop().unwrap();

        assert!(matches!(top, Value::Int32(6)));
        assert!(matches!(second, Value::Int32(5)));
        assert!(matches!(third, Value::Int32(6)));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_length() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: [1 2 3 4 5] length should give us 5
        execute_string("[1 2 3 4 5] length", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(5)));

        // Test: [] length should give us 0
        execute_string("[] length", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));
    }

    #[test]
    fn test_prelude_null_predicate() {
        let mut interp = setup_interpreter_with_prelude();

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
    fn test_prelude_while_loop_counter() {
        let mut interp = setup_interpreter_with_prelude();

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
        assert!(matches!(result, Value::Int32(5)));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_while_sum_accumulator() {
        let mut interp = setup_interpreter_with_prelude();

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
        assert!(matches!(result, Value::Int32(15)));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_while_empty_body() {
        let mut interp = setup_interpreter_with_prelude();

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
        assert!(matches!(result, Value::Int32(42)));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_question_dup() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: truthy value should be duplicated
        execute_string("42 ?dup", &mut interp).unwrap();

        let top = interp.pop().unwrap();
        let second = interp.pop().unwrap();
        assert!(matches!(top, Value::Int32(42)));
        assert!(matches!(second, Value::Int32(42)));

        // Test: falsy value (0) should not be duplicated
        execute_string("0 ?dup", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));
        // Should be empty now - no duplication occurred
        assert!(interp.pop().is_err());

        // Test: false should not be duplicated
        execute_string("false ?dup", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_and_short_circuit() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: true and true -> returns second value
        execute_string("[5] [10] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(10)));

        // Test: false and anything -> returns false, doesn't execute second
        execute_string("[0] [99] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));

        // Test: short-circuiting - second quotation should not execute
        // This pushes a marker, then does false and [marker-remover]
        // If short-circuiting works, marker should still be there
        execute_string("999 [0] [drop] and", &mut interp).unwrap();

        let and_result = interp.pop().unwrap();
        let marker = interp.pop().unwrap();
        assert!(matches!(and_result, Value::Int32(0)));
        assert!(matches!(marker, Value::Int32(999))); // Marker should still be there
    }

    #[test]
    fn test_prelude_or_short_circuit() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: false or true -> returns second value
        execute_string("[0] [42] or", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(42)));

        // Test: true or anything -> returns first value, doesn't execute second
        execute_string("[5] [99] or", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(5)));

        // Test: short-circuiting - second quotation should not execute
        execute_string("888 [7] [drop] or", &mut interp).unwrap();

        let or_result = interp.pop().unwrap();
        let marker = interp.pop().unwrap();
        assert!(matches!(or_result, Value::Int32(7)));
        assert!(matches!(marker, Value::Int32(888))); // Marker should still be there
    }

    #[test]
    fn test_prelude_and_or_chaining() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: chaining and operations - all true
        execute_string("[1] [2] and [3] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(3)));

        // Test: chaining or operations - first true
        execute_string("[1] [2] or [3] or", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(1)));

        // Test: mixed and/or
        execute_string("[0] [5] or [10] and", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(10)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_prelude_imaginary_constant() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter_with_prelude();

        // Test: 'i' should be defined as 0+1i (GaussianInt)
        execute_string("i", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(1)));

        // Test: using 'i' in arithmetic: i + i = 0+2i
        execute_string("i i +", &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(2)));
    }

    #[test]
    fn test_prelude_each_basic() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: [1 2 3] [drop] each should consume all elements
        execute_string("[10 20 30] [drop] each", &mut interp).unwrap();

        // Stack should be empty - each consumes the list and function consumes elements
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_each_empty_list() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: empty list should not execute function
        execute_string("42 [] [drop] each", &mut interp).unwrap();

        // The 42 should still be on stack (function never executed)
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(42)));

        // Stack should be empty now
        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_prelude_each_with_print() {
        let mut interp = setup_interpreter_with_prelude();

        // Test: [1 2 3] [.] each should print 1, 2, 3 (side effects only)
        // Since [.] consumes its argument and returns nothing, stack should be clean
        execute_string("[1 2 3] [.] each", &mut interp).unwrap();

        // Stack should be empty - pure side effects
        assert!(interp.pop().is_err());
    }
}
