// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Modulo operation with zero checking
// Stack-based modulo: ( n1 n2 -- remainder )
pub fn mod_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;
    if b == 0.0 {
        return Err(RuntimeError::ModuloByZero);
    }
    interp.push(Value::Number(a % b));
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
    fn test_mod_builtin() {
        let mut interp = setup_interpreter();

        // Test basic modulo: 13 % 5 = 3
        interp.push(Value::Number(13.0));
        interp.push(Value::Number(5.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test with exact division: 12 % 4 = 0
        interp.push(Value::Number(12.0));
        interp.push(Value::Number(4.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));

        // Test with negative numbers: -7 % 3 (result depends on implementation)
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(3.0));
        mod_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::Number(n) = result {
            // Rust's % follows the dividend's sign
            assert_eq!(n, -1.0);
        } else {
            panic!("Expected number");
        }
    }

    #[test]
    fn test_mod_builtin_modulo_by_zero() {
        let mut interp = setup_interpreter();

        // Test modulo by zero
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::ModuloByZero)));

        // After modulo by zero error, stack should be empty
        // (both operands were popped before the error was detected)
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_mod_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_mod_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::String("not a number".into()));
        let result = mod_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}