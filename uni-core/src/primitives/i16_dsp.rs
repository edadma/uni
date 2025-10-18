// RUST CONCEPT: I16 buffer DSP utilities
// Statistical and analysis operations for audio/signal processing
use crate::compat::ToString;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: i16-max primitive
// Stack: ( buffer -- buffer max-value )
// Finds the maximum value in the buffer
pub fn i16_max_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I16Buffer(buffer) => {
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
            "i16-max expects an i16-buffer".to_string(),
        )),
    }
}

// RUST CONCEPT: i16-min primitive
// Stack: ( buffer -- buffer min-value )
// Finds the minimum value in the buffer
pub fn i16_min_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I16Buffer(buffer) => {
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
            "i16-min expects an i16-buffer".to_string(),
        )),
    }
}

// RUST CONCEPT: i16-avg primitive
// Stack: ( buffer -- buffer average )
// Computes the average (mean) value of the buffer
pub fn i16_avg_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let buffer_val = interp.pop()?;

    match &buffer_val {
        Value::I16Buffer(buffer) => {
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
            "i16-avg expects an i16-buffer".to_string(),
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

    // i16-max tests
    #[test]
    fn test_i16_max_positive() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i16, 30, 20, 50, 40];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_max_builtin(&mut interp).unwrap();

        let max = interp.pop().unwrap();
        assert!(matches!(max, Value::Int32(50)));
    }

    #[test]
    fn test_i16_max_negative() {
        let mut interp = setup_interpreter();

        let buffer = vec![-100i16, -50, -200, -10];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_max_builtin(&mut interp).unwrap();

        let max = interp.pop().unwrap();
        assert!(matches!(max, Value::Int32(-10)));
    }

    #[test]
    fn test_i16_max_mixed() {
        let mut interp = setup_interpreter();

        let buffer = vec![-100i16, 200, -50, 100];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_max_builtin(&mut interp).unwrap();

        let max = interp.pop().unwrap();
        assert!(matches!(max, Value::Int32(200)));
    }

    #[test]
    fn test_i16_max_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        let result = i16_max_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    // i16-min tests
    #[test]
    fn test_i16_min_positive() {
        let mut interp = setup_interpreter();

        let buffer = vec![50i16, 10, 30, 20, 40];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_min_builtin(&mut interp).unwrap();

        let min = interp.pop().unwrap();
        assert!(matches!(min, Value::Int32(10)));
    }

    #[test]
    fn test_i16_min_negative() {
        let mut interp = setup_interpreter();

        let buffer = vec![-100i16, -50, -200, -10];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_min_builtin(&mut interp).unwrap();

        let min = interp.pop().unwrap();
        assert!(matches!(min, Value::Int32(-200)));
    }

    #[test]
    fn test_i16_min_mixed() {
        let mut interp = setup_interpreter();

        let buffer = vec![100i16, -200, 50, -100];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_min_builtin(&mut interp).unwrap();

        let min = interp.pop().unwrap();
        assert!(matches!(min, Value::Int32(-200)));
    }

    #[test]
    fn test_i16_min_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        let result = i16_min_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    // i16-avg tests
    #[test]
    fn test_i16_avg_positive() {
        let mut interp = setup_interpreter();

        let buffer = vec![10i16, 20, 30, 40, 50];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_avg_builtin(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        assert!(matches!(avg, Value::Int32(30))); // (10+20+30+40+50)/5 = 30
    }

    #[test]
    fn test_i16_avg_negative() {
        let mut interp = setup_interpreter();

        let buffer = vec![-10i16, -20, -30];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_avg_builtin(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        assert!(matches!(avg, Value::Int32(-20))); // (-10-20-30)/3 = -20
    }

    #[test]
    fn test_i16_avg_mixed() {
        let mut interp = setup_interpreter();

        let buffer = vec![-100i16, 100, -50, 50];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_avg_builtin(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        assert!(matches!(avg, Value::Int32(0))); // Sum is 0
    }

    #[test]
    fn test_i16_avg_single() {
        let mut interp = setup_interpreter();

        let buffer = vec![42i16];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_avg_builtin(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        assert!(matches!(avg, Value::Int32(42)));
    }

    #[test]
    fn test_i16_avg_empty() {
        let mut interp = setup_interpreter();

        let buffer = vec![];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        let result = i16_avg_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::DomainError(_))));
    }

    #[test]
    fn test_i16_avg_large_values() {
        let mut interp = setup_interpreter();

        // Test that we don't overflow with large values
        let buffer = vec![10000i16, 10000, 10000, 10000];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        interp.push(buffer_val);
        i16_avg_builtin(&mut interp).unwrap();

        let avg = interp.pop().unwrap();
        assert!(matches!(avg, Value::Int32(10000)));
    }

    #[test]
    fn test_buffer_returned_after_operations() {
        let mut interp = setup_interpreter();

        let buffer = vec![1i16, 2, 3];
        let buffer_val = Value::I16Buffer(Rc::new(RefCell::new(buffer)));

        // Test that buffer is returned for all operations
        interp.push(buffer_val.clone());
        i16_max_builtin(&mut interp).unwrap();
        let _ = interp.pop(); // Pop max value
        let buf = interp.pop().unwrap();
        assert!(matches!(buf, Value::I16Buffer(_)));

        interp.push(buffer_val.clone());
        i16_min_builtin(&mut interp).unwrap();
        let _ = interp.pop(); // Pop min value
        let buf = interp.pop().unwrap();
        assert!(matches!(buf, Value::I16Buffer(_)));

        interp.push(buffer_val);
        i16_avg_builtin(&mut interp).unwrap();
        let _ = interp.pop(); // Pop avg value
        let buf = interp.pop().unwrap();
        assert!(matches!(buf, Value::I16Buffer(_)));
    }
}
