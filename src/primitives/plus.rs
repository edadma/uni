// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Polymorphic addition - numbers and string concatenation
// Stack-based addition: ( n1 n2 -- sum ) or ( str1 any -- concat ) or ( any str2 -- concat )
pub fn add_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let b = interp.pop()?;
    let a = interp.pop()?;

    // RUST CONCEPT: Pattern matching for polymorphic behavior
    match (&a, &b) {
        // Both numbers - do arithmetic addition
        (Value::Number(n1), Value::Number(n2)) => {
            interp.push(Value::Number(n1 + n2));
        },
        // At least one string - do string concatenation
        (Value::String(_), _) | (_, Value::String(_)) => {
            // For string concatenation, extract the actual string content
            let str_a = match &a {
                Value::String(s) => s.as_ref(),
                _ => &a.to_string()  // Convert non-strings using Display
            };
            let str_b = match &b {
                Value::String(s) => s.as_ref(),
                _ => &b.to_string()  // Convert non-strings using Display
            };
            let result = format!("{}{}", str_a, str_b);
            interp.push(Value::String(result.into()));
        },
        // Neither number nor string - type error
        _ => {
            return Err(RuntimeError::TypeError(
                "Addition requires numbers or at least one string".to_string()
            ));
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
    fn test_add_builtin() {
        let mut interp = setup_interpreter();

        // Test basic addition
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(5.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 8.0));

        // Test with negative numbers
        interp.push(Value::Number(-2.0));
        interp.push(Value::Number(7.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 5.0));

        // Test with zero
        interp.push(Value::Number(0.0));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_add_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test with only one element
        interp.push(Value::Number(5.0));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_add_builtin_string_concatenation() {
        let mut interp = setup_interpreter();

        // Test string + string
        interp.push(Value::String("Hello ".into()));
        interp.push(Value::String("World".into()));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "Hello World"));

        // Test string + number
        interp.push(Value::String("Count: ".into()));
        interp.push(Value::Number(42.0));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "Count: 42"));

        // Test number + string
        interp.push(Value::Number(3.14));
        interp.push(Value::String(" is pi".into()));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "3.14 is pi"));

        // Test string + boolean
        interp.push(Value::String("Result: ".into()));
        interp.push(Value::Boolean(true));
        add_builtin(&mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::String(s) if s.as_ref() == "Result: true"));
    }

    #[test]
    fn test_add_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with incompatible types (no numbers or strings)
        interp.push(Value::Boolean(true));
        interp.push(Value::Boolean(false));
        let result = add_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}