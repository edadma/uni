use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

pub fn bit_or_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    // Convert to integers for bitwise operations
    let a_int = a as i64;
    let b_int = b as i64;

    let result = a_int | b_int;
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
    fn test_bit_or_basic() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0)); // 101 in binary
        interp.push(Value::Number(3.0)); // 011 in binary

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.0)); // 111 in binary
    }

    #[test]
    fn test_bit_or_no_overlap() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(8.0)); // 1000 in binary
        interp.push(Value::Number(4.0)); // 0100 in binary

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 12.0)); // 1100 in binary
    }

    #[test]
    fn test_bit_or_with_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0));

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_bit_or_same_values() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(15.0));
        interp.push(Value::Number(15.0));

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 15.0));
    }

    #[test]
    fn test_bit_or_powers_of_two() {
        let mut interp = setup_interpreter();

        let test_cases = [
            (1.0, 2.0, 3.0),  // 01 | 10 = 11
            (4.0, 8.0, 12.0), // 0100 | 1000 = 1100
            (1.0, 4.0, 5.0),  // 0001 | 0100 = 0101
            (2.0, 8.0, 10.0), // 0010 | 1000 = 1010
        ];

        for (a, b, expected) in test_cases {
            interp.push(Value::Number(a));
            interp.push(Value::Number(b));
            bit_or_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if n == expected),
                "{} | {} should be {}, got {:?}",
                a,
                b,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_bit_or_alternating_patterns() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(170.0)); // 10101010 in binary
        interp.push(Value::Number(85.0)); // 01010101 in binary

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 255.0)); // 11111111 in binary
    }

    #[test]
    fn test_bit_or_large_numbers() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(512.0)); // 1000000000 in binary
        interp.push(Value::Number(255.0)); // 0011111111 in binary

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 767.0)); // 1011111111 in binary
    }

    #[test]
    fn test_bit_or_negative_numbers() {
        let mut interp = setup_interpreter();

        // Test with negative numbers (two's complement)
        interp.push(Value::Number(-1.0)); // All bits set
        interp.push(Value::Number(42.0));

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0)); // -1 | anything = -1
    }

    #[test]
    fn test_bit_or_fractional_truncation() {
        let mut interp = setup_interpreter();

        // Fractional parts should be truncated
        interp.push(Value::Number(5.7));
        interp.push(Value::Number(2.9));

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 7.0)); // 5 | 2 = 7
    }

    #[test]
    fn test_bit_or_commutative() {
        let mut interp = setup_interpreter();

        let a = 13.0;
        let b = 7.0;

        // Test a | b
        interp.push(Value::Number(a));
        interp.push(Value::Number(b));
        bit_or_builtin(&mut interp).unwrap();
        let result1 = interp.pop().unwrap();

        // Test b | a
        interp.push(Value::Number(b));
        interp.push(Value::Number(a));
        bit_or_builtin(&mut interp).unwrap();
        let result2 = interp.pop().unwrap();

        assert!(matches!((result1, result2), (Value::Number(n1), Value::Number(n2)) if n1 == n2));
    }

    #[test]
    fn test_bit_or_with_all_bits_set() {
        let mut interp = setup_interpreter();

        // Test OR with a number that has many bits set
        interp.push(Value::Number(123.0)); // Arbitrary number
        interp.push(Value::Number(255.0)); // All bits set in byte

        bit_or_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 255.0)); // Should be 255
    }

    #[test]
    fn test_bit_or_identity() {
        let mut interp = setup_interpreter();

        // OR with 0 should be identity
        let values = [1.0, 7.0, 15.0, 31.0, 63.0];

        for value in values {
            interp.push(Value::Number(value));
            interp.push(Value::Number(0.0));
            bit_or_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Number(n) if n == value),
                "{} | 0 should be {}, got {:?}",
                value,
                value,
                result
            );
        }
    }

    #[test]
    fn test_bit_or_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = bit_or_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        interp.push(Value::Number(5.0));
        let result = bit_or_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_bit_or_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(5.0));

        let result = bit_or_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
