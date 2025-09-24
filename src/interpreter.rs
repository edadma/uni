use std::rc::Rc;
use std::collections::HashMap;
use crate::value::{Value, RuntimeError};

// RUST CONCEPT: Dictionary entry with metadata
// Each entry contains the value and a flag indicating execution behavior
#[derive(Debug, Clone)]
pub struct DictEntry {
    pub value: Value,
    pub is_executable: bool,  // true = execute lists (def), false = push as data (val)
}

pub struct Interpreter {
    pub stack: Vec<Value>,
    pub dictionary: HashMap<Rc<str>, DictEntry>,
    pub atoms: HashMap<String, Rc<str>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            stack: Vec::new(),
            dictionary: HashMap::new(),
            atoms: HashMap::new(),
        };

        // RUST CONCEPT: Automatic initialization
        // Load builtins first (primitives and core operations)
        crate::builtins::register_builtins(&mut interpreter);

        // Then load stdlib (higher-level operations built on primitives)
        if let Err(_e) = crate::stdlib::load_stdlib(&mut interpreter) {
            // In a constructor, we can't easily return errors
            // For now, just continue without stdlib
            // TODO: Better error handling for stdlib loading
        }

        interpreter
    }

    pub fn intern_atom(&mut self, text: &str) -> Rc<str> {
        if let Some(existing) = self.atoms.get(text) {
            existing.clone()
        } else {
            let atom: Rc<str> = text.into();
            self.atoms.insert(text.to_string(), atom.clone());
            atom
        }
    }

    pub fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn pop_number(&mut self) -> Result<f64, RuntimeError> {
        let value = self.pop()?;
        match value {
            Value::Number(n) => Ok(n),
            _ => Err(RuntimeError::TypeError("Expected number".to_string())),
        }
    }

    pub fn make_list(&self, items: Vec<Value>) -> Value {
        items.into_iter().rev().fold(Value::Nil, |acc, item| {
            Value::Pair(Rc::new(item), Rc::new(acc))
        })
    }

    pub fn pop_string(&mut self) -> Result<Rc<str>, RuntimeError> {
        let value = self.pop()?;
        match value {
            Value::String(s) => Ok(s),
            _ => Err(RuntimeError::TypeError("Expected string".to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_interning() {
        let mut interp = Interpreter::new();

        let atom1 = interp.intern_atom("hello");
        let atom2 = interp.intern_atom("hello");

        assert!(Rc::ptr_eq(&atom1, &atom2));
    }

    #[test]
    fn test_stack_operations() {
        let mut interp = Interpreter::new();

        interp.push(Value::Number(42.0));
        let popped = interp.pop().unwrap();

        match popped {
            Value::Number(n) => assert_eq!(n, 42.0),
            _ => panic!("Expected number"),
        }

        assert!(interp.pop().is_err());
    }

    #[test]
    fn test_list_construction() {
        let interp = Interpreter::new();

        let empty = interp.make_list(vec![]);
        match empty {
            Value::Nil => (),
            _ => panic!("Expected Nil for empty list"),
        }

        let single = interp.make_list(vec![Value::Number(42.0)]);
        match single {
            Value::Pair(car, cdr) => {
                match (car.as_ref(), cdr.as_ref()) {
                    (Value::Number(n), Value::Nil) => assert_eq!(*n, 42.0),
                    _ => panic!("Expected Pair(42, Nil)"),
                }
            },
            _ => panic!("Expected Pair for single element list"),
        }

        // Test multi-element list
        let multi = interp.make_list(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        match multi {
            Value::Pair(car1, cdr1) => {
                match (car1.as_ref(), cdr1.as_ref()) {
                    (Value::Number(n1), Value::Pair(car2, cdr2)) => {
                        assert_eq!(*n1, 1.0);
                        match (car2.as_ref(), cdr2.as_ref()) {
                            (Value::Number(n2), Value::Pair(car3, cdr3)) => {
                                assert_eq!(*n2, 2.0);
                                match (car3.as_ref(), cdr3.as_ref()) {
                                    (Value::Number(n3), Value::Nil) => assert_eq!(*n3, 3.0),
                                    _ => panic!("Expected third element to be 3.0 followed by Nil"),
                                }
                            },
                            _ => panic!("Expected second element to be 2.0"),
                        }
                    },
                    _ => panic!("Expected first element to be 1.0"),
                }
            },
            _ => panic!("Expected Pair for multi-element list"),
        }
    }

    #[test]
    fn test_pop_number_success() {
        let mut interp = Interpreter::new();
        interp.push(Value::Number(42.0));
        assert_eq!(interp.pop_number().unwrap(), 42.0);
    }

    #[test]
    fn test_pop_number_type_error() {
        let mut interp = Interpreter::new();
        interp.push(Value::Nil);
        assert!(matches!(
            interp.pop_number(),
            Err(RuntimeError::TypeError(msg)) if msg == "Expected number"
        ));
    }

    #[test]
    fn test_pop_number_underflow() {
        let mut interp = Interpreter::new();
        assert!(matches!(
            interp.pop_number(),
            Err(RuntimeError::StackUnderflow)
        ));
    }

    #[test]
    fn test_dictionary_operations() {
        let mut interp = Interpreter::new();

        // Test inserting and retrieving from dictionary
        let key = interp.intern_atom("test");
        let entry = DictEntry {
            value: Value::Number(99.0),
            is_executable: false,  // Constants are not executable
        };
        interp.dictionary.insert(key.clone(), entry);

        match interp.dictionary.get(&key) {
            Some(dict_entry) => {
                match &dict_entry.value {
                    Value::Number(n) => assert_eq!(*n, 99.0),
                    _ => panic!("Expected to find Number(99.0) in dictionary entry"),
                }
                assert!(!dict_entry.is_executable);
            },
            _ => panic!("Expected to find dictionary entry"),
        }

        // Test that non-existent keys return None
        let missing = interp.intern_atom("missing");
        assert!(interp.dictionary.get(&missing).is_none());
    }

    #[test]
    fn test_atom_interning_different_atoms() {
        let mut interp = Interpreter::new();

        let atom1 = interp.intern_atom("hello");
        let atom2 = interp.intern_atom("world");
        let atom3 = interp.intern_atom("hello");

        // Same text should return same Rc
        assert!(Rc::ptr_eq(&atom1, &atom3));

        // Different text should return different Rc
        assert!(!Rc::ptr_eq(&atom1, &atom2));

        // Verify the actual content
        assert_eq!(&*atom1, "hello");
        assert_eq!(&*atom2, "world");
    }

    #[test]
    fn test_pop_string() {
        let mut interp = Interpreter::new();

        // Test successful pop_string
        let string_val: Rc<str> = "hello world".into();
        interp.push(Value::String(string_val));
        let s = interp.pop_string().unwrap();
        assert_eq!(&*s, "hello world");

        // Test type error when popping non-string
        interp.push(Value::Number(42.0));
        assert!(matches!(
            interp.pop_string(),
            Err(RuntimeError::TypeError(msg)) if msg == "Expected string"
        ));

        // Test stack underflow
        assert!(matches!(
            interp.pop_string(),
            Err(RuntimeError::StackUnderflow)
        ));
    }

    #[test]
    fn test_string_vs_atom_distinction() {
        let mut interp = Interpreter::new();

        // Strings are not interned - each is separate
        let string1 = Value::String("hello".into());
        let string2 = Value::String("hello".into());

        // Atoms are interned - same text gives same Rc
        let atom1 = Value::Atom(interp.intern_atom("hello"));
        let atom2 = Value::Atom(interp.intern_atom("hello"));

        // Strings with same content are different Rc objects (not interned)
        if let (Value::String(s1), Value::String(s2)) = (&string1, &string2) {
            assert_eq!(s1, s2); // Same content
            assert!(!Rc::ptr_eq(s1, s2)); // Different Rc objects
        }

        // Atoms with same content share the same object
        if let (Value::Atom(a1), Value::Atom(a2)) = (&atom1, &atom2) {
            assert_eq!(a1, a2); // Same content
            assert!(Rc::ptr_eq(a1, a2)); // Same object
        }
    }
}