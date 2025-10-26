// CAR/HEAD primitive - get the first element of a pair

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// CAR: ( [a|b] -- a )
pub fn car_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let pair = interp.pop()?;
    match pair {
        Value::Pair(car, _) => {
            interp.push((*car).clone());
            Ok(())
        }
        _ => Err(RuntimeError::TypeError("CAR requires a pair".to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use crate::compat::Rc;

    #[test]
    fn test_car_impl() {
        let mut interp = AsyncInterpreter::new();

        let pair = Value::Pair(Rc::new(Value::Number(42.0)), Rc::new(Value::Nil));
        interp.push(pair);
        car_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }
}
