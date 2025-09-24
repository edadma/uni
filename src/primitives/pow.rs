use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn pow_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let exponent = interp.pop_number()?;
    let base = interp.pop_number()?;

    let result = base.powf(exponent);

    // Check for invalid results (NaN or infinite)
    if result.is_nan() {
        return Err(RuntimeError::DomainError("pow result is NaN".to_string()));
    }
    if result.is_infinite() {
        return Err(RuntimeError::DomainError("pow result is infinite".to_string()));
    }

    interp.push(Value::Number(result));
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
    fn test_pow_positive_integers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));
    }

    #[test]
    fn test_pow_zero_exponent() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_pow_one_base() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(100.0));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_pow_negative_exponent() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(-2.0));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 0.25).abs() < f64::EPSILON));
    }

    #[test]
    fn test_pow_fractional_exponent() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(4.0));
        interp.push(Value::Number(0.5));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 2.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_pow_negative_base_integer_exponent() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-2.0));
        interp.push(Value::Number(3.0));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -8.0));
    }

    #[test]
    fn test_pow_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = pow_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(2.0));
        let result = pow_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_pow_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(2.0));

        let result = pow_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_pow_square_root() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(9.0));
        interp.push(Value::Number(0.5));

        pow_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 3.0).abs() < f64::EPSILON));
    }
}