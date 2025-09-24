// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Arithmetic operations with overflow considerations
// Stack-based multiplication: ( n1 n2 -- product )
pub fn mul_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;
    interp.push(Value::Number(a * b));
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
    fn test_mul_builtin() {
        let mut interp = setup_interpreter();

        // Test basic multiplication: 4 * 3 = 12
        interp.push(Value::Number(4.0));
        interp.push(Value::Number(3.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 12.0));

        // Test with negative numbers: -2 * 7 = -14
        interp.push(Value::Number(-2.0));
        interp.push(Value::Number(7.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -14.0));

        // Test with zero
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));

        // Test with fractional numbers
        interp.push(Value::Number(2.5));
        interp.push(Value::Number(4.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }

    #[test]
    fn test_mul_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = mul_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = mul_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_mul_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        let atom = interp.intern_atom("not-a-number");
        interp.push(Value::Atom(atom));
        interp.push(Value::Number(5.0));
        let result = mul_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}