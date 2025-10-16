use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use crate::compat::ToString;

pub fn sqrt_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;

    if n < 0.0 {
        return Err(RuntimeError::DomainError(
            "sqrt of negative number".to_string(),
        ));
    }

    interp.push(Value::Number(n.sqrt()));
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
    fn test_sqrt_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(9.0));

        sqrt_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_sqrt_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        sqrt_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_sqrt_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0));

        sqrt_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        let expected = 2.0_f64.sqrt();
        assert!(matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON));
    }

    #[test]
    fn test_sqrt_large_number() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(100.0));

        sqrt_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }

    #[test]
    fn test_sqrt_negative_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-4.0));

        let result = sqrt_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_sqrt_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = sqrt_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_sqrt_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = sqrt_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_sqrt_perfect_squares() {
        let mut interp = setup_interpreter();

        let test_cases = [(4.0, 2.0), (16.0, 4.0), (25.0, 5.0), (49.0, 7.0)];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            sqrt_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON));
        }
    }
}
