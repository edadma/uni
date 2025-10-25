use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

pub fn bit_not_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let a = interp.pop_number()?;
    let a_int = a as i64;
    let result = !a_int;
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
        interp.push(Value::Number(0.0));
        bit_not_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -1.0));
    }
}
