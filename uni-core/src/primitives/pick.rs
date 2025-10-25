// RUST CONCEPT: ANS Forth pick primitive
// pick ( n -- value ) - Copy the nth item from the stack to the top
// n=0: dup, n=1: over, n=2: pick third item, etc.
use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

pub fn pick_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let n = interp.pop_integer()?;

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
