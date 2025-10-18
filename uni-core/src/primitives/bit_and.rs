use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn bit_and_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    // Convert to integers for bitwise operations
    let a_int = a as i64;
    let b_int = b as i64;

    let result = a_int & b_int;
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
    fn test_bit_and_basic() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0)); // 101 in binary
        interp.push(Value::Number(3.0)); // 011 in binary

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0)); // 001 in binary
    }

    #[test]
    fn test_bit_and_all_ones() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(15.0)); // 1111 in binary
        interp.push(Value::Number(7.0)); // 0111 in binary

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.0)); // 0111 in binary
    }

    #[test]
    fn test_bit_and_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(255.0));
        interp.push(Value::Number(0.0));

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_bit_and_same_values() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(42.0));

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_bit_and_powers_of_two() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (8.0, 4.0, 0.0),   // 1000 & 0100 = 0000
            (12.0, 10.0, 8.0), // 1100 & 1010 = 1000
            (7.0, 7.0, 7.0),   // 0111 & 0111 = 0111
        ];

        for (a, b, expected) in test_cases {
            interp.push(Value::Number(a));
            interp.push(Value::Number(b));
            bit_and_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if n == expected),
                "{} & {} should be {}, got {:?}",
                a,
                b,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_bit_and_large_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1023.0)); // 1111111111 in binary (10 bits)
        interp.push(Value::Number(512.0)); // 1000000000 in binary

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 512.0));
    }

    #[test]
    fn test_bit_and_negative_numbers() {
        let mut interp = setup_interpreter();

        // Test with negative numbers (two's complement)
        interp.push(Value::Number(-1.0));
        interp.push(Value::Number(7.0));

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.0)); // -1 has all bits set
    }

    #[test]
    fn test_bit_and_fractional_truncation() {
        let mut interp = setup_interpreter();

        // Fractional parts should be truncated
        interp.push(Value::Number(5.7));
        interp.push(Value::Number(3.9));

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0)); // 5 & 3 = 1
    }

    #[test]
    fn test_bit_and_identity_mask() {
        let mut interp = setup_interpreter();

        // Test common bit masks
        let value = 170.0; // 10101010 in binary
        interp.push(Value::Number(value));
        interp.push(Value::Number(255.0)); // 11111111 in binary

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == value)); // Should be unchanged
    }

    #[test]
    fn test_bit_and_alternating_pattern() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(170.0)); // 10101010 in binary
        interp.push(Value::Number(85.0)); // 01010101 in binary

        bit_and_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0)); // No common bits
    }

    #[test]
    fn test_bit_and_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = bit_and_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(5.0));
        let result = bit_and_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_bit_and_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(5.0));

        let result = bit_and_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
