// Natural logarithm primitive

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(feature = "std"))]
use num_traits::Float;

// RUST CONCEPT: Natural logarithm with domain checking
// Stack-based log: ( n -- ln(n) )
pub fn log_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()?;

    if n <= 0.0 {
        return Err(RuntimeError::DomainError(
            "log of non-positive number".to_string(),
        ));
    }

    interp.push(Value::Number(n.ln()));
    Ok(())
}
