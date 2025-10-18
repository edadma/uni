// RUST CONCEPT: Forth-style space output
// Outputs a single space character
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: SPACE builtin
// Outputs a space character to the terminal
// Stack effect: ( -- )
pub fn space_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let _ = interp.write_str(" ");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_space_basic() {
        let mut interp = setup_interpreter();

        // SPACE should succeed
        let result = space_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should remain unchanged (empty)
        assert!(interp.stack.is_empty());
    }

    #[test]
    fn test_space_with_values_on_stack() {
        let mut interp = setup_interpreter();

        // Put some values on stack
        interp.push(crate::value::Value::Int32(42));
        interp.push(crate::value::Value::Int32(100));

        // SPACE should not affect stack
        space_builtin(&mut interp).unwrap();

        assert_eq!(interp.stack.len(), 2);
    }
}
