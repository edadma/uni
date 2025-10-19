// RUST CONCEPT: Platform-agnostic time primitive
// Gets current timestamp from injected TimeSource
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};
use num_bigint::BigInt;

// RUST CONCEPT: current-timestamp primitive
// Calls the platform's TimeSource to get current time in milliseconds since Unix epoch
// Stack effect: ( -- timestamp )
// Returns BigInt for arbitrary precision (timestamps can be large)
pub fn current_timestamp_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Check if a time source is available
    if let Some(time_source) = &interp.time_source {
        let timestamp_millis = time_source.now_timestamp_millis();
        interp.push(Value::Integer(BigInt::from(timestamp_millis)));
        Ok(())
    } else {
        // No time source available - return error
        Err(RuntimeError::TypeError(
            "No time source available - platform must inject TimeSource".into(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_current_timestamp_no_source() {
        let mut interp = setup_interpreter();

        // Without a time source, should return error
        let result = current_timestamp_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_current_timestamp_with_source() {
        use crate::time_source::TimeSource;

        struct TestTimeSource;
        impl TimeSource for TestTimeSource {
            fn now_timestamp_millis(&self) -> i64 {
                1234567890123 // Fixed timestamp for testing
            }
            fn now_offset_minutes(&self) -> i32 {
                0
            }
        }

        let mut interp = setup_interpreter();
        interp.set_time_source(Box::new(TestTimeSource));

        // Should succeed and push timestamp
        let result = current_timestamp_builtin(&mut interp);
        assert!(result.is_ok());

        let timestamp = interp.pop().unwrap();
        assert!(matches!(timestamp, Value::Integer(ref i) if i == &BigInt::from(1234567890123i64)));
    }
}
