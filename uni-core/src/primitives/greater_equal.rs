// Greater than or equal comparison primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Greater than or equal: ( a b -- bool )
pub fn greater_equal_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    let result = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => i1 >= i2,
        (Value::Number(n1), Value::Number(n2)) => n1 >= n2,
        (Value::Int32(i), Value::Number(n)) => (*i as f64) >= *n,
        (Value::Number(n), Value::Int32(i)) => *n >= (*i as f64),
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
