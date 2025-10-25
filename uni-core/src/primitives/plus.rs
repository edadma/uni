// Addition primitive - handles numeric addition and string concatenation

use crate::compat::{format, ToString};
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Polymorphic addition - multiple numeric types and string concatenation
// Stack-based addition: ( n1 n2 -- sum ) or ( str1 any -- concat ) or ( any str2 -- concat )
// Supports automatic type promotion: Integer < Rational < GaussianInt < Number < Complex
pub fn add_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b =
        interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;
    let a =
        interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;

    // Handle string concatenation first
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

    // For numeric types, use type promotion
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        // RUST CONCEPT: Fast path for Int32 - checked arithmetic for embedded safety
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_add(*i2) {
                Some(result) => Value::Int32(result),
                // Overflow: promote to BigInt
                None => Value::Integer(num_bigint::BigInt::from(*i1) + num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 + i2),
        (Value::Rational(r1), Value::Rational(r2)) => Value::Rational(r1 + r2).demote(),
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 + n2),
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(re1, im1), Value::GaussianInt(re2, im2)) => {
            Value::GaussianInt(re1 + re2, im1 + im2).demote()
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 + c2),
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
