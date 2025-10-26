// Multiplication primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Polymorphic multiplication - multiple numeric types
// Stack-based multiplication: ( n1 n2 -- product )
// Supports automatic type promotion: Integer < Rational < GaussianInt < Number < Complex
pub fn mul_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'*' requires exactly 2 values on the stack (e.g., '4 3 *')")?;
    let a = interp.pop_with_context("'*' requires exactly 2 values on the stack (e.g., '4 3 *')")?;

    // For numeric types, use type promotion
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        // RUST CONCEPT: Fast path for Int32 - checked arithmetic for embedded safety
        (Value::Int32(i1), Value::Int32(i2)) => {
            match i1.checked_mul(*i2) {
                Some(result) => Value::Int32(result),
                // Overflow: promote to BigInt
                None => Value::Integer(num_bigint::BigInt::from(*i1) * num_bigint::BigInt::from(*i2)),
            }
        }
        (Value::Integer(i1), Value::Integer(i2)) => Value::Integer(i1 * i2),
        (Value::Rational(r1), Value::Rational(r2)) => Value::Rational(r1 * r2).demote(),
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 * n2),
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(a_re, a_im), Value::GaussianInt(b_re, b_im)) => {
            // (a+bi)(c+di) = (ac-bd)+(ad+bc)i
            let ac = a_re * b_re;
            let bd = a_im * b_im;
            let ad = a_re * b_im;
            let bc = a_im * b_re;
            Value::GaussianInt(&ac - &bd, ad + bc).demote()
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 * c2),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot multiply {:?} and {:?}",
                a, b
            )))
        }
    };

    interp.push(result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_mul_impl() {
        let mut interp = AsyncInterpreter::new();

        // Test basic multiplication
        interp.push(Value::Number(6.0));
        interp.push(Value::Number(7.0));
        mul_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));

        // Test with negative numbers
        interp.push(Value::Number(-3.0));
        interp.push(Value::Number(4.0));
        mul_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -12.0));
    }

    #[test]
    fn test_mul_impl_integers() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Int32(12));
        interp.push(Value::Int32(5));
        mul_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(60)));
    }
}
