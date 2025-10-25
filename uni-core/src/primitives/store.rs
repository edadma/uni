// Forth-style variable store (!)
// Stores a value into a variable

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Store primitive (!)
// Stack-based: ( value var -- )
pub fn store_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let var = interp.pop()?;
    let value = interp.pop()?;

    match var {
        Value::Variable(ref cell) => {
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
    use crate::interpreter::AsyncInterpreter;
    use crate::value::Value;
    use crate::compat::Rc;
    
    #[cfg(not(target_os = "none"))]
    use std::cell::RefCell;
    #[cfg(target_os = "none")]
    use core::cell::RefCell;

    #[test]
    fn test_store_to_variable() {
        let mut interp = AsyncInterpreter::new();

        // Create a variable with initial value 0
        let var = Rc::new(RefCell::new(Value::Int32(0)));
        
        // Store 42 to the variable
        interp.push(Value::Int32(42));
        interp.push(Value::Variable(var.clone()));
        
        store_impl(&mut interp).unwrap();

        // Verify the variable now contains 42
        let stored = var.borrow().clone();
        assert!(matches!(stored, Value::Int32(42)));
    }

    #[test]
    fn test_store_type_error() {
        let mut interp = AsyncInterpreter::new();

        // Try to store to non-variable
        interp.push(Value::Int32(100));
        interp.push(Value::Int32(42));
        
        let result = store_impl(&mut interp);
        assert!(matches!(result, Err(crate::value::RuntimeError::TypeError(_))));
    }
}
