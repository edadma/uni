// Cosine primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Cosine trigonometric function
// Stack-based cos: ( radians -- cos(radians) )
pub fn cos_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.cos()));
    Ok(())
}
