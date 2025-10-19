// RUST CONCEPT: Platform-agnostic timezone offset primitive
// Gets current timezone offset from injected TimeSource
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

// RUST CONCEPT: current-offset primitive
// Calls the platform's TimeSource to get timezone offset in minutes from UTC
// Stack effect: ( -- offset )
// Returns Int32 (timezone offsets are small: -720 to +840 minutes)
pub fn current_offset_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Check if a time source is available
    if let Some(time_source) = &interp.time_source {
        let offset_minutes = time_source.now_offset_minutes();
        interp.push(Value::Int32(offset_minutes));
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
    fn test_current_offset_no_source() {
        let mut interp = setup_interpreter();

        // Without a time source, should return error
        let result = current_offset_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_current_offset_with_source() {
        use crate::time_source::TimeSource;

        struct TestTimeSource;
        impl TimeSource for TestTimeSource {
            fn now_timestamp_millis(&self) -> i64 {
                0
            }
            fn now_offset_minutes(&self) -> i32 {
                -300 // EST: UTC-5 hours
            }
        }

        let mut interp = setup_interpreter();
        interp.set_time_source(Box::new(TestTimeSource));

        // Should succeed and push offset
        let result = current_offset_builtin(&mut interp);
        assert!(result.is_ok());

        let offset = interp.pop().unwrap();
        assert!(matches!(offset, Value::Int32(-300)));
    }

    #[test]
    fn test_current_offset_utc() {
        use crate::time_source::TimeSource;

        struct UtcTimeSource;
        impl TimeSource for UtcTimeSource {
            fn now_timestamp_millis(&self) -> i64 {
                0
            }
            fn now_offset_minutes(&self) -> i32 {
                0 // UTC
            }
        }

        let mut interp = setup_interpreter();
        interp.set_time_source(Box::new(UtcTimeSource));

        let result = current_offset_builtin(&mut interp);
        assert!(result.is_ok());

        let offset = interp.pop().unwrap();
        assert!(matches!(offset, Value::Int32(0)));
    }
}
