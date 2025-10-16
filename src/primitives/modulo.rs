// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_traits::Zero;

// RUST CONCEPT: Modulo operation with zero checking and type promotion
// Stack-based modulo: ( n1 n2 -- remainder )
// Supports all numeric types with automatic promotion
pub fn mod_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Check for modulo by zero
    let is_zero = match &b {
        Value::Int32(i) => *i == 0,
        Value::Integer(i) => i.is_zero(),
        Value::Rational(r) => r.is_zero(),
        Value::Number(n) => *n == 0.0,
        _ => false,
    };
    if is_zero {
        return Err(RuntimeError::ModuloByZero);
    }

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Int32(i1), Value::Int32(i2)) => Value::Int32(i1 % i2),
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 % i2),
        (Value::Rational(r1), Value::Rational(r2)) => {
            let result = Value::Rational(r1 % r2);
            result.demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 % n2),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot compute modulo of {:?} and {:?}",
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
    fn test_mod_builtin_floats() {
        let mut interp = setup_interpreter();

        // Test basic modulo: 13.0 % 5.0 = 3.0
        interp.push(Value::Number(13.0));
        interp.push(Value::Number(5.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test with exact division: 12.0 % 4.0 = 0.0
        interp.push(Value::Number(12.0));
        interp.push(Value::Number(4.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));

        // Test with negative numbers: -7.0 % 3.0
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(3.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::Number(n) = result {
            // Rust's % follows the dividend's sign
            assert_eq!(n, -1.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_mod_builtin_integers() {
        let mut interp = setup_interpreter();

        // Test Integer % Integer: 13 % 5 = 3
        interp.push(Value::Integer(BigInt::from(13)));
        interp.push(Value::Integer(BigInt::from(5)));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(3)));

        // Test with exact division: 12 % 4 = 0
        interp.push(Value::Integer(BigInt::from(12)));
        interp.push(Value::Integer(BigInt::from(4)));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(0)));

        // Test with negative: -17 % 5 = -2
        interp.push(Value::Integer(BigInt::from(-17)));
        interp.push(Value::Integer(BigInt::from(5)));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if *i == BigInt::from(-2)));
    }

    #[test]
    fn test_mod_builtin_rationals() {
        let mut interp = setup_interpreter();

        // Test Rational % Rational: (7/2) % (3/2) = 1/2
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(7),
            BigInt::from(2),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(2),
        )));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(1));
            assert_eq!(*r.denom(), BigInt::from(2));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn test_mod_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test Integer % Number: 13 % 5.0 = 3.0
        interp.push(Value::Integer(BigInt::from(13)));
        interp.push(Value::Number(5.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test Rational % Integer: (7/2) % 2 = 3/2
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(7),
            BigInt::from(2),
        )));
        interp.push(Value::Integer(BigInt::from(2)));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::Rational(r) = result {
            assert_eq!(*r.numer(), BigInt::from(3));
            assert_eq!(*r.denom(), BigInt::from(2));
        } else {
            panic!("Expected Rational, got {:?}", result);
        }
    }

    #[test]
    fn test_mod_builtin_modulo_by_zero() {
        let mut interp = setup_interpreter();

        // Test modulo by zero with floats
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::ModuloByZero)));

        // Test modulo by zero with integers
        interp.push(Value::Integer(BigInt::from(5)));
        interp.push(Value::Integer(BigInt::from(0)));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::ModuloByZero)));

        // Test modulo by zero with rationals
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(5),
            BigInt::from(1),
        )));
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(0),
            BigInt::from(1),
        )));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::ModuloByZero)));
    }

    #[test]
    fn test_mod_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_mod_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::String("not a number".into()));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
