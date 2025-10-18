// RUST CONCEPT: Forth-style variable creation
// Creates a mutable variable and binds it to a name in the dictionary
use crate::compat::{format, Rc};
use crate::interpreter::Interpreter;
use crate::value::{RuntimeError, Value};

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

// RUST CONCEPT: Variable creation primitive
// Stack-based: ( initial-value 'name -- )
// Creates a Variable containing initial-value and defines it as 'name'
pub fn var_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    let name_val = interp.pop()?;
    let initial_value = interp.pop()?;

    // Extract the atom name (like def, expects Atom not QuotedAtom)
    let name = match name_val {
        Value::Atom(ref atom) => atom.clone(),
        _ => {
            return Err(RuntimeError::TypeError(format!(
                "var expects atom name, got {:?}",
                name_val
            )))
        }
    };

    // Create the variable
    let var = Value::Variable(Rc::new(RefCell::new(initial_value)));

    // Store in dictionary as an executable word
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
    use crate::value::Value;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_var_basic() {
        let mut interp = setup_interpreter();

        // Create a variable: 42 'x var
        interp.push(Value::Int32(42));
        let name_atom = interp.intern_atom("x");
        interp.push(Value::Atom(name_atom.clone()));
        var_builtin(&mut interp).unwrap();

        // Check that 'x' is in the dictionary
        assert!(interp.dictionary.contains_key(&name_atom));

        // Get the dictionary entry
        let entry = interp.dictionary.get(&name_atom).unwrap();

        // Verify it's a Variable
        assert!(matches!(entry.value, Value::Variable(_)));

        // Verify it's executable
        assert!(entry.is_executable);
    }

    #[test]
    fn test_var_wrong_name_type() {
        let mut interp = setup_interpreter();

        // Try to create variable with non-quoted atom
        interp.push(Value::Int32(42));
        interp.push(Value::Int32(5)); // Wrong type for name
        let result = var_builtin(&mut interp);

        assert!(matches!(result, Err(RuntimeError::TypeError(_))));
    }

    #[test]
    fn test_var_stack_underflow() {
        let mut interp = setup_interpreter();

        // Not enough values on stack
        let result = var_builtin(&mut interp);
        assert!(matches!(result, Err(RuntimeError::StackUnderflow)));
    }
}
