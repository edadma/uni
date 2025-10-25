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
