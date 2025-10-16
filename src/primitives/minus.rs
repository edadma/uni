// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Arithmetic operations with stack semantics
// Stack-based subtraction: ( n1 n2 -- difference )
// Supports all numeric types with automatic promotion
pub fn sub_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    // Perform subtraction based on the promoted type
    let result = match (&pa, &pb) {
        // RUST CONCEPT: Fast path for Int32 - checked arithmetic for embedded safety
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_sub(*i2) {
                Some(result) => Value::Int32(result),
                // Overflow: promote to BigInt
                None => Value::Integer(num_bigint::BigInt::from(*i1) - num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 - i2),
        (Value::Rational(r1), Value::Rational(r2)) => {
            let result = Value::Rational(r1 - r2);
            result.demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 - n2),
        (Value::GaussianInt(re1, im1), Value::GaussianInt(re2, im2)) => {
            let result = Value::GaussianInt(re1 - re2, im1 - im2);
            result.demote()
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 - c2),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot subtract {:?} and {:?}",
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
    fn test_sub_builtin() {
        let mut interp = setup_interpreter();

        // Test basic subtraction: 8 - 3 = 5
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(3.0));
        sub_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with negative result: 3 - 8 = -5
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(8.0));
        sub_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -5.0));

        // Test with zero
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0));
        sub_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_sub_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = sub_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = sub_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_sub_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::Boolean(false));
        let result = sub_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
