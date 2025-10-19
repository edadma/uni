// RUST CONCEPT: Local mutable variable creation
// Creates a mutable local variable in the current local frame
use crate::compat::{format, Rc};
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// RUST CONCEPT: Local variable primitive
// Stack-based: ( value 'name -- )
// Creates a local mutable variable with the given value in the current local frame
// The variable can be accessed with @ and modified with !
pub fn lvar_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Check if we have a local frame
    if interp.local_frames.is_empty() {
        return Err(RuntimeError::TypeError(
            "lvar: no local frame (can only be used inside quotations)".to_string(),
        ));
    }

    let name_val = interp.pop()?;
    let initial_value = interp.pop()?;

    // Extract the atom name
    let name = match name_val {
        Value::Atom(ref atom) => atom.clone(),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "lvar expects atom name, got {:?}",
                name_val
            )))
        }
    };

    // Create the variable (using RefCell for mutability)
    let var = Value::Variable(Rc::new(RefCell::new(initial_value)));

    // Store in the current (top) local frame
    let frame = interp
        .local_frames
        .last_mut()
        .expect("Already checked that local_frames is not empty");
    frame.insert(name, var);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_lvar_no_frame_error() {
        let mut interp = Interpreter::new();

        // Try to create local without a frame
        interp.push(Value::Int32(42));
        let name_atom = interp.intern_atom("x");
        interp.push(Value::Atom(name_atom.clone()));
        let result = lvar_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_lvar_with_frame() {
        use std::collections::HashMap;
        let mut interp = Interpreter::new();

        // Push a local frame
        interp.local_frames.push(HashMap::new());

        // Create a local variable: 42 'x lvar
        interp.push(Value::Int32(42));
        let name_atom = interp.intern_atom("x");
        interp.push(Value::Atom(name_atom.clone()));
        lvar_builtin(&mut interp).unwrap();

        // Check that the local frame contains 'x'
        let frame = interp.local_frames.last().unwrap();
        assert!(frame.contains_key(&name_atom));

        // Verify it's a Variable
        let stored_value = frame.get(&name_atom).unwrap();
        assert!(matches!(stored_value, Value::Variable(_)));

        // Cleanup
        interp.local_frames.pop();
    }

    #[test]
    fn test_lvar_wrong_name_type() {
        use std::collections::HashMap;
        let mut interp = Interpreter::new();

        // Push a local frame
        interp.local_frames.push(HashMap::new());

        // Try to create local with wrong name type
        interp.push(Value::Int32(42));
        interp.push(Value::Int32(5)); // Wrong type for name
        let result = lvar_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));

        // Cleanup
        interp.local_frames.pop();
    }
}
