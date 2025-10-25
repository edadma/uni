// Modulo primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_traits::Zero;

// RUST CONCEPT: Modulo operation with zero checking and type promotion
// Stack-based modulo: ( n1 n2 -- remainder )
// Supports all numeric types with automatic promotion
pub fn mod_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'mod' requires exactly 2 values on the stack (e.g., '13 5 mod')")?;
    let a = interp.pop_with_context("'mod' requires exactly 2 values on the stack (e.g., '13 5 mod')")?;

    // Check for modulo by zero
    let is_zero = match &b {
        Value::Int32(i) => *i == 0,
        Value::Integer(i) => i.is_zero(),
        Value::Rational(r) => r.is_zero(),
        Value::Number(n) => *n == 0.0,
        _ => false,
    };
    if is_zero {
        return Err(RuntimeError::ModuloByZero);
    }

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Int32(i1), Value::Int32(i2)) => Value::Int32(i1 % i2),
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 % i2),
        (Value::Rational(r1), Value::Rational(r2)) => {
            let result = Value::Rational(r1 % r2);
            result.demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 % n2),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot compute modulo of {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}
