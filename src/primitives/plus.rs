// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Arithmetic operations and error handling
// Stack-based addition: ( n1 n2 -- sum )
pub fn add_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;
    interp.push(Value::Number(a + b));
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
    fn test_add_builtin() {
        let mut interp = setup_interpreter();

        // Test basic addition
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));

        // Test with negative numbers
        interp.push(Value::Number(-2.0));
        interp.push(Value::Number(7.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with zero
        interp.push(Value::Number(0.0));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_add_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_add_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::String("not a number".into()));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}