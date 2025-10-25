// Forth-style variable creation
// Creates a mutable variable and binds it to a name in the dictionary

use crate::compat::{format, Rc};
use crate::interpreter::AsyncInterpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// Variable creation primitive
// Stack-based: ( initial-value 'name -- )
pub fn var_impl(interp: &mut AsyncInterpreter) -> Result<(), RuntimeError> {
    let name_val = interp.pop()?;
    let initial_value = interp.pop()?;

    let name = match name_val {
        Value::Atom(ref atom) => atom.clone(),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "var expects atom name, got {:?}",
                name_val
            )))
        }
    };

    let var = Value::Variable(Rc::new(RefCell::new(initial_value)));

    let dict_entry = crate::interpreter::DictEntry {
        value: var,
        is_executable: true,
        doc: None,
    };

    interp.dictionary.insert(name, dict_entry);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::AsyncInterpreter;
    use crate::value::Value;

    #[test]
    fn test_var_creation() {
        let mut interp = AsyncInterpreter::new();

        // Create variable: 0 'counter var
        interp.push(Value::Int32(0));
        let name = interp.intern_atom("counter");
        interp.push(Value::Atom(name.clone()));
        
        var_impl(&mut interp).unwrap();

        // Verify variable exists in dictionary
        assert!(interp.dictionary.contains_key(&name));
    }

    #[test]
    fn test_var_executable() {
        let mut interp = AsyncInterpreter::new();

        interp.push(Value::Int32(42));
        let name = interp.intern_atom("x");
        interp.push(Value::Atom(name.clone()));
        var_impl(&mut interp).unwrap();

        // Verify it's marked as executable
        let entry = interp.dictionary.get(&name).unwrap();
        assert!(entry.is_executable);
        assert!(matches!(entry.value, Value::Variable(_)));
    }
}
