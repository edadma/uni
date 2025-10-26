// Get current date/time from platform TimeSource
// Returns a date record with all components

use crate::compat::{Rc, ToString};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// ASYNC CONCEPT: now primitive (sync, wrapped in async)
// Stack: ( -- date_record )
// Gets current date/time from TimeSource and returns as a date record
pub fn now_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
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
    let date_entry = interp.dict_get(&date_type_key).ok_or_else(|| {
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
    #[cfg(not(target_os = "none"))]
    let field_values = vec![
        Value::Int32(components.year),
        Value::Int32(components.month as i32),
        Value::Int32(components.day as i32),
        Value::Int32(components.hour as i32),
        Value::Int32(components.minute as i32),
        Value::Int32(components.second as i32),
        Value::Int32(components.offset_minutes),
    ];

    #[cfg(target_os = "none")]
    let field_values = {
        use crate::compat::Vec;
        let mut v = Vec::new();
        v.push(Value::Int32(components.year));
        v.push(Value::Int32(components.month as i32));
        v.push(Value::Int32(components.day as i32));
        v.push(Value::Int32(components.hour as i32));
        v.push(Value::Int32(components.minute as i32));
        v.push(Value::Int32(components.second as i32));
        v.push(Value::Int32(components.offset_minutes));
        v
    };

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

    #[test]
    #[cfg(feature = "std")]
    fn test_now_with_time_source() {
        use crate::hardware::linux::LinuxTimeSource;
        use crate::primitives::record::make_record_type_impl;

        let mut interp = AsyncInterpreter::new();

        // Inject time source
        interp.set_time_source(Box::new(LinuxTimeSource::new()));

        // Create the date record type first
        // Build field names list: (year month day hour minute second offset)
        let field_names = vec!["year", "month", "day", "hour", "minute", "second", "offset"];
        let mut field_list = Value::Nil;
        for name in field_names.iter().rev() {
            let name_atom = interp.intern_atom(name);
            field_list = Value::Pair(
                Rc::new(Value::Atom(name_atom)),
                Rc::new(field_list)
            );
        }

        // Push arguments for make-record-type: field_list type_name
        interp.push(field_list);
        let date_atom = interp.intern_atom("date");
        interp.push(Value::String(date_atom));
        make_record_type_impl(&mut interp).unwrap();

        // Pop the record type that was returned
        interp.pop().unwrap();

        // Now call now
        now_impl(&mut interp).unwrap();

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
        let mut interp = AsyncInterpreter::new();

        // No time source
        let result = now_impl(&mut interp);
        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }
}
