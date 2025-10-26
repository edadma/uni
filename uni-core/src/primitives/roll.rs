// RUST CONCEPT: ANS Forth roll primitive
// roll ( n -- ) - Move the nth item from stack to top (destructive)
// n=0: no-op, n=1: swap top two items, n=2: move third item to top
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

pub fn roll_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_integer()?;

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

    #[test]
    fn test_roll_impl() {
        let mut interp = AsyncInterpreter::new();

        // Test 1 roll (swap)
        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Int32(1));
        roll_impl(&mut interp).unwrap();

        let top = interp.pop().unwrap();
        assert!(matches!(top, Value::Number(n) if n == 1.0));
        let next = interp.pop().unwrap();
        assert!(matches!(next, Value::Number(n) if n == 2.0));

        // Test 0 roll (no-op)
        interp.push(Value::Number(42.0));
        interp.push(Value::Int32(0));
        roll_impl(&mut interp).unwrap();
        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Number(n) if n == 42.0));
    }
}
