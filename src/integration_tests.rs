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

    // RUST CONCEPT: Record integration tests
    // These tests verify the complete record system works end-to-end

    #[test]
    fn test_record_type_creation() {
        // Test creating a record type with make-record-type
        let code = r#"["name" "age"] "person" make-record-type"#;
        let result = execute_and_get_top(code).unwrap();

        match result {
            Value::RecordType { type_name, field_names } => {
                assert_eq!(&*type_name, "person");
                assert_eq!(field_names.len(), 2);
                assert_eq!(&*field_names[0], "name");
                assert_eq!(&*field_names[1], "age");
            }
            _ => panic!("Expected RecordType, got {:?}", result),
        }
    }

    #[test]
    fn test_record_constructor_and_predicate() {
        // Test that make-record-type creates working constructor and predicate
        let code = r#"
            ["name" "age"] "person" make-record-type drop
            "Alice" 30 make-person
            dup person?
        "#;
        let result = execute_and_get_top(code).unwrap();

        assert!(
            matches!(result, Value::Boolean(true)),
            "Expected person? to return true, got {:?}",
            result
        );
    }

    #[test]
    fn test_record_predicate_false_for_non_records() {
        // Test that type predicates return false for non-matching types
        let code = r#"
            ["name" "age"] "person" make-record-type drop
            42 person?
        "#;
        let result = execute_and_get_top(code).unwrap();

        assert!(
            matches!(result, Value::Boolean(false)),
            "Expected person? to return false for non-record, got {:?}",
            result
        );
    }

    #[test]
    fn test_record_field_accessors() {
        // Test field accessor functions
        let code = r#"
            ["name" "age"] "person" make-record-type drop
            "Charlie" 35 make-person
            dup person-name
            swap person-age
        "#;

        let mut interp = setup_interpreter();
        execute_string(code, &mut interp).unwrap();

        // Stack should have: age, name
        let age = interp.pop().unwrap();
        let name = interp.pop().unwrap();

        assert!(
            matches!(name, Value::String(ref s) if &**s == "Charlie"),
            "Expected name 'Charlie', got {:?}",
            name
        );

        use num_bigint::BigInt;
        assert!(
            matches!(age, Value::Integer(ref i) if i == &BigInt::from(35)),
            "Expected age 35, got {:?}",
            age
        );
    }

    #[test]
    fn test_record_field_mutators() {
        // Test field mutator functions
        let code = r#"
            ["name" "age"] "person" make-record-type drop
            "David" 40 make-person
            "Eve" swap person-name!
            person-name
        "#;

        let result = execute_and_get_top(code).unwrap();

        assert!(
            matches!(result, Value::String(ref s) if &**s == "Eve"),
            "Expected mutated name 'Eve', got {:?}",
            result
        );
    }

    #[test]
    fn test_record_type_of() {
        // Test record-type-of builtin
        let code = r#"
            ["x" "y"] "point" make-record-type drop
            10 20 make-point
            record-type-of
        "#;

        let result = execute_and_get_top(code).unwrap();

        assert!(
            matches!(result, Value::String(ref s) if &**s == "point"),
            "Expected type name 'point', got {:?}",
            result
        );
    }

    #[test]
    fn test_record_general_predicate() {
        // Test general record? predicate from prelude
        let code = r#"
            ["name"] "user" make-record-type drop
            "Bob" make-user
            record?
        "#;

        let result = execute_and_get_top(code).unwrap();

        assert!(
            matches!(result, Value::Boolean(true)),
            "Expected record? to return true, got {:?}",
            result
        );
    }

    #[test]
    fn test_record_predicate_false_for_primitives() {
        // Test that record? returns false for non-records
        let code = r#"
            42 record?
        "#;

        let result = execute_and_get_top(code).unwrap();

        assert!(
            matches!(result, Value::Boolean(false)),
            "Expected record? to return false for integer, got {:?}",
            result
        );
    }

    #[test]
    fn test_multiple_record_types() {
        // Test that different record types don't interfere with each other
        let code = r#"
            ["a" "b"] "foo" make-record-type drop
            ["x" "y"] "bar" make-record-type drop
            1 2 make-foo
            dup foo?
            swap dup bar?
            swap drop drop
            3 4 make-bar
            bar?
        "#;

        let mut interp = setup_interpreter();
        execute_string(code, &mut interp).unwrap();

        // Stack should have: bar-predicate-result
        let bar_result = interp.pop().unwrap();

        assert!(
            matches!(bar_result, Value::Boolean(true)),
            "Expected bar? on bar to be true"
        );

        // Test foo? returns true for foo and false for bar
        let code2 = r#"
            ["a" "b"] "foo" make-record-type drop
            1 2 make-foo
            foo?
        "#;

        let mut interp2 = setup_interpreter();
        execute_string(code2, &mut interp2).unwrap();
        let foo_result = interp2.pop().unwrap();

        assert!(
            matches!(foo_result, Value::Boolean(true)),
            "Expected foo? on foo to be true"
        );

        // Test bar? returns false for foo
        let code3 = r#"
            ["a" "b"] "foo" make-record-type drop
            ["x" "y"] "bar" make-record-type drop
            1 2 make-foo
            bar?
        "#;

        let mut interp3 = setup_interpreter();
        execute_string(code3, &mut interp3).unwrap();
        let foo_bar_result = interp3.pop().unwrap();

        assert!(
            matches!(foo_bar_result, Value::Boolean(false)),
            "Expected bar? on foo to be false"
        );
    }

    #[test]
    fn test_record_with_three_fields() {
        // Test record with multiple fields
        let code = r#"
            ["name" "age" "city"] "person" make-record-type drop
            "Alice" 30 "NYC" make-person
            dup person-name
            swap dup person-age
            swap person-city
        "#;

        let mut interp = setup_interpreter();
        execute_string(code, &mut interp).unwrap();

        // Stack should have: city, age, name
        let city = interp.pop().unwrap();
        let age = interp.pop().unwrap();
        let name = interp.pop().unwrap();

        assert!(
            matches!(name, Value::String(ref s) if &**s == "Alice"),
            "Expected name 'Alice', got {:?}",
            name
        );

        use num_bigint::BigInt;
        assert!(
            matches!(age, Value::Integer(ref i) if i == &BigInt::from(30)),
            "Expected age 30, got {:?}",
            age
        );

        assert!(
            matches!(city, Value::String(ref s) if &**s == "NYC"),
            "Expected city 'NYC', got {:?}",
            city
        );
    }

    #[test]
    fn test_record_mutation_preserves_other_fields() {
        // Test that mutating one field doesn't affect others
        let code = r#"
            ["x" "y"] "point" make-record-type drop
            10 20 make-point
            99 swap point-x!
            dup point-x
            swap point-y
        "#;

        let mut interp = setup_interpreter();
        execute_string(code, &mut interp).unwrap();

        // Stack should have: y, x
        let y = interp.pop().unwrap();
        let x = interp.pop().unwrap();

        use num_bigint::BigInt;
        assert!(
            matches!(x, Value::Integer(ref i) if i == &BigInt::from(99)),
            "Expected mutated x to be 99, got {:?}",
            x
        );

        assert!(
            matches!(y, Value::Integer(ref i) if i == &BigInt::from(20)),
            "Expected y to remain 20, got {:?}",
            y
        );
    }

    // RUST CONCEPT: Date/time integration tests
    // These tests verify the complete date/time system works end-to-end

    #[test]
    fn test_datetime_creation_and_accessors() {
        let code = r#"
            2025 10 1 14 30 45 datetime
            dup year
            swap dup month
            swap dup day
            swap dup hour
            swap dup minute
            swap second
        "#;

        let mut interp = setup_interpreter();
        execute_string(code, &mut interp).unwrap();

        use num_bigint::BigInt;

        let second = interp.pop().unwrap();
        let minute = interp.pop().unwrap();
        let hour = interp.pop().unwrap();
        let day = interp.pop().unwrap();
        let month = interp.pop().unwrap();
        let year = interp.pop().unwrap();

        assert!(matches!(year, Value::Integer(ref i) if i == &BigInt::from(2025)));
        assert!(matches!(month, Value::Integer(ref i) if i == &BigInt::from(10)));
        assert!(matches!(day, Value::Integer(ref i) if i == &BigInt::from(1)));
        assert!(matches!(hour, Value::Integer(ref i) if i == &BigInt::from(14)));
        assert!(matches!(minute, Value::Integer(ref i) if i == &BigInt::from(30)));
        assert!(matches!(second, Value::Integer(ref i) if i == &BigInt::from(45)));
    }

    #[test]
    fn test_datetime_with_offset() {
        let code = r#"
            2025 10 1 14 30 0 -5 datetime-with-offset
        "#;

        let result = execute_and_get_top(code).unwrap();
        assert!(matches!(result, Value::DateTime(_)));
    }

    #[test]
    fn test_duration_creation() {
        let code = r#"
            1 2 30 45 duration
            duration->seconds
        "#;

        let result = execute_and_get_top(code).unwrap();

        use num_bigint::BigInt;
        // 1 day = 86400s, 2 hours = 7200s, 30 min = 1800s, 45s = 45s
        // Total = 95445 seconds
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(95445)));
    }

    #[test]
    fn test_date_arithmetic_add() {
        let code = r#"
            2025 10 1 12 0 0 datetime
            1 0 0 0 duration
            date+
            day
        "#;

        let result = execute_and_get_top(code).unwrap();

        use num_bigint::BigInt;
        // Adding 1 day to Oct 1 should give Oct 2
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(2)));
    }

    #[test]
    fn test_date_arithmetic_subtract_duration() {
        let code = r#"
            2025 10 5 12 0 0 datetime
            2 0 0 0 duration
            date-
            day
        "#;

        let result = execute_and_get_top(code).unwrap();

        use num_bigint::BigInt;
        // Subtracting 2 days from Oct 5 should give Oct 3
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(3)));
    }

    #[test]
    fn test_date_arithmetic_subtract_datetime() {
        let code = r#"
            2025 10 5 12 0 0 datetime
            2025 10 1 12 0 0 datetime
            date-
            duration->seconds
        "#;

        let result = execute_and_get_top(code).unwrap();

        use num_bigint::BigInt;
        // 4 days difference = 4 * 86400 = 345600 seconds
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(345600)));
    }

    #[test]
    fn test_date_comparisons() {
        let mut interp = setup_interpreter();

        // Test date< (less than)
        execute_string("2025 10 1 12 0 0 datetime 2025 10 2 12 0 0 datetime date<", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test date> (greater than)
        execute_string("2025 10 2 12 0 0 datetime 2025 10 1 12 0 0 datetime date>", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test date= (equal)
        execute_string("2025 10 1 12 0 0 datetime 2025 10 1 12 0 0 datetime date=", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_timezone_conversions() {
        let code = r#"
            2025 10 1 14 30 0 datetime
            dup to-utc
            swap to-local
        "#;

        let mut interp = setup_interpreter();
        execute_string(code, &mut interp).unwrap();

        // Both should be datetime values
        let local = interp.pop().unwrap();
        let utc = interp.pop().unwrap();

        assert!(matches!(local, Value::DateTime(_)));
        assert!(matches!(utc, Value::DateTime(_)));
    }

    #[test]
    fn test_datetime_string_conversion() {
        let code = r#"
            2025 10 1 14 30 0 0 datetime-with-offset
            datetime->string
        "#;

        let result = execute_and_get_top(code).unwrap();

        // Should be a string in ISO 8601 format
        assert!(matches!(result, Value::String(_)));
    }

    #[test]
    fn test_timestamp_conversion() {
        let code = r#"
            2025 10 1 12 0 0 0 datetime-with-offset
            timestamp
            timestamp->datetime
        "#;

        let result = execute_and_get_top(code).unwrap();

        // Should get a datetime back
        assert!(matches!(result, Value::DateTime(_)));
    }

    #[test]
    fn test_weekday() {
        let code = r#"
            2025 10 1 12 0 0 datetime
            weekday
        "#;

        let result = execute_and_get_top(code).unwrap();

        // October 1, 2025 is a Wednesday (weekday = 2)
        use num_bigint::BigInt;
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(2)));
    }

    #[test]
    fn test_duration_comparisons() {
        let mut interp = setup_interpreter();

        // Test duration<
        execute_string("1 0 0 0 duration 2 0 0 0 duration duration<", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test duration>
        execute_string("2 0 0 0 duration 1 0 0 0 duration duration>", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test duration=
        execute_string("1 0 0 0 duration 1 0 0 0 duration duration=", &mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }
}
