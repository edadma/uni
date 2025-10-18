// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::{format, ToString};
use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Polymorphic addition - multiple numeric types and string concatenation
// Stack-based addition: ( n1 n2 -- sum ) or ( str1 any -- concat ) or ( any str2 -- concat )
// Supports automatic type promotion: Integer < Rational < GaussianInt < Number < Complex
pub fn add_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b =
        interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;
    let a =
        interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;

    // Handle string concatenation first
    match (&a, &b) {
        (Value::String(_), _) | (_, Value::String(_)) => {
            let str_a = match &a {
                Value::String(s) => s.as_ref(),
                _ => &a.to_string(),
            };
            let str_b = match &b {
                Value::String(s) => s.as_ref(),
                _ => &b.to_string(),
            };
            let result = format!("{}{}", str_a, str_b);
            interp.push(Value::String(result.into()));
            return Ok(());
        }
        _ => {}
    }

    // For numeric types, use type promotion
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        // RUST CONCEPT: Fast path for Int32 - checked arithmetic for embedded safety
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_add(*i2) {
                Some(result) => Value::Int32(result),
                // Overflow: promote to BigInt
                None => Value::Integer(num_bigint::BigInt::from(*i1) + num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 + i2),
        (Value::Rational(r1), Value::Rational(r2)) => Value::Rational(r1 + r2).demote(),
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 + n2),
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(re1, im1), Value::GaussianInt(re2, im2)) => {
            Value::GaussianInt(re1 + re2, im1 + im2).demote()
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 + c2),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot add {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_add_builtin() {
        let mut interp = setup_interpreter();

        // Test basic addition
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));

        // Test with negative numbers
        interp.push(Value::Number(-2.0));
        interp.push(Value::Number(7.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with zero
        interp.push(Value::Number(0.0));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_add_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_add_builtin_string_concatenation() {
        let mut interp = setup_interpreter();

        // Test string + string
        interp.push(Value::String("Hello ".into()));
        interp.push(Value::String("World".into()));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "Hello World"));

        // Test string + number
        interp.push(Value::String("Count: ".into()));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "Count: 42"));

        // Test number + string
        interp.push(Value::Number(3.14));
        interp.push(Value::String(" is pi".into()));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "3.14 is pi"));

        // Test string + boolean
        interp.push(Value::String("Result: ".into()));
        interp.push(Value::Boolean(true));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "Result: true"));
    }

    #[test]
    fn test_add_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with incompatible types (no numbers or strings)
        interp.push(Value::Boolean(true));
        interp.push(Value::Boolean(false));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_add_builtin_position_aware_error() {
        use crate::tokenizer::SourcePos;
        let mut interp = setup_interpreter();

        // Set up a source position for error context (mock for testing)
        let pos = SourcePos::new(2, 15, 20); // Line 2, column 15, offset 20
        interp.current_pos = Some(pos);

        // Test stack underflow with position information
        let result = add_builtin(&mut interp);
        assert!(result.is_err());

        match result.unwrap_err() {
            RuntimeError::StackUnderflowAt { pos, context } => {
                assert_eq!(pos.line, 2);
                assert_eq!(pos.column, 15);
                assert_eq!(pos.offset, 20);
                assert!(context.contains("'+' requires exactly 2 values"));
            }
            _ => panic!("Expected StackUnderflowAt error"),
        }

        // Test with only one element on stack
        interp.push(Value::Number(42.0));
        let result = add_builtin(&mut interp);
        assert!(result.is_err());

        match result.unwrap_err() {
            RuntimeError::StackUnderflowAt { pos, context } => {
                assert_eq!(pos.line, 2);
                assert_eq!(pos.column, 15);
                assert!(context.contains("'+' requires exactly 2 values"));
            }
            _ => panic!("Expected StackUnderflowAt error"),
        }
    }

    #[test]
    fn test_demonstrate_formatted_error_output() {
        use crate::tokenizer::SourcePos;
        let mut interp = setup_interpreter();

        // Set up a source position that represents where '+' appears in source code
        let pos = SourcePos::new(3, 8, 45); // Line 3, column 8, offset 45
        interp.current_pos = Some(pos);

        // Try to add without enough values on stack
        let result = add_builtin(&mut interp);
        assert!(result.is_err());

        // Show the formatted error message
        let error = result.unwrap_err();
        let formatted_error = format!("{}", error);

        // Print to demonstrate nice formatting (won't show in normal test run)
        println!("Demo error message: {}", formatted_error);

        // Verify the error message contains expected components
        assert!(formatted_error.contains("line 3, column 8"));
        assert!(formatted_error.contains("'+' requires exactly 2 values"));
        assert!(formatted_error.contains("Stack underflow"));
    }

    // ========== TESTS FOR NEW NUMBER TYPES ==========

    #[test]
    fn test_add_bigint() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test BigInt + BigInt
        interp.push(Value::Integer(BigInt::from(123)));
        interp.push(Value::Integer(BigInt::from(456)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == BigInt::from(579)));

        // Test large BigInt addition
        let large1 = BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        let large2 = BigInt::parse_bytes(b"987654321098765432109876543210", 10).unwrap();
        let expected = BigInt::parse_bytes(b"1111111110111111111011111111100", 10).unwrap();

        interp.push(Value::Integer(large1));
        interp.push(Value::Integer(large2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == expected));
    }

    #[test]
    fn test_add_rational() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test Rational + Rational: 1/2 + 1/3 = 5/6
        let r1 = BigRational::new(BigInt::from(1), BigInt::from(2));
        let r2 = BigRational::new(BigInt::from(1), BigInt::from(3));
        let expected = BigRational::new(BigInt::from(5), BigInt::from(6));

        interp.push(Value::Rational(r1));
        interp.push(Value::Rational(r2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Rational(r) if r == expected));

        // Test Rational + Rational: 1/3 + 1/3 = 2/3
        let r1 = BigRational::new(BigInt::from(1), BigInt::from(3));
        let r2 = BigRational::new(BigInt::from(1), BigInt::from(3));
        let expected = BigRational::new(BigInt::from(2), BigInt::from(3));

        interp.push(Value::Rational(r1));
        interp.push(Value::Rational(r2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Rational(r) if r == expected));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_complex() {
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test Complex + Complex: (3+4i) + (1+2i) = (4+6i)
        let c1 = Complex64::new(3.0, 4.0);
        let c2 = Complex64::new(1.0, 2.0);
        let expected = Complex64::new(4.0, 6.0);

        interp.push(Value::Complex(c1));
        interp.push(Value::Complex(c2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == expected));

        // Test pure imaginary: 5i + 3i = 8i
        let c1 = Complex64::new(0.0, 5.0);
        let c2 = Complex64::new(0.0, 3.0);
        let expected = Complex64::new(0.0, 8.0);

        interp.push(Value::Complex(c1));
        interp.push(Value::Complex(c2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == expected));
    }

    #[test]
    fn test_add_mixed_float_bigint() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test float + BigInt: 5.0 + 10 = 15.0 (promotes to float since floats are inexact)
        interp.push(Value::Number(5.0));
        interp.push(Value::Integer(BigInt::from(10)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 15.0));

        // Test BigInt + float (reversed order)
        interp.push(Value::Integer(BigInt::from(20)));
        interp.push(Value::Number(7.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 27.0));
    }

    #[test]
    fn test_add_mixed_float_rational() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test float + Rational: 0.5 + 1/2 = 1.0 (promotes to float since floats are inexact)
        interp.push(Value::Number(0.5));
        interp.push(Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(2))));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Result should be Number (float) since mixing exact and inexact promotes to inexact
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_mixed_float_complex() {
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test float + Complex: 3.0 + (2+5i) = (5+5i)
        interp.push(Value::Number(3.0));
        interp.push(Value::Complex(Complex64::new(2.0, 5.0)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(5.0, 5.0)));
    }

    #[test]
    fn test_add_mixed_bigint_rational() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test BigInt + Rational: 2n + 1/2 = 5/2
        interp.push(Value::Integer(BigInt::from(2)));
        interp.push(Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(2))));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        let expected = BigRational::new(BigInt::from(5), BigInt::from(2));
        assert!(matches!(result, Value::Rational(r) if r == expected));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_mixed_bigint_complex() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test BigInt + Complex: 5n + (1+2i) = (6+2i)
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Complex(Complex64::new(1.0, 2.0)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(6.0, 2.0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_mixed_rational_complex() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test Rational + Complex: 1/2 + (3+4i)
        interp.push(Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(2))));
        interp.push(Value::Complex(Complex64::new(3.0, 4.0)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(3.5, 4.0)));
    }

    // ========== EDGE CASE TESTS ==========

    #[test]
    fn test_add_bigint_zero() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test 0n + 0n = 0n
        interp.push(Value::Integer(BigInt::from(0)));
        interp.push(Value::Integer(BigInt::from(0)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == BigInt::from(0)));

        // Test 0n + positive
        interp.push(Value::Integer(BigInt::from(0)));
        interp.push(Value::Integer(BigInt::from(42)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == BigInt::from(42)));
    }

    #[test]
    fn test_add_bigint_negative() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test negative + negative
        interp.push(Value::Integer(BigInt::from(-5)));
        interp.push(Value::Integer(BigInt::from(-3)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == BigInt::from(-8)));

        // Test positive + negative
        interp.push(Value::Integer(BigInt::from(10)));
        interp.push(Value::Integer(BigInt::from(-7)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == BigInt::from(3)));
    }

    #[test]
    fn test_add_rational_zero() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test 0/1 + 0/1 (should demote to Int32(0))
        let zero = BigRational::new(BigInt::from(0), BigInt::from(1));
        interp.push(Value::Rational(zero.clone()));
        interp.push(Value::Rational(zero.clone()));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));

        // Test 0/1 + 3/4
        let frac = BigRational::new(BigInt::from(3), BigInt::from(4));
        interp.push(Value::Rational(zero));
        interp.push(Value::Rational(frac.clone()));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Rational(r) if r == frac));
    }

    #[test]
    fn test_add_rational_negative() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test -1/2 + 1/2 = 0 (demoted to Int32)
        let neg_half = BigRational::new(BigInt::from(-1), BigInt::from(2));
        let pos_half = BigRational::new(BigInt::from(1), BigInt::from(2));
        interp.push(Value::Rational(neg_half));
        interp.push(Value::Rational(pos_half));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));

        // Test -3/4 + -1/4 = -1 (demoted to Int32)
        let r1 = BigRational::new(BigInt::from(-3), BigInt::from(4));
        let r2 = BigRational::new(BigInt::from(-1), BigInt::from(4));
        interp.push(Value::Rational(r1));
        interp.push(Value::Rational(r2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(-1)));
    }

    #[test]
    fn test_add_rational_simplification() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test 1/4 + 1/4 = 1/2 (auto-simplifies from 2/4)
        let quarter = BigRational::new(BigInt::from(1), BigInt::from(4));
        let half = BigRational::new(BigInt::from(1), BigInt::from(2));

        interp.push(Value::Rational(quarter.clone()));
        interp.push(Value::Rational(quarter));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Rational(r) if r == half));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_complex_zero() {
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test 0+0i + 0+0i
        let zero = Complex64::new(0.0, 0.0);
        interp.push(Value::Complex(zero));
        interp.push(Value::Complex(zero));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == zero));

        // Test 0+0i + 3+4i
        let c = Complex64::new(3.0, 4.0);
        interp.push(Value::Complex(zero));
        interp.push(Value::Complex(c));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(3.0, 4.0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_complex_negative() {
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test (3+4i) + (-3-4i) = 0+0i
        let c1 = Complex64::new(3.0, 4.0);
        let c2 = Complex64::new(-3.0, -4.0);
        let zero = Complex64::new(0.0, 0.0);

        interp.push(Value::Complex(c1));
        interp.push(Value::Complex(c2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == zero));

        // Test negative real only: (-5+2i) + (3-1i)
        let c1 = Complex64::new(-5.0, 2.0);
        let c2 = Complex64::new(3.0, -1.0);
        let expected = Complex64::new(-2.0, 1.0);

        interp.push(Value::Complex(c1));
        interp.push(Value::Complex(c2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == expected));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_complex_pure_real() {
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test (5+0i) + (3+0i) = (8+0i)
        let c1 = Complex64::new(5.0, 0.0);
        let c2 = Complex64::new(3.0, 0.0);
        let expected = Complex64::new(8.0, 0.0);

        interp.push(Value::Complex(c1));
        interp.push(Value::Complex(c2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == expected));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_complex_pure_imaginary() {
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test (0+5i) + (0+3i) = (0+8i)
        let c1 = Complex64::new(0.0, 5.0);
        let c2 = Complex64::new(0.0, 3.0);
        let expected = Complex64::new(0.0, 8.0);

        interp.push(Value::Complex(c1));
        interp.push(Value::Complex(c2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == expected));
    }

    #[test]
    fn test_add_mixed_float_bigint_fractional() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test 3.5 + 10 = 13.5 (promotes to float since floats are inexact)
        interp.push(Value::Number(3.5));
        interp.push(Value::Integer(BigInt::from(10)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should be Number (float) since mixing exact and inexact promotes to inexact
        assert!(matches!(result, Value::Number(n) if n == 13.5));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_mixed_types_commutativity() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test that mixed type addition is commutative: a + b == b + a
        // BigInt + Complex
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Complex(Complex64::new(1.0, 2.0)));
        add_builtin(&mut interp).unwrap();
        let result1 = interp.pop().unwrap();

        // Complex + BigInt (reversed)
        interp.push(Value::Complex(Complex64::new(1.0, 2.0)));
        interp.push(Value::Integer(BigInt::from(5)));
        add_builtin(&mut interp).unwrap();
        let result2 = interp.pop().unwrap();

        // Both should be Complex(6.0, 2.0)
        assert!(matches!(result1, Value::Complex(c) if c == Complex64::new(6.0, 2.0)));
        assert!(matches!(result2, Value::Complex(c) if c == Complex64::new(6.0, 2.0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_type_error_with_new_types() {
        let mut interp = setup_interpreter();

        // Test BigInt + Atom (should error)
        use num_bigint::BigInt;
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Atom("foo".into()));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));

        // Test Rational + Boolean (should error)
        use num_rational::BigRational;
        interp.stack.clear();
        interp.push(Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(2))));
        interp.push(Value::Boolean(true));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));

        // Test Complex + Nil (should error)
        use num_complex::Complex64;
        interp.stack.clear();
        interp.push(Value::Complex(Complex64::new(1.0, 2.0)));
        interp.push(Value::Nil);
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_add_very_large_bigint() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test addition of very large numbers that would overflow i64
        let large1 = BigInt::parse_bytes(b"99999999999999999999999999999999", 10).unwrap();
        let large2 = BigInt::parse_bytes(b"11111111111111111111111111111111", 10).unwrap();
        let expected = BigInt::parse_bytes(b"111111111111111111111111111111110", 10).unwrap();

        interp.push(Value::Integer(large1));
        interp.push(Value::Integer(large2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(i) if i == expected));
    }

    #[test]
    fn test_add_infinity_and_nan() {
        let mut interp = setup_interpreter();

        // Test infinity + number
        interp.push(Value::Number(f64::INFINITY));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.is_infinite()));

        // Test NaN + number
        interp.push(Value::Number(f64::NAN));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.is_nan()));
    }

    // ========== GAUSSIAN INTEGER TESTS ==========

    #[test]
    #[cfg(feature = "complex_numbers")]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test GaussianInt + GaussianInt: (3+4i) + (1+2i) = (4+6i)
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::GaussianInt(BigInt::from(1), BigInt::from(2)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(4) && im == BigInt::from(6)
        ));

        // Test negative imaginary: (5+3i) + (2-7i) = (7-4i)
        interp.push(Value::GaussianInt(BigInt::from(5), BigInt::from(3)));
        interp.push(Value::GaussianInt(BigInt::from(2), BigInt::from(-7)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(7) && im == BigInt::from(-4)
        ));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_zero() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test 0+0i + 0+0i = 0 (demoted to Int32)
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(0)));
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(0)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));

        // Test 0+0i + 3+4i = 3+4i
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(0)));
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(3) && im == BigInt::from(4)
        ));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_pure_real() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test (5+0i) + (3+0i) = 8 (demoted to Int32)
        interp.push(Value::GaussianInt(BigInt::from(5), BigInt::from(0)));
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(0)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(8)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_pure_imaginary() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test (0+5i) + (0+3i) = (0+8i)
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(5)));
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(3)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(0) && im == BigInt::from(8)
        ));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_with_bigint() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test GaussianInt + BigInt: (3+4i) + 5 = (8+4i)
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::Integer(BigInt::from(5)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(8) && im == BigInt::from(4)
        ));

        // Test BigInt + GaussianInt (reversed): 7 + (2+3i) = (9+3i)
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::GaussianInt(BigInt::from(2), BigInt::from(3)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(9) && im == BigInt::from(3)
        ));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_with_float_promotes_to_complex() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test GaussianInt + Float promotes to Complex64: (3+4i) + 2.5 = (5.5+4i)
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::Number(2.5));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(5.5, 4.0)));

        // Test Float + GaussianInt (reversed): 1.5 + (2+3i) = (3.5+3i)
        interp.push(Value::Number(1.5));
        interp.push(Value::GaussianInt(BigInt::from(2), BigInt::from(3)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(3.5, 3.0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_with_rational_promotes_to_complex() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        use num_rational::BigRational;
        let mut interp = setup_interpreter();

        // Test GaussianInt + Rational promotes to Complex64: (3+4i) + 1/2 = (3.5+4i)
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(2))));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(3.5, 4.0)));

        // Test Rational + GaussianInt (reversed): 1/4 + (2+5i) = (2.25+5i)
        interp.push(Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(4))));
        interp.push(Value::GaussianInt(BigInt::from(2), BigInt::from(5)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(2.25, 5.0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_with_complex() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = setup_interpreter();

        // Test GaussianInt + Complex: (3+4i) + (1.5+2.5i) = (4.5+6.5i)
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::Complex(Complex64::new(1.5, 2.5)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(4.5, 6.5)));

        // Test Complex + GaussianInt (reversed): (2.5+1.5i) + (5+3i) = (7.5+4.5i)
        interp.push(Value::Complex(Complex64::new(2.5, 1.5)));
        interp.push(Value::GaussianInt(BigInt::from(5), BigInt::from(3)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Complex(c) if c == Complex64::new(7.5, 4.5)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_large_values() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test with large Gaussian integers
        let large_re1 = BigInt::parse_bytes(b"123456789012345", 10).unwrap();
        let large_im1 = BigInt::parse_bytes(b"987654321098765", 10).unwrap();
        let large_re2 = BigInt::parse_bytes(b"111111111111111", 10).unwrap();
        let large_im2 = BigInt::parse_bytes(b"222222222222222", 10).unwrap();

        let expected_re = BigInt::parse_bytes(b"234567900123456", 10).unwrap();
        let expected_im = BigInt::parse_bytes(b"1209876543320987", 10).unwrap();

        interp.push(Value::GaussianInt(large_re1, large_im1));
        interp.push(Value::GaussianInt(large_re2, large_im2));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == expected_re && im == expected_im
        ));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_add_gaussian_int_negative_parts() {
        use num_bigint::BigInt;
        let mut interp = setup_interpreter();

        // Test (-3+4i) + (5-2i) = (2+2i)
        interp.push(Value::GaussianInt(BigInt::from(-3), BigInt::from(4)));
        interp.push(Value::GaussianInt(BigInt::from(5), BigInt::from(-2)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(2) && im == BigInt::from(2)
        ));

        // Test (-5-3i) + (-2-7i) = (-7-10i)
        interp.push(Value::GaussianInt(BigInt::from(-5), BigInt::from(-3)));
        interp.push(Value::GaussianInt(BigInt::from(-2), BigInt::from(-7)));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(re, im) if re == BigInt::from(-7) && im == BigInt::from(-10)
        ));
    }
}
