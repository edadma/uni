// RUST CONCEPT: Forth-style variable store (!)
// Stores a value into a variable
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Store primitive (!)
// Stack-based: ( value var -- )
// Stores value into a Variable
pub fn store_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let var = interp.pop()?;
    let value = interp.pop()?;

    match var {
        Value::Variable(ref cell) => {
            // Replace the value in the RefCell
            *cell.borrow_mut() = value;
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(format!(
            "! expects Variable, got {:?}",
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
    fn test_store_basic() {
        let mut interp = setup_interpreter();

        // Create a variable containing 10
        let var = Value::Variable(Rc::new(RefCell::new(Value::Int32(10))));

        // Store 42 into it
        interp.push(Value::Int32(42));
        interp.push(var.clone());
        store_builtin(&mut interp).unwrap();

        // Verify the variable now contains 42
        if let Value::Variable(cell) = var {
            let value = cell.borrow().clone();
            assert!(matches!(value, Value::Int32(42)));
        } else {
            panic!("Expected Variable");
        }
    }

    #[test]
    fn test_store_wrong_type() {
        let mut interp = setup_interpreter();

        // Try to store into non-variable
        interp.push(Value::Int32(42));
        interp.push(Value::Int32(5)); // Not a variable
        let result = store_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_store_stack_underflow() {
        let mut interp = setup_interpreter();

        // Not enough values
        let result = store_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Only one value
        interp.push(Value::Int32(42));
        let result = store_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_store_multiple_times() {
        let mut interp = setup_interpreter();

        // Create variable
        let var = Value::Variable(Rc::new(RefCell::new(Value::Int32(1))));

        // Store 2
        interp.push(Value::Int32(2));
        interp.push(var.clone());
        store_builtin(&mut interp).unwrap();

        // Store 3
        interp.push(Value::Int32(3));
        interp.push(var.clone());
        store_builtin(&mut interp).unwrap();

        // Verify final value is 3
        if let Value::Variable(cell) = var {
            let value = cell.borrow().clone();
            assert!(matches!(value, Value::Int32(3)));
        } else {
            panic!("Expected Variable");
        }
    }

    #[test]
    fn test_store_different_types() {
        let mut interp = setup_interpreter();

        // Create variable with int
        let var = Value::Variable(Rc::new(RefCell::new(Value::Int32(42))));

        // Store a string into it (changing type is allowed)
        interp.push(Value::String(Rc::from("hello")));
        interp.push(var.clone());
        store_builtin(&mut interp).unwrap();

        // Verify it now contains string
        if let Value::Variable(cell) = var {
            let value = cell.borrow().clone();
            if let Value::String(s) = value {
                assert_eq!(&*s, "hello");
            } else {
                panic!("Expected String");
            }
        } else {
            panic!("Expected Variable");
        }
    }
}
