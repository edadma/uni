use crate::compat::{Rc, String, Vec, Box, ToString};
use crate::tokenizer::SourcePos;
use crate::value::{RuntimeError, Value};
use crate::output::Output;
use crate::time_source::TimeSource;
use num_traits::Zero;

#[cfg(target_os = "none")]
use num_traits::Float;

#[cfg(not(target_os = "none"))]
use std::collections::HashMap;
#[cfg(target_os = "none")]
use alloc::collections::BTreeMap as HashMap;

// RUST CONCEPT: Dictionary entry with metadata
// Each entry contains the value and a flag indicating execution behavior
#[derive(Debug, Clone)]
pub struct DictEntry {
    pub value: Value,
    pub is_executable: bool, // true = execute lists (def), false = push as data (val)
    pub doc: Option<Rc<str>>, // Optional documentation string for help
}

pub struct Interpreter {
    pub stack: Vec<Value>,
    pub return_stack: Vec<Value>, // RUST CONCEPT: Return stack for Forth-like operations
    pub dictionary: HashMap<Rc<str>, DictEntry>,
    pub atoms: HashMap<String, Rc<str>>,
    pub current_pos: Option<SourcePos>, // Track current execution position for error messages
    pending_doc_target: Option<Rc<str>>, // Remember most recent definition for doc
    output: Option<Box<dyn Output>>, // Optional output for print/display (REPL mode)
    time_source: Option<Box<dyn TimeSource>>, // Optional time source for datetime operations

    // Hardware peripherals (micro:bit only)
    #[cfg(feature = "hardware-microbit")]
    pub buttons: Option<microbit::board::Buttons>,
    #[cfg(feature = "hardware-microbit")]
    pub display_buffer: [[u8; 5]; 5],  // Raw pixel buffer for LED matrix
}

impl Interpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            stack: Vec::new(),
            return_stack: Vec::new(), // RUST CONCEPT: Initialize empty return stack
            dictionary: HashMap::new(),
            atoms: HashMap::new(),
            current_pos: None, // No position initially
            pending_doc_target: None,
            output: None, // No output by default (for file execution, tests)
            time_source: None, // No time source by default (platform must inject)

            // Hardware peripherals start as None, set by platform initialization
            #[cfg(feature = "hardware-microbit")]
            buttons: None,
            #[cfg(feature = "hardware-microbit")]
            display_buffer: [[0u8; 5]; 5],  // All LEDs off initially
        };

        // RUST CONCEPT: Automatic initialization
        // Load builtins first (primitives and core operations)
        crate::builtins::register_builtins(&mut interpreter);

        // Then load prelude (higher-level operations built on primitives)
        if let Err(_e) = crate::prelude::load_prelude(&mut interpreter) {
            // In a constructor, we can't easily return errors
            // For now, just continue without prelude
            // TODO: Better error handling for prelude loading
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

    pub fn set_pending_doc_target(&mut self, atom: Rc<str>) {
        self.pending_doc_target = Some(atom);
    }

    pub fn take_pending_doc_target(&mut self) -> Option<Rc<str>> {
        self.pending_doc_target.take()
    }

    pub fn attach_doc(&mut self, atom: &Rc<str>, doc: Rc<str>) -> Result<(), RuntimeError> {
        if let Some(entry) = self.dictionary.get_mut(atom) {
            entry.doc = Some(doc);
            Ok(())
        } else {
            Err(RuntimeError::UndefinedWord(atom.to_string()))
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
            Value::Int32(i) => Ok(i as f64),
            _ => Err(RuntimeError::TypeError("Expected number".to_string())),
        }
    }

    pub fn pop_integer(&mut self) -> Result<usize, RuntimeError> {
        use num_traits::ToPrimitive;
        let value = self.pop()?;
        match value {
            Value::Int32(i) => {
                if i >= 0 {
                    Ok(i as usize)
                } else {
                    Err(RuntimeError::TypeError("Expected non-negative integer".to_string()))
                }
            }
            Value::Integer(i) => i.to_usize().ok_or_else(|| {
                RuntimeError::TypeError("Integer value too large for index".to_string())
            }),
            Value::Number(n) => {
                if n.fract() == 0.0 && n >= 0.0 && n.is_finite() {
                    Ok(n as usize)
                } else {
                    Err(RuntimeError::TypeError("Expected non-negative integer".to_string()))
                }
            }
            _ => Err(RuntimeError::TypeError("Expected integer".to_string())),
        }
    }

    pub fn make_list(&self, items: Vec<Value>) -> Value {
        items.into_iter().rev().fold(Value::Nil, |acc, item| {
            Value::Pair(Rc::new(item), Rc::new(acc))
        })
    }

    pub fn make_array(&self, items: Vec<Value>) -> Value {
        #[cfg(not(target_os = "none"))]
        {
            Value::Array(Rc::new(std::cell::RefCell::new(items)))
        }
        #[cfg(target_os = "none")]
        {
            use core::cell::RefCell;
            Value::Array(Rc::new(RefCell::new(items)))
        }
    }

    pub fn is_null(&self, value: &Value) -> bool {
        matches!(value, Value::Null)
    }

    // RUST CONCEPT: Defensive truthiness check
    // Following JS rules: check for falsy cases explicitly, everything else is truthy
    // This is more maintainable - new types automatically become truthy by default
    pub fn is_truthy(&self, value: &Value) -> bool {
        match value {
            // Falsy cases only:
            Value::Boolean(false) => false,    // false is falsy
            Value::Null => false,              // null is falsy (like JS)
            Value::String(s) if s.is_empty() => false, // "" is falsy (like JS)

            // Zero in all numeric representations is falsy
            Value::Int32(0) => false,
            Value::Number(n) if *n == 0.0 || n.is_nan() => false, // 0 and NaN are falsy (like JS)
            Value::Integer(i) if i.is_zero() => false,
            Value::Rational(r) if r.is_zero() => false,
            #[cfg(feature = "complex_numbers")]
            Value::GaussianInt(re, im) if re.is_zero() && im.is_zero() => false, // 0+0i
            #[cfg(feature = "complex_numbers")]
            Value::Complex(c) if (c.re == 0.0 && c.im == 0.0) || c.re.is_nan() || c.im.is_nan() => false, // 0+0i or NaN parts

            // Everything else is truthy (including Nil, atoms, pairs, arrays, buffers, etc.)
            _ => true,
        }
    }

    // RUST CONCEPT: Return stack operations for Forth-like control structures
    // These operations enable temporary storage of values outside the main computation stack

    pub fn push_return(&mut self, value: Value) {
        self.return_stack.push(value);
    }

    pub fn pop_return(&mut self) -> Result<Value, RuntimeError> {
        self.return_stack.pop().ok_or(RuntimeError::StackUnderflow)
    }

    pub fn peek_return(&self) -> Result<&Value, RuntimeError> {
        self.return_stack.last().ok_or(RuntimeError::StackUnderflow)
    }

    // TODO: Position management for error context - uncomment when connecting to execution pipeline
    // pub fn set_position(&mut self, pos: SourcePos) {
    //     self.current_pos = Some(pos);
    // }

    // TODO: Method for clearing position context - uncomment when needed for multi-statement execution
    // pub fn clear_position(&mut self) {
    //     self.current_pos = None;
    // }

    // Position-aware pop method for better error messages
    pub fn pop_with_context(&mut self, context: &str) -> Result<Value, RuntimeError> {
        if let Some(pos) = &self.current_pos {
            self.stack
                .pop()
                .ok_or_else(|| RuntimeError::StackUnderflowAt {
                    pos: pos.clone(),
                    context: context.to_string(),
                })
        } else {
            self.stack.pop().ok_or(RuntimeError::StackUnderflow)
        }
    }

    // Output management (used by print/help builtins)
    pub fn set_output(&mut self, output: Box<dyn Output>) {
        self.output = Some(output);
    }

    #[allow(dead_code)]
    pub fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub fn set_time_source(&mut self, time_source: Box<dyn TimeSource>) {
        self.time_source = Some(time_source);
    }

    #[allow(dead_code)]
    pub fn has_time_source(&self) -> bool {
        self.time_source.is_some()
    }

    // Write a line to the output if available
    pub fn writeln(&mut self, text: &str) -> Result<(), ()> {
        if let Some(output) = &mut self.output {
            output.write(text.as_bytes())?;
            // Use platform-appropriate line ending
            #[cfg(not(target_os = "none"))]
            output.write(b"\n")?;
            #[cfg(target_os = "none")]
            output.write(b"\r\n")?;
            output.flush()?;
        }
        Ok(())
    }

    // Write text to the output without a newline
    pub fn write_str(&mut self, text: &str) -> Result<(), ()> {
        if let Some(output) = &mut self.output {
            output.write(text.as_bytes())?;
            output.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test-only helper methods
    impl Interpreter {
        fn pop_string(&mut self) -> Result<Rc<str>, RuntimeError> {
            let value = self.pop()?;
            match value {
                Value::String(s) => Ok(s),
                _ => Err(RuntimeError::TypeError("Expected string".to_string())),
            }
        }

        fn pop_boolean(&mut self) -> Result<bool, RuntimeError> {
            let value = self.pop()?;
            match value {
                Value::Boolean(b) => Ok(b),
                _ => Err(RuntimeError::TypeError("Expected boolean".to_string())),
            }
        }
    }

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
            Value::Pair(car, cdr) => match (car.as_ref(), cdr.as_ref()) {
                (Value::Number(n), Value::Nil) => assert_eq!(*n, 42.0),
                _ => panic!("Expected Pair(42, Nil)"),
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
            Value::Pair(car1, cdr1) => match (car1.as_ref(), cdr1.as_ref()) {
                (Value::Number(n1), Value::Pair(car2, cdr2)) => {
                    assert_eq!(*n1, 1.0);
                    match (car2.as_ref(), cdr2.as_ref()) {
                        (Value::Number(n2), Value::Pair(car3, cdr3)) => {
                            assert_eq!(*n2, 2.0);
                            match (car3.as_ref(), cdr3.as_ref()) {
                                (Value::Number(n3), Value::Nil) => assert_eq!(*n3, 3.0),
                                _ => panic!("Expected third element to be 3.0 followed by Nil"),
                            }
                        }
                        _ => panic!("Expected second element to be 2.0"),
                    }
                }
                _ => panic!("Expected first element to be 1.0"),
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
            is_executable: false, // Constants are not executable
            doc: None,
        };
        interp.dictionary.insert(key.clone(), entry);

        match interp.dictionary.get(&key) {
            Some(dict_entry) => {
                match &dict_entry.value {
                    Value::Number(n) => assert_eq!(*n, 99.0),
                    _ => panic!("Expected to find Number(99.0) in dictionary entry"),
                }
                assert!(!dict_entry.is_executable);
            }
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

    #[test]
    fn test_pop_boolean() {
        let mut interp = Interpreter::new();

        // Test successful pop_boolean
        interp.push(Value::Boolean(true));
        assert_eq!(interp.pop_boolean().unwrap(), true);

        interp.push(Value::Boolean(false));
        assert_eq!(interp.pop_boolean().unwrap(), false);

        // Test type error when popping non-boolean
        interp.push(Value::Number(42.0));
        assert!(matches!(
            interp.pop_boolean(),
            Err(RuntimeError::TypeError(msg)) if msg == "Expected boolean"
        ));

        // Test stack underflow
        assert!(matches!(
            interp.pop_boolean(),
            Err(RuntimeError::StackUnderflow)
        ));
    }

    #[test]
    fn test_is_null() {
        let interp = Interpreter::new();

        assert!(interp.is_null(&Value::Null));
        assert!(!interp.is_null(&Value::Nil));
        assert!(!interp.is_null(&Value::Boolean(false)));
        assert!(!interp.is_null(&Value::Number(0.0)));
    }

    #[test]
    fn test_is_truthy() {
        let interp = Interpreter::new();

        // Boolean values
        assert!(interp.is_truthy(&Value::Boolean(true)));
        assert!(!interp.is_truthy(&Value::Boolean(false)));

        // Null is falsy, Nil (empty list) is truthy (like JS)
        assert!(!interp.is_truthy(&Value::Null));
        assert!(interp.is_truthy(&Value::Nil));

        // Numbers: 0 is falsy, NaN is falsy, everything else is truthy
        assert!(!interp.is_truthy(&Value::Number(0.0)));
        assert!(!interp.is_truthy(&Value::Number(f64::NAN))); // NaN is falsy (like JS)
        assert!(interp.is_truthy(&Value::Number(42.0)));
        assert!(interp.is_truthy(&Value::Number(-1.0)));
        assert!(interp.is_truthy(&Value::Number(f64::INFINITY)));
        assert!(interp.is_truthy(&Value::Number(f64::NEG_INFINITY)));

        // Strings: empty is falsy, non-empty is truthy
        assert!(!interp.is_truthy(&Value::String("".into())));
        assert!(interp.is_truthy(&Value::String("hello".into())));

        // Atoms and QuotedAtoms are always truthy
        assert!(interp.is_truthy(&Value::Atom("hello".into())));
        assert!(interp.is_truthy(&Value::QuotedAtom("hello".into())));

        // Pairs are always truthy
        assert!(interp.is_truthy(&Value::Pair(
            Rc::new(Value::Number(1.0)),
            Rc::new(Value::Nil)
        )));
    }
}
