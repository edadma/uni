// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Type checking predicates
// null? ( value -- boolean ) - Check if value is null
// Returns true only for Value::Null, false for all other types including Nil
pub fn null_predicate_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let is_null = interp.is_null(&value);
    interp.push(Value::Boolean(is_null));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use crate::compat::Rc;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_null_predicate_builtin() {
        let mut interp = setup_interpreter();

        // Test null? with null value -> true
        interp.push(Value::Null);
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test null? with non-null values -> false
        let test_cases = vec![
            Value::Boolean(false),
            Value::Boolean(true),
            Value::Number(0.0),
            Value::Number(42.0),
            Value::String("".into()),
            Value::String("hello".into()),
            Value::Nil, // Nil is NOT null
            Value::Atom(interp.intern_atom("test")),
            Value::QuotedAtom(interp.intern_atom("quoted")),
            Value::Pair(Rc::new(Value::Number(1.0)), Rc::new(Value::Nil)),
        ];

        for (i, test_value) in test_cases.into_iter().enumerate() {
            interp.push(test_value.clone());
            null_predicate_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Boolean(false)),
                "Expected false for non-null value #{}: {:?}",
                i,
                test_value
            );
        }
    }

    #[test]
    fn test_null_predicate_null_vs_nil_distinction() {
        let mut interp = setup_interpreter();

        // null is null
        interp.push(Value::Null);
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // nil is NOT null
        interp.push(Value::Nil);
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // boolean false is NOT null
        interp.push(Value::Boolean(false));
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // number 0 is NOT null
        interp.push(Value::Number(0.0));
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // empty string is NOT null
        interp.push(Value::String("".into()));
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_null_predicate_edge_cases() {
        let mut interp = setup_interpreter();

        // Test with various edge case values
        let edge_cases = vec![
            Value::Number(-0.0),           // negative zero
            Value::Number(f64::NAN),       // NaN
            Value::Number(f64::INFINITY),  // infinity
            Value::String("null".into()),  // string "null"
            Value::String("false".into()), // string "false"
        ];

        for (i, test_value) in edge_cases.into_iter().enumerate() {
            interp.push(test_value.clone());
            null_predicate_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(
                matches!(result, Value::Boolean(false)),
                "Expected false for edge case #{}: {:?}",
                i,
                test_value
            );
        }
    }

    #[test]
    fn test_null_predicate_list_with_null() {
        let mut interp = setup_interpreter();

        // Test list containing null -> false (list itself is not null)
        let list_with_null = interp.make_list(vec![Value::Null, Value::Number(1.0)]);
        interp.push(list_with_null);
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Test empty list -> false (nil is not null)
        interp.push(Value::Nil);
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_null_predicate_atom_types() {
        let mut interp = setup_interpreter();

        // Test regular atom
        let atom = interp.intern_atom("null");
        interp.push(Value::Atom(atom));
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Test quoted atom
        let quoted_atom = interp.intern_atom("null");
        interp.push(Value::QuotedAtom(quoted_atom));
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_null_predicate_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = null_predicate_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_null_predicate_multiple_calls() {
        let mut interp = setup_interpreter();

        // Test multiple null checks in sequence
        interp.push(Value::Null);
        interp.push(Value::Number(42.0));
        interp.push(Value::Null);

        // Check third value (null)
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Check second value (number)
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Check first value (null)
        null_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }
}
