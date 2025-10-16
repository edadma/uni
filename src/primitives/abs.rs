use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use num_traits::Signed;

pub fn abs_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let val = interp.pop()?;

    let result = match val {
        Value::Int32(i) => Value::Int32(i.abs()),
        Value::Integer(i) => Value::Integer(i.abs()),
        Value::Rational(r) => Value::Rational(r.abs()),
        Value::Number(n) => Value::Number(n.abs()),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "abs requires a number, got {}",
                val.type_name()
            )));
        }
    };

    interp.push(result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use num_bigint::BigInt;
    use num_rational::BigRational;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_abs_int32_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Int32(5));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(5)));
    }

    #[test]
    fn test_abs_int32_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Int32(-5));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(5)));
    }

    #[test]
    fn test_abs_int32_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Int32(0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Int32(0)));
    }

    #[test]
    fn test_abs_integer_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Integer(BigInt::from(42)));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(42)));
    }

    #[test]
    fn test_abs_integer_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Integer(BigInt::from(-42)));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Integer(ref i) if i == &BigInt::from(42)));
    }

    #[test]
    fn test_abs_rational_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(3),
            BigInt::from(4),
        )));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Rational(ref r) if r == &BigRational::new(BigInt::from(3), BigInt::from(4))));
    }

    #[test]
    fn test_abs_rational_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Rational(BigRational::new(
            BigInt::from(-3),
            BigInt::from(4),
        )));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Rational(ref r) if r == &BigRational::new(BigInt::from(3), BigInt::from(4))));
    }

    #[test]
    fn test_abs_number_positive() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(5.0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_abs_number_negative() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-5.0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));
    }

    #[test]
    fn test_abs_number_zero() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(0.0));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 0.0));
    }

    #[test]
    fn test_abs_number_decimal() {
        let mut interp = setup_interpreter();
        interp.push(Value::Number(-3.14));

        abs_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if (n - 3.14).abs() < f64::EPSILON));
    }

    #[test]
    fn test_abs_stack_underflow() {
        let mut interp = setup_interpreter();

        let result = abs_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_abs_type_error() {
        let mut interp = setup_interpreter();
        interp.push(Value::String("hello".into()));

        let result = abs_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
