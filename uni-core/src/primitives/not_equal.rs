// Not-equal comparison primitive

use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use core::ptr;

// RUST CONCEPT: Comprehensive inequality with support for all value types
// Not equals: ( a b -- bool )
pub fn not_equal_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'!=' requires exactly 2 values on the stack (e.g., '5 3 !=')")?;
    let a = interp.pop_with_context("'!=' requires exactly 2 values on the stack (e.g., '5 3 !=')")?;

    // Use the same equality logic as equals.rs but negate result
    let are_equal = match (&a, &b) {
        (Value::Int32(i1), Value::Int32(i2)) => i1 == i2,
        (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
        (Value::Integer(i1), Value::Integer(i2)) => i1 == i2,
        (Value::Rational(r1), Value::Rational(r2)) => r1 == r2,
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(re1, im1), Value::GaussianInt(re2, im2)) => re1 == re2 && im1 == im2,
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => c1 == c2,
        (Value::String(a), Value::String(b)) => a == b,
        (Value::Boolean(a), Value::Boolean(b)) => a == b,
        (Value::Atom(a), Value::Atom(b)) => a == b,
        (Value::QuotedAtom(a), Value::QuotedAtom(b)) => a == b,
        (Value::Null, Value::Null) => true,
        (Value::Nil, Value::Nil) => true,
        (Value::Pair(a1, a2), Value::Pair(b1, b2)) => {
            // Recursive equality for lists - would need helper function
            // For now, just check reference equality (shallow)
            ptr::eq(a1.as_ref(), b1.as_ref()) && ptr::eq(a2.as_ref(), b2.as_ref())
        }
        _ => false, // Different types are not equal
    };

    interp.push(Value::Boolean(!are_equal));
    Ok(())
}
