use crate::compat::ToString;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use num_traits::Float;

pub fn log_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;

    if n <= 0.0 {
        return Err(RuntimeError::DomainError(
            "log of non-positive number".to_string(),
        ));
    }

    interp.push(Value::Number(n.ln()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use std::f64::consts::E;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_log_e() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(E));

        log_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_log_one() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1.0));

        log_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < f64::EPSILON));
    }

    #[test]
    fn test_log_e_squared() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(E * E));

        log_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 2.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_log_powers_of_e() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (E.powf(0.5), 0.5),
            (E.powf(2.0), 2.0),
            (E.powf(3.0), 3.0),
            (E.powf(-1.0), -1.0),
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            log_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < 1e-14),
                "log({}) should be approximately {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_log_large_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1000.0));

        log_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        let expected = 1000.0_f64.ln();
        assert!(matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON));
    }

    #[test]
    fn test_log_small_positive_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.001));

        log_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        let expected = 0.001_f64.ln();
        assert!(matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON));
    }

    #[test]
    fn test_log_zero_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        let result = log_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_log_negative_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-1.0));

        let result = log_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_log_very_small_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(f64::MIN_POSITIVE));

        log_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should be a very large negative number
        assert!(matches!(result, Value::Number(n) if n < -700.0));
    }

    #[test]
    fn test_log_exp_inverse() {
        let mut interp = setup_interpreter();

        // Test that log(exp(x)) ≈ x for reasonable values
        let test_values: [f64; 5] = [0.0, 1.0, 2.0, -1.0, -2.0];

        for value in test_values {
            let exp_value = value.exp();
            interp.push(Value::Number(exp_value));
            log_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();

            assert!(
                matches!(result, Value::Number(n) if (n - value).abs() < 1e-14),
                "log(exp({})) should be approximately {}, got {:?}",
                value,
                value,
                result
            );
        }
    }

    #[test]
    fn test_log_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = log_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_log_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = log_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
