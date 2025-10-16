use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

pub fn round_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.round()));
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
    fn test_round_positive_up() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.7));

        round_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 4.0));
    }

    #[test]
    fn test_round_positive_down() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.3));

        round_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_round_negative_up() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.3));

        round_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -3.0));
    }

    #[test]
    fn test_round_negative_down() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.7));

        round_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -4.0));
    }

    #[test]
    fn test_round_half_values() {
        let mut interp = setup_interpreter();

        // Rust's f64::round() uses "round half away from zero" strategy
        let test_cases = [
            (0.5, 1.0),
            (1.5, 2.0),
            (2.5, 3.0),
            (-0.5, -1.0),
            (-1.5, -2.0),
            (-2.5, -3.0),
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            round_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if n == expected),
                "round({}) should be {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_round_integer() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));

        round_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_round_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        round_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_round_very_small() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (0.1, 0.0),
            (0.4, 0.0),
            (0.6, 1.0),
            (-0.1, 0.0),
            (-0.4, 0.0),
            (-0.6, -1.0),
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            round_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if n == expected),
                "round({}) should be {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_round_large_numbers() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (999.4, 999.0),
            (999.6, 1000.0),
            (-999.4, -999.0),
            (-999.6, -1000.0),
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            round_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON),
                "round({}) should be {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_round_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = round_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_round_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = round_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
