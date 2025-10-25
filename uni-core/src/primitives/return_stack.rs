// Return stack primitives for Forth-like control structures
// These operations enable complex control structures by providing temporary storage

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

// >r (to-R) - Move value from data stack to return stack
// Stack effect: ( x -- ) R:( -- x )
pub fn to_r_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    interp.push_return(value);
    Ok(())
}

// r> (from-R) - Move value from return stack to data stack
// Stack effect: ( -- x ) R:( x -- )
pub fn from_r_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop_return()?;
    interp.push(value);
    Ok(())
}

// r@ (R-fetch) - Copy top of return stack to data stack
// Stack effect: ( -- x ) R:( x -- x )
pub fn r_fetch_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.peek_return()?.clone();
    interp.push(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::AsyncInterpreter;
    use crate::value::Value;

    #[test]
    fn test_to_r_and_from_r() {
        let mut interp = AsyncInterpreter::new();

        // Push value to data stack
        interp.push(Value::Int32(42));
        
        // Move to return stack
        to_r_impl(&mut interp).unwrap();
        assert!(interp.stack.is_empty());
        assert_eq!(interp.return_stack.len(), 1);

        // Move back to data stack
        from_r_impl(&mut interp).unwrap();
        assert_eq!(interp.stack.len(), 1);
        assert!(interp.return_stack.is_empty());

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(42)));
    }

    #[test]
    fn test_r_fetch() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Int32(100));
        to_r_impl(&mut interp).unwrap();

        // Fetch from return stack (should not remove)
        r_fetch_impl(&mut interp).unwrap();
        
        assert_eq!(interp.stack.len(), 1);
        assert_eq!(interp.return_stack.len(), 1);

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(100)));
    }

    #[test]
    fn test_return_stack_multiple_values() {
        let mut interp = AsyncInterpreter::new();

        // Push multiple values
        interp.push(Value::Int32(1));
        to_r_impl(&mut interp).unwrap();
        interp.push(Value::Int32(2));
        to_r_impl(&mut interp).unwrap();
        interp.push(Value::Int32(3));
        to_r_impl(&mut interp).unwrap();

        // Pop in reverse order
        from_r_impl(&mut interp).unwrap();
        let v3 = interp.pop().unwrap();
        from_r_impl(&mut interp).unwrap();
        let v2 = interp.pop().unwrap();
        from_r_impl(&mut interp).unwrap();
        let v1 = interp.pop().unwrap();

        assert!(matches!(v3, Value::Int32(3)));
        assert!(matches!(v2, Value::Int32(2)));
        assert!(matches!(v1, Value::Int32(1)));
    }
}
