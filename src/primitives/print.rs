// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Print builtin - pops and displays the top stack value
// Usage: 42 pr  (prints "42" and removes it from stack)
// Note: We use "pr" instead of "." because "." is reserved for cons pair notation
pub fn print_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;

    // RUST CONCEPT: User-friendly printing - strings without quotes for readability
    match &value {
        Value::String(s) => {
            // For pr primitive, show strings without quotes for user output
            println!("{}", s);
        },
        _ => {
            // For non-strings, use the standard Display format (with quotes for strings in data structures)
            println!("{}", value);
        }
    }

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
    fn test_print_builtin_number() {
        let mut interp = setup_interpreter();

        // Test printing a number
        interp.push(Value::Number(42.0));

        // Note: We can't easily test the actual output without capturing stdout,
        // but we can test that the function succeeds and pops the value
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty after printing
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_string() {
        let mut interp = setup_interpreter();

        interp.push(Value::String("hello world".into()));
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_boolean() {
        let mut interp = setup_interpreter();

        interp.push(Value::Boolean(true));
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        interp.push(Value::Boolean(false));
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_null() {
        let mut interp = setup_interpreter();

        interp.push(Value::Null);
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_atom() {
        let mut interp = setup_interpreter();

        let atom = interp.intern_atom("test");
        interp.push(Value::Atom(atom));
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_quoted_atom() {
        let mut interp = setup_interpreter();

        let quoted_atom = interp.intern_atom("quoted");
        interp.push(Value::QuotedAtom(quoted_atom));
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_empty_list() {
        let mut interp = setup_interpreter();

        interp.push(Value::Nil);
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_list() {
        let mut interp = setup_interpreter();

        // Test printing a list [1, 2, 3]
        let list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ]);
        interp.push(list);
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_mixed_list() {
        let mut interp = setup_interpreter();

        // Test printing a mixed list ["hello", 42, true]
        let mixed_list = interp.make_list(vec![
            Value::String("hello".into()),
            Value::Number(42.0),
            Value::Boolean(true)
        ]);
        interp.push(mixed_list);
        let result = print_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = print_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_print_builtin_multiple_values() {
        let mut interp = setup_interpreter();

        // Test printing multiple values in sequence
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));

        // Print each value
        let result1 = print_builtin(&mut interp);
        assert!(result1.is_ok());

        let result2 = print_builtin(&mut interp);
        assert!(result2.is_ok());

        let result3 = print_builtin(&mut interp);
        assert!(result3.is_ok());

        // Stack should be empty
        assert!(matches!(interp.pop(), Err(RuntimeError::StackUnderflow)));
    }
}