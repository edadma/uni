// Local constant creation
// Creates an immutable local constant in the current local frame

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(target_os = "none")]
use crate::compat::ToString;

// Stack-based: ( value 'name -- )
// Creates a local constant with the given value in the current local frame
// The constant can be referenced by name without needing @ (fetch)
pub fn lval_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
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

    #[cfg(not(target_os = "none"))]
    use std::collections::HashMap;
    #[cfg(target_os = "none")]
    use alloc::collections::BTreeMap as HashMap;

    #[test]
    fn test_lval_impl() {
        let mut interp = AsyncInterpreter::new();

        // Create a local frame
        interp.local_frames.push(HashMap::new());

        let name = interp.intern_atom("x");
        interp.push(Value::Number(42.0));
        interp.push(Value::Atom(name.clone()));
        lval_impl(&mut interp).unwrap();

        let frame = &interp.local_frames[0];
        let value = frame.get(&name).unwrap();
        assert!(matches!(value, Value::Number(n) if *n == 42.0));

        // Clean up
        interp.local_frames.pop();
    }
}
