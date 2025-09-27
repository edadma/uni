use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn exp_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    let result = n.exp();

    // Check for overflow (infinite result)
    if result.is_infinite() {
        return Err(RuntimeError::DomainError(
            "exp result is infinite (overflow)".to_string(),
        ));
    }

    interp.push(Value::Number(result));
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
    fn test_exp_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        exp_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_exp_one() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1.0));

        exp_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - E).abs() < f64::EPSILON));
    }

    #[test]
    fn test_exp_two() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0));

        exp_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        let expected = E * E;
        assert!(matches!(result, Value::Number(n) if (n - expected).abs() < 1e-14));
    }

    #[test]
    fn test_exp_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-1.0));

        exp_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        let expected = 1.0 / E;
        assert!(matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON));
    }

    #[test]
    fn test_exp_integer_values() {
        let mut interp = setup_interpreter();

        let test_cases = [(-2.0, 1.0 / (E * E)), (0.5, E.sqrt()), (3.0, E.powf(3.0))];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            exp_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < 1e-14),
                "exp({}) should be approximately {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_exp_small_values() {
        let mut interp = setup_interpreter();

        let test_values = [0.1, -0.1, 0.01, -0.01];

        for value in test_values {
            interp.push(Value::Number(value));
            exp_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            let expected = value.exp();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON),
                "exp({}) should be {}, got {:?}",
                value,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_exp_large_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-100.0));

        exp_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should be very close to zero but not exactly zero
        assert!(matches!(result, Value::Number(n) if n > 0.0 && n < 1e-40));
    }

    #[test]
    fn test_exp_overflow() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1000.0)); // Very large value that will cause overflow

        let result = exp_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_exp_near_overflow_boundary() {
        let mut interp = setup_interpreter();

        // Test value that's large but doesn't overflow
        interp.push(Value::Number(700.0));
        let result = exp_builtin(&mut interp);

        // This might succeed or fail depending on the exact boundary,
        // but it shouldn't panic
        match result {
            Ok(_) => {
                // If it succeeds, the result should be finite
                let value = interp.pop().unwrap();
                if let Value::Number(n) = value {
                    assert!(n.is_finite());
                }
            }
            Err(RuntimeError::DomainError(_)) => {
                // Overflow is acceptable for large values
            }
            _ => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn test_exp_log_inverse() {
        let mut interp = setup_interpreter();

        // Test that exp(log(x)) ≈ x for positive values
        let test_values: [f64; 5] = [1.0, 2.0, 10.0, 0.5, 0.1];

        for value in test_values {
            let log_value = value.ln();
            interp.push(Value::Number(log_value));
            exp_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();

            assert!(
                matches!(result, Value::Number(n) if (n - value).abs() < 1e-14),
                "exp(log({})) should be approximately {}, got {:?}",
                value,
                value,
                result
            );
        }
    }

    #[test]
    fn test_exp_additivity() {
        let mut interp = setup_interpreter();

        // Test that exp(a + b) = exp(a) * exp(b)
        let a = 1.5;
        let b = 2.3;

        // Calculate exp(a + b)
        interp.push(Value::Number(a + b));
        exp_builtin(&mut interp).unwrap();
        let combined = interp.pop().unwrap();

        // Calculate exp(a) * exp(b)
        interp.push(Value::Number(a));
        exp_builtin(&mut interp).unwrap();
        let exp_a = interp.pop().unwrap();

        interp.push(Value::Number(b));
        exp_builtin(&mut interp).unwrap();
        let exp_b = interp.pop().unwrap();

        if let (Value::Number(combined_n), Value::Number(exp_a_n), Value::Number(exp_b_n)) =
            (combined, exp_a, exp_b)
        {
            let product = exp_a_n * exp_b_n;
            assert!(
                (combined_n - product).abs() < 1e-14,
                "exp({} + {}) should equal exp({}) * exp({})",
                a,
                b,
                a,
                b
            );
        }
    }

    #[test]
    fn test_exp_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = exp_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_exp_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = exp_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
