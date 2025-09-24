use crate::interpreter::Interpreter;
use crate::value::{Value, RuntimeError};

pub fn add_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop_number()?;
    let a = interp.pop_number()?;
    interp.push(Value::Number(a + b));
    Ok(())
}

pub fn register_builtins(interp: &mut Interpreter) {
    let add_atom = interp.intern_atom("+");
    interp.dictionary.insert(add_atom, Value::Builtin(add_builtin));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_builtin() {
        let mut interp = Interpreter::new();

        // Test successful addition
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));
        assert!(add_builtin(&mut interp).is_ok());

        match interp.pop() {
            Ok(Value::Number(n)) => assert_eq!(n, 8.0),
            _ => panic!("Expected Number(8.0)"),
        }

        // Test with insufficient arguments
        interp.push(Value::Number(1.0));
        match add_builtin(&mut interp) {
            Err(RuntimeError::StackUnderflow) => (),
            _ => panic!("Expected StackUnderflow"),
        }

        // Test with wrong types
        interp.push(Value::Nil);
        interp.push(Value::Number(1.0));
        match add_builtin(&mut interp) {
            Err(RuntimeError::TypeError(msg)) => assert_eq!(msg, "Expected number"),
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_register_builtins() {
        let mut interp = Interpreter::new();
        register_builtins(&mut interp);

        // Check that + is registered
        let plus = interp.intern_atom("+");
        match interp.dictionary.get(&plus) {
            Some(Value::Builtin(_)) => (),
            _ => panic!("Expected + to be registered as a builtin"),
        }

        // Test that the registered builtin works
        interp.push(Value::Number(10.0));
        interp.push(Value::Number(20.0));
        if let Some(Value::Builtin(func)) = interp.dictionary.get(&plus) {
            assert!(func(&mut interp).is_ok());
            match interp.pop() {
                Ok(Value::Number(n)) => assert_eq!(n, 30.0),
                _ => panic!("Expected Number(30.0)"),
            }
        } else {
            panic!("+ should be a builtin");
        }
    }
}