// DEF primitive - define an executable word

use crate::compat::ToString;
use crate::interpreter::{AsyncInterpreter, DictEntry};
use crate::value::{RuntimeError, Value};

// DEF: ( 'name [body] -- ) - Define an executable word
pub fn def_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let body = interp.pop()?;
    let name_value = interp.pop()?;

    let atom = match name_value {
        Value::Atom(atom) => atom,
        _ => return Err(RuntimeError::TypeError(
            "DEF requires an atom as the first argument (use 'name [...] def)".to_string()
        )),
    };

    // Store pending doc target for doc string attachment
    interp.set_pending_doc_target(atom.clone());

    // Insert into dictionary as executable
    interp.dictionary.insert(
        atom,
        DictEntry {
            value: body,
            is_executable: true,
            doc: None,
        },
    );

    Ok(())
}
