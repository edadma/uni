// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: Clear builtin - removes all items from the stack
// Usage: clear  (empties the entire stack)
pub fn clear_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    interp.stack.clear();
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
    fn test_clear_builtin_empty_stack() {
        let mut interp = setup_interpreter();

        // Clear on empty stack should succeed
        let result = clear_builtin(&mut interp);
        assert!(result.is_ok());
        assert!(interp.stack.is_empty());
    }

    #[test]
    fn test_clear_builtin_with_values() {
        let mut interp = setup_interpreter();

        // Push some values
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));

        assert_eq!(interp.stack.len(), 3);

        // Clear should remove all values
        let result = clear_builtin(&mut interp);
        assert!(result.is_ok());
        assert!(interp.stack.is_empty());
    }

    #[test]
    fn test_clear_builtin_multiple_times() {
        let mut interp = setup_interpreter();

        // Push values
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));

        // Clear
        clear_builtin(&mut interp).unwrap();
        assert!(interp.stack.is_empty());

        // Push more values
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(4.0));

        // Clear again
        clear_builtin(&mut interp).unwrap();
        assert!(interp.stack.is_empty());
    }

    #[test]
    fn test_clear_builtin_many_items() {
        let mut interp = setup_interpreter();

        // Push many values
        for i in 0..100 {
            interp.push(Value::Number(i as f64));
        }

        assert_eq!(interp.stack.len(), 100);

        // Clear should remove all
        let result = clear_builtin(&mut interp);
        assert!(result.is_ok());
        assert!(interp.stack.is_empty());
    }
}
