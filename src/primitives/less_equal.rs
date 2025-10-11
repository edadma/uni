use crate::interpreter::Interpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

pub fn less_equal_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Promote to common type for comparison
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Int32(i1), Value::Int32(i2)) => i1 <= i2,
        (Value::Integer(i1), Value::Integer(i2)) => i1 <= i2,
        (Value::Rational(r1), Value::Rational(r2)) => r1 <= r2,
        (Value::Number(n1), Value::Number(n2)) => n1 <= n2,
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot compare {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(Value::Boolean(result));
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
    fn test_less_equal_true_less() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));

        less_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_less_equal_true_equal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(5.0));

        less_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_less_equal_false() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(3.0));

        less_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_less_equal_negative_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-10.0));
        interp.push(Value::Number(-10.0));

        less_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_less_equal_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = less_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(1.0));
        let result = less_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_less_equal_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(5.0));

        let result = less_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
