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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use crate::compat::Rc;

    #[test]
    fn test_cdr_impl() {
        let mut interp = AsyncInterpreter::new();

        let pair = Value::Pair(Rc::new(Value::Number(42.0)), Rc::new(Value::Nil));
        interp.push(pair);
        cdr_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Nil));
    }
}
