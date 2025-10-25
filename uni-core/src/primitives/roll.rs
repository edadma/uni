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
