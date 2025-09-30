use crate::tokenizer::SourcePos;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Atom(Rc<str>),       // Interned atoms for efficiency
    QuotedAtom(Rc<str>), // Quoted atoms - push without executing
    String(Rc<str>),     // Literal strings - ref counted but not interned
    Boolean(bool),       // True/false boolean values
    Null,                // Null/undefined value (distinct from Nil empty list)
    Pair(Rc<Value>, Rc<Value>),
    Array(Rc<RefCell<Vec<Value>>>),
    Nil,
    Builtin(fn(&mut crate::interpreter::Interpreter) -> Result<(), RuntimeError>),
}

#[derive(Debug)]
pub enum RuntimeError {
    StackUnderflow,
    StackUnderflowAt { pos: SourcePos, context: String },
    TypeError(String),
    UndefinedWord(String),
    DivisionByZero,
    ModuloByZero,
    DomainError(String),
}

// RUST CONCEPT: Implementing traits for custom error types
// The Display trait allows us to convert errors to strings
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::StackUnderflow => write!(f, "Stack underflow"),
            RuntimeError::StackUnderflowAt { pos, context } => {
                write!(
                    f,
                    "Stack underflow at line {}, column {}: {}",
                    pos.line, pos.column, context
                )
            }
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::UndefinedWord(word) => write!(f, "Undefined word: {}", word),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::ModuloByZero => write!(f, "Modulo by zero"),
            RuntimeError::DomainError(msg) => write!(f, "Domain error: {}", msg),
        }
    }
}

// RUST CONCEPT: Implementing Display for Value types
// This is the "data display" mode - strings WITH quotes for data structures
impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Atom(a) => write!(f, "{}", a),
            Value::QuotedAtom(a) => write!(f, "'{}", a),
            Value::String(s) => write!(f, "\"{}\"", s), // Strings WITH quotes
            Value::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Value::Null => write!(f, "null"),
            Value::Pair(head, tail) => {
                write!(f, "[")?;
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
                write!(f, "]")
            }
            Value::Array(elements) => {
                let elements_ref = elements.borrow();
                write!(f, "#[")?;
                let mut iter = elements_ref.iter();
                if let Some(first) = iter.next() {
                    write!(f, "{}", first)?;
                    for elem in iter {
                        write!(f, " {}", elem)?;
                    }
                }
                write!(f, "]")
            }
            Value::Nil => write!(f, "[]"),
            Value::Builtin(_) => write!(f, "<builtin>"),
        }
    }
}
