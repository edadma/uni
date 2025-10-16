// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_traits::{Zero, Float};

// RUST CONCEPT: Floor division with zero checking and type promotion
// Stack-based floor division: ( n1 n2 -- quotient )
// Like Python's //, always returns the floor of the quotient
pub fn floor_div_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Check for division by zero
    let is_zero = match &b {
        Value::Integer(i) => i.is_zero(),
        Value::Rational(r) => r.is_zero(),
        Value::Number(n) => *n == 0.0,
        _ => false,
    };
    if is_zero {
        return Err(RuntimeError::DivisionByZero);
    }

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Integer(i1), Value::Integer(i2)) => {
            // Integer division in Rust uses truncation, not floor
            // For floor division: floor(a/b) = (a - (a % b)) / b when signs differ
            let quotient = i1 / i2;
            let remainder = i1 % i2;

            // Adjust for floor semantics if signs differ and there's a remainder
            if (i1.sign() != i2.sign()) && !remainder.is_zero() {
                Value::Integer(quotient - 1)
            } else {
                Value::Integer(quotient)
            }
        }
        (Value::Rational(r1), Value::Rational(r2)) => {
            // For rationals, divide and take floor
            let division = r1 / r2;
            let floor_val = division.floor();
            Value::Rational(floor_val).demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number((n1 / n2).floor()),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot floor divide {:?} and {:?}",
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
    use num_bigint::BigInt;
    use num_rational::BigRational;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_floor_div_builtin_floats() {
        let mut interp = setup_interpreter();

        // Test basic floor division: 7.0 // 2.0 = 3.0
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(2.0));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test exact division: 10.0 // 2.0 = 5.0
        interp.push(Value::Number(10.0));
        interp.push(Value::Number(2.0));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with negative dividend: -7.0 // 2.0 = -4.0 (floor, not truncate)
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(2.0));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -4.0));

        // Test with negative divisor: 7.0 // -2.0 = -4.0
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(-2.0));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::Number(n) = result {
            // 7.0 / -2.0 = -3.5, floor = -4.0
            assert_eq!(n, -4.0, "Expected -4.0, got {}", n);
        } else {
            panic!("Expected Number");
        }

        // Test with both negative: -7.0 // -2.0 = 3.0
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(-2.0));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_floor_div_builtin_integers() {
        let mut interp = setup_interpreter();

        // Test Integer // Integer: 7 // 2 = 3
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Integer(BigInt::from(2)));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(3)));

        // Test exact division: 10 // 2 = 5
        interp.push(Value::Integer(BigInt::from(10)));
        interp.push(Value::Integer(BigInt::from(2)));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(5)));

        // Test with negative dividend: -7 // 2 = -4 (floor, not truncate)
        interp.push(Value::Integer(BigInt::from(-7)));
        interp.push(Value::Integer(BigInt::from(2)));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-4)));

        // Test with negative divisor: 7 // -2 = -4
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Integer(BigInt::from(-2)));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-4)));

        // Test with both negative: -7 // -2 = 3
        interp.push(Value::Integer(BigInt::from(-7)));
        interp.push(Value::Integer(BigInt::from(-2)));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(3)));
    }

    #[test]
    fn test_floor_div_builtin_rationals() {
        let mut interp = setup_interpreter();

        // Test Rational // Rational: (7/2) // (3/2) = 2
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(7),
            BigInt::from(2),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(2),
        )));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // (7/2) / (3/2) = 7/3 = 2.333..., floor = 2 (demoted to Int32)
        assert!(matches!(result, Value::Int32(2)));

        // Test (5/2) // (1/2) = 5
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(5),
            BigInt::from(2),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(1),
            BigInt::from(2),
        )));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // (5/2) / (1/2) = 5 (demoted to Int32)
        assert!(matches!(result, Value::Int32(5)));
    }

    #[test]
    fn test_floor_div_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test Integer // Number: 7 // 2.0 = 3.0
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Number(2.0));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test Rational // Integer: (7/2) // 2 = 1
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(7),
            BigInt::from(2),
        )));
        interp.push(Value::Integer(BigInt::from(2)));
        floor_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // (7/2) / 2 = 7/4 = 1.75, floor = 1 (demoted to Int32)
        assert!(matches!(result, Value::Int32(1)));
    }

    #[test]
    fn test_floor_div_builtin_division_by_zero() {
        let mut interp = setup_interpreter();

        // Test division by zero with floats
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));
        let result = floor_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));

        // Test division by zero with integers
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Integer(BigInt::from(0)));
        let result = floor_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_floor_div_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = floor_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = floor_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_floor_div_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::Null);
        let result = floor_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_floor_div_vs_regular_div() {
        let mut interp = setup_interpreter();

        // Verify floor division differs from truncation for negative numbers
        // -7 / 2 would be -3.5, truncation gives -3, floor gives -4
        interp.push(Value::Integer(BigInt::from(-7)));
        interp.push(Value::Integer(BigInt::from(2)));
        floor_div_builtin(&mut interp).unwrap();

        let floor_result = interp.pop().unwrap();
        assert!(matches!(floor_result, Value::Integer(ref i) if *i == BigInt::from(-4)));
    }
}
