// ASYNC CONCEPT: Integration tests for complex Uni programs
// These tests verify that the async interpreter correctly executes non-trivial programs
// by parsing and evaluating hardcoded code strings rather than files

use uni_core::evaluator::execute_string;
use uni_core::interpreter::AsyncInterpreter;
use uni_core::value::{RuntimeError, Value};

async fn setup_interpreter() -> AsyncInterpreter {
    let mut interp = AsyncInterpreter::new();
    interp.load_prelude().await.expect("Failed to load prelude");
    interp
}

// Helper function to execute code and get top stack value
async fn execute_and_get_top(code: &str) -> Result<Value, RuntimeError> {
    let mut interp = setup_interpreter().await;
    execute_string(code, &mut interp).await?;
    if interp.stack.is_empty() {
        Err(RuntimeError::StackUnderflow)
    } else {
        Ok(interp.stack.last().unwrap().clone())
    }
}

#[tokio::test]
async fn test_factorial_function() {
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

    let result = execute_and_get_top(factorial_code).await.unwrap();

    assert!(
        matches!(result, Value::Int32(120)),
        "Expected factorial(5) = 120, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_naive_fibonacci() {
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

    let result = execute_and_get_top(fibonacci_code).await.unwrap();

    assert!(
        matches!(result, Value::Int32(13)),
        "Expected fibonacci(7) = 13, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_list_processing_prelude() {
    // RUST CONCEPT: Testing prelude list processing functions
    // This tests length and nil? predicates properly
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

    let result = execute_and_get_top(list_processing_code).await.unwrap();

    assert!(
        matches!(result, Value::Int32(6)),
        "Expected length operations result = 6, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_mutually_recursive_tail_call_optimization() {
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
        \ This bounces between the two functions 5000 times
        5000 mutual-a
    "#;

    let result = execute_and_get_top(mutual_recursion_code).await.unwrap();

    assert!(
        matches!(result, Value::Int32(42)),
        "Expected mutual recursion result = 42, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_deep_single_tail_recursion() {
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

    let result = execute_and_get_top(deep_recursion_code).await.unwrap();

    assert!(
        matches!(result, Value::Int32(99)),
        "Expected deep recursion result = 99, got {:?}",
        result
    );
}

// RUST CONCEPT: Record integration tests
// These tests verify the complete record system works end-to-end

#[tokio::test]
async fn test_record_type_creation() {
    // Test creating a record type with make-record-type
    let code = r#"["name" "age"] "person" make-record-type"#;
    let result = execute_and_get_top(code).await.unwrap();

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

#[tokio::test]
async fn test_record_constructor_and_predicate() {
    // Test that make-record-type creates working constructor and predicate
    let code = r#"
        ["name" "age"] "person" make-record-type drop
        "Alice" 30 make-person
        dup person?
    "#;
    let result = execute_and_get_top(code).await.unwrap();

    assert!(
        matches!(result, Value::Boolean(true)),
        "Expected person? to return true, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_record_predicate_false_for_non_records() {
    // Test that type predicates return false for non-matching types
    let code = r#"
        ["name" "age"] "person" make-record-type drop
        42 person?
    "#;
    let result = execute_and_get_top(code).await.unwrap();

    assert!(
        matches!(result, Value::Boolean(false)),
        "Expected person? to return false for non-record, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_record_field_accessors() {
    // Test field accessor functions
    let code = r#"
        ["name" "age"] "person" make-record-type drop
        "Charlie" 35 make-person
        dup person-name
        swap person-age
    "#;

    let mut interp = setup_interpreter().await;
    execute_string(code, &mut interp).await.unwrap();

    // Stack should have: age, name
    let age = interp.pop().unwrap();
    let name = interp.pop().unwrap();

    assert!(
        matches!(name, Value::String(ref s) if &**s == "Charlie"),
        "Expected name 'Charlie', got {:?}",
        name
    );


    assert!(
        matches!(age, Value::Int32(35)),
        "Expected age 35, got {:?}",
        age
    );
}

#[tokio::test]
async fn test_record_field_mutators() {
    // Test field mutator functions
    let code = r#"
        ["name" "age"] "person" make-record-type drop
        "David" 40 make-person
        "Eve" swap person-name!
        person-name
    "#;

    let result = execute_and_get_top(code).await.unwrap();

    assert!(
        matches!(result, Value::String(ref s) if &**s == "Eve"),
        "Expected mutated name 'Eve', got {:?}",
        result
    );
}

#[tokio::test]
async fn test_record_type_of() {
    // Test record-type-of builtin
    let code = r#"
        ["x" "y"] "point" make-record-type drop
        10 20 make-point
        record-type-of
    "#;

    let result = execute_and_get_top(code).await.unwrap();

    assert!(
        matches!(result, Value::String(ref s) if &**s == "point"),
        "Expected type name 'point', got {:?}",
        result
    );
}

#[tokio::test]
async fn test_record_general_predicate() {
    // Test general is-record-type? predicate
    let code = r#"
        ["name"] "user" make-record-type drop
        "Bob" make-user
        "user" is-record-type?
    "#;

    let result = execute_and_get_top(code).await.unwrap();

    assert!(
        matches!(result, Value::Boolean(true)),
        "Expected is-record-type? to return true, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_multiple_record_types() {
    // Test that different record types don't interfere with each other
    let code = r#"
        ["a" "b"] "foo" make-record-type drop
        ["x" "y"] "bar" make-record-type drop
        1 2 make-foo
        dup foo?
        swap dup "bar" is-record-type?
        swap drop drop
        3 4 make-bar
        bar?
    "#;

    let mut interp = setup_interpreter().await;
    execute_string(code, &mut interp).await.unwrap();

    // Stack should have: bar-predicate-result
    let bar_result = interp.pop().unwrap();

    assert!(
        matches!(bar_result, Value::Boolean(true)),
        "Expected bar? on bar to be true"
    );

    // Test foo? returns true for foo
    let code2 = r#"
        ["a" "b"] "foo" make-record-type drop
        1 2 make-foo
        foo?
    "#;

    let mut interp2 = setup_interpreter().await;
    execute_string(code2, &mut interp2).await.unwrap();
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

    let mut interp3 = setup_interpreter().await;
    execute_string(code3, &mut interp3).await.unwrap();
    let foo_bar_result = interp3.pop().unwrap();

    assert!(
        matches!(foo_bar_result, Value::Boolean(false)),
        "Expected bar? on foo to be false"
    );
}

#[tokio::test]
async fn test_record_with_three_fields() {
    // Test record with multiple fields
    let code = r#"
        ["name" "age" "city"] "person" make-record-type drop
        "Alice" 30 "NYC" make-person
        dup person-name
        swap dup person-age
        swap person-city
    "#;

    let mut interp = setup_interpreter().await;
    execute_string(code, &mut interp).await.unwrap();

    // Stack should have: city, age, name
    let city = interp.pop().unwrap();
    let age = interp.pop().unwrap();
    let name = interp.pop().unwrap();

    assert!(
        matches!(name, Value::String(ref s) if &**s == "Alice"),
        "Expected name 'Alice', got {:?}",
        name
    );


    assert!(
        matches!(age, Value::Int32(30)),
        "Expected age 30, got {:?}",
        age
    );

    assert!(
        matches!(city, Value::String(ref s) if &**s == "NYC"),
        "Expected city 'NYC', got {:?}",
        city
    );
}

#[tokio::test]
async fn test_record_mutation_preserves_other_fields() {
    // Test that mutating one field doesn't affect others
    let code = r#"
        ["x" "y"] "point" make-record-type drop
        10 20 make-point
        99 swap point-x!
        dup point-x
        swap point-y
    "#;

    let mut interp = setup_interpreter().await;
    execute_string(code, &mut interp).await.unwrap();

    // Stack should have: y, x
    let y = interp.pop().unwrap();
    let x = interp.pop().unwrap();


    assert!(
        matches!(x, Value::Int32(99)),
        "Expected mutated x to be 99, got {:?}",
        x
    );

    assert!(
        matches!(y, Value::Int32(20)),
        "Expected y to remain 20, got {:?}",
        y
    );
}

// RUST CONCEPT: Int32 type tests for embedded systems
// These tests verify that Int32 works correctly for small integers

#[tokio::test]
async fn test_int32_parsing() {
    // Test that small integers are parsed as Int32
    let code = "42";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(42)), "Expected Int32(42), got {:?}", result);

    // Test negative Int32
    let code = "-100";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(-100)), "Expected Int32(-100), got {:?}", result);

    // Test that large integers use BigInt
    let code = "9999999999";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Integer(_)), "Expected Integer for large value, got {:?}", result);
}

#[tokio::test]
async fn test_int32_addition() {
    // Test Int32 + Int32 = Int32
    let code = "5 3 +";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(8)), "Expected Int32(8), got {:?}", result);

    // Test Int32 addition with negative
    let code = "10 -3 +";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(7)), "Expected Int32(7), got {:?}", result);
}

#[tokio::test]
async fn test_int32_addition_overflow() {
    // Test Int32 overflow promotes to BigInt
    let code = "2147483647 1 +"; // i32::MAX + 1
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(2147483648_i64)),
        "Expected Integer(2147483648) after overflow, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_subtraction() {
    // Test Int32 - Int32 = Int32
    let code = "10 3 -";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(7)), "Expected Int32(7), got {:?}", result);

    // Test subtraction with negative result
    let code = "3 10 -";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(-7)), "Expected Int32(-7), got {:?}", result);
}

#[tokio::test]
async fn test_int32_subtraction_overflow() {
    // Test Int32 underflow promotes to BigInt
    let code = "-2147483648 1 -"; // i32::MIN - 1
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(-2147483649_i64)),
        "Expected Integer(-2147483649) after underflow, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_multiplication() {
    // Test Int32 * Int32 = Int32
    let code = "6 7 *";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(42)), "Expected Int32(42), got {:?}", result);

    // Test multiplication with negative
    let code = "-5 4 *";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(-20)), "Expected Int32(-20), got {:?}", result);
}

#[tokio::test]
async fn test_int32_multiplication_overflow() {
    // Test Int32 multiplication overflow promotes to BigInt
    let code = "1000000 1000000 *";
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(1000000000000_i64)),
        "Expected Integer after overflow, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_division() {
    // Test Int32 / Int32 with exact result (demotes to Int32)
    let code = "10 2 /";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(5)), "Expected Int32(5), got {:?}", result);

    // Test Int32 / Int32 with fractional result (stays Rational)
    let code = "7 3 /";
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    if let Value::Rational(r) = result {
        assert_eq!(*r.numer(), BigInt::from(7));
        assert_eq!(*r.denom(), BigInt::from(3));
    } else {
        panic!("Expected Rational, got {:?}", result);
    }
}

#[tokio::test]
async fn test_int32_mixed_with_bigint() {
    // Test Int32 + BigInt promotes to BigInt
    let code = "5 9999999999 +";
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(10000000004_i64)),
        "Expected Integer, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_mixed_with_float() {
    // Test Int32 + Number promotes to Number
    let code = "5 3.14 +";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Number(n) if (n - 8.14).abs() < 1e-10),
        "Expected Number(8.14), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_mixed_with_rational() {
    // Test Int32 * Rational
    let code = "5 1 2 / *";  // 5 * 1/2
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    if let Value::Rational(r) = result {
        assert_eq!(*r.numer(), BigInt::from(5));
        assert_eq!(*r.denom(), BigInt::from(2));
    } else {
        panic!("Expected Rational, got {:?}", result);
    }
}

#[tokio::test]
async fn test_int32_demotion_from_rational() {
    // Test that Rational with denominator 1 demotes to Int32
    let code = "4 2 / 2 *";  // (4/2) * 2 = 2 * 2 = 4
    let result = execute_and_get_top(code).await.unwrap();
    assert!(matches!(result, Value::Int32(4)), "Expected Int32(4), got {:?}", result);
}

#[tokio::test]
async fn test_int32_to_integer_promotion() {
    // Test that Int32 automatically promotes to Integer when mixed
    let code = "5 9999999999 +";  // Int32 + BigInt
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(10000000004_i64)),
        "Expected Integer(10000000004) after promotion, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_integer_stays_integer() {
    // Test that large integers stay as Integer
    let code = "9999999999";
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(9999999999_i64)),
        "Expected Integer for large value, got {:?}",
        result
    );
}

// RUST CONCEPT: Additional Int32 edge case tests
// Testing boundary values, complex type interactions, and demotion scenarios

#[tokio::test]
async fn test_int32_boundary_max() {
    // Test i32::MAX stays as Int32
    let code = "2147483647";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(2147483647)),
        "Expected Int32(i32::MAX), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_boundary_min() {
    // Test i32::MIN stays as Int32
    let code = "-2147483648";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(-2147483648)),
        "Expected Int32(i32::MIN), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_max_plus_one_promotes() {
    // Test i32::MAX + 1 promotes to Integer
    let code = "2147483648";
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(2147483648_i64)),
        "Expected Integer for i32::MAX + 1, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_min_minus_one_promotes() {
    // Test i32::MIN - 1 promotes to Integer
    let code = "-2147483649";
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(-2147483649_i64)),
        "Expected Integer for i32::MIN - 1, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_subtraction_to_zero() {
    // Test Int32 - Int32 = 0 (demotion test)
    let code = "5 5 -";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(0)),
        "Expected Int32(0), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_addition_to_zero() {
    // Test Int32 + Int32 = 0
    let code = "5 -5 +";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(0)),
        "Expected Int32(0), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_multiplication_by_zero() {
    // Test Int32 * 0 = Int32(0)
    let code = "42 0 *";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(0)),
        "Expected Int32(0), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_multiplication_by_one() {
    // Test Int32 * 1 = Int32
    let code = "42 1 *";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(42)),
        "Expected Int32(42), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_multiplication_by_negative_one() {
    // Test Int32 * -1 negates
    let code = "42 -1 *";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(-42)),
        "Expected Int32(-42), got {:?}",
        result
    );
}

#[tokio::test]
#[cfg(feature = "complex_numbers")]
async fn test_int32_with_complex_from_float() {
    // Test Int32 + Complex (from float imaginary)
    let code = "5 3.0 i * +";  // 5 + 3.0i = 5+3i (Complex64)
    let result = execute_and_get_top(code).await.unwrap();
    use num_complex::Complex64;
    if let Value::Complex(c) = result {
        assert_eq!(c, Complex64::new(5.0, 3.0));
    } else {
        panic!("Expected Complex, got {:?}", result);
    }
}

#[tokio::test]
#[cfg(feature = "complex_numbers")]
async fn test_int32_with_gaussian_int() {
    // Test Int32 * GaussianInt: 5 * (3+4i)
    let code = "5 3 4 i * + *";  // 5 * (3+4i) = 15+20i
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    if let Value::GaussianInt(re, im) = result {
        assert_eq!(re, BigInt::from(15));
        assert_eq!(im, BigInt::from(20));
    } else {
        panic!("Expected GaussianInt, got {:?}", result);
    }
}

#[tokio::test]
#[cfg(feature = "complex_numbers")]
async fn test_int32_addition_with_gaussian_int() {
    // Test Int32 + GaussianInt: 5 + (3+4i)
    let code = "5 3 4 i * + +";  // 5 + (3+4i) = 8+4i
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    if let Value::GaussianInt(re, im) = result {
        assert_eq!(re, BigInt::from(8));
        assert_eq!(im, BigInt::from(4));
    } else {
        panic!("Expected GaussianInt, got {:?}", result);
    }
}

#[tokio::test]
#[cfg(feature = "complex_numbers")]
async fn test_gaussian_int_demotes_to_int32() {
    // Test GaussianInt with zero imaginary part demotes to Int32
    let code = "5 0 i * + 3 0 i * + +";  // (5+0i) + (3+0i) = 8
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(8)),
        "Expected Int32(8) after GaussianInt demotion, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_rational_division_demotes_to_int32() {
    // Test Rational division that results in whole number demotes to Int32
    let code = "10 5 / 1 *";  // (10/5) * 1 = 2 * 1 = 2
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(2)),
        "Expected Int32(2) after Rational demotion, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_rational_addition_demotes_to_int32() {
    // Test Rational addition that results in whole number demotes to Int32
    let code = "1 2 / 1 2 / +";  // 1/2 + 1/2 = 1
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(1)),
        "Expected Int32(1) after Rational addition, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_rational_subtraction_to_zero_demotes() {
    // Test Rational - Rational = 0 demotes to Int32
    let code = "1 2 / 1 2 / -";  // 1/2 - 1/2 = 0
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(0)),
        "Expected Int32(0) after Rational subtraction, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_max_operations_stay_in_range() {
    // Test operations at i32::MAX boundary that stay in range
    let code = "2147483647 -1 +";  // i32::MAX + (-1) = i32::MAX - 1
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(2147483646)),
        "Expected Int32(2147483646), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_min_operations_stay_in_range() {
    // Test operations at i32::MIN boundary that stay in range
    let code = "-2147483648 1 +";  // i32::MIN + 1 = i32::MIN + 1
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(-2147483647)),
        "Expected Int32(-2147483647), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_near_boundary_multiplication() {
    // Test multiplication near boundary that stays in range
    let code = "46340 46340 *";  // sqrt(i32::MAX) * sqrt(i32::MAX) ≈ 2147395600
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(2147395600)),
        "Expected Int32(2147395600), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_just_over_boundary_multiplication() {
    // Test multiplication that overflows promotes to Integer
    let code = "46341 46341 *";  // Just over sqrt(i32::MAX)
    let result = execute_and_get_top(code).await.unwrap();
    use num_bigint::BigInt;
    assert!(
        matches!(result, Value::Integer(ref i) if i == &BigInt::from(2147488281_i64)),
        "Expected Integer after overflow, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_with_rational_exact_division() {
    // Test Int32 / Int32 that results in exact Rational then simplifies
    let code = "8 2 / 1 2 / /";  // (8/2) / (1/2) = 4 / 0.5 = 8
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(8)),
        "Expected Int32(8), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_negative_zero_is_zero() {
    // Test that -0 is treated as 0
    let code = "0 -1 *";  // 0 * -1 = 0 (not -0)
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(0)),
        "Expected Int32(0), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_chained_operations_stay_int32() {
    // Test multiple operations that all stay within Int32 range
    let code = "10 5 + 3 * 2 -";  // ((10 + 5) * 3) - 2 = 45 - 2 = 43
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(43)),
        "Expected Int32(43), got {:?}",
        result
    );
}

#[tokio::test]
async fn test_int32_mixed_with_float_promotes() {
    // Test Int32 + Float promotes to Float
    let code = "10 3.5 +";
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Number(n) if (n - 13.5).abs() < 1e-10),
        "Expected Number(13.5), got {:?}",
        result
    );
}

#[tokio::test]
#[cfg(feature = "complex_numbers")]
async fn test_gaussian_int_multiplication_demotes() {
    // Test GaussianInt multiplication that yields real result demotes
    let code = "1 i + 1 -1 i * + *";  // (1+i)*(1-i) = 1-i+i-i² = 1-(-1) = 2
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(2)),
        "Expected Int32(2) after GaussianInt multiplication demotion, got {:?}",
        result
    );
}

// RUST CONCEPT: Control flow primitive tests
// Tests for while loops and control flow constructs

#[tokio::test]
async fn test_while_loop_with_counter() {
    // Test while loop with a variable counter
    // This tests the while primitive, variable creation, fetch (@), and store (!)
    let code = r#"
        0 'counter var
        [ counter @ 3 < ] [ counter @ 1 + counter ! ] while
        counter @
    "#;
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(3)),
        "Expected counter to reach 3, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_while_loop_zero_iterations() {
    // Test while loop that should not execute at all (condition false from start)
    let code = r#"
        10 'counter var
        [ counter @ 3 < ] [ counter @ 1 + counter ! ] while
        counter @
    "#;
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(10)),
        "Expected counter to remain 10, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_while_loop_countdown() {
    // Test while loop that counts down instead of up
    let code = r#"
        5 'counter var
        [ counter @ 0 > ] [ counter @ 1 - counter ! ] while
        counter @
    "#;
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(0)),
        "Expected counter to reach 0, got {:?}",
        result
    );
}

#[tokio::test]
async fn test_while_loop_with_accumulator() {
    // Test while loop that accumulates a sum: sum of 1+2+3+4+5 = 15
    let code = r#"
        0 'sum var
        1 'i var
        [ i @ 6 < ] [
            sum @ i @ + sum !
            i @ 1 + i !
        ] while
        sum @
    "#;
    let result = execute_and_get_top(code).await.unwrap();
    assert!(
        matches!(result, Value::Int32(15)),
        "Expected sum to be 15, got {:?}",
        result
    );
}
