use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn cos_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.cos()));
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
    fn test_cos_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        cos_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_cos_pi_half() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI / 2.0));

        cos_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < 1e-15)); // Very small due to floating point precision
    }

    #[test]
    fn test_cos_pi() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI));

        cos_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n + 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_cos_three_pi_half() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.0 * PI / 2.0));

        cos_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < 1e-15));
    }

    #[test]
    fn test_cos_two_pi() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(2.0 * PI));

        cos_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_cos_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-PI / 2.0));

        cos_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < 1e-15));
    }

    #[test]
    fn test_cos_arbitrary_values() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (PI / 6.0, 3.0_f64.sqrt() / 2.0), // cos(30°) = √3/2
            (PI / 4.0, 2.0_f64.sqrt() / 2.0), // cos(45°) = √2/2
            (PI / 3.0, 0.5),                  // cos(60°) = 0.5
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            cos_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if (n - expected).abs() < 1e-15),
                   "cos({}) should be approximately {}, got {:?}", input, expected, result);
        }
    }

    #[test]
    fn test_cos_symmetry() {
        let mut interp = setup_interpreter();

        // cos(-x) = cos(x)
        let angle = PI / 3.0;

        interp.push(Value::Number(angle));
        cos_builtin(&mut interp).unwrap();
        let positive_result = interp.pop().unwrap();

        interp.push(Value::Number(-angle));
        cos_builtin(&mut interp).unwrap();
        let negative_result = interp.pop().unwrap();

        if let (Value::Number(pos), Value::Number(neg)) = (positive_result, negative_result) {
            assert!((pos - neg).abs() < f64::EPSILON, "cos should be even function");
        }
    }

    #[test]
    fn test_cos_large_values() {
        let mut interp = setup_interpreter();

        // Test that cos works for large values (should still be in [-1, 1])
        interp.push(Value::Number(1000.0));
        cos_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n >= -1.0 && n <= 1.0));
    }

    #[test]
    fn test_cos_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = cos_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_cos_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = cos_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}