// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;
use std::rc::Rc;

// RUST CONCEPT: Cons cell construction (fundamental list operation)
// Stack-based cons: ( element list -- new-list )
// Prepends element to the front of list, creating a new cons cell
pub fn cons_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let cdr = interp.pop()?;  // The list (or tail)
    let car = interp.pop()?;  // The element (or head)

    // RUST CONCEPT: Creating new Pair with Rc for shared ownership
    let new_pair = Value::Pair(Rc::new(car), Rc::new(cdr));
    interp.push(new_pair);
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
    fn test_cons_builtin_basic() {
        let mut interp = setup_interpreter();

        // Test: 1 [] cons -> [1]
        interp.push(Value::Number(1.0));
        interp.push(Value::Nil);
        cons_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));
                assert!(matches!(cdr.as_ref(), Value::Nil));
            },
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_cons_builtin_onto_list() {
        let mut interp = setup_interpreter();

        // Create initial list [2, 3]
        let initial_list = interp.make_list(vec![Value::Number(2.0), Value::Number(3.0)]);

        // Test: 1 [2, 3] cons -> [1, 2, 3]
        interp.push(Value::Number(1.0));
        interp.push(initial_list);
        cons_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                // First element should be 1
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));

                // Rest should be [2, 3]
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 2.0));
                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                assert!(matches!(car3.as_ref(), Value::Number(n) if *n == 3.0));
                                assert!(matches!(cdr3.as_ref(), Value::Nil));
                            },
                            _ => panic!("Expected third element"),
                        }
                    },
                    _ => panic!("Expected second element"),
                }
            },
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_cons_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test: "hello" [42, true] cons -> ["hello", 42, true]
        let mixed_list = interp.make_list(vec![
            Value::Number(42.0),
            Value::Boolean(true),
        ]);

        interp.push(Value::String("hello".into()));
        interp.push(mixed_list);
        cons_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                // First element should be "hello"
                assert!(matches!(car.as_ref(), Value::String(s) if s.as_ref() == "hello"));

                // Rest should be [42, true]
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 42.0));
                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                assert!(matches!(car3.as_ref(), Value::Boolean(true)));
                                assert!(matches!(cdr3.as_ref(), Value::Nil));
                            },
                            _ => panic!("Expected third element"),
                        }
                    },
                    _ => panic!("Expected second element"),
                }
            },
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_cons_builtin_onto_non_list() {
        let mut interp = setup_interpreter();

        // Test: 1 42 cons -> creates improper list (1 . 42)
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(42.0));
        cons_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));
                assert!(matches!(cdr.as_ref(), Value::Number(n) if *n == 42.0));
            },
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_cons_builtin_with_null() {
        let mut interp = setup_interpreter();

        // Test: null 1 cons -> (null . 1)
        interp.push(Value::Null);
        interp.push(Value::Number(1.0));
        cons_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Null));
                assert!(matches!(cdr.as_ref(), Value::Number(n) if *n == 1.0));
            },
            _ => panic!("Expected Pair"),
        }
    }

    #[test]
    fn test_cons_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = cons_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = cons_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}