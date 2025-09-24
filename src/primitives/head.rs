// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: List head extraction (car in Lisp terminology)
// Stack-based head: ( list -- first-element )
// Returns the first element of a list, or error if not a list
pub fn head_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let list = interp.pop()?;

    match list {
        Value::Pair(car, _cdr) => {
            // RUST CONCEPT: Cloning Rc just increments reference count
            interp.push((*car).clone());
            Ok(())
        },
        Value::Nil => {
            Err(RuntimeError::TypeError("Cannot take head of empty list".to_string()))
        },
        _ => {
            Err(RuntimeError::TypeError("head requires a list".to_string()))
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
    fn test_head_builtin_basic() {
        let mut interp = setup_interpreter();

        // Create list [1, 2, 3] and take head
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);

        interp.push(list);
        head_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_head_builtin_single_element() {
        let mut interp = setup_interpreter();

        // Create single-element list ["hello"] and take head
        let list = interp.make_list(vec![Value::String("hello".into())]);

        interp.push(list);
        head_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_head_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Create mixed-type list [true, 42, "world"] and take head
        let list = interp.make_list(vec![
            Value::Boolean(true),
            Value::Number(42.0),
            Value::String("world".into()),
        ]);

        interp.push(list);
        head_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }

    #[test]
    fn test_head_builtin_nested_list() {
        let mut interp = setup_interpreter();

        // Create nested list [[1, 2], 3] and take head
        let inner_list = interp.make_list(vec![Value::Number(1.0), Value::Number(2.0)]);
        let outer_list = interp.make_list(vec![inner_list, Value::Number(3.0)]);

        interp.push(outer_list);
        head_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        // Should get [1, 2] as the head
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::Number(n) if *n == 1.0));
                match cdr.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 2.0));
                        assert!(matches!(cdr2.as_ref(), Value::Nil));
                    },
                    _ => panic!("Expected second element in inner list"),
                }
            },
            _ => panic!("Expected inner list as head"),
        }
    }

    #[test]
    fn test_head_builtin_empty_list() {
        let mut interp = setup_interpreter();

        // Test head of empty list
        interp.push(Value::Nil);
        let result = head_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(msg)) if msg.contains("empty list")));
    }

    #[test]
    fn test_head_builtin_non_list() {
        let mut interp = setup_interpreter();

        // Test head of non-list values
        let test_cases = vec![
            Value::Number(42.0),
            Value::String("not a list".into()),
            Value::Boolean(true),
            Value::Null,
            Value::Atom(interp.intern_atom("atom")),
        ];

        for test_value in test_cases {
            interp.push(test_value);
            let result = head_builtin(&mut interp);
            assert!(matches!(result, Err(RuntimeError::TypeError(msg)) if msg.contains("requires a list")));

            // Clean up stack for next test
            let _ = interp.pop();
        }
    }

    #[test]
    fn test_head_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = head_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}