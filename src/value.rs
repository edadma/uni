use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Atom(Rc<str>),              // Interned atoms for efficiency
    QuotedAtom(Rc<str>),        // Quoted atoms - push without executing
    String(Rc<str>),            // Literal strings - ref counted but not interned
    Boolean(bool),              // True/false boolean values
    Null,                       // Null/undefined value (distinct from Nil empty list)
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

// RUST CONCEPT: Implementing Display for Value types
// This allows Values to be printed in a readable format
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Atom(a) => write!(f, "{}", a),
            Value::QuotedAtom(a) => write!(f, "'{}", a),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Null => write!(f, "null"),
            Value::Pair(head, tail) => {
                write!(f, "(")?;
                write!(f, "{}", head)?;
                let mut current = tail;
                loop {
                    match current.as_ref() {
                        Value::Nil => break,
                        Value::Pair(h, t) => {
                            write!(f, " {}", h)?;
                            current = t;
                        }
                        other => {
                            write!(f, " . {}", other)?;
                            break;
                        }
                    }
                }
                write!(f, ")")
            }
            Value::Nil => write!(f, "nil"),
            Value::Builtin(_) => write!(f, "<builtin>"),
        }
    }
}