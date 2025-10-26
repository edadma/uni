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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_cons_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(1.0));
        interp.push(Value::Nil);
        cons_impl(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));
                assert!(matches!(cdr.as_ref(), Value::Nil));
            }
            _ => panic!("Expected pair"),
        }
    }
}
