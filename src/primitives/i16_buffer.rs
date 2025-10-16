// RUST CONCEPT: I16 buffer primitive for audio and DSP
// Creates a new i16 buffer with specified size, initialized to zeros
use crate::compat::Rc;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

#[cfg(target_os = "none")]
use alloc::vec;

// RUST CONCEPT: Create i16 buffer primitive
// Stack: ( size -- buffer )
// Creates a new i16 buffer with 'size' samples, all initialized to 0
pub fn i16_buffer_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let size = interp.pop_integer()?;

    // Create vector of zeros with the specified size
    let buffer = vec![0i16; size];

    // Wrap in Rc<RefCell<>> for shared mutable access
    let buffer_value = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

    interp.push(buffer_value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_i16_buffer_create_empty() {
        let mut interp = setup_interpreter();

        // Create a buffer of size 0
        interp.push(Value::Int32(0));
        let result = i16_buffer_builtin(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::I16Buffer(buf) => {
                assert_eq!(buf.borrow().len(), 0);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_buffer_create_small() {
        let mut interp = setup_interpreter();

        // Create a buffer of size 10
        interp.push(Value::Int32(10));
        let result = i16_buffer_builtin(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::I16Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed.len(), 10);
                // All values should be initialized to 0
                assert!(borrowed.iter().all(|&x| x == 0));
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_buffer_create_large() {
        let mut interp = setup_interpreter();

        // Create a buffer of size 1000
        interp.push(Value::Int32(1000));
        let result = i16_buffer_builtin(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::I16Buffer(buf) => {
                assert_eq!(buf.borrow().len(), 1000);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_buffer_stack_underflow() {
        let mut interp = setup_interpreter();

        // Try to create buffer without size on stack
        let result = i16_buffer_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }

    #[test]
    fn test_i16_buffer_type_error() {
        let mut interp = setup_interpreter();

        // Push a non-integer value
        interp.push(Value::String("not a number".into()));
        let result = i16_buffer_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
