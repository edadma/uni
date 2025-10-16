use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

pub fn ceil_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.ceil()));
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
    fn test_ceil_positive_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.2));

        ceil_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 4.0));
    }

    #[test]
    fn test_ceil_negative_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.7));

        ceil_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -3.0));
    }

    #[test]
    fn test_ceil_integer() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));

        ceil_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_ceil_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        ceil_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_ceil_very_small_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.1));

        ceil_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_ceil_very_small_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-0.1));

        ceil_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_ceil_large_numbers() {
        let mut interp = setup_interpreter();

        let test_cases = [(999.1, 1000.0), (-999.1, -999.0), (1e10 + 0.5, 1e10 + 1.0)];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            ceil_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < f64::EPSILON),
                "ceil({}) should be {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_ceil_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = ceil_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_ceil_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = ceil_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
