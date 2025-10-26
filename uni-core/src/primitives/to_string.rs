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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_to_string_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(42.0));
        to_string_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));

        interp.push(Value::Boolean(true));
        to_string_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "true"));
    }
}
