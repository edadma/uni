// RUST CONCEPT: String conversion primitive
// Converts any value to its string representation using Display trait
use crate::compat::ToString;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Universal string conversion
// Stack-based conversion: ( any -- string )
pub fn to_string_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;

    // RUST CONCEPT: Use Display trait for consistent string representation
    let string_result = value.to_string();
    interp.push(Value::String(string_result.into()));

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
    fn test_to_string_builtin_number() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(42.0));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "42"));
    }

    #[test]
    fn test_to_string_builtin_negative_number() {
        let mut interp = setup_interpreter();

        interp.push(Value::Number(-3.14));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "-3.14"));
    }

    #[test]
    fn test_to_string_builtin_boolean() {
        let mut interp = setup_interpreter();

        interp.push(Value::Boolean(true));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "true"));

        interp.push(Value::Boolean(false));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "false"));
    }

    #[test]
    fn test_to_string_builtin_string() {
        let mut interp = setup_interpreter();

        // Converting string to string should add quotes (data representation)
        interp.push(Value::String("hello".into()));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "\"hello\""));
    }

    #[test]
    fn test_to_string_builtin_atom() {
        let mut interp = setup_interpreter();

        let atom = interp.intern_atom("test");
        interp.push(Value::Atom(atom));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "test"));
    }

    #[test]
    fn test_to_string_builtin_quoted_atom() {
        let mut interp = setup_interpreter();

        let quoted_atom = interp.intern_atom("quoted");
        interp.push(Value::QuotedAtom(quoted_atom));
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "'quoted"));
    }

    #[test]
    fn test_to_string_builtin_null() {
        let mut interp = setup_interpreter();

        interp.push(Value::Null);
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "null"));
    }

    #[test]
    fn test_to_string_builtin_empty_list() {
        let mut interp = setup_interpreter();

        interp.push(Value::Nil);
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "[]"));
    }

    #[test]
    fn test_to_string_builtin_list() {
        let mut interp = setup_interpreter();

        // Test converting a list to string
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        interp.push(list);
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "[1 2 3]"));
    }

    #[test]
    fn test_to_string_builtin_mixed_list() {
        let mut interp = setup_interpreter();

        // Test converting a mixed list to string
        let mixed_list = interp.make_list(vec![
            Value::String("hello".into()),
            Value::Number(42.0),
            Value::Boolean(true),
        ]);
        interp.push(mixed_list);
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "[\"hello\" 42 true]"));
    }

    #[test]
    fn test_to_string_builtin_array() {
        let mut interp = setup_interpreter();

        let array = interp.make_array(vec![Value::Number(1.0), Value::Number(2.0)]);
        interp.push(array);
        to_string_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "#[1 2]"));
    }

    #[test]
    fn test_to_string_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = to_string_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}
