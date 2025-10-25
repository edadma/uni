// List construction from multiple stack elements
// Creates a list from the top 'count' stack elements

use crate::compat::{ToString, Vec};
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

#[cfg(target_os = "none")]
use num_traits::Float;

// Stack-based list: ( element1 element2 ... elementN count -- list )
// Creates a list from the top 'count' stack elements in reverse order
pub fn list_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let count_value = interp.pop_number()?;

    // Input validation
    if count_value < 0.0 || count_value.fract() != 0.0 {
        return Err(RuntimeError::TypeError(
            "list count must be a non-negative integer".to_string(),
        ));
    }

    let count = count_value as usize;

    // Collect elements from stack
    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        elements.push(interp.pop()?);
    }

    // Elements are in reverse order (stack is LIFO)
    // We need to reverse them to get the correct list order
    elements.reverse();

    // Use interpreter's list construction helper
    let list = interp.make_list(elements);
    interp.push(list);
    Ok(())
}
