use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use crate::compat::Rc;

pub fn type_of_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let type_name = value.type_name();
    interp.push(Value::String(Rc::<str>::from(type_name)));
    Ok(())
}
