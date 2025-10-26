// Division primitive

use crate::compat::format;
use crate::interpreter::AsyncInterpreter;
use crate::primitives::numeric_promotion::promote_pair;
use crate::value::{RuntimeError, Value};
use num_rational::BigRational;
use num_traits::Zero;

// RUST CONCEPT: Division with zero checking and type promotion
// Stack-based division: ( n1 n2 -- quotient )
// Special case: Integer / Integer promotes to Rational for exact division
pub fn div_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_with_context("'/' requires exactly 2 values on the stack (e.g., '15 3 /')")?;
    let a = interp.pop_with_context("'/' requires exactly 2 values on the stack (e.g., '15 3 /')")?;

    // Check for division by zero first
    let is_zero = match &b {
        Value::Int32(i) => *i == 0,
        Value::Integer(i) => i.is_zero(),
        Value::Rational(r) => r.is_zero(),
        Value::Number(n) => *n == 0.0,
        #[cfg(feature = "complex_numbers")]
        Value::Complex(c) => c.re == 0.0 && c.im == 0.0,
        _ => false,
    };
    if is_zero {
        return Err(RuntimeError::DivisionByZero);
    }

    // Special case: Int32 / Int32 and Integer / Integer should promote to Rational
    // This ensures exact division results (e.g., 1/2 = 1/2, not 0.5)
    let result = if let (Value::Int32(ia), Value::Int32(ib)) = (&a, &b) {
        // Int32 / Int32 -> Rational (then demote if denominator is 1)
        let result = Value::Rational(BigRational::new(
            num_bigint::BigInt::from(*ia),
            num_bigint::BigInt::from(*ib),
        ));
        result.demote()
    } else if let (Value::Integer(ia), Value::Integer(ib)) = (&a, &b) {
        let result = Value::Rational(BigRational::new(ia.clone(), ib.clone()));
        result.demote()
    } else {
        // For all other type combinations, use standard promotion
        let (pa, pb) = promote_pair(&a, &b);

        match (&pa, &pb) {
            (Value::Rational(r1), Value::Rational(r2)) => {
                let result = Value::Rational(r1 / r2);
                result.demote()
            }
            (Value::Number(n1), Value::Number(n2)) => Value::Number(n1 / n2),
            #[cfg(feature = "complex_numbers")]
            (Value::Complex(c1), Value::Complex(c2)) => Value::Complex(c1 / c2),
            _ => {
                return Err(RuntimeError::TypeError(format!(
                    "Cannot divide {:?} and {:?}",
                    a, b
                )))
            }
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
    fn test_div_impl() {
        let mut interp = AsyncInterpreter::new();

        // Test basic division
        interp.push(Value::Number(20.0));
        interp.push(Value::Number(4.0));
        div_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test fractional division
        interp.push(Value::Number(7.0));
        interp.push(Value::Number(2.0));
        div_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 3.5));
    }

    #[test]
    fn test_div_impl_by_zero() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(10.0));
        interp.push(Value::Number(0.0));
        let result = div_impl(&mut interp);
        assert!(result.is_err());
    }
}
