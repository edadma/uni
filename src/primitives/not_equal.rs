use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use core::ptr;

pub fn not_equal_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Use the same equality logic as equals.rs but negate result
    let are_equal = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => i1 == i2,
        (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
        (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
        (Value::Rational(r1), Value::Rational(r2)) => r1 == r2,
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(re1, im1), Value::GaussianInt(re2, im2)) => re1 == re2 && im1 == im2,
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => c1 == c2,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Atom(a), Value::Atom(b)) => a == b,
        (Value::QuotedAtom(a), Value::QuotedAtom(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::Nil, Value::Nil) => true,
        (Value::Pair(a1, a2), Value::Pair(b1, b2)) => {
            // Recursive equality for lists - would need helper function
            // For now, just check reference equality (shallow)
            ptr::eq(a1.as_ref(), b1.as_ref()) && ptr::eq(a2.as_ref(), b2.as_ref())
        }
        _ => false, // Different types are not equal
    };

    interp.push(Value::Boolean(!are_equal));
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
    fn test_not_equal_numbers_true() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(7.0));

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_not_equal_numbers_false() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(5.0));

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_not_equal_strings_true() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::String("world".into()));

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_not_equal_strings_false() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::String("hello".into()));

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_not_equal_different_types() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::String("5".into()));

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_not_equal_booleans() {
        let mut interp = setup_interpreter();
        interp.push(Value::Boolean(true));
        interp.push(Value::Boolean(false));

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_not_equal_null_values() {
        let mut interp = setup_interpreter();
        interp.push(Value::Null);
        interp.push(Value::Null);

        not_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_not_equal_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = not_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(1.0));
        let result = not_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}
