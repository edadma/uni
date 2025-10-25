// RUST CONCEPT: Numeric type promotion system
// This module handles automatic type promotion for arithmetic operations
// It ensures that operations between different numeric types produce sensible results

use crate::value::Value;
use num_bigint::BigInt;
#[cfg(feature = "complex_numbers")]
use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::ToPrimitive;

// RUST CONCEPT: Type promotion hierarchy
// The promotion hierarchy prioritizes exactness:
// Integer < Rational (both exact)
// Number and Complex (both inexact, same level)
// GaussianInt < Complex (exact complex to inexact complex)
//
// Mixing exact and inexact types promotes to inexact
// Once you introduce a float, precision is lost

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NumericType {
    Int32,       // i32 - fixed-size signed integer (embedded-friendly)
    Integer,     // BigInt - exact integers
    Rational,    // BigRational - exact fractions (still exact)
    #[cfg(feature = "complex_numbers")]
    GaussianInt, // Gaussian integers (a+bi where a,b are integers, exact)
    Number,      // f64 - floating point (inexact, loses precision)
    #[cfg(feature = "complex_numbers")]
    Complex,     // Complex64 - complex floating point (inexact)
}

// RUST CONCEPT: Determine the type of a numeric value
fn numeric_type(val: &Value) -> Option<NumericType> {
    match val {
        Value::Int32(_) => Some(NumericType::Int32),
        Value::Integer(_) => Some(NumericType::Integer),
        Value::Rational(_) => Some(NumericType::Rational),
        Value::Number(_) => Some(NumericType::Number),
        #[cfg(feature = "complex_numbers")]
        Value::GaussianInt(_, _) => Some(NumericType::GaussianInt),
        #[cfg(feature = "complex_numbers")]
        Value::Complex(_) => Some(NumericType::Complex),
        _ => None,
    }
}

// RUST CONCEPT: Promote a value to a target numeric type
fn promote_to(val: &Value, target: NumericType) -> Value {
    match (val, target) {
        // Already the target type
        (Value::Int32(_), NumericType::Int32) => val.clone(),
        (Value::Integer(_), NumericType::Integer) => val.clone(),
        (Value::Rational(_), NumericType::Rational) => val.clone(),
        (Value::Number(_), NumericType::Number) => val.clone(),
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(_, _), NumericType::GaussianInt) => val.clone(),
        #[cfg(feature = "complex_numbers")]
        (Value::Complex(_), NumericType::Complex) => val.clone(),

        // Promote Int32 to higher types
        (Value::Int32(i), NumericType::Integer) => Value::Integer(BigInt::from(*i)),
        (Value::Int32(i), NumericType::Rational) => {
            Value::Rational(BigRational::from(BigInt::from(*i)))
        }
        (Value::Int32(i), NumericType::Number) => Value::Number(*i as f64),
        #[cfg(feature = "complex_numbers")]
        (Value::Int32(i), NumericType::GaussianInt) => {
            Value::GaussianInt(BigInt::from(*i), BigInt::from(0))
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Int32(i), NumericType::Complex) => Value::Complex(Complex64::new(*i as f64, 0.0)),

        // Promote Integer to higher types
        (Value::Integer(i), NumericType::Rational) => {
            Value::Rational(BigRational::from(i.clone()))
        }
        (Value::Integer(i), NumericType::Number) => {
            Value::Number(i.to_f64().unwrap_or(f64::INFINITY))
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Integer(i), NumericType::GaussianInt) => {
            Value::GaussianInt(i.clone(), BigInt::from(0))
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Integer(i), NumericType::Complex) => {
            let n = i.to_f64().unwrap_or(f64::INFINITY);
            Value::Complex(Complex64::new(n, 0.0))
        }

        // Promote Rational to higher types
        (Value::Rational(r), NumericType::Number) => {
            let numer = r.numer().to_f64().unwrap_or(0.0);
            let denom = r.denom().to_f64().unwrap_or(1.0);
            Value::Number(numer / denom)
        }
        #[cfg(feature = "complex_numbers")]
        (Value::Rational(r), NumericType::Complex) => {
            let numer = r.numer().to_f64().unwrap_or(0.0);
            let denom = r.denom().to_f64().unwrap_or(1.0);
            Value::Complex(Complex64::new(numer / denom, 0.0))
        }

        // Promote Number to Complex
        #[cfg(feature = "complex_numbers")]
        (Value::Number(n), NumericType::Complex) => Value::Complex(Complex64::new(*n, 0.0)),

        // Promote GaussianInt to Complex
        #[cfg(feature = "complex_numbers")]
        (Value::GaussianInt(re, im), NumericType::Complex) => {
            let re_f = re.to_f64().unwrap_or(f64::INFINITY);
            let im_f = im.to_f64().unwrap_or(f64::INFINITY);
            Value::Complex(Complex64::new(re_f, im_f))
        }

        // Invalid promotions (can't demote or cross between incompatible types)
        _ => val.clone(), // Fallback: return unchanged
    }
}

// RUST CONCEPT: Promote two values to a common type for arithmetic
// Returns (promoted_a, promoted_b)
pub fn promote_pair(a: &Value, b: &Value) -> (Value, Value) {
    let type_a = numeric_type(a);
    let type_b = numeric_type(b);

    match (type_a, type_b) {
        (Some(ta), Some(tb)) => {
            // Determine the target type (the "higher" one in the hierarchy)
            let target = if ta >= tb { ta.clone() } else { tb.clone() };

            // Handle special case: GaussianInt + Number should go to Complex
            #[cfg(feature = "complex_numbers")]
            let target = match (&ta, &tb) {
                (NumericType::GaussianInt, NumericType::Number)
                | (NumericType::Number, NumericType::GaussianInt) => NumericType::Complex,
                (NumericType::GaussianInt, NumericType::Rational)
                | (NumericType::Rational, NumericType::GaussianInt) => NumericType::Complex,
                _ => target,
            };

            #[cfg(not(feature = "complex_numbers"))]
            let target = target;

            (promote_to(a, target.clone()), promote_to(b, target))
        }
        // If either value is not numeric, return unchanged
        _ => (a.clone(), b.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promote_same_types() {
        let a = Value::Integer(BigInt::from(5));
        let b = Value::Integer(BigInt::from(3));
        let (pa, pb) = promote_pair(&a, &b);
        assert!(matches!(pa, Value::Integer(_)));
        assert!(matches!(pb, Value::Integer(_)));
    }

    #[test]
    fn test_promote_integer_to_rational() {
        let a = Value::Integer(BigInt::from(5));
        let b = Value::Rational(BigRational::from(BigInt::from(3)));
        let (pa, pb) = promote_pair(&a, &b);
        assert!(matches!(pa, Value::Rational(_)));
        assert!(matches!(pb, Value::Rational(_)));
    }

    #[test]
    fn test_promote_integer_to_number() {
        let a = Value::Integer(BigInt::from(5));
        let b = Value::Number(3.14);
        let (pa, pb) = promote_pair(&a, &b);
        assert!(matches!(pa, Value::Number(_)));
        assert!(matches!(pb, Value::Number(_)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_promote_number_to_complex() {
        let a = Value::Number(5.0);
        let b = Value::Complex(Complex64::new(3.0, 4.0));
        let (pa, pb) = promote_pair(&a, &b);
        assert!(matches!(pa, Value::Complex(_)));
        assert!(matches!(pb, Value::Complex(_)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_promote_gaussian_to_complex() {
        let a = Value::GaussianInt(BigInt::from(5), BigInt::from(2));
        let b = Value::Number(3.14);
        let (pa, pb) = promote_pair(&a, &b);
        assert!(matches!(pa, Value::Complex(_)));
        assert!(matches!(pb, Value::Complex(_)));
    }
}
