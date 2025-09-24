// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::RuntimeError;
use crate::interpreter::Interpreter;

// RUST CONCEPT: Stack manipulation - removing top element
// Stack-based drop: ( value -- )
pub fn drop_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: We don't need to do anything with the popped value
    // Just pop it and discard it
    interp.pop()?;
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
    fn test_drop_builtin() {
        let mut interp = setup_interpreter();

        // Test dropping a number
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(17.0));
        drop_builtin(&mut interp).unwrap();

        // Should have only 42.0 left
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Stack should now be empty
        let empty_result = interp.pop();
        assert!(matches!(empty_result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_drop_builtin_various_types() {
        let mut interp = setup_interpreter();

        // Test dropping different value types
        interp.push(Value::Number(1.0));
        interp.push(Value::String("drop me".into()));
        drop_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));

        interp.push(Value::Boolean(true));
        interp.push(Value::Null);
        drop_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        let keep_atom = interp.intern_atom("keep");
        interp.push(Value::Atom(keep_atom));
        interp.push(Value::Nil);
        drop_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        if let Value::Atom(atom) = result {
            assert_eq!(&*atom, "keep");
        } else {
            panic!("Expected atom");
        }
    }

    #[test]
    fn test_drop_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = drop_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}