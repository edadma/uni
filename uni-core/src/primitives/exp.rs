// Exponential primitive

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Exponential function with overflow checking
// Stack-based exp: ( n -- e^n )
pub fn exp_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;
    let result = n.exp();

    // Check for overflow (infinite result)
    if result.is_infinite() {
        return Err(RuntimeError::DomainError(
            "exp result is infinite (overflow)".to_string(),
        ));
    }

    interp.push(Value::Number(result));
    Ok(())
}
