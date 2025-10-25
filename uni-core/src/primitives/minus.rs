// Subtraction primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// Subtraction: ( a b -- a-b )
pub fn sub_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'-' requires exactly 2 values on the stack")?;
    let a = interp.pop_with_context("'-' requires exactly 2 values on the stack")?;

    let result = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_sub(*i2) {
                Some(result) => Value::Int32(result),
                None => Value::Integer(num_bigint::BigInt::from(*i1) - num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 - n2),
        (Value::Int32(i), Value::Number(n)) => Value::Number(*i as f64 - n),
        (Value::Number(n), Value::Int32(i)) => Value::Number(n - *i as f64),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot subtract {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}
