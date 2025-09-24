// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::RuntimeError;
use crate::interpreter::Interpreter;

// RUST CONCEPT: ANS Forth pick primitive
// pick ( n -- value ) - Copy the nth item from the stack to the top
// n=0: dup, n=1: over, n=2: pick third item, etc.
// Example: 1 2 3 4  2 pick  ->  1 2 3 4 2 (copied item at depth 2 to top)
pub fn pick_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()? as usize;

    // RUST CONCEPT: Bounds checking
    // We need at least n+1 items on the remaining stack
    if interp.stack.len() < n + 1 {
        return Err(RuntimeError::StackUnderflow);
    }

    // RUST CONCEPT: Vec indexing from the end
    // Get the item at position n from the top (0-indexed)
    let stack_len = interp.stack.len();
    let item = interp.stack[stack_len - n - 1].clone();

    // Push a copy to the top
    interp.stack.push(item);

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
    fn test_pick_builtin_dup() {
        let mut interp = setup_interpreter();

        // Test: 1 2 3  0 pick  ->  1 2 3 3 (dup)
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(0.0));
        pick_builtin(&mut interp).unwrap();

        // Stack should be: 1 2 3 3
        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 3.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 3.0));

        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Number(n) if n == 2.0));

        let fourth = interp.pop().unwrap();
        assert!(matches!(fourth, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_pick_builtin_over() {
        let mut interp = setup_interpreter();

        // Test: 1 2 3  1 pick  ->  1 2 3 2 (over)
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(1.0));
        pick_builtin(&mut interp).unwrap();

        // Stack should be: 1 2 3 2
        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 2.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 3.0));

        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Number(n) if n == 2.0));

        let fourth = interp.pop().unwrap();
        assert!(matches!(fourth, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_pick_builtin_deep() {
        let mut interp = setup_interpreter();

        // Test: 1 2 3 4 5  3 pick  ->  1 2 3 4 5 2
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(4.0));
        interp.push(Value::Number(5.0));
        interp.push(Value::Number(3.0));
        pick_builtin(&mut interp).unwrap();

        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 2.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 5.0));

        // Verify original stack is preserved
        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Number(n) if n == 4.0));

        let fourth = interp.pop().unwrap();
        assert!(matches!(fourth, Value::Number(n) if n == 3.0));

        let fifth = interp.pop().unwrap();
        assert!(matches!(fifth, Value::Number(n) if n == 2.0));

        let sixth = interp.pop().unwrap();
        assert!(matches!(sixth, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_pick_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test with mixed value types
        interp.push(Value::String("hello".into()));
        interp.push(Value::Boolean(true));
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(2.0)); // pick depth
        pick_builtin(&mut interp).unwrap();

        // Should copy "hello" to top
        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::String(s) if s.as_ref() == "hello"));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 42.0));

        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Boolean(true)));

        // Original should still be there
        let fourth = interp.pop().unwrap();
        assert!(matches!(fourth, Value::String(s) if s.as_ref() == "hello"));
    }

    #[test]
    fn test_pick_builtin_single_element() {
        let mut interp = setup_interpreter();

        // Test: 42  0 pick  ->  42 42
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(0.0));
        pick_builtin(&mut interp).unwrap();

        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 42.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 42.0));
    }

    #[test]
    fn test_pick_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = pick_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test when n >= available items
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(1.0)); // Want to pick item at depth 1 but only have 1 item total
        let result = pick_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_pick_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with non-number pick count
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::String("not a number".into()));
        let result = pick_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_pick_builtin_preserves_original_stack() {
        let mut interp = setup_interpreter();

        // Verify that pick doesn't modify the original stack items
        interp.push(Value::Number(10.0));
        interp.push(Value::Number(20.0));
        interp.push(Value::Number(30.0));
        interp.push(Value::Number(1.0)); // pick second item (20)
        pick_builtin(&mut interp).unwrap();

        // Should have: 10 20 30 20
        let copied = interp.pop().unwrap();
        assert!(matches!(copied, Value::Number(n) if n == 20.0));

        // Original stack should be intact
        let original_top = interp.pop().unwrap();
        assert!(matches!(original_top, Value::Number(n) if n == 30.0));

        let original_second = interp.pop().unwrap();
        assert!(matches!(original_second, Value::Number(n) if n == 20.0));

        let original_third = interp.pop().unwrap();
        assert!(matches!(original_third, Value::Number(n) if n == 10.0));
    }
}