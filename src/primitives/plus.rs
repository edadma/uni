// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: Polymorphic addition - numbers and string concatenation
// Stack-based addition: ( n1 n2 -- sum ) or ( str1 any -- concat ) or ( any str2 -- concat )
pub fn add_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Context-aware error messages using position-aware pop
    let b =
        interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;
    let a =
        interp.pop_with_context("'+' requires exactly 2 values on the stack (e.g., '5 3 +')")?;

    // RUST CONCEPT: Pattern matching for polymorphic behavior
    match (&a, &b) {
        // Both numbers - do arithmetic addition
        (Value::Number(n1), Value::Number(n2)) => {
            interp.push(Value::Number(n1 + n2));
        }
        // At least one string - do string concatenation
        (Value::String(_), _) | (_, Value::String(_)) => {
            // For string concatenation, extract the actual string content
            let str_a = match &a {
                Value::String(s) => s.as_ref(),
                _ => &a.to_string(), // Convert non-strings using Display
            };
            let str_b = match &b {
                Value::String(s) => s.as_ref(),
                _ => &b.to_string(), // Convert non-strings using Display
            };
            let result = format!("{}{}", str_a, str_b);
            interp.push(Value::String(result.into()));
        }
        // Neither number nor string - type error
        _ => {
            return Err(RuntimeError::TypeError(
                "Addition requires numbers or at least one string".to_string(),
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

    #[test]
    fn test_add_builtin_position_aware_error() {
        use crate::tokenizer::SourcePos;
        let mut interp = setup_interpreter();

        // Set up a source position for error context (mock for testing)
        let pos = SourcePos::new(2, 15, 20); // Line 2, column 15, offset 20
        interp.current_pos = Some(pos);

        // Test stack underflow with position information
        let result = add_builtin(&mut interp);
        assert!(result.is_err());

        match result.unwrap_err() {
            RuntimeError::StackUnderflowAt { pos, context } => {
                assert_eq!(pos.line, 2);
                assert_eq!(pos.column, 15);
                assert_eq!(pos.offset, 20);
                assert!(context.contains("'+' requires exactly 2 values"));
            }
            _ => panic!("Expected StackUnderflowAt error"),
        }

        // Test with only one element on stack
        interp.push(Value::Number(42.0));
        let result = add_builtin(&mut interp);
        assert!(result.is_err());

        match result.unwrap_err() {
            RuntimeError::StackUnderflowAt { pos, context } => {
                assert_eq!(pos.line, 2);
                assert_eq!(pos.column, 15);
                assert!(context.contains("'+' requires exactly 2 values"));
            }
            _ => panic!("Expected StackUnderflowAt error"),
        }
    }

    #[test]
    fn test_demonstrate_formatted_error_output() {
        use crate::tokenizer::SourcePos;
        let mut interp = setup_interpreter();

        // Set up a source position that represents where '+' appears in source code
        let pos = SourcePos::new(3, 8, 45); // Line 3, column 8, offset 45
        interp.current_pos = Some(pos);

        // Try to add without enough values on stack
        let result = add_builtin(&mut interp);
        assert!(result.is_err());

        // Show the formatted error message
        let error = result.unwrap_err();
        let formatted_error = format!("{}", error);

        // Print to demonstrate nice formatting (won't show in normal test run)
        println!("Demo error message: {}", formatted_error);

        // Verify the error message contains expected components
        assert!(formatted_error.contains("line 3, column 8"));
        assert!(formatted_error.contains("'+' requires exactly 2 values"));
        assert!(formatted_error.contains("Stack underflow"));
    }
}
