// Floor division primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_traits::Zero;

// RUST CONCEPT: Conditional imports for no_std
// On std platforms, f64 has .floor() built-in
// On no_std platforms (target_os = "none"), we need the Float trait for .floor()
#[cfg(target_os = "none")]
use num_traits::Float;

// RUST CONCEPT: Floor division with zero checking and type promotion
// Stack-based floor division: ( n1 n2 -- quotient )
// Like Python's //, always returns the floor of the quotient
pub fn floor_div_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'//' requires exactly 2 values on the stack (e.g., '7 2 //')")?;
    let a = interp.pop_with_context("'//' requires exactly 2 values on the stack (e.g., '7 2 //')")?;

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
            // Int32 division in Rust uses truncation, not floor
            // For floor division, adjust if signs differ and there's a remainder
            let quotient = i1 / i2;
            let remainder = i1 % i2;

            // Adjust for floor semantics if signs differ and there's a remainder
            if (i1.signum() != i2.signum()) && remainder != 0 {
                Value::Int32(quotient - 1)
            } else {
                Value::Int32(quotient)
            }
        }
        (Value::Integer(i1), Value::Integer(i2)) => {
            // Integer division in Rust uses truncation, not floor
            // For floor division: floor(a/b) = (a - (a % b)) / b when signs differ
            let quotient = i1 / i2;
            let remainder = i1 % i2;

            // Adjust for floor semantics if signs differ and there's a remainder
            if (i1.sign() != i2.sign()) && !remainder.is_zero() {
                Value::Integer(quotient - 1)
            } else {
                Value::Integer(quotient)
            }
        }
        (Value::Rational(r1), Value::Rational(r2)) => {
            // For rationals, divide and take floor
            let division = r1 / r2;
            let floor_val = division.floor();
            Value::Rational(floor_val).demote()
        }
        (Value::Number(n1), Value::Number(n2)) => Value::Number((n1 / n2).floor()),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "Cannot floor divide {:?} and {:?}",
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
    fn test_floor_div_impl() {
        let mut interp = AsyncInterpreter::new();

        // Test positive floor division
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(2.0));
        floor_div_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.0));

        // Test negative floor division
        interp.push(Value::Number(-7.0));
        interp.push(Value::Number(2.0));
        floor_div_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -4.0));
    }
}
