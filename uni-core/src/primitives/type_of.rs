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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_type_of_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(42.0));
        type_of_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "number"));

        interp.push(Value::Boolean(true));
        type_of_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "boolean"));

        interp.push(Value::Int32(100));
        type_of_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "int32"));
    }
}
