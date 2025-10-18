// RUST CONCEPT: I16 buffer indexed write primitive
// Set the value at a specific index in an i16 buffer
use crate::compat::{format, ToString};
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use num_traits::ToPrimitive;

// RUST CONCEPT: i16-set primitive
// Stack: ( value buffer index -- buffer )
// Following Forth convention where value comes first
// Sets the i16 value at the specified index and returns the buffer
pub fn i16_set_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let index = interp.pop_integer()?;
    let buffer_val = interp.pop()?;
    let value_to_set = interp.pop()?;

    // Convert value to i16
    let i16_value = match value_to_set {
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
                "i16-set expects a number for value".to_string(),
            ))
        }
    };

    match buffer_val {
        Value::I16Buffer(buffer) => {
            let mut borrowed = buffer.borrow_mut();

            if index >= borrowed.len() {
                return Err(RuntimeError::DomainError(format!(
                    "Index {} out of bounds for buffer of length {}",
                    index,
                    borrowed.len()
                )));
            }

            borrowed[index] = i16_value;
            drop(borrowed); // Release borrow before pushing

            // Push the buffer back on the stack for chaining
            interp.push(Value::I16Buffer(buffer));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i16-set expects an i16-buffer".to_string(),
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
    fn test_i16_set_basic() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 5];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Set index 2 to value 42
        interp.push(Value::Int32(42));
        interp.push(buffer_val.clone());
        interp.push(Value::Int32(2));
        i16_set_builtin(&mut interp).unwrap();

        // Check the buffer was returned
        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed[2], 42);
                assert_eq!(borrowed[0], 0); // Other values unchanged
                assert_eq!(borrowed[4], 0);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_set_negative_value() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(-1000));
        interp.push(buffer_val);
        interp.push(Value::Int32(1));
        i16_set_builtin(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                assert_eq!(buf.borrow()[1], -1000);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_set_max_value() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(32767)); // i16::MAX
        interp.push(buffer_val);
        interp.push(Value::Int32(0));
        i16_set_builtin(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                assert_eq!(buf.borrow()[0], 32767);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_set_min_value() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(-32768)); // i16::MIN
        interp.push(buffer_val);
        interp.push(Value::Int32(0));
        i16_set_builtin(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                assert_eq!(buf.borrow()[0], -32768);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_set_out_of_bounds() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(42));
        interp.push(buffer_val);
        interp.push(Value::Int32(5));
        let result = i16_set_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_i16_set_value_out_of_range() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Try to set value larger than i16::MAX
        interp.push(Value::Int32(40000));
        interp.push(buffer_val);
        interp.push(Value::Int32(0));
        let result = i16_set_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_i16_set_chaining() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 5];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Set multiple values by chaining
        interp.push(Value::Int32(10));
        interp.push(buffer_val);
        interp.push(Value::Int32(0));
        i16_set_builtin(&mut interp).unwrap();

        // Buffer is now on stack from first call
        // For second call we need: value buffer index
        // Stack is currently: buffer
        // So: push value, swap to get: value buffer, then push index
        let buffer_from_stack = interp.pop().unwrap();
        interp.push(Value::Int32(20));
        interp.push(buffer_from_stack);
        interp.push(Value::Int32(1));
        i16_set_builtin(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I16Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed[0], 10);
                assert_eq!(borrowed[1], 20);
            }
            _ => panic!("Expected I16Buffer"),
        }
    }

    #[test]
    fn test_i16_set_type_error_buffer() {
        let mut interp = setup_interpreter();

        interp.push(Value::Int32(100));
        interp.push(Value::Number(42.0));
        interp.push(Value::Int32(0));
        let result = i16_set_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_i16_set_type_error_value() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i16; 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::String("not a number".into()));
        interp.push(buffer_val);
        interp.push(Value::Int32(0));
        let result = i16_set_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
