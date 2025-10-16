use crate::compat::ToString;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

pub fn tan_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    let result = n.tan();

    // Check for invalid results (infinite values occur at odd multiples of π/2)
    if result.is_infinite() {
        return Err(RuntimeError::DomainError(
            "tan is undefined (infinite)".to_string(),
        ));
    }

    interp.push(Value::Number(result));
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
    fn test_tan_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        tan_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < f64::EPSILON));
    }

    #[test]
    fn test_tan_pi() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI));

        tan_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n.abs() < 1e-15)); // Should be very close to 0
    }

    #[test]
    fn test_tan_pi_quarter() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI / 4.0));

        tan_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_tan_negative_pi_quarter() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-PI / 4.0));

        tan_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n + 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn test_tan_arbitrary_values() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (PI / 6.0, 1.0 / 3.0_f64.sqrt()), // tan(30°) = 1/√3
            (PI / 3.0, 3.0_f64.sqrt()),       // tan(60°) = √3
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            tan_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if (n - expected).abs() < 1e-15),
                "tan({}) should be approximately {}, got {:?}",
                input,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_tan_antisymmetry() {
        let mut interp = setup_interpreter();

        // tan(-x) = -tan(x)
        let angle = PI / 6.0;

        interp.push(Value::Number(angle));
        tan_builtin(&mut interp).unwrap();
        let positive_result = interp.pop().unwrap();

        interp.push(Value::Number(-angle));
        tan_builtin(&mut interp).unwrap();
        let negative_result = interp.pop().unwrap();

        if let (Value::Number(pos), Value::Number(neg)) = (positive_result, negative_result) {
            assert!(
                (pos + neg).abs() < f64::EPSILON,
                "tan should be odd function"
            );
        }
    }

    #[test]
    fn test_tan_near_asymptotes() {
        let mut interp = setup_interpreter();

        // Test values very close to π/2 where tan approaches infinity
        // We'll use values that are close but not exactly π/2 to avoid the infinite check
        let near_pi_half = PI / 2.0 - 1e-10;
        interp.push(Value::Number(near_pi_half));
        tan_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();

        // Should be a very large positive number
        assert!(matches!(result, Value::Number(n) if n > 1e9));
    }

    #[test]
    fn test_tan_exactly_pi_half() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(PI / 2.0));

        // Due to floating point precision, tan(π/2) might not be exactly infinite
        // Let's just check that we get a very large number
        let result = tan_builtin(&mut interp);
        if let Ok(()) = result {
            let value = interp.pop().unwrap();
            if let Value::Number(n) = value {
                assert!(
                    n.abs() > 1e10,
                    "tan(π/2) should be a very large number, got {}",
                    n
                );
            }
        }
        // If we get a domain error, that's also acceptable
    }

    #[test]
    fn test_tan_three_pi_half() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(3.0 * PI / 2.0));

        // Due to floating point precision, tan(3π/2) might not be exactly infinite
        // Let's just check that we get a very large number
        let result = tan_builtin(&mut interp);
        if let Ok(()) = result {
            let value = interp.pop().unwrap();
            if let Value::Number(n) = value {
                assert!(
                    n.abs() > 1e10,
                    "tan(3π/2) should be a very large number, got {}",
                    n
                );
            }
        }
        // If we get a domain error, that's also acceptable
    }

    #[test]
    fn test_tan_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = tan_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_tan_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = tan_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_tan_periodicity() {
        let mut interp = setup_interpreter();

        // tan has period π, so tan(x) = tan(x + π)
        let angle = PI / 6.0;

        interp.push(Value::Number(angle));
        tan_builtin(&mut interp).unwrap();
        let first_result = interp.pop().unwrap();

        interp.push(Value::Number(angle + PI));
        tan_builtin(&mut interp).unwrap();
        let second_result = interp.pop().unwrap();

        if let (Value::Number(first), Value::Number(second)) = (first_result, second_result) {
            assert!((first - second).abs() < 1e-15, "tan should have period π");
        }
    }
}
