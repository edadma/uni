// Clear builtin - removes all items from the stack
// Usage: clear  (empties the entire stack)

use crate::interpreter::AsyncInterpreter;
use crate::value::RuntimeError;

pub fn clear_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    interp.stack.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_clear_impl() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Number(1.0));
        interp.push(Value::Number(2.0));
        interp.push(Value::Number(3.0));
        clear_impl(&mut interp).unwrap();

        assert_eq!(interp.stack.len(), 0);
    }
}
