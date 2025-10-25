// Square root primitive

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Float trait needed for no_std environments
// In std environments, f64 has these methods built-in
// In no_std (like micro:bit), we need the Float trait from num_traits with libm
#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Square root with domain checking
// Stack-based sqrt: ( n -- sqrt(n) )
pub fn sqrt_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;

    if n < 0.0 {
        return Err(RuntimeError::DomainError(
            "sqrt of negative number".to_string(),
        ));
    }

    interp.push(Value::Number(n.sqrt()));
    Ok(())
}
