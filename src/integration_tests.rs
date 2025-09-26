// RUST CONCEPT: Integration tests for complex Uni programs
// These tests verify that the interpreter correctly executes non-trivial programs
// by parsing and evaluating hardcoded code strings rather than files

// Imports are only needed in test module

#[cfg(test)]
mod tests {
    use crate::interpreter::Interpreter;
    use crate::value::{Value, RuntimeError};
    use crate::evaluator::execute_string;

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
        assert!(matches!(result, Value::Number(120.0)),
                "Expected factorial(5) = 120, got {:?}", result);
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
        assert!(matches!(result, Value::Number(13.0)),
                "Expected fibonacci(7) = 13, got {:?}", result);
    }

    #[test]
    fn test_list_processing_stdlib() {
        // RUST CONCEPT: Testing stdlib list processing functions
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
        assert!(matches!(result, Value::Number(6.0)),
                "Expected length operations result = 6, got {:?}", result);
    }
}