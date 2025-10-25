// Stack manipulation primitives
// Note: swap, dup, over, rot are defined in the prelude using pick and roll

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

// Drop: ( a -- )
pub fn drop_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    interp.pop()?;
    Ok(())
}
