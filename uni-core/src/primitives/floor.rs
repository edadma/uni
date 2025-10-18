use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

pub fn floor_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.floor()));
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
    fn test_floor_positive_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.7));

        floor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));
    }

    #[test]
    fn test_floor_negative_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.2));

        floor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -4.0));
    }

    #[test]
    fn test_floor_integer() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));

        floor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_floor_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        floor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_floor_very_small_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.1));

        floor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_floor_very_small_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-0.1));

        floor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0));
    }

    #[test]
    fn test_floor_large_numbers() {
        let mut interp = setup_interpreter();

        let test_cases = [(999.9, 999.0), (-999.9, -1000.0), (1e10 + 0.5, 1e10)];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            floor_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON),
                "floor({}) should be {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_floor_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = floor_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_floor_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = floor_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
