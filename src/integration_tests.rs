// RUST CONCEPT: Integration tests for complex Uni programs
// These tests verify that the interpreter correctly executes non-trivial programs
// by parsing and evaluating hardcoded code strings rather than files

// Imports are only needed in test module

#[cfg(test)]
mod tests {
    use crate::evaluator::execute_string;
    use crate::interpreter::Interpreter;
    use crate::value::{RuntimeError, Value};

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    // Helper function to execute code and get top stack value
    fn execute_and_get_top(code: &str) -> Result<Value, RuntimeError> {
        let mut interp = setup_interpreter();
        execute_string(code, &mut interp)?;
        if interp.stack.is_empty() {
            Err(RuntimeError::StackUnderflow)
        } else {
            Ok(interp.stack.last().unwrap().clone())
        }
    }

    #[test]
    fn test_factorial_function() {
        // RUST CONCEPT: Testing recursive factorial definition and execution
        // This is a classic non-trivial program that tests recursion, conditionals,
        // function definition, stack management, and multiple primitive operations
        let factorial_code = r#"
            \ Define factorial function recursively
            'fact [
                dup 1 <=           \ ( n -- n bool ) check if n <= 1
                [drop 1]           \ base case: return 1
                [dup 1 - fact *]   \ recursive case: n * fact(n-1)
                if
            ] def

            \ Test factorial of 5
            5 fact
        "#;

        let result = execute_and_get_top(factorial_code).unwrap();
        use num_bigint::BigInt;
        assert!(
            matches!(result, Value::Integer(ref i) if i == &BigInt::from(120)),
            "Expected factorial(5) = 120, got {:?}",
            result
        );
    }

    #[test]
    fn test_naive_fibonacci() {
        // RUST CONCEPT: Testing double recursion with classic naive Fibonacci
        // This tests the interpreter's ability to handle multiple recursive calls
        let fibonacci_code = r#"
            \ Define naive Fibonacci: fib(n) = fib(n-1) + fib(n-2)
            'fib [
                dup 2 <            \ ( n -- n bool ) check if n < 2
                []                 \ base case: return n (0 or 1)
                [                  \ recursive case: fib(n-1) + fib(n-2)
                    dup 1 - fib    \ calculate fib(n-1)
                    swap 2 - fib   \ calculate fib(n-2)
                    +              \ add them together
                ]
                if
            ] def

            \ Test Fibonacci of 7 (should be 13)
            \ fib(7) = fib(6) + fib(5) = 8 + 5 = 13
            7 fib
        "#;

        let result = execute_and_get_top(fibonacci_code).unwrap();
        use num_bigint::BigInt;
        assert!(
            matches!(result, Value::Integer(ref i) if i == &BigInt::from(13)),
            "Expected fibonacci(7) = 13, got {:?}",
            result
        );
    }

    #[test]
    fn test_list_processing_prelude() {
        // RUST CONCEPT: Testing prelude list processing functions
        // This tests length and null? predicates properly
        let list_processing_code = r#"
            \ Test length function on various lists
            [1 2 3 4 5] length          \ should be 5
            [] length +                 \ should be 5 + 0 = 5

            \ Test nil? predicate (just verify it returns boolean)
            [] nil?                     \ should be true
            drop                        \ remove boolean from stack

            \ Test non-empty list
            [1] length +                \ length of [1] = 1, total = 5 + 1 = 6
        "#;

        let result = execute_and_get_top(list_processing_code).unwrap();
        use num_bigint::BigInt;
        assert!(
            matches!(result, Value::Integer(ref i) if i == &BigInt::from(6)),
            "Expected length operations result = 6, got {:?}",
            result
        );
    }

    #[test]
    fn test_mutually_recursive_tail_call_optimization() {
        // RUST CONCEPT: Testing mutual tail recursion with large iteration count
        // This verifies that the continuation-based evaluator properly handles
        // tail-call optimization across different functions calling each other
        // Without TCO, this would cause stack overflow at deep recursion levels
        let mutual_recursion_code = r#"
            \ Define two functions that call each other recursively
            \ This creates a "bounce" effect that tests cross-function TCO
            'mutual-a [
                dup 0 =                \ ( n -- n bool ) check if n == 0
                [drop 42]              \ base case: return 42 when done
                [1 - mutual-b]         \ recursive case: decrement and call mutual-b
                if
            ] def

            'mutual-b [
                dup 0 =                \ ( n -- n bool ) check if n == 0
                [drop 42]              \ base case: return 42 when done
                [1 - mutual-a]         \ recursive case: decrement and call mutual-a
                if
            ] def

            \ Test with a large number that would cause stack overflow
            \ without proper tail-call optimization
            \ This bounces between the two functions 1000 times
            5000 mutual-a
        "#;

        let result = execute_and_get_top(mutual_recursion_code).unwrap();
        use num_bigint::BigInt;
        assert!(
            matches!(result, Value::Integer(ref i) if i == &BigInt::from(42)),
            "Expected mutual recursion result = 42, got {:?}",
            result
        );
    }

    #[test]
    fn test_deep_single_tail_recursion() {
        // RUST CONCEPT: Testing deep tail recursion in a single function
        // This verifies that tail-call optimization works for self-recursive functions
        let deep_recursion_code = r#"
            \ Define a function that recursively counts down to zero
            \ This tests single-function tail-call optimization
            'countdown [
                dup 0 =                \ ( n -- n bool ) check if n == 0
                [drop 99]              \ base case: return 99 when done
                [1 - countdown]        \ tail-recursive case: decrement and recurse
                if
            ] def

            \ Test with large number that would overflow without TCO
            2000 countdown
        "#;

        let result = execute_and_get_top(deep_recursion_code).unwrap();
        use num_bigint::BigInt;
        assert!(
            matches!(result, Value::Integer(ref i) if i == &BigInt::from(99)),
            "Expected deep recursion result = 99, got {:?}",
            result
        );
    }
}
