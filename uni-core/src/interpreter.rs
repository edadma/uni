use crate::compat::{Rc, String, Vec, Box, ToString};
use crate::tokenizer::SourcePos;
use crate::value::{RuntimeError, Value};
use crate::output::AsyncOutput;
use crate::time_source::TimeSource;
use num_traits::Zero;

#[cfg(target_os = "none")]
use num_traits::Float;

#[cfg(not(target_os = "none"))]
use std::collections::HashMap;
#[cfg(target_os = "none")]
use alloc::collections::BTreeMap as HashMap;

// ASYNC CONCEPT: Dictionary entry with metadata
// Each entry contains the value and a flag indicating execution behavior
#[derive(Clone)]
pub struct DictEntry {
    pub value: Value,
    pub is_executable: bool, // true = execute lists (def), false = push as data (val)
    pub doc: Option<Rc<str>>, // Optional documentation string for help
}

// Implement Debug manually since Value doesn't auto-derive Debug
impl core::fmt::Debug for DictEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DictEntry")
            .field("value", &self.value)
            .field("is_executable", &self.is_executable)
            .field("doc", &self.doc)
            .finish()
    }
}

pub struct AsyncInterpreter {
    pub stack: Vec<Value>,
    pub return_stack: Vec<Value>, // Return stack for Forth-like operations
    pub dictionary: HashMap<Rc<str>, DictEntry>,
    pub atoms: HashMap<String, Rc<str>>,
    pub local_frames: Vec<HashMap<Rc<str>, Value>>, // Stack of local variable frames for lexical scoping
    pub current_pos: Option<SourcePos>, // Track current execution position for error messages
    pending_doc_target: Option<Rc<str>>, // Remember most recent definition for doc

    // ASYNC CONCEPT: AsyncOutput instead of Output
    async_output: Option<Box<dyn AsyncOutput>>, // Optional async output for print/display (REPL mode)

    // Time source for date/time operations (public for primitive access)
    pub time_source: Option<Box<dyn TimeSource>>, // Optional time source for datetime operations
}

impl AsyncInterpreter {
    pub fn new() -> Self {
        let mut interpreter = Self {
            stack: Vec::new(),
            return_stack: Vec::new(),
            dictionary: HashMap::new(),
            atoms: HashMap::new(),
            local_frames: Vec::new(),
            current_pos: None,
            pending_doc_target: None,
            async_output: None,
            time_source: None, // No time source by default (platform must inject)
        };

        // ASYNC CONCEPT: Automatic initialization
        // Load builtins first (primitives and core operations)
        crate::builtins::register_async_builtins(&mut interpreter);

        interpreter
    }

    // ASYNC CONCEPT: Async prelude loading
    // Must be called after new() to load prelude definitions
    pub async fn load_prelude(&mut self) -> Result<(), crate::value::RuntimeError> {
        crate::prelude::load_prelude(self).await
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

    // ASYNC CONCEPT: AsyncOutput management (used by print/help builtins)
    pub fn set_async_output(&mut self, output: Box<dyn AsyncOutput>) {
        self.async_output = Some(output);
    }

    #[allow(dead_code)]
    pub fn has_async_output(&self) -> bool {
        self.async_output.is_some()
    }

    // ASYNC CONCEPT: Async write operations
    // These are async methods that await the output operations

    /// Write a line to the async output if available
    pub async fn writeln_async(&mut self, text: &str) -> Result<(), ()> {
        if let Some(output) = &mut self.async_output {
            output.write(text.as_bytes()).await?;
            // Use platform-appropriate line ending
            #[cfg(not(target_os = "none"))]
            output.write(b"\n").await?;
            #[cfg(target_os = "none")]
            output.write(b"\r\n").await?;
            output.flush().await?;
        }
        Ok(())
    }

    /// Write text to the async output without a newline
    pub async fn write_str_async(&mut self, text: &str) -> Result<(), ()> {
        if let Some(output) = &mut self.async_output {
            output.write(text.as_bytes()).await?;
            output.flush().await?;
        }
        Ok(())
    }

    // RUST CONCEPT: TimeSource management for datetime operations
    pub fn set_time_source(&mut self, time_source: Box<dyn TimeSource>) {
        self.time_source = Some(time_source);
    }

    pub fn has_time_source(&self) -> bool {
        self.time_source.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atom_interning() {
        let mut interp = AsyncInterpreter::new();

        let atom1 = interp.intern_atom("hello");
        let atom2 = interp.intern_atom("hello");

        assert!(Rc::ptr_eq(&atom1, &atom2));
    }

    #[test]
    fn test_stack_operations() {
        let mut interp = AsyncInterpreter::new();

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
        let interp = AsyncInterpreter::new();

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
    }

    #[test]
    fn test_is_truthy() {
        let interp = AsyncInterpreter::new();

        // Boolean values
        assert!(interp.is_truthy(&Value::Boolean(true)));
        assert!(!interp.is_truthy(&Value::Boolean(false)));

        // Null is falsy, Nil (empty list) is truthy
        assert!(!interp.is_truthy(&Value::Null));
        assert!(interp.is_truthy(&Value::Nil));

        // Numbers: 0 is falsy, everything else is truthy
        assert!(!interp.is_truthy(&Value::Number(0.0)));
        assert!(interp.is_truthy(&Value::Number(42.0)));
    }
}
