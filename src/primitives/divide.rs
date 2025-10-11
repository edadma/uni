// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_rational::BigRational;
use num_traits::Zero;

// RUST CONCEPT: Division with zero checking and type promotion
// Stack-based division: ( n1 n2 -- quotient )
// Special case: Integer / Integer promotes to Rational for exact division
pub fn div_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Check for division by zero first
    let is_zero = match &b {
        Value::Int32(i) => *i == 0,
        Value::Integer(i) => i.is_zero(),
        Value::Rational(r) => r.is_zero(),
        Value::Number(n) => *n == 0.0,
        _ => false,
    };
    if is_zero {
        return Err(RuntimeError::DivisionByZero);
    }

    // Special case: Int32 / Int32 and Integer / Integer should promote to Rational
    // This ensures exact division results (e.g., 1/2 = 1/2, not 0.5)
    let result = if let (Value::Int32(ia), Value::Int32(ib)) = (&a, &b) {
        // Int32 / Int32 -> Rational (then demote if denominator is 1)
        let result = Value::Rational(BigRational::new(
            num_bigint::BigInt::from(*ia),
            num_bigint::BigInt::from(*ib),
        ));
        result.demote()
    } else if let (Value::Integer(ia), Value::Integer(ib)) = (&a, &b) {
        let result = Value::Rational(BigRational::new(ia.clone(), ib.clone()));
        result.demote()
    } else {
        // For all other type combinations, use standard promotion
        let (pa, pb) = promote_pair(&a, &b);

        match (&pa, &pb) {
            (Value::Rational(r1), Value::Rational(r2)) => {
                let result = Value::Rational(r1 / r2);
                result.demote()
            }
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 / n2),
            (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 / c2),
            _ => {
                return Err(RuntimeError::TypeError(format!(
                    "Cannot divide {:?} and {:?}",
                    a, b
                )))
            }
        }
    };

    interp.push(result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use num_bigint::BigInt;
    use num_rational::BigRational;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_div_builtin_floats() {
        let mut interp = setup_interpreter();

        // Test basic division: 12.0 / 4.0 = 3.0
        interp.push(Value::Number(12.0));
        interp.push(Value::Number(4.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test with fractional result: 7.0 / 2.0 = 3.5
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.5));

        // Test with negative numbers: -8.0 / 2.0 = -4.0
        interp.push(Value::Number(-8.0));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -4.0));
    }

    #[test]
    fn test_div_builtin_integers_to_rational() {
        let mut interp = setup_interpreter();

        // Test Integer / Integer -> Rational: 1 / 2 = 1/2
        interp.push(Value::Integer(BigInt::from(1)));
        interp.push(Value::Integer(BigInt::from(2)));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(1));
            assert_eq!(*r.denom(), BigInt::from(2));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }

        // Test Integer / Integer with exact division -> demotes to Int32: 10 / 2 = 5
        interp.push(Value::Integer(BigInt::from(10)));
        interp.push(Value::Integer(BigInt::from(2)));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(5)));
    }

    #[test]
    fn test_div_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test Integer / Number: 10 / 2.0 = 5.0
        interp.push(Value::Integer(BigInt::from(10)));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test Rational / Number: (1/2) / 2.0 = 0.25
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(1),
            BigInt::from(2),
        )));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.25));
    }

    #[test]
    fn test_div_builtin_rationals() {
        let mut interp = setup_interpreter();

        // Test Rational / Rational: (1/2) / (1/4) = 2
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(1),
            BigInt::from(2),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(1),
            BigInt::from(4),
        )));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should demote to Int32(2)
        assert!(matches!(result, Value::Int32(2)));
    }

    #[test]
    fn test_div_builtin_division_by_zero() {
        let mut interp = setup_interpreter();

        // Test division by zero with floats
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));

        // Test division by zero with integers
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Integer(BigInt::from(0)));
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));

        // Test division by zero with rationals
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(5),
            BigInt::from(1),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(0),
            BigInt::from(1),
        )));
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_div_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_div_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::Null);
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_div_integer_by_integer_exact_division() {
        let mut interp = setup_interpreter();

        // Test cases where integer division results in whole numbers (should demote)

        // 100 / 10 = 10
        interp.push(Value::Integer(BigInt::from(100)));
        interp.push(Value::Integer(BigInt::from(10)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(10)));

        // 1000 / 100 = 10
        interp.push(Value::Integer(BigInt::from(1000)));
        interp.push(Value::Integer(BigInt::from(100)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(10)));

        // -20 / 4 = -5
        interp.push(Value::Integer(BigInt::from(-20)));
        interp.push(Value::Integer(BigInt::from(4)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(-5)));

        // 20 / -4 = -5
        interp.push(Value::Integer(BigInt::from(20)));
        interp.push(Value::Integer(BigInt::from(-4)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(-5)));
    }

    #[test]
    fn test_div_integer_by_integer_fractional_result() {
        let mut interp = setup_interpreter();

        // Test cases where integer division results in fractions (stays Rational)

        // 1 / 3 = 1/3
        interp.push(Value::Integer(BigInt::from(1)));
        interp.push(Value::Integer(BigInt::from(3)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(1));
            assert_eq!(*r.denom(), BigInt::from(3));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }

        // 7 / 4 = 7/4
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Integer(BigInt::from(4)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(7));
            assert_eq!(*r.denom(), BigInt::from(4));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }

        // 2 / 5 = 2/5
        interp.push(Value::Integer(BigInt::from(2)));
        interp.push(Value::Integer(BigInt::from(5)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(2));
            assert_eq!(*r.denom(), BigInt::from(5));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }

        // -3 / 7 = -3/7
        interp.push(Value::Integer(BigInt::from(-3)));
        interp.push(Value::Integer(BigInt::from(7)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(-3));
            assert_eq!(*r.denom(), BigInt::from(7));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn test_div_integer_by_integer_simplification() {
        let mut interp = setup_interpreter();

        // Test that results are automatically simplified

        // 2 / 4 = 1/2 (simplified from 2/4)
        interp.push(Value::Integer(BigInt::from(2)));
        interp.push(Value::Integer(BigInt::from(4)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(1));
            assert_eq!(*r.denom(), BigInt::from(2));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }

        // 6 / 9 = 2/3 (simplified from 6/9)
        interp.push(Value::Integer(BigInt::from(6)));
        interp.push(Value::Integer(BigInt::from(9)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(2));
            assert_eq!(*r.denom(), BigInt::from(3));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }

        // 15 / 25 = 3/5 (simplified from 15/25)
        interp.push(Value::Integer(BigInt::from(15)));
        interp.push(Value::Integer(BigInt::from(25)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(3));
            assert_eq!(*r.denom(), BigInt::from(5));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn test_div_mixed_integer_rational() {
        let mut interp = setup_interpreter();

        // Test Integer / Rational (should use promotion)
        // 10 / (1/2) = 20
        interp.push(Value::Integer(BigInt::from(10)));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(1),
            BigInt::from(2),
        )));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(20)));

        // Test Rational / Integer (should use promotion)
        // (3/4) / 2 = 3/8
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(4),
        )));
        interp.push(Value::Integer(BigInt::from(2)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(3));
            assert_eq!(*r.denom(), BigInt::from(8));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn test_div_rational_non_simplifying() {
        let mut interp = setup_interpreter();

        // Test Rational / Rational that doesn't simplify to integer
        // (2/3) / (4/5) = (2/3) * (5/4) = 10/12 = 5/6
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(2),
            BigInt::from(3),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(4),
            BigInt::from(5),
        )));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(5));
            assert_eq!(*r.denom(), BigInt::from(6));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn test_div_mixed_number_integer() {
        let mut interp = setup_interpreter();

        // Test Number / Integer (should promote to Number)
        // 10.5 / 2 = 5.25
        interp.push(Value::Number(10.5));
        interp.push(Value::Integer(BigInt::from(2)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.25));

        // Test Integer / Number (should promote to Number)
        // 15 / 2.0 = 7.5
        interp.push(Value::Integer(BigInt::from(15)));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.5));
    }

    #[test]
    fn test_div_mixed_number_rational() {
        let mut interp = setup_interpreter();

        // Test Number / Rational (should promote to Number)
        // 8.0 / (1/2) = 16.0
        interp.push(Value::Number(8.0));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(1),
            BigInt::from(2),
        )));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 16.0));

        // Test Rational / Number (should promote to Number)
        // (3/4) / 2.0 = 0.375
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(4),
        )));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 0.375).abs() < 1e-10));
    }

    #[test]
    fn test_div_complex_numbers() {
        use num_complex::Complex64;

        let mut interp = setup_interpreter();

        // Test Complex / Complex: (4+2i) / (1+1i) = (6+(-2)i) / 2 = 3-i
        interp.push(Value::Complex(Complex64::new(4.0, 2.0)));
        interp.push(Value::Complex(Complex64::new(1.0, 1.0)));
        div_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Complex(c) = result {
            assert!((c.re - 3.0).abs() < 1e-10);
            assert!((c.im - (-1.0)).abs() < 1e-10);
        } else {
            panic!("Expected Complex, got {:?}", result);
        }
    }

    #[test]
    fn test_div_large_integers() {
        let mut interp = setup_interpreter();

        // Test with large coprime integers (no common factors)
        // Using prime-like numbers to ensure they don't divide evenly
        let large1 = BigInt::parse_bytes(b"123456789012345678901234567891", 10).unwrap();
        let large2 = BigInt::parse_bytes(b"987654321098765432109876543211", 10).unwrap();

        interp.push(Value::Integer(large1.clone()));
        interp.push(Value::Integer(large2.clone()));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should be a rational since these don't divide evenly
        // The rational will be simplified, so we just check it's a Rational type
        assert!(matches!(result, Value::Rational(_)));
    }
}
