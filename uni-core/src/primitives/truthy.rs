// Truthy predicate - tests if a value is truthy

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn truthy_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let is_truthy = interp.is_truthy(&value);
    interp.push(Value::Boolean(is_truthy));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_truthy_impl() {
        let mut interp = AsyncInterpreter::new();

        // Test truthy values
        interp.push(Value::Boolean(true));
        truthy_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        interp.push(Value::Number(42.0));
        truthy_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test falsy values
        interp.push(Value::Boolean(false));
        truthy_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }
}
