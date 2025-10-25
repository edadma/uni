// Local mutable variable creation
// Creates a mutable local variable in the current local frame

use crate::compat::{format, Rc};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

#[cfg(target_os = "none")]
use crate::compat::ToString;

// Stack-based: ( value 'name -- )
// Creates a local mutable variable with the given value in the current local frame
// The variable can be accessed with @ and modified with !
pub fn lvar_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
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
