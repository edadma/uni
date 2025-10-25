// Floor primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Floor function rounds down to nearest integer
// Stack-based floor: ( n -- floor(n) )
pub fn floor_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.floor()));
    Ok(())
}
