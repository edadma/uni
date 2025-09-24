// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Arithmetic operations with stack semantics
// Stack-based subtraction: ( n1 n2 -- difference )
pub fn sub_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;
    interp.push(Value::Number(a - b));
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
    fn test_sub_builtin() {
        let mut interp = setup_interpreter();

        // Test basic subtraction: 8 - 3 = 5
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(3.0));
        sub_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with negative result: 3 - 8 = -5
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(8.0));
        sub_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -5.0));

        // Test with zero
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0));
        sub_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_sub_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = sub_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = sub_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_sub_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::Boolean(false));
        let result = sub_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}