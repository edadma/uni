// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use num_bigint::BigInt;
use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::ToPrimitive;

// RUST CONCEPT: Polymorphic multiplication - multiple numeric types
// Stack-based multiplication: ( n1 n2 -- product )
// Supports: f64, BigInt, BigRational, GaussianInt, Complex64
pub fn mul_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    match (&a, &b) {
        // ========== NUMERIC MULTIPLICATIONS ==========

        // Float * Float
        (Value::Number(n1), Value::Number(n2)) => {
            interp.push(Value::Number(n1 * n2));
        }

        // BigInt * BigInt
        (Value::Integer(i1), Value::Integer(i2)) => {
            interp.push(Value::Integer(i1 * i2));
        }

        // Rational * Rational
        (Value::Rational(r1), Value::Rational(r2)) => {
            let result = Value::Rational(r1 * r2);
            interp.push(result.demote());
        }

        // Complex * Complex
        (Value::Complex(c1), Value::Complex(c2)) => {
            interp.push(Value::Complex(c1 * c2));
        }

        // GaussianInt * GaussianInt: (a+bi)(c+di) = (ac-bd)+(ad+bc)i
        (Value::GaussianInt(a_re, a_im), Value::GaussianInt(b_re, b_im)) => {
            let ac = a_re * b_re;
            let bd = a_im * b_im;
            let ad = a_re * b_im;
            let bc = a_im * b_re;
            let result = Value::GaussianInt(&ac - &bd, ad + bc);
            interp.push(result.demote());
        }

        // ========== MIXED NUMERIC TYPE MULTIPLICATIONS ==========

        // Float * Integer
        (Value::Number(n), Value::Integer(i)) | (Value::Integer(i), Value::Number(n)) => {
            if n.fract() == 0.0 && n.is_finite() {
                let n_int = BigInt::from(*n as i64);
                interp.push(Value::Integer(&n_int * i));
            } else {
                let r = float_to_rational(*n);
                let i_rat = BigRational::from(i.clone());
                interp.push(Value::Rational(r * i_rat));
            }
        }

        // Float * Rational
        (Value::Number(n), Value::Rational(r)) | (Value::Rational(r), Value::Number(n)) => {
            let n_rat = float_to_rational(*n);
            interp.push(Value::Rational(n_rat * r));
        }

        // Float * Complex
        (Value::Number(n), Value::Complex(c)) | (Value::Complex(c), Value::Number(n)) => {
            interp.push(Value::Complex(Complex64::new(*n, 0.0) * c));
        }

        // Integer * Rational
        (Value::Integer(i), Value::Rational(r)) | (Value::Rational(r), Value::Integer(i)) => {
            let i_rat = BigRational::from(i.clone());
            let result = Value::Rational(i_rat * r);
            interp.push(result.demote());
        }

        // Integer * Complex
        (Value::Integer(i), Value::Complex(c)) | (Value::Complex(c), Value::Integer(i)) => {
            let i_float = i.to_f64().unwrap_or(f64::INFINITY);
            interp.push(Value::Complex(Complex64::new(i_float, 0.0) * c));
        }

        // Rational * Complex
        (Value::Rational(r), Value::Complex(c)) | (Value::Complex(c), Value::Rational(r)) => {
            let r_float = rational_to_float(r);
            interp.push(Value::Complex(Complex64::new(r_float, 0.0) * c));
        }

        // GaussianInt * Integer: (a+bi) * c = ac + bci
        (Value::GaussianInt(re, im), Value::Integer(i))
        | (Value::Integer(i), Value::GaussianInt(re, im)) => {
            let result = Value::GaussianInt(re * i, im * i);
            interp.push(result.demote());
        }

        // GaussianInt * Float -> Complex64 (promote)
        (Value::GaussianInt(re, im), Value::Number(n))
        | (Value::Number(n), Value::GaussianInt(re, im)) => {
            let re_float = re.to_f64().unwrap_or(f64::INFINITY);
            let im_float = im.to_f64().unwrap_or(f64::INFINITY);
            interp.push(Value::Complex(
                Complex64::new(re_float, im_float) * Complex64::new(*n, 0.0),
            ));
        }

        // GaussianInt * Rational -> Complex64 (promote)
        (Value::GaussianInt(re, im), Value::Rational(r))
        | (Value::Rational(r), Value::GaussianInt(re, im)) => {
            let re_float = re.to_f64().unwrap_or(f64::INFINITY);
            let im_float = im.to_f64().unwrap_or(f64::INFINITY);
            let r_float = rational_to_float(r);
            interp.push(Value::Complex(
                Complex64::new(re_float, im_float) * Complex64::new(r_float, 0.0),
            ));
        }

        // GaussianInt * Complex -> Complex64
        (Value::GaussianInt(re, im), Value::Complex(c))
        | (Value::Complex(c), Value::GaussianInt(re, im)) => {
            let re_float = re.to_f64().unwrap_or(f64::INFINITY);
            let im_float = im.to_f64().unwrap_or(f64::INFINITY);
            interp.push(Value::Complex(Complex64::new(re_float, im_float) * c));
        }

        _ => {
            return Err(RuntimeError::TypeError(
                "Multiplication requires numbers".to_string(),
            ));
        }
    }

    Ok(())
}

// Helper function to convert f64 to BigRational
fn float_to_rational(f: f64) -> BigRational {
    use num_bigint::ToBigInt;
    use num_traits::Zero;

    if !f.is_finite() {
        return BigRational::zero();
    }

    let denominator = 1_000_000_000i64;
    let numerator = (f * denominator as f64).round() as i64;

    BigRational::new(
        numerator.to_bigint().unwrap(),
        denominator.to_bigint().unwrap(),
    )
}

// Helper function to convert BigRational to f64
fn rational_to_float(r: &BigRational) -> f64 {
    let numer = r.numer().to_f64().unwrap_or(0.0);
    let denom = r.denom().to_f64().unwrap_or(1.0);
    numer / denom
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_mul_builtin() {
        let mut interp = setup_interpreter();

        // Test basic multiplication: 4 * 3 = 12
        interp.push(Value::Number(4.0));
        interp.push(Value::Number(3.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 12.0));

        // Test with negative numbers: -2 * 7 = -14
        interp.push(Value::Number(-2.0));
        interp.push(Value::Number(7.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == -14.0));

        // Test with zero
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));

        // Test with fractional numbers
        interp.push(Value::Number(2.5));
        interp.push(Value::Number(4.0));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 10.0));
    }

    #[test]
    fn test_mul_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = mul_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = mul_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_mul_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with wrong types
        let atom = interp.intern_atom("not-a-number");
        interp.push(Value::Atom(atom));
        interp.push(Value::Number(5.0));
        let result = mul_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_mul_gaussian_int() {
        let mut interp = setup_interpreter();

        // Test i * i = -1 (fundamental property of i, demoted to Integer)
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(1))); // i
        interp.push(Value::GaussianInt(BigInt::from(0), BigInt::from(1))); // i
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(-1)));

        // Test (3+4i) * (2+i) = 6+3i+8i+4i² = 6+11i-4 = 2+11i
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::GaussianInt(BigInt::from(2), BigInt::from(1)));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(2) && im == &BigInt::from(11)
        ));
    }

    #[test]
    fn test_mul_gaussian_int_with_integer() {
        let mut interp = setup_interpreter();

        // Test (3+4i) * 5 = 15+20i
        interp.push(Value::GaussianInt(BigInt::from(3), BigInt::from(4)));
        interp.push(Value::Integer(BigInt::from(5)));
        mul_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(
            result,
            Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(15) && im == &BigInt::from(20)
        ));
    }
}
