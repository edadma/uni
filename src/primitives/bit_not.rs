use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn bit_not_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;

    // Convert to integer for bitwise operation
    let n_int = n as i64;

    let result = !n_int;
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
    fn test_bit_not_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        bit_not_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0)); // ~0 = -1 in two's complement
    }

    #[test]
    fn test_bit_not_negative_one() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-1.0));

        bit_not_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0)); // ~(-1) = 0
    }

    #[test]
    fn test_bit_not_small_positive() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (1.0, -2.0),   // ~1 = -2
            (2.0, -3.0),   // ~2 = -3
            (3.0, -4.0),   // ~3 = -4
            (4.0, -5.0),   // ~4 = -5
            (5.0, -6.0),   // ~5 = -6
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            bit_not_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "~{} should be {}, got {:?}", input, expected, result);
        }
    }

    #[test]
    fn test_bit_not_powers_of_two() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (8.0, -9.0),     // ~1000 = ...11110111 = -9
            (16.0, -17.0),   // ~10000 = ...11101111 = -17
            (32.0, -33.0),   // ~100000 = ...11011111 = -33
            (64.0, -65.0),   // ~1000000 = ...10111111 = -65
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            bit_not_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "~{} should be {}, got {:?}", input, expected, result);
        }
    }

    #[test]
    fn test_bit_not_byte_values() {
        let mut interp = setup_interpreter();

        // Test some common byte patterns
        let test_cases = [
            (255.0, -256.0), // ~11111111 = -256
            (170.0, -171.0), // ~10101010 = -171
            (85.0, -86.0),   // ~01010101 = -86
            (15.0, -16.0),   // ~00001111 = -16
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            bit_not_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "~{} should be {}, got {:?}", input, expected, result);
        }
    }

    #[test]
    fn test_bit_not_negative_numbers() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (-2.0, 1.0),   // ~(-2) = 1
            (-3.0, 2.0),   // ~(-3) = 2
            (-4.0, 3.0),   // ~(-4) = 3
            (-10.0, 9.0),  // ~(-10) = 9
        ];

        for (input, expected) in test_cases {
            interp.push(Value::Number(input));
            bit_not_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "~{} should be {}, got {:?}", input, expected, result);
        }
    }

    #[test]
    fn test_bit_not_large_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1024.0)); // 2^10

        bit_not_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1025.0)); // ~1024 = -1025
    }

    #[test]
    fn test_bit_not_fractional_truncation() {
        let mut interp = setup_interpreter();

        // Fractional parts should be truncated
        interp.push(Value::Number(5.7));

        bit_not_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -6.0)); // ~5 = -6
    }

    #[test]
    fn test_bit_not_double_negation() {
        let mut interp = setup_interpreter();

        // Test that ~~x = x
        let test_values = [0.0, 1.0, 5.0, 42.0, 255.0];

        for value in test_values {
            // First NOT
            interp.push(Value::Number(value));
            bit_not_builtin(&mut interp).unwrap();
            let first_not = interp.pop().unwrap();

            // Second NOT
            interp.push(first_not);
            bit_not_builtin(&mut interp).unwrap();
            let double_not = interp.pop().unwrap();

            assert!(matches!(double_not, Value::Number(n) if n == value),
                   "~~{} should equal {}, got {:?}", value, value, double_not);
        }
    }

    #[test]
    fn test_bit_not_complement_property() {

        // Test that n & ~n = 0 for any n
        let test_values = [1.0, 7.0, 15.0, 42.0, 255.0];

        for value in test_values {
            let value_int = value as i64;
            let not_value_int = !value_int;
            let and_result = value_int & not_value_int;

            assert_eq!(and_result, 0, "{} & ~{} should be 0", value, value);
        }
    }

    #[test]
    fn test_bit_not_relationship_with_minus_one() {
        let mut interp = setup_interpreter();

        // Test that ~n = -n - 1
        let test_values = [0.0, 1.0, 5.0, 10.0, 42.0];

        for value in test_values {
            interp.push(Value::Number(value));
            bit_not_builtin(&mut interp).unwrap();
            let not_result = interp.pop().unwrap();

            if let Value::Number(not_n) = not_result {
                let expected = -value - 1.0;
                assert!((not_n - expected).abs() < f64::EPSILON,
                       "~{} should equal {} - 1 = {}, got {}", value, -value, expected, not_n);
            }
        }
    }

    #[test]
    fn test_bit_not_alternating_pattern() {
        let mut interp = setup_interpreter();

        // Test NOT on alternating bit pattern
        interp.push(Value::Number(170.0)); // 10101010 in binary

        bit_not_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -171.0)); // Should flip all bits
    }

    #[test]
    fn test_bit_not_mask_generation() {
        let mut interp = setup_interpreter();

        // NOT of 0 gives all 1s (mask)
        interp.push(Value::Number(0.0));
        bit_not_builtin(&mut interp).unwrap();
        let all_ones = interp.pop().unwrap();

        // Should be -1 (all bits set in two's complement)
        assert!(matches!(all_ones, Value::Number(n) if n == -1.0));
    }

    #[test]
    fn test_bit_not_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = bit_not_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_bit_not_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = bit_not_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}