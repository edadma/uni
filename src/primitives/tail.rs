// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: List operations - tail builtin
// tail ( list -- list ) - Get rest of list after first element
// Example: [1 2 3] tail -> [2 3]
// Example: [42] tail -> []
// Example: [] tail -> []
pub fn tail_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let list = interp.pop()?;

    match list {
        Value::Pair(_, cdr) => {
            // Return the rest of the list (cdr)
            interp.push((*cdr).clone());
            Ok(())
        }
        Value::Nil => {
            // Tail of empty list is empty list
            interp.push(Value::Nil);
            Ok(())
        }
        _ => {
            // Not a list
            Err(RuntimeError::TypeError("tail expects a list".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_tail_builtin_basic() {
        let mut interp = setup_interpreter();

        // Test tail of [1 2 3] -> [2 3]
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        interp.push(list);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();

        // Verify structure: [2 3]
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 2.0));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 3.0));
                        assert!(matches!(cdr2.as_ref(), Value::Nil));
                    }
                    _ => panic!("Expected second element in tail"),
                }
            }
            _ => panic!("Expected list structure for tail result"),
        }
    }

    #[test]
    fn test_tail_builtin_single_element() {
        let mut interp = setup_interpreter();

        // Test tail of [42] -> []
        let single = interp.make_list(vec![Value::Number(42.0)]);
        interp.push(single);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_tail_builtin_empty_list() {
        let mut interp = setup_interpreter();

        // Test tail of [] -> []
        interp.push(Value::Nil);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_tail_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test tail of ["hello" 42 true] -> [42 true]
        let mixed_list = interp.make_list(vec![
            Value::String("hello".into()),
            Value::Number(42.0),
            Value::Boolean(true),
        ]);
        interp.push(mixed_list);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();

        // Verify structure: [42 true]
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 42.0));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Boolean(true)));
                        assert!(matches!(cdr2.as_ref(), Value::Nil));
                    }
                    _ => panic!("Expected second element in tail"),
                }
            }
            _ => panic!("Expected list structure for tail result"),
        }
    }

    #[test]
    fn test_tail_builtin_nested_lists() {
        let mut interp = setup_interpreter();

        // Test tail with nested structure: [1 [2 3] 4] -> [[2 3] 4]
        let inner_list = interp.make_list(vec![Value::Number(2.0), Value::Number(3.0)]);
        let outer_list = interp.make_list(vec![Value::Number(1.0), inner_list, Value::Number(4.0)]);

        interp.push(outer_list);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();

        // Verify structure: [[2 3] 4]
        match result {
            Value::Pair(car, cdr) => {
                // First element should be [2 3]
                assert!(matches!(car.as_ref(), Value::Pair(_, _)));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 4.0));
                        assert!(matches!(cdr2.as_ref(), Value::Nil));
                    }
                    _ => panic!("Expected second element in tail"),
                }
            }
            _ => panic!("Expected list structure for tail result"),
        }
    }

    #[test]
    fn test_tail_builtin_improper_list() {
        let mut interp = setup_interpreter();

        // Test tail of improper list (1 . 2) -> 2
        use std::rc::Rc;
        let improper = Value::Pair(Rc::new(Value::Number(1.0)), Rc::new(Value::Number(2.0)));

        interp.push(improper);
        tail_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_tail_builtin_non_list_error() {
        let mut interp = setup_interpreter();

        // Test tail of number should error
        interp.push(Value::Number(42.0));
        let result = tail_builtin(&mut interp);
        assert!(
            matches!(result, Err(RuntimeError::TypeError(msg)) if msg.contains("tail expects a list"))
        );

        // Test tail of atom should error
        let atom = interp.intern_atom("test");
        interp.push(Value::Atom(atom));
        let result = tail_builtin(&mut interp);
        assert!(
            matches!(result, Err(RuntimeError::TypeError(msg)) if msg.contains("tail expects a list"))
        );

        // Test tail of string should error
        interp.push(Value::String("hello".into()));
        let result = tail_builtin(&mut interp);
        assert!(
            matches!(result, Err(RuntimeError::TypeError(msg)) if msg.contains("tail expects a list"))
        );
    }

    #[test]
    fn test_tail_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = tail_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_tail_builtin_preserves_structure() {
        let mut interp = setup_interpreter();

        // Verify that tail preserves the original list structure (shares cdr)
        let original_list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);

        // Take tail
        interp.push(original_list);
        tail_builtin(&mut interp).unwrap();
        let tail_result = interp.pop().unwrap();

        // Verify tail is exactly [2 3] with correct structure
        match tail_result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 2.0));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 3.0));
                        assert!(matches!(cdr2.as_ref(), Value::Nil));
                    }
                    _ => panic!("Tail structure incorrect"),
                }
            }
            _ => panic!("Expected proper tail structure"),
        }
    }
}
