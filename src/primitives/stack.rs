// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::compat::format;
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

#[cfg(not(target_os = "none"))]
use std::vec::Vec;
#[cfg(target_os = "none")]
use alloc::vec::Vec;

// RUST CONCEPT: Stack builtin - displays the current stack contents
// Usage: stack  (displays all stack items from top to bottom)
pub fn stack_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    if interp.stack.is_empty() {
        let _ = interp.writeln("Stack is empty");
    } else {
        // RUST CONCEPT: Collect all lines first to avoid borrow checker issues
        // We can't iterate over stack (immutable borrow) while calling writeln (mutable borrow)
        let mut lines = Vec::new();

        let msg = format!("Stack ({} items):", interp.stack.len());
        lines.push(msg);

        // RUST CONCEPT: Platform-specific limits
        // Show fewer items on micro:bit due to limited screen space
        let limit = if cfg!(target_os = "none") { 5 } else { 10 };

        for (i, value) in interp.stack.iter().rev().enumerate() {
            if i >= limit {
                let msg = format!("  ... and {} more", interp.stack.len() - limit);
                lines.push(msg);
                break;
            }
            let msg = format!("  {}: {}", i, value);
            lines.push(msg);
        }

        // Now write all lines with mutable borrow
        for line in lines {
            let _ = interp.writeln(&line);
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
    fn test_stack_builtin_empty() {
        let mut interp = setup_interpreter();

        // Stack should succeed with empty stack
        let result = stack_builtin(&mut interp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stack_builtin_with_values() {
        let mut interp = setup_interpreter();

        // Push some values
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));

        // Stack should succeed and display values
        let result = stack_builtin(&mut interp);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stack_builtin_no_stack_effect() {
        let mut interp = setup_interpreter();

        // Push some values on the stack
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));

        let stack_before = interp.stack.len();

        // Stack should not affect the stack
        let result = stack_builtin(&mut interp);
        assert!(result.is_ok());

        let stack_after = interp.stack.len();
        assert_eq!(stack_before, stack_after);
    }

    #[test]
    fn test_stack_builtin_many_items() {
        let mut interp = setup_interpreter();

        // Push many values (more than the display limit)
        for i in 0..20 {
            interp.push(Value::Number(i as f64));
        }

        // Stack should succeed and truncate display
        let result = stack_builtin(&mut interp);
        assert!(result.is_ok());

        // All items should still be on stack
        assert_eq!(interp.stack.len(), 20);
    }
}
