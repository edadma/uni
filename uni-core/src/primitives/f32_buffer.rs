// F32 buffer primitives for floating point data and DSP
// Complete set of operations for working with f32 buffers

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

// Create f32 buffer primitive
// Stack: ( size -- buffer )
// Creates a new f32 buffer with 'size' elements, all initialized to 0.0
pub fn f32_buffer_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let size = interp.pop_integer()?;

    // Create vector of zeros with the specified size
    let buffer = vec![0.0f32; size];

    // Wrap in Rc<RefCell<>> for shared mutable access
    let buffer_value = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

    interp.push(buffer_value);
    Ok(())
}

// f32-ref primitive
// Stack: ( buffer index -- value )
// Gets the f32 value at the specified index
pub fn f32_ref_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let index = interp.pop_integer()?;
    let buffer_val = interp.pop()?;

    match buffer_val {
        Value::F32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if index >= borrowed.len() {
                return Err(RuntimeError::DomainError(format!(
                    "Index {} out of bounds for buffer of length {}",
                    index,
                    borrowed.len()
                )));
            }

            let value = borrowed[index];
            interp.push(Value::Number(value as f64));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32@ expects an f32-buffer".to_string(),
        )),
    }
}

// f32-set primitive
// Stack: ( value buffer index -- buffer )
// Following Forth convention where value comes first
// Sets the f32 value at the specified index and returns the buffer
pub fn f32_set_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let index = interp.pop_integer()?;
    let buffer_val = interp.pop()?;
    let value_to_set = interp.pop()?;

    // Convert value to f32
    let f32_value = match value_to_set {
        Value::Int32(i) => i as f32,
        Value::Number(n) => n as f32,
        Value::Integer(i) => i.to_f32().ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Value {} out of f32 range",
                i
            ))
        })?,
        _ => {
            return Err(RuntimeError::TypeError(
                "f32! expects a number for value".to_string(),
            ))
        }
    };

    match buffer_val {
        Value::F32Buffer(buffer) => {
            let mut borrowed = buffer.borrow_mut();

            if index >= borrowed.len() {
                return Err(RuntimeError::DomainError(format!(
                    "Index {} out of bounds for buffer of length {}",
                    index,
                    borrowed.len()
                )));
            }

            borrowed[index] = f32_value;
            drop(borrowed); // Release borrow before pushing

            // Push the buffer back on the stack for chaining
            interp.push(Value::F32Buffer(buffer));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32! expects an f32-buffer".to_string(),
        )),
    }
}

// f32-length primitive
// Stack: ( buffer -- buffer length )
// Returns the length of the buffer (keeps buffer on stack)
pub fn f32_length_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::F32Buffer(buffer) => {
            let len = buffer.borrow().len();
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Int32(len as i32));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32-length expects an f32-buffer".to_string(),
        )),
    }
}

// f32-push primitive
// Stack: ( value buffer -- buffer )
// Following Forth convention where value comes first
// Appends a value to the end of the buffer
pub fn f32_push_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;
    let value_to_push = interp.pop()?;

    // Convert value to f32
    let f32_value = match value_to_push {
        Value::Int32(i) => i as f32,
        Value::Number(n) => n as f32,
        Value::Integer(i) => i.to_f32().ok_or_else(|| {
            RuntimeError::DomainError(format!(
                "Value {} out of f32 range",
                i
            ))
        })?,
        _ => {
            return Err(RuntimeError::TypeError(
                "f32-push! expects a number for value".to_string(),
            ))
        }
    };

    match buffer_val {
        Value::F32Buffer(buffer) => {
            buffer.borrow_mut().push(f32_value);
            interp.push(Value::F32Buffer(buffer));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32-push! expects an f32-buffer".to_string(),
        )),
    }
}

// f32-pop primitive
// Stack: ( buffer -- buffer value )
// Removes and returns the last value from the buffer
pub fn f32_pop_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match buffer_val {
        Value::F32Buffer(buffer) => {
            let popped = buffer.borrow_mut().pop().ok_or_else(|| {
                RuntimeError::DomainError("Cannot pop from empty buffer".to_string())
            })?;

            interp.push(Value::F32Buffer(buffer));
            interp.push(Value::Number(popped as f64));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32-pop! expects an f32-buffer".to_string(),
        )),
    }
}

// f32-max primitive
// Stack: ( buffer -- buffer max-value )
// Finds the maximum value in the buffer
pub fn f32_max_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::F32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if borrowed.is_empty() {
                return Err(RuntimeError::DomainError(
                    "Cannot find max of empty buffer".to_string(),
                ));
            }

            let max = borrowed.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            drop(borrowed); // Release borrow before pushing
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Number(max as f64));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32-max expects an f32-buffer".to_string(),
        )),
    }
}

// f32-min primitive
// Stack: ( buffer -- buffer min-value )
// Finds the minimum value in the buffer
pub fn f32_min_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::F32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if borrowed.is_empty() {
                return Err(RuntimeError::DomainError(
                    "Cannot find min of empty buffer".to_string(),
                ));
            }

            let min = borrowed.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            drop(borrowed); // Release borrow before pushing
            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Number(min as f64));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32-min expects an f32-buffer".to_string(),
        )),
    }
}

// f32-avg primitive
// Stack: ( buffer -- buffer average )
// Computes the average (mean) value of the buffer
pub fn f32_avg_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::F32Buffer(buffer) => {
            let borrowed = buffer.borrow();

            if borrowed.is_empty() {
                return Err(RuntimeError::DomainError(
                    "Cannot find average of empty buffer".to_string(),
                ));
            }

            // Use f64 for accumulation to maintain precision
            let sum: f64 = borrowed.iter().map(|&x| x as f64).sum();
            let avg = sum / borrowed.len() as f64;
            drop(borrowed); // Release borrow before pushing

            interp.push(buffer_val); // Push buffer back
            interp.push(Value::Number(avg));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "f32-avg expects an f32-buffer".to_string(),
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
    fn test_f32_buffer_create_empty() {
        let mut interp = setup_interpreter();

        interp.push(Value::Int32(0));
        let result = f32_buffer_impl(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::F32Buffer(buf) => {
                assert_eq!(buf.borrow().len(), 0);
            }
            _ => panic!("Expected F32Buffer"),
        }
    }

    #[test]
    fn test_f32_buffer_create_small() {
        let mut interp = setup_interpreter();

        interp.push(Value::Int32(10));
        let result = f32_buffer_impl(&mut interp);
        assert!(result.is_ok());

        let buffer = interp.pop().unwrap();
        match buffer {
            Value::F32Buffer(buf) => {
                let borrowed = buf.borrow();
                assert_eq!(borrowed.len(), 10);
                assert!(borrowed.iter().all(|&x| x == 0.0));
            }
            _ => panic!("Expected F32Buffer"),
        }
    }

    #[test]
    fn test_f32_ref_basic() {
        let mut interp = setup_interpreter();

        let buffer = vec![1.5f32, 2.5, 3.5, 4.5, 5.5];
        let buffer_val = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        interp.push(Value::Int32(2));
        f32_ref_impl(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        match value {
            Value::Number(n) => assert!((n - 3.5).abs() < 0.001),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_f32_set_basic() {
        let mut interp = setup_interpreter();

        let buffer = vec![0.0f32; 5];
        let buffer_val = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Number(3.14));
        interp.push(buffer_val);
        interp.push(Value::Int32(2));
        f32_set_impl(&mut interp).unwrap();

        let returned = interp.pop().unwrap();
        match returned {
            Value::F32Buffer(buf) => {
                let borrowed = buf.borrow();
                assert!((borrowed[2] - 3.14).abs() < 0.01);
                assert_eq!(borrowed[0], 0.0);
            }
            _ => panic!("Expected F32Buffer"),
        }
    }

    #[test]
    fn test_f32_push_and_pop() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(Value::Number(2.71));
        interp.push(buffer_val);
        f32_push_impl(&mut interp).unwrap();

        let buf = interp.pop().unwrap();
        interp.push(buf);
        f32_pop_impl(&mut interp).unwrap();

        let value = interp.pop().unwrap();
        match value {
            Value::Number(n) => assert!((n - 2.71).abs() < 0.01),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_f32_max() {
        let mut interp = setup_interpreter();

        let buffer = vec![1.5f32, 3.7, 2.1, 5.9, 4.2];
        let buffer_val = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        f32_max_impl(&mut interp).unwrap();

        let max = interp.pop().unwrap();
        match max {
            Value::Number(n) => assert!((n - 5.9).abs() < 0.01),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_f32_min() {
        let mut interp = setup_interpreter();

        let buffer = vec![5.5f32, 1.2, 3.3, 2.2, 4.4];
        let buffer_val = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        f32_min_impl(&mut interp).unwrap();

        let min = interp.pop().unwrap();
        match min {
            Value::Number(n) => assert!((n - 1.2).abs() < 0.01),
            _ => panic!("Expected Number"),
        }
    }

    #[test]
    fn test_f32_avg() {
        let mut interp = setup_interpreter();

        let buffer = vec![1.0f32, 2.0, 3.0, 4.0, 5.0];
        let buffer_val = Value::F32Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        f32_avg_impl(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        match avg {
            Value::Number(n) => assert!((n - 3.0).abs() < 0.01),
            _ => panic!("Expected Number"),
        }
    }
}
