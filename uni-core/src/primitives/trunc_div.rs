// Truncating division primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_traits::Zero;

// RUST CONCEPT: Conditional imports for no_std
// On std platforms, f64 has .trunc() built-in
// On no_std platforms (target_os = "none"), we need the Float trait for .trunc()
#[cfg(target_os = "none")]
use num_traits::Float;

// RUST CONCEPT: Truncating division with zero checking and type promotion
// Stack-based truncating division: ( n1 n2 -- quotient )
// Rounds toward zero (like C/Java/Rust integer division)
pub fn trunc_div_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'div' requires exactly 2 values on the stack (e.g., '7 2 div')")?;
    let a = interp.pop_with_context("'div' requires exactly 2 values on the stack (e.g., '7 2 div')")?;

    // Check for division by zero
    let is_zero = match &b {
        Value::Int32(i) => *i == 0,
        Value::Integer(i) => i.is_zero(),
        Value::Rational(r) => r.is_zero(),
        Value::Number(n) => *n == 0.0,
        _ => false,
    };
    if is_zero {
        return Err(RuntimeError::DivisionByZero);
    }

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Int32(i1), Value::Int32(i2)) => {
            // Int32 division in Rust already truncates toward zero
            Value::Int32(i1 / i2)
        }
        (Value::Integer(i1), Value::Integer(i2)) => {
            // Integer division in Rust already truncates toward zero
            Value::Integer(i1 / i2)
        }
        (Value::Rational(r1), Value::Rational(r2)) => {
            // For rationals, divide and truncate toward zero
            let division = r1 / r2;
            let trunc_val = division.trunc();
            Value::Rational(trunc_val).demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number((n1 / n2).trunc()),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot truncating divide {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}
