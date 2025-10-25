// Power primitive

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Exponentiation with NaN/infinity checking
// Stack-based pow: ( base exponent -- base^exponent )
pub fn pow_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let exponent = interp.pop_number()?;
    let base = interp.pop_number()?;

    let result = base.powf(exponent);

    // Check for invalid results (NaN or infinite)
    if result.is_nan() {
        return Err(RuntimeError::DomainError("pow result is NaN".to_string()));
    }
    if result.is_infinite() {
        return Err(RuntimeError::DomainError(
            "pow result is infinite".to_string(),
        ));
    }

    interp.push(Value::Number(result));
    Ok(())
}
