use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Atom(Rc<str>),              // Interned atoms for efficiency
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