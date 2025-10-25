// Equality comparison primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Equals: ( a b -- bool )
pub fn equals_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // Simple value equality
    let result = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => *i1 == *i2,
        (Value::Number(n1), Value::Number(n2)) => n1 == n2,
        (Value::Int32(i), Value::Number(n)) | (Value::Number(n), Value::Int32(i)) => {
            *i as f64 == *n
        }
        (Value::Boolean(b1), Value::Boolean(b2)) => b1 == b2,
        (Value::Null, Value::Null) => true,
        (Value::String(s1), Value::String(s2)) => s1 == s2,
        _ => false,
    };

    interp.push(Value::Boolean(result));
    Ok(())
}
