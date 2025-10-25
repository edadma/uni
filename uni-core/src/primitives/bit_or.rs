use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn bit_or_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;

    let a_int = a as i64;
    let b_int = b as i64;

    let result = a_int | b_int;
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
        interp.push(Value::Number(8.0));
        interp.push(Value::Number(4.0));
        bit_or_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 12.0));
    }
}
