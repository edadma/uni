use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn max_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    interp.push(Value::Number(a.max(b)));
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
    fn test_max_first_larger() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(3.0));

        max_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.0));
    }

    #[test]
    fn test_max_second_larger() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(8.0));

        max_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_max_equal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(5.0));

        max_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_max_negative_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-10.0));
        interp.push(Value::Number(-5.0));

        max_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -5.0));
    }

    #[test]
    fn test_max_mixed_signs() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.0));
        interp.push(Value::Number(5.0));

        max_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_max_decimals() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.14));
        interp.push(Value::Number(2.71));

        max_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_max_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = max_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(1.0));
        let result = max_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_max_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::String("hello".into()));

        let result = max_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}