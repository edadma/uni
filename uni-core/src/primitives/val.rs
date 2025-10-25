// VAL primitive - define a non-executable constant

use crate::compat::ToString;
use crate::interpreter::{AsyncInterpreter, DictEntry};
use crate::value::{RuntimeError, Value};

// VAL: ( 'name value -- ) - Define a non-executable constant
pub fn val_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let value = interp.pop()?;
    let name_value = interp.pop()?;

    let atom = match name_value {
        Value::Atom(atom) => atom,
        _ => return Err(RuntimeError::TypeError(
            "VAL requires an atom as the first argument (use 'name value val)".to_string()
        )),
    };

    // Insert into dictionary as non-executable (constant)
    interp.dictionary.insert(
        atom,
        DictEntry {
            value,
            is_executable: false,
            doc: None,
        },
    );

    Ok(())
}
