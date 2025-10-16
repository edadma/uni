// RUST CONCEPT: I16 buffer indexed read primitive
// Get the value at a specific index in an i16 buffer
use crate::compat::{format, ToString};
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: i16-ref primitive
// Stack: ( buffer index -- value )
// Gets the i16 value at the specified index
pub fn i16_ref_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let index = interp.pop_integer()?;
    let buffer_val = interp.pop()?;

    match buffer_val {
        Value::I16Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if index >= borrowed.len() {
                return Err(RuntimeError::DomainError(format!(
                    "Index {} out of bounds for buffer of length {}",
                    index,
                    borrowed.len()
                )));
            }

            let value = borrowed[index];
            // Convert i16 to Int32 for consistency with other numeric types
            interp.push(Value::Int32(value as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i16-ref expects an i16-buffer".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compat::Rc;

    #[cfg(not(target_os = "none"))]
    use std::cell::RefCell;
    #[cfg(target_os = "none")]
    use core::cell::RefCell;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_i16_ref_basic() {
        let mut interp = setup_interpreter();

        // Create a buffer with some values
        let buffer = vec![10i16, 20, 30, 40, 50];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Get value at index 2 (should be 30)
        interp.push(buffer_val);
        interp.push(Value::Int32(2));
        let result = i16_ref_builtin(&mut interp);
        assert!(result.is_ok());

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(30)));
    }

    #[test]
    fn test_i16_ref_first_element() {
        let mut interp = setup_interpreter();

        let buffer = vec![100i16, 200, 300];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        interp.push(Value::Int32(0));
        i16_ref_builtin(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(100)));
    }

    #[test]
    fn test_i16_ref_last_element() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i16, 20, 30];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        interp.push(Value::Int32(2));
        i16_ref_builtin(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(30)));
    }

    #[test]
    fn test_i16_ref_out_of_bounds() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i16, 20, 30];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        interp.push(Value::Int32(5));
        let result = i16_ref_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_i16_ref_negative_values() {
        let mut interp = setup_interpreter();

        let buffer = vec![-100i16, -200, -300];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        interp.push(Value::Int32(1));
        i16_ref_builtin(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(-200)));
    }

    #[test]
    fn test_i16_ref_type_error() {
        let mut interp = setup_interpreter();

        // Try with non-buffer value
        interp.push(Value::Number(42.0));
        interp.push(Value::Int32(0));
        let result = i16_ref_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_i16_ref_stack_underflow() {
        let mut interp = setup_interpreter();

        // No values on stack
        let result = i16_ref_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}
