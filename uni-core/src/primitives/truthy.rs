// Truthy predicate - tests if a value is truthy

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn truthy_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let is_truthy = interp.is_truthy(&value);
    interp.push(Value::Boolean(is_truthy));
    Ok(())
}
