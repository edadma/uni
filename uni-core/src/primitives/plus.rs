// Addition primitive - handles numeric addition and string concatenation

use crate::compat::{format, ToString};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Addition: ( a b -- a+b )
pub fn add_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;
    let a = interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;

    // Handle string concatenation
    match (&a, &b) {
        (Value::String(_), _) | (_, Value::String(_)) => {
            let str_a = match &a {
                Value::String(s) => s.as_ref(),
                _ => &a.to_string(),
            };
            let str_b = match &b {
                Value::String(s) => s.as_ref(),
                _ => &b.to_string(),
            };
            let result = format!("{}{}", str_a, str_b);
            interp.push(Value::String(result.into()));
            return Ok(());
        }
        _ => {}
    }

    // Numeric addition
    let result = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_add(*i2) {
                Some(result) => Value::Int32(result),
                None => Value::Integer(num_bigint::BigInt::from(*i1) + num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 + n2),
        (Value::Int32(i), Value::Number(n)) | (Value::Number(n), Value::Int32(i)) => {
            Value::Number(*i as f64 + n)
        }
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot add {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}
