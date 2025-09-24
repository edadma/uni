// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: The val builtin - defines constants only (like Scheme's define for constants)
// Usage: 'constant-name value val
// Examples:
//   'pi 3.14159 val         - defines a constant
//   'greeting "Hello!" val  - defines a string constant
// Unlike def, val is specifically for constants that shouldn't be evaluated
pub fn val_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Same implementation as def for now
    // The distinction is semantic - val is for constants, def is for procedures
    let definition = interp.pop()?;  // The constant value
    let name_value = interp.pop()?;  // Name of the constant

    match name_value {
        Value::Atom(atom_name) => {
            // RUST CONCEPT: Creating dict entry with constant flag
            use crate::interpreter::DictEntry;
            let entry = DictEntry {
                value: definition,
                is_executable: false,  // val marks entries as constants
            };
            interp.dictionary.insert(atom_name, entry);
            Ok(())
        },
        _ => Err(RuntimeError::TypeError(
            "val expects an atom as the constant name (use 'name value val)".to_string()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;
    use crate::interpreter::DictEntry;

    fn setup_interpreter() -> Interpreter {
        Interpreter::new()
    }

    #[test]
    fn test_val_builtin_constants() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing val for defining constants
        // Define pi: 'pi 3.14159 val
        let pi_atom = interp.intern_atom("pi");
        interp.push(Value::Atom(pi_atom.clone()));
        interp.push(Value::Number(3.14159));
        val_builtin(&mut interp).unwrap();

        // Verify it was stored
        match interp.dictionary.get(&pi_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert!((n - 3.14159).abs() < 1e-10),
            _ => panic!("Expected pi constant"),
        }

        // Define string constant: 'greeting "Hello!" val
        let greeting_atom = interp.intern_atom("greeting");
        let hello_str: std::rc::Rc<str> = "Hello!".into();
        interp.push(Value::Atom(greeting_atom.clone()));
        interp.push(Value::String(hello_str));
        val_builtin(&mut interp).unwrap();

        // Verify string constant
        match interp.dictionary.get(&greeting_atom) {
            Some(DictEntry { value: Value::String(s), .. }) => assert_eq!(s.as_ref(), "Hello!"),
            _ => panic!("Expected greeting string constant"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_error_cases() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing val error handling
        // Same error conditions as def
        assert!(val_builtin(&mut interp).is_err()); // Empty stack

        interp.push(Value::Number(42.0));
        assert!(val_builtin(&mut interp).is_err()); // Only one argument

        // Non-atom name should fail
        interp.push(Value::Number(42.0));        // Invalid name
        interp.push(Value::Number(123.0));       // Value
        let result = val_builtin(&mut interp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("val expects an atom"));

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_constant_flag() {
        let mut interp = setup_interpreter();

        // Test that val marks entries as non-executable (constants)
        let test_atom = interp.intern_atom("test_constant");
        interp.push(Value::Atom(test_atom.clone()));
        interp.push(Value::Number(42.0));
        val_builtin(&mut interp).unwrap();

        // Verify executable flag is set to false
        match interp.dictionary.get(&test_atom) {
            Some(DictEntry { is_executable, .. }) => assert!(!*is_executable),
            _ => panic!("Expected test_constant to be defined"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_various_types() {
        let mut interp = setup_interpreter();

        // Test defining various constant types
        let constants_to_test = vec![
            ("bool_const", Value::Boolean(false)),
            ("null_const", Value::Null),
            ("string_const", Value::String("constant value".into())),
            ("atom_const", Value::Atom(interp.intern_atom("constant_atom"))),
        ];

        for (name, value) in constants_to_test {
            let name_atom = interp.intern_atom(name);
            interp.push(Value::Atom(name_atom.clone()));
            interp.push(value.clone());
            val_builtin(&mut interp).unwrap();

            // Verify it was stored as non-executable
            match interp.dictionary.get(&name_atom) {
                Some(DictEntry { value: stored_value, is_executable }) => {
                    assert!(!*is_executable, "Constants should not be executable");

                    // Basic type check
                    match (&value, stored_value) {
                        (Value::Boolean(b1), Value::Boolean(b2)) => assert_eq!(b1, b2),
                        (Value::Null, Value::Null) => (),
                        (Value::String(s1), Value::String(s2)) => assert_eq!(s1, s2),
                        (Value::Atom(a1), Value::Atom(a2)) => assert_eq!(a1, a2),
                        _ => panic!("Value type mismatch for {}", name),
                    }
                },
                _ => panic!("Expected {} to be defined", name),
            }
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_redefinition() {
        let mut interp = setup_interpreter();

        // Test redefining a constant
        let const_atom = interp.intern_atom("my_const");

        // First definition
        interp.push(Value::Atom(const_atom.clone()));
        interp.push(Value::Number(100.0));
        val_builtin(&mut interp).unwrap();

        // Verify first definition
        match interp.dictionary.get(&const_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert_eq!(*n, 100.0),
            _ => panic!("Expected first definition"),
        }

        // Redefine the constant
        interp.push(Value::Atom(const_atom.clone()));
        interp.push(Value::Number(200.0));
        val_builtin(&mut interp).unwrap();

        // Verify redefinition
        match interp.dictionary.get(&const_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert_eq!(*n, 200.0),
            _ => panic!("Expected redefinition"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_with_nil() {
        let mut interp = setup_interpreter();

        // Test defining nil as a constant
        let nil_atom = interp.intern_atom("nil_const");
        interp.push(Value::Atom(nil_atom.clone()));
        interp.push(Value::Nil);
        val_builtin(&mut interp).unwrap();

        // Verify nil constant
        match interp.dictionary.get(&nil_atom) {
            Some(DictEntry { value: Value::Nil, is_executable }) => {
                assert!(!*is_executable, "Nil constant should not be executable");
            },
            _ => panic!("Expected nil_const to be defined as Nil"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_val_builtin_with_list() {
        let mut interp = setup_interpreter();

        // Test defining a list as a constant (even though it's unusual)
        let list_atom = interp.intern_atom("list_const");
        let constant_list = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0)
        ]);

        interp.push(Value::Atom(list_atom.clone()));
        interp.push(constant_list);
        val_builtin(&mut interp).unwrap();

        // Verify list constant is non-executable
        match interp.dictionary.get(&list_atom) {
            Some(DictEntry { value: Value::Pair(_, _), is_executable }) => {
                assert!(!*is_executable, "List constants should not be executable");
            },
            _ => panic!("Expected list_const to be defined as a list"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }
}