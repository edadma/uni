// Clear builtin - removes all items from the stack
// Usage: clear  (empties the entire stack)

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

pub fn clear_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    interp.stack.clear();
    Ok(())
}
