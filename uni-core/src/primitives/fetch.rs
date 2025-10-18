// RUST CONCEPT: Forth-style variable fetch (@)
// Fetches the value stored in a variable
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Fetch primitive (@)
// Stack-based: ( var -- value )
// Reads the value from a Variable
pub fn fetch_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let var = interp.pop()?;

    match var {
        Value::Variable(ref cell) => {
            // Borrow the value from the RefCell and clone it
            let value = cell.borrow().clone();
            interp.push(value);
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(format!(
            "@ expects Variable, got {:?}",
            var
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::Rc;
    use crate::value::Value;

    #[cfg(not(target_os = "none"))]
    use std::cell::RefCell;
    #[cfg(target_os = "none")]
    use core::cell::RefCell;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_fetch_basic() {
        let mut interp = setup_interpreter();

        // Create a variable containing 42
        let var = Value::Variable(Rc::new(RefCell::new(Value::Int32(42))));
        interp.push(var);

        // Fetch the value
        fetch_builtin(&mut interp).unwrap();

        // Check result
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(42)));
    }

    #[test]
    fn test_fetch_wrong_type() {
        let mut interp = setup_interpreter();

        // Try to fetch from non-variable
        interp.push(Value::Int32(42));
        let result = fetch_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_fetch_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = fetch_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_fetch_different_types() {
        let mut interp = setup_interpreter();

        // Test with string
        let var = Value::Variable(Rc::new(RefCell::new(Value::String(Rc::from("hello")))));
        interp.push(var);
        fetch_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        if let Value::String(s) = result {
            assert_eq!(&*s, "hello");
        } else {
            panic!("Expected String");
        }

        // Test with boolean
        let var = Value::Variable(Rc::new(RefCell::new(Value::Boolean(true))));
        interp.push(var);
        fetch_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }
}
