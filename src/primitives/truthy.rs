// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;
use std::rc::Rc;

// RUST CONCEPT: Truthiness predicate for all value types
// Stack-based truthy?: ( value -- boolean )
// Returns true if value is considered "truthy" in conditional contexts
pub fn truthy_predicate_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let is_truthy = interp.is_truthy(&value);
    interp.push(Value::Boolean(is_truthy));
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
    fn test_truthy_predicate_builtin() {
        let mut interp = setup_interpreter();

        // Test truthy values -> true
        let truthy_cases = vec![
            Value::Boolean(true),
            Value::Number(1.0),
            Value::Number(-1.0),
            Value::Number(42.0),
            Value::String("hello".into()),
            Value::String("false".into()),  // String "false" is truthy!
            Value::Atom(interp.intern_atom("test")),
            Value::QuotedAtom(interp.intern_atom("quoted")),
            Value::Pair(Rc::new(Value::Number(1.0)), Rc::new(Value::Nil)),
        ];

        for (i, test_value) in truthy_cases.into_iter().enumerate() {
            interp.push(test_value.clone());
            truthy_predicate_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Boolean(true)),
                "Expected true for truthy value #{}: {:?}", i, test_value);
        }

        // Test falsy values -> false
        let falsy_cases = vec![
            Value::Boolean(false),
            Value::Number(0.0),
            Value::String("".into()),
            Value::Null,
            Value::Nil,
        ];

        for (i, test_value) in falsy_cases.into_iter().enumerate() {
            interp.push(test_value.clone());
            truthy_predicate_builtin(&mut interp).unwrap();
            let result = interp.pop().unwrap();
            assert!(matches!(result, Value::Boolean(false)),
                "Expected false for falsy value #{}: {:?}", i, test_value);
        }

        // Test stack underflow
        let result = truthy_predicate_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_truthy_predicate_edge_cases() {
        let mut interp = setup_interpreter();

        // Edge case: empty string is falsy
        interp.push(Value::String("".into()));
        truthy_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Edge case: zero is falsy
        interp.push(Value::Number(0.0));
        truthy_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Edge case: negative zero is falsy
        interp.push(Value::Number(-0.0));
        truthy_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));

        // Edge case: non-empty list is truthy
        let list = interp.make_list(vec![Value::Null]);
        interp.push(list);
        truthy_predicate_builtin(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));
    }
}