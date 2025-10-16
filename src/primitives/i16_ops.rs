// RUST CONCEPT: I16 buffer operations (length, push, pop)
// Basic operations for working with i16 buffers
use crate::compat::{format, ToString};
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use num_traits::ToPrimitive;

// RUST CONCEPT: i16-length primitive
// Stack: ( buffer -- buffer length )
// Returns the length of the buffer (keeps buffer on stack)
pub fn i16_length_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I16Buffer(buffer) => {
            let len = buffer.borrow().len();
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Int32(len as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i16-length expects an i16-buffer".to_string(),
        )),
    }
}

// RUST CONCEPT: i16-push primitive
// Stack: ( value buffer -- buffer )
// Following Forth convention where value comes first
// Appends a value to the end of the buffer
pub fn i16_push_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;
    let value_to_push = interp.pop()?;

    // Convert value to i16
    let i16_value = match value_to_push {
        Value::Int32(i) => {
            if i < i16::MIN as i32 || i > i16::MAX as i32 {
                return Err(RuntimeError::DomainError(format!(
                    "Value {} out of i16 range ({} to {})",
                    i,
                    i16::MIN,
                    i16::MAX
                )));
            }
            i as i16
        }
        Value::Number(n) => {
            if n < i16::MIN as f64 || n > i16::MAX as f64 {
                return Err(RuntimeError::DomainError(format!(
                    "Value {} out of i16 range ({} to {})",
                    n,
                    i16::MIN,
                    i16::MAX
                )));
            }
            n as i16
        }
        Value::Integer(i) => i.to_i16().ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Value {} out of i16 range ({} to {})",
                i,
                i16::MIN,
                i16::MAX
            ))
        })?,
        _ => {
            return Err(RuntimeError::TypeError(
                "i16-push expects a number for value".to_string(),
            ))
        }
    };

    match buffer_val {
        Value::I16Buffer(buffer) => {
            buffer.borrow_mut().push(i16_value);
            interp.push(Value::I16Buffer(buffer));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i16-push expects an i16-buffer".to_string(),
        )),
    }
}

// RUST CONCEPT: i16-pop primitive
// Stack: ( buffer -- buffer value )
// Removes and returns the last value from the buffer
pub fn i16_pop_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match buffer_val {
        Value::I16Buffer(buffer) => {
            let popped = buffer.borrow_mut().pop().ok_or_else(|| {
                RuntimeError::DomainError("Cannot pop from empty buffer".to_string())
            })?;

            interp.push(Value::I16Buffer(buffer));
            interp.push(Value::Int32(popped as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i16-pop expects an i16-buffer".to_string(),
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

    // i16-length tests
    #[test]
    fn test_i16_length_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_length_builtin(&mut interp).unwrap();

        let len = interp.pop().unwrap();
        assert!(matches!(len, Value::Int32(0)));
    }

    #[test]
    fn test_i16_length_non_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![1i16, 2, 3, 4, 5];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_length_builtin(&mut interp).unwrap();

        let len = interp.pop().unwrap();
        assert!(matches!(len, Value::Int32(5)));

        // Buffer should still be on stack
        let buf = interp.pop().unwrap();
        assert!(matches!(buf, Value::I16Buffer(_)));
    }

    // i16-push tests
    #[test]
    fn test_i16_push_to_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(42));
        interp.push(buffer_val);
        i16_push_builtin(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed.len(), 1);
                assert_eq!(borrowed[0], 42);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_push_multiple() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Push three values
        interp.push(Value::Int32(10));
        interp.push(buffer_val);
        i16_push_builtin(&mut interp).unwrap();
        // Buffer is now on stack, need to arrange: value buffer
        let buffer_from_stack = interp.pop().unwrap();
        interp.push(Value::Int32(20));
        interp.push(buffer_from_stack);
        i16_push_builtin(&mut interp).unwrap();
        let buffer_from_stack = interp.pop().unwrap();
        interp.push(Value::Int32(30));
        interp.push(buffer_from_stack);
        i16_push_builtin(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed.len(), 3);
                assert_eq!(&*borrowed, &[10i16, 20, 30]);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_push_out_of_range() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(40000)); // > i16::MAX
        interp.push(buffer_val);
        let result = i16_push_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    // i16-pop tests
    #[test]
    fn test_i16_pop_single() {
        let mut interp = setup_interpreter();

        let buffer = vec![42i16];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_pop_builtin(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(42)));

        let buf = interp.pop().unwrap();
        match buf {
            Value::I16Buffer(b) => {
                assert_eq!(b.borrow().len(), 0);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_pop_multiple() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i16, 20, 30];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_pop_builtin(&mut interp).unwrap();

        let val1 = interp.pop().unwrap();
        assert!(matches!(val1, Value::Int32(30))); // Last pushed

        let buf = interp.pop().unwrap();
        interp.push(buf);
        i16_pop_builtin(&mut interp).unwrap();

        let val2 = interp.pop().unwrap();
        assert!(matches!(val2, Value::Int32(20)));
    }

    #[test]
    fn test_i16_pop_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        let result = i16_pop_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_i16_pop_negative() {
        let mut interp = setup_interpreter();

        let buffer = vec![-100i16, -200];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_pop_builtin(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(-200)));
    }

    #[test]
    fn test_push_pop_round_trip() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Push, then pop
        interp.push(Value::Int32(12345));
        interp.push(buffer_val);
        i16_push_builtin(&mut interp).unwrap();
        i16_pop_builtin(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(12345)));
    }
}
