// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_traits::Zero;
#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Truncating division with zero checking and type promotion
// Stack-based truncating division: ( n1 n2 -- quotient )
// Rounds toward zero (like C/Java/Rust integer division)
pub fn trunc_div_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Check for division by zero
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

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Int32(i1), Value::Int32(i2)) => {
            // Int32 division in Rust already truncates toward zero
            Value::Int32(i1 / i2)
        }
        (Value::Integer(i1), Value::Integer(i2)) => {
            // Integer division in Rust already truncates toward zero
            Value::Integer(i1 / i2)
        }
        (Value::Rational(r1), Value::Rational(r2)) => {
            // For rationals, divide and truncate toward zero
            let division = r1 / r2;
            let trunc_val = division.trunc();
            Value::Rational(trunc_val).demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 / n2).trunc(),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot truncating divide {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}

// RUST CONCEPT: Extension trait to add trunc method to Value
trait Truncate {
    fn trunc(self) -> Self;
}

impl Truncate for Value {
    fn trunc(self) -> Self {
        match self {
            Value::Number(n) => Value::Number(n.trunc()),
            other => other,
        }
    }
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
    fn test_trunc_div_builtin_floats() {
        let mut interp = setup_interpreter();

        // Test basic truncating division: 7.0 div 2.0 = 3.0
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(2.0));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test exact division: 10.0 div 2.0 = 5.0
        interp.push(Value::Number(10.0));
        interp.push(Value::Number(2.0));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with negative dividend: -7.0 div 2.0 = -3.0 (truncate toward zero)
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(2.0));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -3.0));

        // Test with negative divisor: 7.0 div -2.0 = -3.0
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(-2.0));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -3.0));

        // Test with both negative: -7.0 div -2.0 = 3.0
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(-2.0));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_trunc_div_builtin_integers() {
        let mut interp = setup_interpreter();

        // Test Integer div Integer: 7 div 2 = 3
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Integer(BigInt::from(2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(3)));

        // Test exact division: 10 div 2 = 5
        interp.push(Value::Integer(BigInt::from(10)));
        interp.push(Value::Integer(BigInt::from(2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(5)));

        // Test with negative dividend: -7 div 2 = -3 (truncate toward zero)
        interp.push(Value::Integer(BigInt::from(-7)));
        interp.push(Value::Integer(BigInt::from(2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-3)));

        // Test with negative divisor: 7 div -2 = -3
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Integer(BigInt::from(-2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-3)));

        // Test with both negative: -7 div -2 = 3
        interp.push(Value::Integer(BigInt::from(-7)));
        interp.push(Value::Integer(BigInt::from(-2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(3)));
    }

    #[test]
    fn test_trunc_div_builtin_rationals() {
        let mut interp = setup_interpreter();

        // Test Rational div Rational: (7/2) div (3/2) = 2
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(7),
            BigInt::from(2),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(2),
        )));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // (7/2) / (3/2) = 7/3 = 2.333..., trunc = 2 (demoted to Int32)
        assert!(matches!(result, Value::Int32(2)));

        // Test negative: (-7/2) div (3/2) = -2 (truncate toward zero, not floor)
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(-7),
            BigInt::from(2),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(2),
        )));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // (-7/2) / (3/2) = -7/3 = -2.333..., trunc = -2 (not floor which is -3, demoted to Int32)
        assert!(matches!(result, Value::Int32(-2)));
    }

    #[test]
    fn test_trunc_div_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test Integer div Number: 7 div 2.0 = 3.0
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Number(2.0));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test Rational div Integer: (7/2) div 2 = 1
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(7),
            BigInt::from(2),
        )));
        interp.push(Value::Integer(BigInt::from(2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // (7/2) / 2 = 7/4 = 1.75, trunc = 1 (demoted to Int32)
        assert!(matches!(result, Value::Int32(1)));
    }

    #[test]
    fn test_trunc_div_vs_floor_div() {
        let mut interp = setup_interpreter();

        // Key difference: with negative results
        // -7 div 2: truncate toward zero = -3
        // -7 // 2: floor = -4

        // Test truncate: -7 div 2 = -3
        interp.push(Value::Integer(BigInt::from(-7)));
        interp.push(Value::Integer(BigInt::from(2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-3)));

        // Compare with positive: 7 div 2 = 3 (same magnitude)
        interp.push(Value::Integer(BigInt::from(7)));
        interp.push(Value::Integer(BigInt::from(2)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(3)));
    }

    #[test]
    fn test_trunc_div_builtin_division_by_zero() {
        let mut interp = setup_interpreter();

        // Test division by zero with floats
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));
        let result = trunc_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));

        // Test division by zero with integers
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Integer(BigInt::from(0)));
        let result = trunc_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));
    }

    #[test]
    fn test_trunc_div_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = trunc_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = trunc_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_trunc_div_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::Null);
        let result = trunc_div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_trunc_div_large_numbers() {
        let mut interp = setup_interpreter();

        // Test with large integers
        interp.push(Value::Integer(BigInt::from(1000000)));
        interp.push(Value::Integer(BigInt::from(3)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(333333)));

        // Test with large negative
        interp.push(Value::Integer(BigInt::from(-1000000)));
        interp.push(Value::Integer(BigInt::from(3)));
        trunc_div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-333333)));
    }
}
