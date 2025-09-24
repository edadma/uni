use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn shl_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let shift_amount = interp.pop_number()?;
    let value = interp.pop_number()?;

    // Convert to integers for shift operations
    let value_int = value as i64;
    let shift_int = shift_amount as u32;

    // Check for reasonable shift amounts to prevent overflow
    if shift_int > 63 {
        return Err(RuntimeError::DomainError("shift amount too large".to_string()));
    }

    let result = value_int << shift_int;
    interp.push(Value::Number(result as f64));
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
    fn test_shl_basic() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(3.0)); // Shift left by 3

        shl_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0)); // 1 << 3 = 8
    }

    #[test]
    fn test_shl_zero_shift() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0)); // Shift left by 0

        shl_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0)); // No change
    }

    #[test]
    fn test_shl_powers_of_two() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (1.0, 1.0, 2.0),   // 1 << 1 = 2
            (1.0, 2.0, 4.0),   // 1 << 2 = 4
            (1.0, 4.0, 16.0),  // 1 << 4 = 16
            (1.0, 8.0, 256.0), // 1 << 8 = 256
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shl_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} << {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shl_multiple_bits() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (3.0, 2.0, 12.0),  // 11 << 2 = 1100 = 12
            (5.0, 1.0, 10.0),  // 101 << 1 = 1010 = 10
            (7.0, 3.0, 56.0),  // 111 << 3 = 111000 = 56
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shl_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} << {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shl_zero_value() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));
        interp.push(Value::Number(5.0)); // Shift any amount

        shl_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0)); // 0 shifted is still 0
    }

    #[test]
    fn test_shl_negative_numbers() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (-1.0, 1.0, -2.0),   // -1 << 1 = -2
            (-2.0, 2.0, -8.0),   // -2 << 2 = -8
            (-4.0, 1.0, -8.0),   // -4 << 1 = -8
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shl_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} << {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shl_large_shifts() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (1.0, 10.0, 1024.0), // 1 << 10 = 1024
            (1.0, 20.0, 1048576.0), // 1 << 20 = 2^20
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shl_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} << {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shl_fractional_truncation() {
        let mut interp = setup_interpreter();

        // Both value and shift should be truncated
        interp.push(Value::Number(3.7));
        interp.push(Value::Number(2.9)); // Should truncate to 2

        shl_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 12.0)); // 3 << 2 = 12
    }

    #[test]
    fn test_shl_multiplication_equivalence() {
        let mut interp = setup_interpreter();

        // Left shift by n is equivalent to multiplication by 2^n
        let test_cases = [
            (5.0, 1.0),  // 5 << 1 = 5 * 2 = 10
            (7.0, 2.0),  // 7 << 2 = 7 * 4 = 28
            (3.0, 4.0),  // 3 << 4 = 3 * 16 = 48
        ];

        for (value, shift) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shl_builtin(&mut interp).unwrap();
            let shift_result = interp.pop().unwrap();

            let expected = value * (2.0_f64.powf(shift));

            assert!(matches!(shift_result, Value::Number(n) if n == expected),
                   "{} << {} should equal {} * 2^{} = {}, got {:?}",
                   value, shift, value, shift, expected, shift_result);
        }
    }

    #[test]
    fn test_shl_excessive_shift_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(100.0)); // Shift amount too large

        let result = shl_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_shl_boundary_shift() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(63.0)); // Maximum allowed shift

        let result = shl_builtin(&mut interp);
        // Should succeed (might overflow but shouldn't error)
        assert!(result.is_ok());
    }

    #[test]
    fn test_shl_bit_pattern_preservation() {
        let mut interp = setup_interpreter();

        // Test that bit patterns are preserved correctly
        interp.push(Value::Number(170.0)); // 10101010 in binary
        interp.push(Value::Number(1.0));   // Shift left by 1

        shl_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 340.0)); // 101010100 in binary
    }

    #[test]
    fn test_shl_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = shl_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(5.0));
        let result = shl_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_shl_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(2.0));

        let result = shl_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}