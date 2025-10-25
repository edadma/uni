// Ceiling primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Ceiling function rounds up to nearest integer
// Stack-based ceil: ( n -- ceil(n) )
pub fn ceil_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.ceil()));
    Ok(())
}
