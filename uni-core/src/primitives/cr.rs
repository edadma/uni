// RUST CONCEPT: Forth-style carriage return
// Outputs a newline character
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: CR (carriage return) builtin
// Outputs a newline to the terminal
// Stack effect: ( -- )
pub fn cr_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let _ = interp.write_str("\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_cr_basic() {
        let mut interp = setup_interpreter();

        // CR should succeed
        let result = cr_builtin(&mut interp);
        assert!(result.is_ok());

        // Stack should remain unchanged (empty)
        assert!(interp.stack.is_empty());
    }

    #[test]
    fn test_cr_with_values_on_stack() {
        let mut interp = setup_interpreter();

        // Put some values on stack
        interp.push(crate::value::Value::Int32(42));
        interp.push(crate::value::Value::Int32(100));

        // CR should not affect stack
        cr_builtin(&mut interp).unwrap();

        assert_eq!(interp.stack.len(), 2);
    }
}
