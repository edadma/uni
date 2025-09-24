// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::RuntimeError;
use crate::interpreter::Interpreter;

// RUST CONCEPT: List construction from multiple stack elements
// Stack-based list: ( element1 element2 ... elementN count -- list )
// Creates a list from the top 'count' stack elements in reverse order
pub fn list_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let count_value = interp.pop_number()?;

    // RUST CONCEPT: Input validation
    if count_value < 0.0 || count_value.fract() != 0.0 {
        return Err(RuntimeError::TypeError("list count must be a non-negative integer".to_string()));
    }

    let count = count_value as usize;

    // RUST CONCEPT: Collect elements from stack
    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        elements.push(interp.pop()?);
    }

    // RUST CONCEPT: Elements are in reverse order (stack is LIFO)
    // We need to reverse them to get the correct list order
    elements.reverse();

    // RUST CONCEPT: Use interpreter's list construction helper
    let list = interp.make_list(elements);
    interp.push(list);
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
    fn test_list_builtin_basic() {
        let mut interp = setup_interpreter();

        // Test: 1 2 3 3 list -> [1, 2, 3]
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(3.0)); // count
        list_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();

        // Verify the list structure: [1, 2, 3]
        match result {
            Value::Pair(car1, cdr1) => {
                assert!(matches!(car1.as_ref(), Value::Number(n) if *n == 1.0));
                match cdr1.as_ref() {
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
            _ => panic!("Expected list structure"),
        }
    }

    #[test]
    fn test_list_builtin_empty() {
        let mut interp = setup_interpreter();

        // Test: 0 list -> []
        interp.push(Value::Number(0.0));
        list_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Nil));
    }

    #[test]
    fn test_list_builtin_single_element() {
        let mut interp = setup_interpreter();

        // Test: "hello" 1 list -> ["hello"]
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(1.0));
        list_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        match result {
            Value::Pair(car, cdr) => {
                assert!(matches!(car.as_ref(), Value::String(s) if s.as_ref() == "hello"));
                assert!(matches!(cdr.as_ref(), Value::Nil));
            },
            _ => panic!("Expected single-element list"),
        }
    }

    #[test]
    fn test_list_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test: "hello" 42 true null 4 list -> ["hello", 42, true, null]
        interp.push(Value::String("hello".into()));
        interp.push(Value::Number(42.0));
        interp.push(Value::Boolean(true));
        interp.push(Value::Null);
        interp.push(Value::Number(4.0)); // count
        list_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();

        // Verify mixed-type list: ["hello", 42, true, null]
        match result {
            Value::Pair(car1, cdr1) => {
                assert!(matches!(car1.as_ref(), Value::String(s) if s.as_ref() == "hello"));
                match cdr1.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Number(n) if *n == 42.0));
                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                assert!(matches!(car3.as_ref(), Value::Boolean(true)));
                                match cdr3.as_ref() {
                                    Value::Pair(car4, cdr4) => {
                                        assert!(matches!(car4.as_ref(), Value::Null));
                                        assert!(matches!(cdr4.as_ref(), Value::Nil));
                                    },
                                    _ => panic!("Expected fourth element"),
                                }
                            },
                            _ => panic!("Expected third element"),
                        }
                    },
                    _ => panic!("Expected second element"),
                }
            },
            _ => panic!("Expected list structure"),
        }
    }

    #[test]
    fn test_list_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = list_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test when count > available elements
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(5.0)); // count = 5, but only 1 element available
        let result = list_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_list_builtin_count_type_error() {
        let mut interp = setup_interpreter();

        // Test with non-number count
        interp.push(Value::Number(1.0));
        interp.push(Value::String("not a count".into()));
        let result = list_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_list_builtin_negative_count() {
        let mut interp = setup_interpreter();

        // Test with negative count (should give validation error)
        interp.push(Value::Number(-1.0));
        let result = list_builtin(&mut interp);

        // Should get validation error for negative count
        assert!(matches!(result, Err(RuntimeError::TypeError(msg)) if msg.contains("non-negative integer")));
    }

    #[test]
    fn test_list_builtin_preserves_order() {
        let mut interp = setup_interpreter();

        // Test that elements appear in the correct order
        // Push in order: a, b, c
        let atom_a = interp.intern_atom("a");
        let atom_b = interp.intern_atom("b");
        let atom_c = interp.intern_atom("c");

        interp.push(Value::Atom(atom_a.clone()));
        interp.push(Value::Atom(atom_b.clone()));
        interp.push(Value::Atom(atom_c.clone()));
        interp.push(Value::Number(3.0));
        list_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();

        // Should get [a, b, c] (same order as pushed)
        match result {
            Value::Pair(car1, cdr1) => {
                assert!(matches!(car1.as_ref(), Value::Atom(a) if a == &atom_a));
                match cdr1.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Atom(b) if b == &atom_b));
                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                assert!(matches!(car3.as_ref(), Value::Atom(c) if c == &atom_c));
                                assert!(matches!(cdr3.as_ref(), Value::Nil));
                            },
                            _ => panic!("Expected third element"),
                        }
                    },
                    _ => panic!("Expected second element"),
                }
            },
            _ => panic!("Expected list structure"),
        }
    }
}