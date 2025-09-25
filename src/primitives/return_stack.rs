// RUST CONCEPT: Return stack primitives for Forth-like control structures
// These operations enable complex control structures by providing temporary storage
use crate::value::RuntimeError;
use crate::interpreter::Interpreter;

// RUST CONCEPT: >r (to-R) - Move value from data stack to return stack
// Stack effect: ( x -- ) R:( -- x )
// This moves the top value from the data stack to the return stack
pub fn to_r_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Pop from data stack and push to return stack
    let value = interp.pop()?;
    interp.push_return(value);
    Ok(())
}

// RUST CONCEPT: r> (from-R) - Move value from return stack to data stack
// Stack effect: ( -- x ) R:( x -- )
// This moves the top value from the return stack to the data stack
pub fn from_r_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Pop from return stack and push to data stack
    let value = interp.pop_return()?;
    interp.push(value);
    Ok(())
}

// RUST CONCEPT: r@ (R-fetch) - Copy top of return stack to data stack
// Stack effect: ( -- x ) R:( x -- x )
// This copies (non-destructively) the top value from return stack to data stack
pub fn r_fetch_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Peek at return stack and clone value to data stack
    let value = interp.peek_return()?.clone();
    interp.push(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_to_r_builtin() {
        let mut interp = setup_interpreter();

        // Test moving number to return stack
        interp.push(Value::Number(42.0));
        to_r_builtin(&mut interp).unwrap();

        // Data stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));

        // Return stack should have the value
        let r_value = interp.pop_return().unwrap();
        assert!(matches!(r_value, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_from_r_builtin() {
        let mut interp = setup_interpreter();

        // Push value to return stack directly for testing
        interp.push_return(Value::Number(17.0));
        from_r_builtin(&mut interp).unwrap();

        // Return stack should be empty
        assert!(matches!(interp.pop_return(), Err(RuntimeError::StackUnderflow)));

        // Data stack should have the value
        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Number(n) if n == 17.0));
    }

    #[test]
    fn test_r_fetch_builtin() {
        let mut interp = setup_interpreter();

        // Push value to return stack directly for testing
        interp.push_return(Value::Number(99.0));
        r_fetch_builtin(&mut interp).unwrap();

        // Return stack should still have the value (non-destructive)
        let r_value = interp.pop_return().unwrap();
        assert!(matches!(r_value, Value::Number(n) if n == 99.0));

        // Data stack should also have the value (copy)
        let d_value = interp.pop().unwrap();
        assert!(matches!(d_value, Value::Number(n) if n == 99.0));
    }

    #[test]
    fn test_return_stack_various_types() {
        let mut interp = setup_interpreter();

        // Test with different value types
        let atom = interp.intern_atom("test");
        interp.push(Value::Atom(atom.clone()));
        to_r_builtin(&mut interp).unwrap();

        interp.push(Value::Boolean(true));
        to_r_builtin(&mut interp).unwrap();

        interp.push(Value::String("hello".into()));
        to_r_builtin(&mut interp).unwrap();

        // Pop them back in reverse order
        from_r_builtin(&mut interp).unwrap();
        let str_val = interp.pop().unwrap();
        assert!(matches!(str_val, Value::String(s) if &*s == "hello"));

        from_r_builtin(&mut interp).unwrap();
        let bool_val = interp.pop().unwrap();
        assert!(matches!(bool_val, Value::Boolean(true)));

        from_r_builtin(&mut interp).unwrap();
        let atom_val = interp.pop().unwrap();
        if let Value::Atom(a) = atom_val {
            assert_eq!(&*a, "test");
        } else {
            panic!("Expected atom");
        }
    }

    #[test]
    fn test_return_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test from_r with empty return stack
        assert!(matches!(from_r_builtin(&mut interp), Err(RuntimeError::StackUnderflow)));

        // Test r_fetch with empty return stack
        assert!(matches!(r_fetch_builtin(&mut interp), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_data_stack_underflow_for_to_r() {
        let mut interp = setup_interpreter();

        // Test to_r with empty data stack
        assert!(matches!(to_r_builtin(&mut interp), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_return_stack_sequence() {
        let mut interp = setup_interpreter();

        // Test a sequence: push two values, move to R, then fetch and pop
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));

        to_r_builtin(&mut interp).unwrap();  // 2 -> R
        to_r_builtin(&mut interp).unwrap();  // 1 -> R

        // Return stack now has: [2, 1] (top is 1)
        // Data stack is empty

        r_fetch_builtin(&mut interp).unwrap();  // Copy 1 to data stack
        let peek_val = interp.pop().unwrap();
        assert!(matches!(peek_val, Value::Number(n) if n == 1.0));

        from_r_builtin(&mut interp).unwrap();  // Move 1 from R to data
        let val1 = interp.pop().unwrap();
        assert!(matches!(val1, Value::Number(n) if n == 1.0));

        from_r_builtin(&mut interp).unwrap();  // Move 2 from R to data
        let val2 = interp.pop().unwrap();
        assert!(matches!(val2, Value::Number(n) if n == 2.0));

        // Both stacks should now be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
        assert!(matches!(interp.pop_return(), Err(RuntimeError::StackUnderflow)));
    }
}