use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

pub fn bit_xor_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    // Convert to integers for bitwise operations
    let a_int = a as i64;
    let b_int = b as i64;

    let result = a_int ^ b_int;
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
    fn test_bit_xor_basic() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));  // 101 in binary
        interp.push(Value::Number(3.0));  // 011 in binary

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 6.0)); // 110 in binary
    }

    #[test]
    fn test_bit_xor_same_values() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(42.0));

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0)); // Any value XOR itself = 0
    }

    #[test]
    fn test_bit_xor_with_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(123.0));
        interp.push(Value::Number(0.0));

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 123.0)); // XOR with 0 is identity
    }

    #[test]
    fn test_bit_xor_with_all_ones() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(170.0)); // 10101010 in binary
        interp.push(Value::Number(255.0)); // 11111111 in binary

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 85.0)); // 01010101 in binary (inverted)
    }

    #[test]
    fn test_bit_xor_alternating_patterns() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(170.0)); // 10101010 in binary
        interp.push(Value::Number(85.0));  // 01010101 in binary

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 255.0)); // 11111111 in binary
    }

    #[test]
    fn test_bit_xor_powers_of_two() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (1.0, 2.0, 3.0),     // 01 ^ 10 = 11
            (4.0, 8.0, 12.0),    // 0100 ^ 1000 = 1100
            (1.0, 1.0, 0.0),     // 01 ^ 01 = 00
            (15.0, 8.0, 7.0),    // 1111 ^ 1000 = 0111
        ];

        for (a, b, expected) in test_cases {
            interp.push(Value::Number(a));
            interp.push(Value::Number(b));
            bit_xor_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Number(n) if n == expected),
                   "{} ^ {} should be {}, got {:?}", a, b, expected, result);
        }
    }

    #[test]
    fn test_bit_xor_large_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(1023.0)); // 1111111111 in binary (10 bits)
        interp.push(Value::Number(512.0));  // 1000000000 in binary

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 511.0)); // 0111111111 in binary
    }

    #[test]
    fn test_bit_xor_negative_numbers() {
        let mut interp = setup_interpreter();

        // Test with negative numbers (two's complement)
        interp.push(Value::Number(-1.0)); // All bits set
        interp.push(Value::Number(42.0));

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // -1 XOR 42 should give ~42 = -43 in two's complement
        assert!(matches!(result, Value::Number(n) if n == -43.0));
    }

    #[test]
    fn test_bit_xor_fractional_truncation() {
        let mut interp = setup_interpreter();

        // Fractional parts should be truncated
        interp.push(Value::Number(5.7));
        interp.push(Value::Number(3.9));

        bit_xor_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 6.0)); // 5 ^ 3 = 6
    }

    #[test]
    fn test_bit_xor_commutative() {
        let mut interp = setup_interpreter();

        let a = 13.0;
        let b = 7.0;

        // Test a ^ b
        interp.push(Value::Number(a));
        interp.push(Value::Number(b));
        bit_xor_builtin(&mut interp).unwrap();
        let result1 = interp.pop().unwrap();

        // Test b ^ a
        interp.push(Value::Number(b));
        interp.push(Value::Number(a));
        bit_xor_builtin(&mut interp).unwrap();
        let result2 = interp.pop().unwrap();

        assert!(matches!((result1, result2), (Value::Number(n1), Value::Number(n2)) if n1 == n2));
    }

    #[test]
    fn test_bit_xor_self_inverse() {
        let mut interp = setup_interpreter();

        // Test that (a ^ b) ^ b = a
        let a = 123.0;
        let b = 45.0;

        // First: a ^ b
        interp.push(Value::Number(a));
        interp.push(Value::Number(b));
        bit_xor_builtin(&mut interp).unwrap();
        let intermediate = interp.pop().unwrap();

        // Then: (a ^ b) ^ b
        interp.push(intermediate);
        interp.push(Value::Number(b));
        bit_xor_builtin(&mut interp).unwrap();
        let final_result = interp.pop().unwrap();

        assert!(matches!(final_result, Value::Number(n) if n == a),
               "({} ^ {}) ^ {} should equal {}", a, b, b, a);
    }

    #[test]
    fn test_bit_xor_toggle_bits() {
        let mut interp = setup_interpreter();

        // Test bit toggling with mask
        let value = 15.0;  // 1111 in binary
        let mask = 5.0;    // 0101 in binary

        interp.push(Value::Number(value));
        interp.push(Value::Number(mask));
        bit_xor_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();

        assert!(matches!(result, Value::Number(n) if n == 10.0)); // 1010 in binary
    }

    #[test]
    fn test_bit_xor_encryption_like() {
        let mut interp = setup_interpreter();

        // Simulate simple XOR encryption/decryption
        let plaintext = 100.0;
        let key = 77.0;

        // Encrypt: plaintext ^ key
        interp.push(Value::Number(plaintext));
        interp.push(Value::Number(key));
        bit_xor_builtin(&mut interp).unwrap();
        let encrypted = interp.pop().unwrap();

        // Decrypt: encrypted ^ key
        interp.push(encrypted);
        interp.push(Value::Number(key));
        bit_xor_builtin(&mut interp).unwrap();
        let decrypted = interp.pop().unwrap();

        assert!(matches!(decrypted, Value::Number(n) if n == plaintext),
               "XOR encryption should be reversible");
    }

    #[test]
    fn test_bit_xor_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = bit_xor_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(5.0));
        let result = bit_xor_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_bit_xor_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(5.0));

        let result = bit_xor_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}