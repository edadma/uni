// String conversion primitive
// Converts any value to its string representation

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn to_string_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let string_result = value.to_string();
    interp.push(Value::String(string_result.into()));
    Ok(())
}
