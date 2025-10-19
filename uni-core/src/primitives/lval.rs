// RUST CONCEPT: Local constant creation
// Creates an immutable local constant in the current local frame
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(target_os = "none")]
use crate::compat::ToString;

// RUST CONCEPT: Local constant primitive
// Stack-based: ( value 'name -- )
// Creates a local constant with the given value in the current local frame
// The constant can be referenced by name without needing @ (fetch)
pub fn lval_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Check if we have a local frame
    if interp.local_frames.is_empty() {
        return Err(RuntimeError::TypeError(
            "lval: no local frame (can only be used inside quotations)".to_string(),
        ));
    }

    let name_val = interp.pop()?;
    let value = interp.pop()?;

    // Extract the atom name
    let name = match name_val {
        Value::Atom(ref atom) => atom.clone(),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "lval expects atom name, got {:?}",
                name_val
            )))
        }
    };

    // Store in the current (top) local frame
    let frame = interp
        .local_frames
        .last_mut()
        .expect("Already checked that local_frames is not empty");
    frame.insert(name, value);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_lval_no_frame_error() {
        let mut interp = Interpreter::new();

        // Try to create local without a frame
        interp.push(Value::Int32(42));
        let name_atom = interp.intern_atom("x");
        interp.push(Value::Atom(name_atom.clone()));
        let result = lval_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_lval_with_frame() {
        use std::collections::HashMap;
        let mut interp = Interpreter::new();

        // Push a local frame
        interp.local_frames.push(HashMap::new());

        // Create a local constant: 42 'x lval
        interp.push(Value::Int32(42));
        let name_atom = interp.intern_atom("x");
        interp.push(Value::Atom(name_atom.clone()));
        lval_builtin(&mut interp).unwrap();

        // Check that the local frame contains 'x'
        let frame = interp.local_frames.last().unwrap();
        assert!(frame.contains_key(&name_atom));

        // Verify the value
        let stored_value = frame.get(&name_atom).unwrap();
        assert!(matches!(stored_value, Value::Int32(42)));

        // Cleanup
        interp.local_frames.pop();
    }

    #[test]
    fn test_lval_wrong_name_type() {
        use std::collections::HashMap;
        let mut interp = Interpreter::new();

        // Push a local frame
        interp.local_frames.push(HashMap::new());

        // Try to create local with wrong name type
        interp.push(Value::Int32(42));
        interp.push(Value::Int32(5)); // Wrong type for name
        let result = lval_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));

        // Cleanup
        interp.local_frames.pop();
    }
}
