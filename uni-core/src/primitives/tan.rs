// Tangent primitive

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Tangent trigonometric function with infinity checking
// Stack-based tan: ( radians -- tan(radians) )
pub fn tan_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    let result = n.tan();

    // Check for invalid results (infinite values occur at odd multiples of π/2)
    if result.is_infinite() {
        return Err(RuntimeError::DomainError(
            "tan is undefined (infinite)".to_string(),
        ));
    }

    interp.push(Value::Number(result));
    Ok(())
}
