// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Division with zero checking
// Stack-based division: ( n1 n2 -- quotient )
pub fn div_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;
    if b == 0.0 {
        return Err(RuntimeError::DivisionByZero);
    }
    interp.push(Value::Number(a / b));
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
    fn test_div_builtin() {
        let mut interp = setup_interpreter();

        // Test basic division: 12 / 4 = 3
        interp.push(Value::Number(12.0));
        interp.push(Value::Number(4.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test with fractional result: 7 / 2 = 3.5
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.5));

        // Test with negative numbers: -8 / 2 = -4
        interp.push(Value::Number(-8.0));
        interp.push(Value::Number(2.0));
        div_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -4.0));
    }

    #[test]
    fn test_div_builtin_division_by_zero() {
        let mut interp = setup_interpreter();

        // Test division by zero
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(0.0));
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DivisionByZero)));

        // After division by zero error, stack should be empty
        // (both operands were popped before the error was detected)
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_div_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_div_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        interp.push(Value::Number(5.0));
        interp.push(Value::Null);
        let result = div_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
