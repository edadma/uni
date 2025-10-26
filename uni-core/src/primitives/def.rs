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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn test_def_impl() {
        let mut interp = AsyncInterpreter::new();

        let name = interp.intern_atom("test");
        interp.push(Value::Atom(name.clone()));
        interp.push(interp.make_list(vec![Value::Number(42.0)]));
        def_impl(&mut interp).unwrap();

        let entry = interp.dictionary.get(&name).unwrap();
        assert!(entry.is_executable);
    }
}
