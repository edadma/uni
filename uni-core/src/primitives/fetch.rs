// Forth-style variable fetch (@)
// Fetches the value stored in a variable

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Fetch primitive (@)
// Stack-based: ( var -- value )
pub fn fetch_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let var = interp.pop()?;

    match var {
        Value::Variable(ref cell) => {
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
    use crate::interpreter::AsyncInterpreter;
    use crate::value::Value;
    use crate::compat::Rc;
    
    #[cfg(not(target_os = "none"))]
    use std::cell::RefCell;
    #[cfg(target_os = "none")]
    use core::cell::RefCell;

    #[test]
    fn test_fetch_from_variable() {
        let mut interp = AsyncInterpreter::new();

        // Create a variable with value 100
        let var = Value::Variable(Rc::new(RefCell::new(Value::Int32(100))));
        interp.push(var);
        
        fetch_impl(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(100)));
    }

    #[test]
    fn test_fetch_type_error() {
        let mut interp = AsyncInterpreter::new();

        // Try to fetch from non-variable
        interp.push(Value::Int32(42));
        
        let result = fetch_impl(&mut interp);
        assert!(matches!(result, Err(crate::value::RuntimeError::TypeError(_))));
    }
}
