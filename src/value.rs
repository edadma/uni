use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Atom(Rc<str>),              // Interned atoms for efficiency
    QuotedAtom(Rc<str>),        // Quoted atoms - push without executing
    String(Rc<str>),            // Literal strings - ref counted but not interned
    Pair(Rc<Value>, Rc<Value>),
    Nil,
    Builtin(fn(&mut crate::interpreter::Interpreter) -> Result<(), RuntimeError>),
}

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    TypeError(String),
    UndefinedWord(String),
}

// RUST CONCEPT: Implementing traits for custom error types
// The Display trait allows us to convert errors to strings
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::StackUnderflow => write!(f, "Stack underflow"),
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::UndefinedWord(word) => write!(f, "Undefined word: {}", word),
        }
    }
}