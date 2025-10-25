// Absolute value primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use num_traits::Signed;

// RUST CONCEPT: Absolute value for all numeric types
// Stack-based abs: ( n -- |n| )
pub fn abs_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let val = interp.pop_with_context("'abs' requires exactly 1 value on the stack (e.g., '-5 abs')")?;

    let result = match val {
        Value::Int32(i) => Value::Int32(i.abs()),
        Value::Integer(i) => Value::Integer(i.abs()),
        Value::Rational(r) => Value::Rational(r.abs()),
        Value::Number(n) => Value::Number(n.abs()),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "abs requires a number, got {}",
                val.type_name()
            )));
        }
    };

    interp.push(result);
    Ok(())
}
