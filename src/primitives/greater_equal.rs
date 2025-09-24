use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn greater_equal_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    interp.push(Value::Boolean(a >= b));
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
    fn test_greater_equal_true_greater() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(3.0));

        greater_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_greater_equal_true_equal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(5.0));

        greater_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_greater_equal_false() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(7.0));

        greater_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_greater_equal_negative_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-5.0));
        interp.push(Value::Number(-5.0));

        greater_equal_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_greater_equal_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = greater_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(1.0));
        let result = greater_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_greater_equal_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::String("hello".into()));

        let result = greater_equal_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}