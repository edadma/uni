// Sine primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Sine trigonometric function
// Stack-based sin: ( radians -- sin(radians) )
pub fn sin_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    interp.push(Value::Number(n.sin()));
    Ok(())
}
