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
    interp.dict_insert(
        atom,
        DictEntry {
            value,
            is_executable: false,
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
    fn test_val_impl() {
        let mut interp = AsyncInterpreter::new();

        let name = interp.intern_atom("pi");
        interp.push(Value::Atom(name.clone()));
        interp.push(Value::Number(3.14159));
        val_impl(&mut interp).unwrap();

        let entry = interp.dict_get(&name).unwrap();
        assert!(!entry.is_executable);
        assert!(matches!(entry.value, Value::Number(n) if (n - 3.14159).abs() < 0.001));
    }
}
