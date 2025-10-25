use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn shr_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let shift = interp.pop_number()?;
    let value = interp.pop_number()?;

    let value_int = value as i64;
    let shift_int = shift as u32;

    let result = value_int >> shift_int;
    interp.push(Value::Number(result as f64));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::AsyncInterpreter;
    use crate::value::Value;

    #[test]
    fn test_basic_operation() {
        let mut interp = AsyncInterpreter::new();
        interp.push(Value::Number(16.0));
        interp.push(Value::Number(2.0));
        shr_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 4.0));
    }
}
