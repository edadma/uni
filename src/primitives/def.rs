// RUST CONCEPT: Modular primitive organization
// Each primitive gets its own file with implementation and tests
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: The def builtin - defines new words in the dictionary
// Usage: 'word-name definition def
// Examples:
//   'square [dup *] def     - defines a procedure
//   'pi 3.14159 def         - defines a constant
pub fn def_builtin(interp: &mut Interpreter) -> Result<(), RuntimeError> {
    // RUST CONCEPT: Stack-based parameter passing
    // def expects two values on the stack:
    // 1. The definition (top of stack) - can be any Value
    // 2. The word name (second on stack) - must be an Atom

    let definition = interp.pop()?;  // What to define the word as
    let name_value = interp.pop()?;  // Name of the word to define

    // RUST CONCEPT: Pattern matching for type checking
    // The word name must be an Atom (interned string)
    match name_value {
        Value::Atom(atom_name) => {
            // RUST CONCEPT: Creating dict entry with executable flag
            use crate::interpreter::DictEntry;
            let entry = DictEntry {
                value: definition,
                is_executable: true,  // def marks entries as executable
            };
            interp.dictionary.insert(atom_name, entry);
            Ok(())
        },

        // RUST CONCEPT: Descriptive error messages
        // If the name isn't an atom, we can't use it as a dictionary key
        _ => Err(RuntimeError::TypeError(
            "def expects an atom as the word name (use 'word-name definition def)".to_string()
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
    fn test_def_builtin_constant() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing constant definition
        // Define pi as 3.14159: 'pi 3.14159 def
        let pi_atom = interp.intern_atom("pi");
        interp.push(Value::Atom(pi_atom.clone()));      // Word name
        interp.push(Value::Number(3.14159));    // Definition
        def_builtin(&mut interp).unwrap();

        // Verify it was stored in dictionary
        let pi_lookup = interp.intern_atom("pi");
        assert!(interp.dictionary.contains_key(&pi_lookup));

        // Verify we can retrieve the constant
        match interp.dictionary.get(&pi_lookup) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert!((n - 3.14159).abs() < 1e-10),
            _ => panic!("Expected pi to be defined as a number"),
        }

        // Clear stack before next test
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_procedure() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing procedure definition
        // Define square as [dup *]: 'square [dup *] def
        let square_atom = interp.intern_atom("square");
        let dup_atom = interp.intern_atom("dup");
        let mul_atom = interp.intern_atom("*");

        // Create the procedure list [dup *]
        let square_proc = interp.make_list(vec![
            Value::Atom(dup_atom),
            Value::Atom(mul_atom),
        ]);

        interp.push(Value::Atom(square_atom.clone()));  // Word name
        interp.push(square_proc);               // Definition
        def_builtin(&mut interp).unwrap();

        // Verify it was stored in dictionary
        let square_lookup = interp.intern_atom("square");
        assert!(interp.dictionary.contains_key(&square_lookup));

        // Verify we can retrieve the procedure
        match interp.dictionary.get(&square_lookup) {
            Some(DictEntry { value: Value::Pair(_, _), .. }) => (), // It's a list (procedure)
            _ => panic!("Expected square to be defined as a list"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_string_definition() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing string definition
        // Define greeting as "Hello, Uni!": 'greeting "Hello, Uni!" def
        let greeting_atom = interp.intern_atom("greeting");
        let greeting_string: std::rc::Rc<str> = "Hello, Uni!".into();

        interp.push(Value::Atom(greeting_atom.clone()));           // Word name
        interp.push(Value::String(greeting_string));       // Definition
        def_builtin(&mut interp).unwrap();

        // Verify it was stored
        let greeting_lookup = interp.intern_atom("greeting");
        match interp.dictionary.get(&greeting_lookup) {
            Some(DictEntry { value: Value::String(s), .. }) => assert_eq!(s.as_ref(), "Hello, Uni!"),
            _ => panic!("Expected greeting to be defined as a string"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_error_cases() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing error handling
        // def requires exactly two arguments
        assert!(def_builtin(&mut interp).is_err()); // Empty stack

        interp.push(Value::Number(42.0));
        assert!(def_builtin(&mut interp).is_err()); // Only one argument

        // RUST CONCEPT: Testing type safety
        // First argument (word name) must be an Atom
        interp.push(Value::Number(42.0));        // Invalid name (not atom)
        interp.push(Value::Number(123.0));       // Definition
        let result = def_builtin(&mut interp);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("def expects an atom"));

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_redefinition() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing word redefinition
        // First define foo as 123
        let foo_atom = interp.intern_atom("foo");
        interp.push(Value::Atom(foo_atom.clone()));
        interp.push(Value::Number(123.0));
        def_builtin(&mut interp).unwrap();

        // Verify first definition
        match interp.dictionary.get(&foo_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert_eq!(*n, 123.0),
            _ => panic!("Expected foo to be 123"),
        }

        // Redefine foo as 456
        interp.push(Value::Atom(foo_atom.clone()));
        interp.push(Value::Number(456.0));
        def_builtin(&mut interp).unwrap();

        // Verify redefinition worked
        match interp.dictionary.get(&foo_atom) {
            Some(DictEntry { value: Value::Number(n), .. }) => assert_eq!(*n, 456.0),
            _ => panic!("Expected foo to be redefined as 456"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_with_nil() {
        let mut interp = setup_interpreter();

        // RUST CONCEPT: Testing edge case - defining with empty list
        let empty_atom = interp.intern_atom("empty");
        interp.push(Value::Atom(empty_atom.clone()));
        interp.push(Value::Nil);
        def_builtin(&mut interp).unwrap();

        // Verify nil definition
        match interp.dictionary.get(&empty_atom) {
            Some(DictEntry { value: Value::Nil, .. }) => (),
            _ => panic!("Expected empty to be defined as Nil"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_executable_flag() {
        let mut interp = setup_interpreter();

        // Test that def marks entries as executable
        let test_atom = interp.intern_atom("test");
        interp.push(Value::Atom(test_atom.clone()));
        interp.push(Value::Number(42.0));
        def_builtin(&mut interp).unwrap();

        // Verify executable flag is set to true
        match interp.dictionary.get(&test_atom) {
            Some(DictEntry { is_executable, .. }) => assert!(*is_executable),
            _ => panic!("Expected test to be defined"),
        }

        // Clear stack
        while interp.pop().is_ok() {}
    }

    #[test]
    fn test_def_builtin_various_types() {
        let mut interp = setup_interpreter();

        // Test defining various value types
        let types_to_test = vec![
            ("bool_val", Value::Boolean(true)),
            ("null_val", Value::Null),
            ("atom_val", Value::Atom(interp.intern_atom("test_atom"))),
        ];

        for (name, value) in types_to_test {
            let name_atom = interp.intern_atom(name);
            interp.push(Value::Atom(name_atom.clone()));
            interp.push(value.clone());
            def_builtin(&mut interp).unwrap();

            // Verify it was stored
            assert!(interp.dictionary.contains_key(&name_atom));
            match interp.dictionary.get(&name_atom) {
                Some(DictEntry { value: stored_value, .. }) => {
                    // Basic type check - more detailed equality would need custom implementation
                    match (&value, stored_value) {
                        (Value::Boolean(b1), Value::Boolean(b2)) => assert_eq!(b1, b2),
                        (Value::Null, Value::Null) => (),
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
}