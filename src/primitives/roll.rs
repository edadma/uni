// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::interpreter::Interpreter;
use crate::value::RuntimeError;

// RUST CONCEPT: ANS Forth roll primitive
// roll ( n -- ) - Move the nth item from stack to top (destructive)
// n=0: no-op, n=1: swap top two items, n=2: move third item to top
// Example: 1 2 3 4  2 roll  ->  1 3 4 2 (item at depth 2 moved to top)
pub fn roll_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_number()? as usize;

    // RUST CONCEPT: Bounds checking
    // We need at least n+1 items on the stack
    if interp.stack.len() < n + 1 {
        return Err(RuntimeError::StackUnderflow);
    }

    if n == 0 {
        // n=0: no operation
        return Ok(());
    }

    // RUST CONCEPT: Vec manipulation
    // Remove the item at position n from the end (0-indexed from top)
    let stack_len = interp.stack.len();
    let item = interp.stack.remove(stack_len - n - 1);

    // Push it to the top
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
    fn test_roll_builtin_basic() {
        let mut interp = setup_interpreter();

        // Test: 1 2 3 4  2 roll  ->  1 3 4 2
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(4.0));
        interp.push(Value::Number(2.0));
        roll_builtin(&mut interp).unwrap();

        // Stack should now be: 1 3 4 2 (from bottom to top)
        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 2.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 4.0));

        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Number(n) if n == 3.0));

        let fourth = interp.pop().unwrap();
        assert!(matches!(fourth, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_roll_builtin_no_op() {
        let mut interp = setup_interpreter();

        // Test: 1 2 3 4  0 roll  ->  1 2 3 4 (no change)
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        interp.push(Value::Number(4.0));
        interp.push(Value::Number(0.0));
        roll_builtin(&mut interp).unwrap();

        // Stack should be unchanged: 1 2 3 4
        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 4.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 3.0));

        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Number(n) if n == 2.0));

        let fourth = interp.pop().unwrap();
        assert!(matches!(fourth, Value::Number(n) if n == 1.0));
    }

    #[test]
    fn test_roll_builtin_swap() {
        let mut interp = setup_interpreter();

        // Test: 1 2  1 roll  ->  2 1 (swap top two items)
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(1.0));
        roll_builtin(&mut interp).unwrap();

        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 1.0));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 2.0));
    }

    #[test]
    fn test_roll_builtin_mixed_types() {
        let mut interp = setup_interpreter();

        // Test with mixed value types
        interp.push(Value::String("hello".into()));
        interp.push(Value::Boolean(true));
        interp.push(Value::Number(42.0));
        interp.push(Value::Number(2.0)); // roll depth
        roll_builtin(&mut interp).unwrap();

        // Should move "hello" to top
        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::String(s) if s.as_ref() == "hello"));

        let second = interp.pop().unwrap();
        assert!(matches!(second, Value::Number(n) if n == 42.0));

        let third = interp.pop().unwrap();
        assert!(matches!(third, Value::Boolean(true)));
    }

    #[test]
    fn test_roll_builtin_stack_underflow() {
        let mut interp = setup_interpreter();

        // Test with empty stack
        let result = roll_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));

        // Test when n > available items
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0)); // Want to roll 2 items but only have 1
        let result = roll_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_roll_builtin_type_error() {
        let mut interp = setup_interpreter();

        // Test with non-number roll count
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::String("not a number".into()));
        let result = roll_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
