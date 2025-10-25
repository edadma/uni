// Type introspection - returns type name as string

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn type_of_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let type_name = value.type_name().to_string();
    interp.push(Value::String(type_name.into()));
    Ok(())
}
