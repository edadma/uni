use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn sin_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.sin()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use std::f64::consts::PI;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_sin_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        sin_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < f64::EPSILON));
    }

    #[test]
    fn test_sin_pi_half() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI / 2.0));

        sin_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_sin_pi() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI));

        sin_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < 1e-15)); // Very small due to floating point precision
    }

    #[test]
    fn test_sin_three_pi_half() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.0 * PI / 2.0));

        sin_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n + 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_sin_two_pi() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0 * PI));

        sin_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < 1e-15));
    }

    #[test]
    fn test_sin_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-PI / 2.0));

        sin_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n + 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_sin_arbitrary_values() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (PI / 6.0, 0.5),                  // sin(30°) = 0.5
            (PI / 4.0, 2.0_f64.sqrt() / 2.0), // sin(45°) = √2/2
            (PI / 3.0, 3.0_f64.sqrt() / 2.0), // sin(60°) = √3/2
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            sin_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < 1e-15),
                "sin({}) should be approximately {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_sin_large_values() {
        let mut interp = setup_interpreter();

        // Test that sin works for large values (should still be in [-1, 1])
        interp.push(Value::Number(1000.0));
        sin_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n >= -1.0 && n <= 1.0));
    }

    #[test]
    fn test_sin_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = sin_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_sin_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = sin_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
