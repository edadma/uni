// CONS primitive - construct a pair

use crate::compat::Rc;
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

// CONS: ( a b -- [a|b] )
pub fn cons_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let cdr = interp.pop()?;
    let car = interp.pop()?;
    interp.push(Value::Pair(Rc::new(car), Rc::new(cdr)));
    Ok(())
}
