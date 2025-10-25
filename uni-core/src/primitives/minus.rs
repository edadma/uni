// Subtraction primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Polymorphic subtraction - multiple numeric types
// Stack-based subtraction: ( n1 n2 -- difference )
// Supports automatic type promotion: Integer < Rational < GaussianInt < Number < Complex
pub fn sub_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'-' requires exactly 2 values on the stack (e.g., '10 3 -')")?;
    let a = interp.pop_with_context("'-' requires exactly 2 values on the stack (e.g., '10 3 -')")?;

    // For numeric types, use type promotion
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        // RUST CONCEPT: Fast path for Int32 - checked arithmetic for embedded safety
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_sub(*i2) {
                Some(result) => Value::Int32(result),
                // Overflow: promote to BigInt
                None => Value::Integer(num_bigint::BigInt::from(*i1) - num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 - i2),
        (Value::Rational(r1), Value::Rational(r2)) => Value::Rational(r1 - r2).demote(),
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 - n2),
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(re1, im1), Value::GaussianInt(re2, im2)) => {
            Value::GaussianInt(re1 - re2, im1 - im2).demote()
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 - c2),
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
