// Round primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Round function rounds to nearest integer
// Stack-based round: ( n -- round(n) )
// Uses "round half away from zero" strategy
pub fn round_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.round()));
    Ok(())
}
