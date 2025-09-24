use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn abs_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.abs()));
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
    fn test_abs_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_abs_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-5.0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_abs_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_abs_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.14));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_abs_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = abs_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_abs_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = abs_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}