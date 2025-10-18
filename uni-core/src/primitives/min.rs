use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn min_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    interp.push(Value::Number(a.min(b)));
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
    fn test_min_first_smaller() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(7.0));

        min_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_min_second_smaller() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(2.0));

        min_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_min_equal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(5.0));

        min_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_min_negative_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-10.0));
        interp.push(Value::Number(-5.0));

        min_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -10.0));
    }

    #[test]
    fn test_min_mixed_signs() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.0));
        interp.push(Value::Number(5.0));

        min_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -3.0));
    }

    #[test]
    fn test_min_decimals() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.14));
        interp.push(Value::Number(2.71));

        min_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 2.71).abs() < f64::EPSILON));
    }

    #[test]
    fn test_min_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = min_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(1.0));
        let result = min_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_min_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(5.0));

        let result = min_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
