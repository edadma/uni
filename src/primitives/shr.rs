use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn shr_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let shift_amount = interp.pop_number()?;
    let value = interp.pop_number()?;

    // Convert to integers for shift operations
    let value_int = value as i64;
    let shift_int = shift_amount as u32;

    // Check for reasonable shift amounts
    if shift_int > 63 {
        return Err(RuntimeError::DomainError("shift amount too large".to_string()));
    }

    // Use arithmetic right shift (preserves sign bit)
    let result = value_int >> shift_int;
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
    fn test_shr_basic() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(3.0)); // Shift right by 3

        shr_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0)); // 8 >> 3 = 1
    }

    #[test]
    fn test_shr_zero_shift() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0)); // Shift right by 0

        shr_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0)); // No change
    }

    #[test]
    fn test_shr_powers_of_two() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (16.0, 1.0, 8.0),   // 16 >> 1 = 8
            (32.0, 2.0, 8.0),   // 32 >> 2 = 8
            (64.0, 4.0, 4.0),   // 64 >> 4 = 4
            (256.0, 8.0, 1.0),  // 256 >> 8 = 1
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shr_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} >> {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shr_multiple_bits() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (12.0, 2.0, 3.0),   // 1100 >> 2 = 11 = 3
            (10.0, 1.0, 5.0),   // 1010 >> 1 = 101 = 5
            (56.0, 3.0, 7.0),   // 111000 >> 3 = 111 = 7
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shr_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} >> {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shr_zero_value() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));
        interp.push(Value::Number(5.0)); // Shift any amount

        shr_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0)); // 0 shifted is still 0
    }

    #[test]
    fn test_shr_negative_numbers_arithmetic() {
        let mut interp = setup_interpreter();

        // Arithmetic right shift preserves sign (fills with sign bit)
        let test_cases = [
            (-8.0, 1.0, -4.0),   // -8 >> 1 = -4
            (-4.0, 2.0, -1.0),   // -4 >> 2 = -1
            (-1.0, 1.0, -1.0),   // -1 >> 1 = -1 (sign extension)
            (-2.0, 1.0, -1.0),   // -2 >> 1 = -1
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shr_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} >> {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shr_large_shifts() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (1024.0, 10.0, 1.0),     // 1024 >> 10 = 1
            (1048576.0, 20.0, 1.0),  // 2^20 >> 20 = 1
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shr_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} >> {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shr_fractional_truncation() {
        let mut interp = setup_interpreter();

        // Both value and shift should be truncated
        interp.push(Value::Number(15.7));
        interp.push(Value::Number(2.9)); // Should truncate to 2

        shr_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0)); // 15 >> 2 = 3
    }

    #[test]
    fn test_shr_division_equivalence_positive() {
        let mut interp = setup_interpreter();

        // Right shift by n is roughly equivalent to division by 2^n for positive numbers
        let test_cases = [
            (20.0, 2.0),  // 20 >> 2 = 5 (20 / 4 = 5)
            (48.0, 4.0),  // 48 >> 4 = 3 (48 / 16 = 3)
        ];

        for (value, shift) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shr_builtin(&mut interp).unwrap();
            let shift_result = interp.pop().unwrap();

            let expected = (value / (2.0_f64.powf(shift))).floor();

            assert!(matches!(shift_result, Value::Number(n) if n == expected),
                   "{} >> {} should equal floor({} / 2^{}) = {}, got {:?}",
                   value, shift, value, shift, expected, shift_result);
        }
    }

    #[test]
    fn test_shr_sign_extension() {
        let mut interp = setup_interpreter();

        // Test that negative numbers maintain their sign with large shifts
        interp.push(Value::Number(-100.0));
        interp.push(Value::Number(50.0)); // Very large shift

        shr_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0)); // Should be -1 due to sign extension
    }

    #[test]
    fn test_shr_excessive_shift_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(100.0)); // Shift amount too large

        let result = shr_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_shr_boundary_shift() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1024.0));
        interp.push(Value::Number(63.0)); // Maximum allowed shift

        let result = shr_builtin(&mut interp);
        // Should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_shr_bit_pattern() {
        let mut interp = setup_interpreter();

        // Test specific bit pattern
        interp.push(Value::Number(340.0)); // 101010100 in binary
        interp.push(Value::Number(2.0));   // Shift right by 2

        shr_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 85.0)); // 1010101 in binary
    }

    #[test]
    fn test_shr_odd_numbers() {
        let mut interp = setup_interpreter();

        // Test that odd numbers are handled correctly (truncation behavior)
        let test_cases = [
            (15.0, 1.0, 7.0),   // 15 >> 1 = 7 (not 7.5)
            (7.0, 1.0, 3.0),    // 7 >> 1 = 3 (not 3.5)
            (5.0, 2.0, 1.0),    // 5 >> 2 = 1 (not 1.25)
        ];

        for (value, shift, expected) in test_cases {
            interp.push(Value::Number(value));
            interp.push(Value::Number(shift));
            shr_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} >> {} should be {}, got {:?}", value, shift, expected, result);
        }
    }

    #[test]
    fn test_shr_reversibility_with_shl() {

        // Test that (x << n) >> n = x for values that don't overflow
        let value = 42.0;
        let shift = 3.0;

        // Simulate left shift then right shift
        let value_int = value as i64;
        let shifted_left = value_int << (shift as u32);
        let shifted_back = shifted_left >> (shift as u32);

        assert_eq!(shifted_back, value_int,
                  "({} << {}) >> {} should equal {}", value, shift, shift, value);
    }

    #[test]
    fn test_shr_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = shr_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(8.0));
        let result = shr_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_shr_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(2.0));

        let result = shr_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}