// RUST CONCEPT: Record operations for Scheme-style records
// This module implements record types similar to R7RS Scheme's define-record-type
// Records are named product types with labeled fields

use crate::interpreter::{DictEntry, Interpreter};
use crate::value::{RuntimeError, Value};
use std::cell::RefCell;
use std::rc::Rc;

// RUST CONCEPT: make-record-type builtin
// Creates a record type and defines constructor, predicate, accessors, and mutators
// Stack: field_names_list type_name -- record_type
// Side effect: Defines make-<type>, <type>?, <type>-<field>, <type>-<field>! for each field
pub fn make_record_type_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Pop arguments in reverse order (stack is LIFO)
    let type_name_val = interp.pop()?;
    let field_names_list = interp.pop()?;

    // RUST CONCEPT: Extract type name string
    let type_name = match type_name_val {
        Value::String(s) => s,
        Value::Atom(a) => a,
        _ => {
            return Err(RuntimeError::TypeError(
                "make-record-type: type name must be string or atom".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Extract field names from list
    let mut field_names: Vec<Rc<str>> = Vec::new();
    let mut current = &field_names_list;

    loop {
        match current {
            Value::Nil => break,
            Value::Pair(head, tail) => {
                match head.as_ref() {
                    Value::String(s) => field_names.push(s.clone()),
                    Value::Atom(a) => field_names.push(a.clone()),
                    _ => {
                        return Err(RuntimeError::TypeError(
                            "make-record-type: field names must be strings or atoms".to_string(),
                        ))
                    }
                }
                current = tail.as_ref();
            }
            _ => {
                return Err(RuntimeError::TypeError(
                    "make-record-type: field names must be a list".to_string(),
                ))
            }
        }
    }

    // RUST CONCEPT: Create the record type descriptor
    let record_type = Value::RecordType {
        type_name: type_name.clone(),
        field_names: Rc::new(field_names.clone()),
    };

    // RUST CONCEPT: Store record type in dictionary for later use
    let record_type_atom = interp.intern_atom(&format!("<record-type:{}>", type_name));
    interp.dictionary.insert(
        record_type_atom.clone(),
        DictEntry {
            value: record_type.clone(),
            is_executable: false,
            doc: None,
        },
    );

    // RUST CONCEPT: Generate constructor (make-<type>)
    // The constructor takes field values from stack and creates a record instance
    let constructor_name = format!("make-{}", type_name);
    let constructor_atom = interp.intern_atom(&constructor_name);

    // RUST CONCEPT: Clone data for closure capture
    let constructor_type_name = type_name.clone();
    let constructor_field_count = field_names.len();

    // RUST CONCEPT: Create a list that will be executed to construct the record
    // This creates a quotation that captures the type name and field count
    let constructor_code = format!(
        "[{} \"{}\" construct-record]",
        constructor_field_count, constructor_type_name
    );

    // Parse and store as executable definition
    use crate::parser::parse;
    let parsed_values = parse(&constructor_code, interp)
        .map_err(|e| RuntimeError::TypeError(format!("Failed to parse constructor: {:?}", e)))?;

    if let Some(parsed) = parsed_values.into_iter().next() {
        interp.dictionary.insert(
            constructor_atom.clone(),
            DictEntry {
                value: parsed,
                is_executable: true,
                doc: Some(Rc::<str>::from(format!(
                    "Constructor for {} record type. Takes {} field values from stack.",
                    type_name, constructor_field_count
                ))),
            },
        );
    }

    // RUST CONCEPT: Generate type predicate (<type>?)
    // The predicate checks if a value is a record of this type
    let predicate_name = format!("{}?", type_name);
    let predicate_atom = interp.intern_atom(&predicate_name);

    // Store the predicate as a quotation that checks the record type
    let predicate_code = format!("[\"{}\" is-record-type?]", type_name);
    let parsed_values = parse(&predicate_code, interp)
        .map_err(|e| RuntimeError::TypeError(format!("Failed to parse predicate: {:?}", e)))?;

    if let Some(parsed) = parsed_values.into_iter().next() {
        interp.dictionary.insert(
            predicate_atom.clone(),
            DictEntry {
                value: parsed,
                is_executable: true,
                doc: Some(Rc::<str>::from(format!(
                    "Type predicate for {} record type.",
                    type_name
                ))),
            },
        );
    }

    // RUST CONCEPT: Generate field accessors (<type>-<field>)
    // Each accessor extracts a specific field from a record
    for (field_index, field_name) in field_names.iter().enumerate() {
        let accessor_name = format!("{}-{}", type_name, field_name);
        let accessor_atom = interp.intern_atom(&accessor_name);

        let accessor_code = format!("[\"{}\" {} get-record-field]", type_name, field_index);
        let parsed_values = parse(&accessor_code, interp)
            .map_err(|e| RuntimeError::TypeError(format!("Failed to parse accessor: {:?}", e)))?;

        if let Some(parsed) = parsed_values.into_iter().next() {
            interp.dictionary.insert(
                accessor_atom.clone(),
                DictEntry {
                    value: parsed,
                    is_executable: true,
                    doc: Some(Rc::<str>::from(format!(
                        "Get {} field from {} record.",
                        field_name, type_name
                    ))),
                },
            );
        }

        // RUST CONCEPT: Generate field mutators (<type>-<field>!)
        // Each mutator sets a specific field in a record
        let mutator_name = format!("{}-{}!", type_name, field_name);
        let mutator_atom = interp.intern_atom(&mutator_name);

        let mutator_code = format!("[\"{}\" {} set-record-field!]", type_name, field_index);
        let parsed_values = parse(&mutator_code, interp)
            .map_err(|e| RuntimeError::TypeError(format!("Failed to parse mutator: {:?}", e)))?;

        if let Some(parsed) = parsed_values.into_iter().next() {
            interp.dictionary.insert(
                mutator_atom.clone(),
                DictEntry {
                    value: parsed,
                    is_executable: true,
                    doc: Some(Rc::<str>::from(format!(
                        "Set {} field in {} record.",
                        field_name, type_name
                    ))),
                },
            );
        }
    }

    // RUST CONCEPT: Push the record type descriptor to stack
    interp.push(record_type);

    Ok(())
}

// RUST CONCEPT: Helper builtin to construct record instances
// Stack: field_values... field_count type_name -- record
pub fn construct_record_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let type_name_val = interp.pop()?;
    let field_count_val = interp.pop()?;

    // Extract type name
    let type_name = match type_name_val {
        Value::String(s) => s,
        Value::Atom(a) => a,
        _ => {
            return Err(RuntimeError::TypeError(
                "construct-record: type name must be string or atom".to_string(),
            ))
        }
    };

    // Extract field count
    let field_count = match field_count_val {
        Value::Int32(i) if i >= 0 => i as usize,
        Value::Integer(i) => {
            use num_traits::ToPrimitive;
            i.to_usize().ok_or_else(|| {
                RuntimeError::TypeError("construct-record: field count too large".to_string())
            })?
        }
        Value::Number(n) if n.fract() == 0.0 && n >= 0.0 => n as usize,
        _ => {
            return Err(RuntimeError::TypeError(
                "construct-record: field count must be non-negative integer".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Pop field values from stack in reverse order
    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        fields.push(interp.pop()?);
    }
    fields.reverse(); // Reverse to get correct field order

    // RUST CONCEPT: Create record instance
    let record = Value::Record {
        type_name,
        fields: Rc::new(RefCell::new(fields)),
    };

    interp.push(record);
    Ok(())
}

// RUST CONCEPT: Check if value is a record of specific type
// Stack: value type_name -- boolean
pub fn is_record_type_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let type_name_val = interp.pop()?;
    let value = interp.pop()?;

    // Extract type name
    let expected_type_name = match type_name_val {
        Value::String(s) => s,
        Value::Atom(a) => a,
        _ => {
            return Err(RuntimeError::TypeError(
                "is-record-type?: type name must be string or atom".to_string(),
            ))
        }
    };

    // Check if value is a record of the specified type
    let result = match value {
        Value::Record { type_name, .. } => type_name == expected_type_name,
        _ => false,
    };

    interp.push(Value::Boolean(result));
    Ok(())
}

// RUST CONCEPT: Get field from record
// Stack: record type_name field_index -- value
pub fn get_record_field_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let field_index_val = interp.pop()?;
    let type_name_val = interp.pop()?;
    let record = interp.pop()?;

    // Extract type name
    let expected_type_name = match type_name_val {
        Value::String(s) => s,
        Value::Atom(a) => a,
        _ => {
            return Err(RuntimeError::TypeError(
                "get-record-field: type name must be string or atom".to_string(),
            ))
        }
    };

    // Extract field index
    let field_index = match field_index_val {
        Value::Int32(i) if i >= 0 => i as usize,
        Value::Integer(i) => {
            use num_traits::ToPrimitive;
            i.to_usize().ok_or_else(|| {
                RuntimeError::TypeError("get-record-field: field index too large".to_string())
            })?
        }
        Value::Number(n) if n.fract() == 0.0 && n >= 0.0 => n as usize,
        _ => {
            return Err(RuntimeError::TypeError(
                "get-record-field: field index must be non-negative integer".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Pattern matching to extract record fields
    match record {
        Value::Record { type_name, fields } => {
            // Verify record type
            if type_name != expected_type_name {
                return Err(RuntimeError::TypeError(format!(
                    "get-record-field: expected {} record, got {}",
                    expected_type_name, type_name
                )));
            }

            // RUST CONCEPT: Borrow the RefCell to access the vector
            let fields_ref = fields.borrow();

            // Get field value
            let field_value = fields_ref.get(field_index).ok_or_else(|| {
                RuntimeError::TypeError(format!(
                    "get-record-field: field index {} out of bounds for {} record",
                    field_index, type_name
                ))
            })?;

            interp.push(field_value.clone());
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "get-record-field: expected record".to_string(),
        )),
    }
}

// RUST CONCEPT: Set field in record
// Stack: new_value record type_name field_index -- record
pub fn set_record_field_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let field_index_val = interp.pop()?;
    let type_name_val = interp.pop()?;
    let record = interp.pop()?;
    let new_value = interp.pop()?;

    // Extract type name
    let expected_type_name = match type_name_val {
        Value::String(s) => s,
        Value::Atom(a) => a,
        _ => {
            return Err(RuntimeError::TypeError(
                "set-record-field!: type name must be string or atom".to_string(),
            ))
        }
    };

    // Extract field index
    let field_index = match field_index_val {
        Value::Int32(i) if i >= 0 => i as usize,
        Value::Integer(i) => {
            use num_traits::ToPrimitive;
            i.to_usize().ok_or_else(|| {
                RuntimeError::TypeError("set-record-field!: field index too large".to_string())
            })?
        }
        Value::Number(n) if n.fract() == 0.0 && n >= 0.0 => n as usize,
        _ => {
            return Err(RuntimeError::TypeError(
                "set-record-field!: field index must be non-negative integer".to_string(),
            ))
        }
    };

    // RUST CONCEPT: Pattern matching to extract and modify record fields
    match record {
        Value::Record { type_name, fields } => {
            // Verify record type
            if type_name != expected_type_name {
                return Err(RuntimeError::TypeError(format!(
                    "set-record-field!: expected {} record, got {}",
                    expected_type_name, type_name
                )));
            }

            // RUST CONCEPT: Borrow the RefCell mutably to modify the vector
            let mut fields_ref = fields.borrow_mut();

            // Set field value
            if field_index >= fields_ref.len() {
                return Err(RuntimeError::TypeError(format!(
                    "set-record-field!: field index {} out of bounds for {} record",
                    field_index, type_name
                )));
            }

            fields_ref[field_index] = new_value;

            // RUST CONCEPT: Drop the mutable borrow before pushing
            drop(fields_ref);

            // Push the record back (for chaining)
            interp.push(Value::Record { type_name, fields });
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "set-record-field!: expected record".to_string(),
        )),
    }
}

// RUST CONCEPT: Get record type name
// Stack: record -- type_name
pub fn record_type_of_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let record = interp.pop()?;

    match record {
        Value::Record { type_name, .. } => {
            interp.push(Value::String(type_name));
            Ok(())
        }
        _ => Err(RuntimeError::TypeError(
            "record-type-of: expected record".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluator::execute_string;

    #[test]
    fn test_make_record_type() {
        let mut interp = Interpreter::new();

        // Create a person record type with name and age fields
        execute_string(r#"["name" "age"] "person" make-record-type"#, &mut interp).unwrap();

        // Should have record type on stack
        let record_type = interp.pop().unwrap();
        assert!(matches!(record_type, Value::RecordType { .. }));

        if let Value::RecordType {
            type_name,
            field_names,
        } = record_type
        {
            assert_eq!(&*type_name, "person");
            assert_eq!(field_names.len(), 2);
            assert_eq!(&*field_names[0], "name");
            assert_eq!(&*field_names[1], "age");
        }
    }

    #[test]
    fn test_construct_record() {
        let mut interp = Interpreter::new();

        // Define record type
        execute_string(r#"["name" "age"] "person" make-record-type drop"#, &mut interp).unwrap();

        // Construct a record instance
        execute_string(r#""Alice" 30 make-person"#, &mut interp).unwrap();

        // Should have record on stack
        let record = interp.pop().unwrap();
        assert!(matches!(record, Value::Record { .. }));

        if let Value::Record { type_name, fields } = record {
            assert_eq!(&*type_name, "person");
            let fields_ref = fields.borrow();
            assert_eq!(fields_ref.len(), 2);
        }
    }

    #[test]
    fn test_record_predicate() {
        let mut interp = Interpreter::new();

        // Define record type
        execute_string(r#"["name" "age"] "person" make-record-type drop"#, &mut interp).unwrap();

        // Create instance and test predicate
        execute_string(r#""Bob" 25 make-person person?"#, &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(true)));

        // Test predicate on non-record
        execute_string(r#"42 person?"#, &mut interp).unwrap();

        let result = interp.pop().unwrap();
        assert!(matches!(result, Value::Boolean(false)));
    }

    #[test]
    fn test_record_accessors() {
        let mut interp = Interpreter::new();

        // Define record type and create instance
        execute_string(
            r#"["name" "age"] "person" make-record-type drop
               "Charlie" 35 make-person"#,
            &mut interp,
        )
        .unwrap();

        // Access name field
        execute_string(r#"dup person-name"#, &mut interp).unwrap();
        let name = interp.pop().unwrap();
        assert!(matches!(name, Value::String(ref s) if &**s == "Charlie"));

        // Access age field
        execute_string(r#"person-age"#, &mut interp).unwrap();
        let age = interp.pop().unwrap();
        assert!(matches!(age, Value::Int32(35)));
    }

    #[test]
    fn test_record_mutators() {
        let mut interp = Interpreter::new();

        // Define record type and create instance
        execute_string(
            r#"["name" "age"] "person" make-record-type drop
               "David" 40 make-person"#,
            &mut interp,
        )
        .unwrap();

        // Mutate name field
        execute_string(r#""Eve" swap person-name!"#, &mut interp).unwrap();

        // Verify mutation
        execute_string(r#"person-name"#, &mut interp).unwrap();
        let name = interp.pop().unwrap();
        assert!(matches!(name, Value::String(ref s) if &**s == "Eve"));
    }

    #[test]
    fn test_record_type_of() {
        let mut interp = Interpreter::new();

        // Define record type and create instance
        execute_string(
            r#"["x" "y"] "point" make-record-type drop
               10 20 make-point"#,
            &mut interp,
        )
        .unwrap();

        // Get type name
        execute_string(r#"record-type-of"#, &mut interp).unwrap();
        let type_name = interp.pop().unwrap();
        assert!(matches!(type_name, Value::String(ref s) if &**s == "point"));
    }
}
