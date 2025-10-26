// Less than comparison primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Less than: ( a b -- bool )
pub fn less_than_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    let result = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => i1 < i2,
        (Value::Number(n1), Value::Number(n2)) => n1 < n2,
        (Value::Int32(i), Value::Number(n)) => (*i as f64) < *n,
        (Value::Number(n), Value::Int32(i)) => *n < (*i as f64),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot compare {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(Value::Boolean(result));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_less_than_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(3.0));
        interp.push(Value::Number(7.0));
        less_than_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        interp.push(Value::Number(10.0));
        interp.push(Value::Number(5.0));
        less_than_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }
}
