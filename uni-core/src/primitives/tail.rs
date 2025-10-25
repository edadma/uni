// CDR/TAIL primitive - get the rest of a pair

use crate::compat::ToString;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// CDR: ( [a|b] -- b )
pub fn cdr_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let pair = interp.pop()?;
    match pair {
        Value::Pair(_, cdr) => {
            interp.push((*cdr).clone());
            Ok(())
        }
        _ => Err(RuntimeError::TypeError("CDR requires a pair".to_string())),
    }
}
