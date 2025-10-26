// Minimum primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Minimum of two values with type promotion
// Stack-based min: ( n1 n2 -- min )
pub fn min_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'min' requires exactly 2 values on the stack (e.g., '3 7 min')")?;
    let a = interp.pop_with_context("'min' requires exactly 2 values on the stack (e.g., '3 7 min')")?;

    // Promote both values to a common type
    let (pa, pb) = promote_pair(&a, &b);

    let result = match (&pa, &pb) {
        (Value::Int32(i1), Value::Int32(i2)) => {
            if i1 <= i2 { Value::Int32(*i1) } else { Value::Int32(*i2) }
        }
        (Value::Integer(i1), Value::Integer(i2)) => {
            if i1 <= i2 { Value::Integer(i1.clone()) } else { Value::Integer(i2.clone()) }
        }
        (Value::Rational(r1), Value::Rational(r2)) => {
            if r1 <= r2 { Value::Rational(r1.clone()) } else { Value::Rational(r2.clone()) }
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number(n1.min(*n2)),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot compute min of {:?} and {:?}",
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
    fn test_min_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(3.0));
        interp.push(Value::Number(7.0));
        min_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        interp.push(Value::Int32(10));
        interp.push(Value::Int32(5));
        min_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(5)));
    }
}
