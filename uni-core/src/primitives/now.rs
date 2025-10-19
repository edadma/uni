// RUST CONCEPT: Get current date/time from platform TimeSource
// Returns a date record with all components
use crate::compat::Rc;
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// RUST CONCEPT: Platform-specific imports for no_std
#[cfg(target_os = "none")]
use crate::compat::ToString;

#[cfg(target_os = "none")]
use alloc::vec;

// RUST CONCEPT: now primitive
// Stack: ( -- date_record )
// Gets current date/time from TimeSource and returns as a date record
pub fn now_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // Check if we have a time source
    let time_source = interp.time_source.as_ref().ok_or_else(|| {
        RuntimeError::TypeError(
            "now: no time source available - platform must inject TimeSource".to_string(),
        )
    })?;

    // Get date components from time source
    let components = time_source.now();

    // Look up the date record type in the dictionary
    // Record types are stored with the key "<record-type:typename>"
    let date_type_key = interp.intern_atom("<record-type:date>");
    let date_entry = interp.dictionary.get(&date_type_key).ok_or_else(|| {
        RuntimeError::TypeError(
            "now: date record type not found - prelude not loaded?".to_string(),
        )
    })?;

    // Verify it's a record type
    match &date_entry.value {
        Value::RecordType { .. } => {},
        _ => {
            return Err(RuntimeError::TypeError(
                "now: 'date' is not a record type".to_string(),
            ))
        }
    }

    // Create field values vector
    let field_values = vec![
        Value::Int32(components.year),
        Value::Int32(components.month as i32),
        Value::Int32(components.day as i32),
        Value::Int32(components.hour as i32),
        Value::Int32(components.minute as i32),
        Value::Int32(components.second as i32),
        Value::Int32(components.offset_minutes),
    ];

    // Create the record
    // Use just "date" as the type name, not the internal key
    let date_type_name = interp.intern_atom("date");
    let record = Value::Record {
        type_name: date_type_name,
        fields: Rc::new(RefCell::new(field_values)),
    };

    interp.push(record);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::execute_string;

    #[test]
    #[cfg(feature = "std")]
    fn test_now_with_time_source() {
        use crate::builtins::register_builtins;
        use crate::hardware::linux_time::LinuxTimeSource;

        let mut interp = Interpreter::new();
        register_builtins(&mut interp);

        // Inject time source
        interp.set_time_source(Box::new(LinuxTimeSource::new()));

        // Load prelude to get date record type
        // Use list builder to create field names list
        execute_string(r#"
            "year" "month" "day" "hour" "minute" "second" "offset" 7 list "date" make-record-type drop
        "#, &mut interp).unwrap();

        // Call now
        now_builtin(&mut interp).unwrap();

        // Should have a record on the stack
        assert_eq!(interp.stack.len(), 1);

        let value = interp.pop().unwrap();
        match value {
            Value::Record { type_name, fields } => {
                assert_eq!(&*type_name, "date");
                assert_eq!(fields.borrow().len(), 7);
            }
            _ => panic!("Expected record, got {:?}", value),
        }
    }

    #[test]
    fn test_now_without_time_source() {
        let mut interp = Interpreter::new();

        // No time source
        let result = now_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
