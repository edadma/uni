// I32 buffer primitives for integer data and DSP
// Complete set of operations for working with i32 buffers

use crate::compat::{format, Rc, ToString};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};
use num_traits::ToPrimitive;

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

#[cfg(target_os = "none")]
use alloc::vec;

// Create i32 buffer primitive
// Stack: ( size -- buffer )
// Creates a new i32 buffer with 'size' elements, all initialized to 0
pub fn i32_buffer_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let size = interp.pop_integer()?;

    // Create vector of zeros with the specified size
    let buffer = vec![0i32; size];

    // Wrap in Rc<RefCell<>> for shared mutable access
    let buffer_value = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

    interp.push(buffer_value);
    Ok(())
}

// i32-ref primitive
// Stack: ( buffer index -- value )
// Gets the i32 value at the specified index
pub fn i32_ref_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let index = interp.pop_integer()?;
    let buffer_val = interp.pop()?;

    match buffer_val {
        Value::I32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if index >= borrowed.len() {
                return Err(RuntimeError::DomainError(format!(
                    "Index {} out of bounds for buffer of length {}",
                    index,
                    borrowed.len()
                )));
            }

            let value = borrowed[index];
            interp.push(Value::Int32(value));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32@ expects an i32-buffer".to_string(),
        )),
    }
}

// i32-set primitive
// Stack: ( value buffer index -- buffer )
// Following Forth convention where value comes first
// Sets the i32 value at the specified index and returns the buffer
pub fn i32_set_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let index = interp.pop_integer()?;
    let buffer_val = interp.pop()?;
    let value_to_set = interp.pop()?;

    // Convert value to i32
    let i32_value = match value_to_set {
        Value::Int32(i) => i,
        Value::Number(n) => n as i32,
        Value::Integer(i) => i.to_i32().ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Value {} out of i32 range",
                i
            ))
        })?,
        _ => {
            return Err(RuntimeError::TypeError(
                "i32! expects a number for value".to_string(),
            ))
        }
    };

    match buffer_val {
        Value::I32Buffer(buffer) => {
            let mut borrowed = buffer.borrow_mut();

            if index >= borrowed.len() {
                return Err(RuntimeError::DomainError(format!(
                    "Index {} out of bounds for buffer of length {}",
                    index,
                    borrowed.len()
                )));
            }

            borrowed[index] = i32_value;
            drop(borrowed); // Release borrow before pushing

            // Push the buffer back on the stack for chaining
            interp.push(Value::I32Buffer(buffer));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32! expects an i32-buffer".to_string(),
        )),
    }
}

// i32-length primitive
// Stack: ( buffer -- buffer length )
// Returns the length of the buffer (keeps buffer on stack)
pub fn i32_length_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I32Buffer(buffer) => {
            let len = buffer.borrow().len();
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Int32(len as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32-length expects an i16-buffer".to_string(),
        )),
    }
}

// i32-push primitive
// Stack: ( value buffer -- buffer )
// Following Forth convention where value comes first
// Appends a value to the end of the buffer
pub fn i32_push_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;
    let value_to_push = interp.pop()?;

    // Convert value to i16
    let i32_value = match value_to_push {
        Value::Int32(i) => {
            if i < i32::MIN as i32 || i > i32::MAX as i32 {
                return Err(RuntimeError::DomainError(format!(
                    "Value {} out of i16 range ({} to {})",
                    i,
                    i32::MIN,
                    i32::MAX
                )));
            }
            i
        }
        Value::Number(n) => {
            if n < i32::MIN as f64 || n > i32::MAX as f64 {
                return Err(RuntimeError::DomainError(format!(
                    "Value {} out of i16 range ({} to {})",
                    n,
                    i32::MIN,
                    i32::MAX
                )));
            }
            n as i32
        }
        Value::Integer(i) => i.to_i32().ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Value {} out of i16 range ({} to {})",
                i,
                i32::MIN,
                i32::MAX
            ))
        })?,
        _ => {
            return Err(RuntimeError::TypeError(
                "i32-push! expects a number for value".to_string(),
            ))
        }
    };

    match buffer_val {
        Value::I32Buffer(buffer) => {
            buffer.borrow_mut().push(i32_value);
            interp.push(Value::I32Buffer(buffer));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32-push! expects an i32-buffer".to_string(),
        )),
    }
}

// i32-pop primitive
// Stack: ( buffer -- buffer value )
// Removes and returns the last value from the buffer
pub fn i32_pop_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match buffer_val {
        Value::I32Buffer(buffer) => {
            let popped = buffer.borrow_mut().pop().ok_or_else(|| {
                RuntimeError::DomainError("Cannot pop from empty buffer".to_string())
            })?;

            interp.push(Value::I32Buffer(buffer));
            interp.push(Value::Int32(popped as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32-pop! expects an i16-buffer".to_string(),
        )),
    }
}

// i32-max primitive
// Stack: ( buffer -- buffer max-value )
// Finds the maximum value in the buffer
pub fn i32_max_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if borrowed.is_empty() {
                return Err(RuntimeError::DomainError(
                    "Cannot find max of empty buffer".to_string(),
                ));
            }

            let max = *borrowed.iter().max().unwrap();
            drop(borrowed); // Release borrow before pushing
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Int32(max as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32-max expects an i16-buffer".to_string(),
        )),
    }
}

// i32-min primitive
// Stack: ( buffer -- buffer min-value )
// Finds the minimum value in the buffer
pub fn i32_min_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if borrowed.is_empty() {
                return Err(RuntimeError::DomainError(
                    "Cannot find min of empty buffer".to_string(),
                ));
            }

            let min = *borrowed.iter().min().unwrap();
            drop(borrowed); // Release borrow before pushing
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Int32(min as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32-min expects an i16-buffer".to_string(),
        )),
    }
}

// i32-avg primitive
// Stack: ( buffer -- buffer average )
// Computes the average (mean) value of the buffer
pub fn i32_avg_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if borrowed.is_empty() {
                return Err(RuntimeError::DomainError(
                    "Cannot find average of empty buffer".to_string(),
                ));
            }

            // Use i64 to avoid overflow when summing i16 values
            let sum: i64 = borrowed.iter().map(|&x| x as i64).sum();
            let avg = sum / borrowed.len() as i64;
            drop(borrowed); // Release borrow before pushing

            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Int32(avg as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "i32-avg expects an i16-buffer".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::AsyncInterpreter;

    fn setup_interpreter() -> AsyncInterpreter {
        AsyncInterpreter::new()
    }

    #[test]
    fn test_i32_buffer_create_empty() {
        let mut interp = setup_interpreter();

        // Create a buffer of size 0
        interp.push(Value::Int32(0));
        let result = i32_buffer_impl(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::I32Buffer(buf) => {
                assert_eq!(buf.borrow().len(), 0);
            }
            _ => panic!("Expected I32Buffer"),
        }
    }

    #[test]
    fn test_i32_buffer_create_small() {
        let mut interp = setup_interpreter();

        // Create a buffer of size 10
        interp.push(Value::Int32(10));
        let result = i32_buffer_impl(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::I32Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed.len(), 10);
                // All values should be initialized to 0
                assert!(borrowed.iter().all(|&x| x == 0));
            }
            _ => panic!("Expected I32Buffer"),
        }
    }

    #[test]
    fn test_i32_buffer_create_large() {
        let mut interp = setup_interpreter();

        // Create a buffer of size 1000
        interp.push(Value::Int32(1000));
        let result = i32_buffer_impl(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::I32Buffer(buf) => {
                assert_eq!(buf.borrow().len(), 1000);
            }
            _ => panic!("Expected I32Buffer"),
        }
    }

    #[test]
    fn test_i32_ref_basic() {
        let mut interp = setup_interpreter();

        // Create a buffer with some values
        let buffer = vec![10i32, 20, 30, 40, 50];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        // Get value at index 2 (should be 30)
        interp.push(buffer_val);
        interp.push(Value::Int32(2));
        let result = i32_ref_impl(&mut interp);
        assert!(result.is_ok());

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(30)));
    }

    #[test]
    fn test_i32_set_basic() {
        let mut interp = setup_interpreter();

        let buffer = vec![0i32; 5];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        // Set index 2 to value 42
        interp.push(Value::Int32(42));
        interp.push(buffer_val.clone());
        interp.push(Value::Int32(2));
        i32_set_impl(&mut interp).unwrap();

        // Check the buffer was returned
        let returned = interp.pop().unwrap();
        match returned {
            Value::I32Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed[2], 42);
                assert_eq!(borrowed[0], 0); // Other values unchanged
                assert_eq!(borrowed[4], 0);
            }
            _ => panic!("Expected I32Buffer"),
        }
    }

    #[test]
    fn test_i32_length() {
        let mut interp = setup_interpreter();

        let buffer = vec![1i32, 2, 3, 4, 5];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i32_length_impl(&mut interp).unwrap();

        let len = interp.pop().unwrap();
        assert!(matches!(len, Value::Int32(5)));

        // Buffer should still be on stack
        let buf = interp.pop().unwrap();
        assert!(matches!(buf, Value::I32Buffer(_)));
    }

    #[test]
    fn test_i32_push_to_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Int32(42));
        interp.push(buffer_val);
        i32_push_impl(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::I32Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed.len(), 1);
                assert_eq!(borrowed[0], 42);
            }
            _ => panic!("Expected I32Buffer"),
        }
    }

    #[test]
    fn test_i32_pop_single() {
        let mut interp = setup_interpreter();

        let buffer = vec![42i32];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i32_pop_impl(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        assert!(matches!(value, Value::Int32(42)));

        let buf = interp.pop().unwrap();
        match buf {
            Value::I32Buffer(b) => {
                assert_eq!(b.borrow().len(), 0);
            }
            _ => panic!("Expected I32Buffer"),
        }
    }

    #[test]
    fn test_i32_max() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i32, 30, 20, 50, 40];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i32_max_impl(&mut interp).unwrap();

        let max = interp.pop().unwrap();
        assert!(matches!(max, Value::Int32(50)));
    }

    #[test]
    fn test_i32_min() {
        let mut interp = setup_interpreter();

        let buffer = vec![50i32, 10, 30, 20, 40];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i32_min_impl(&mut interp).unwrap();

        let min = interp.pop().unwrap();
        assert!(matches!(min, Value::Int32(10)));
    }

    #[test]
    fn test_i32_avg() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i32, 20, 30, 40, 50];
        let buffer_val = Value::I32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i32_avg_impl(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        assert!(matches!(avg, Value::Int32(30))); // (10+20+30+40+50)/5 = 30
    }
}
