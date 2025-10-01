use crate::tokenizer::SourcePos;
use num_bigint::BigInt;
use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::{One, Zero};
use std::cell::RefCell;
use std::rc::Rc;

// RUST CONCEPT: Using the num ecosystem for arbitrary precision and special number types
// BigInt: Arbitrary precision integers (unlimited size)
// BigRational: Exact rational numbers (fractions)
// GaussianInt: Gaussian integers (a + bi where a, b are integers)
// Complex64: Complex numbers with f64 components

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),                    // Floating point number (default)
    Integer(BigInt),                // Arbitrary precision integer
    Rational(BigRational),          // Exact rational number (fraction)
    GaussianInt(BigInt, BigInt),    // Gaussian integer (real, imaginary) - both integers
    Complex(Complex64),             // Complex number (a + bi) - floating point components
    Atom(Rc<str>),                  // Interned atoms for efficiency
    QuotedAtom(Rc<str>),            // Quoted atoms - push without executing
    String(Rc<str>),                // Literal strings - ref counted but not interned
    Boolean(bool),                  // True/false boolean values
    Null,                           // Null/undefined value (distinct from Nil empty list)
    Pair(Rc<Value>, Rc<Value>),    // Cons cell for lists
    Array(Rc<RefCell<Vec<Value>>>), // Mutable array/vector
    Nil,                            // Empty list marker
    Builtin(fn(&mut crate::interpreter::Interpreter) -> Result<(), RuntimeError>),
}

impl Value {
    // RUST CONCEPT: Get the type name of a value
    // Returns a string describing the type for display and debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Integer(_) => "integer",
            Value::Rational(_) => "rational",
            Value::GaussianInt(_, _) => "gaussian",
            Value::Complex(_) => "complex",
            Value::Atom(_) => "atom",
            Value::QuotedAtom(_) => "quoted-atom",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "null",
            Value::Pair(_, _) => "list",
            Value::Array(_) => "vector",
            Value::Nil => "nil",
            Value::Builtin(_) => "builtin",
        }
    }

    // RUST CONCEPT: Automatic numeric type demotion for cleaner results
    // This function attempts to demote numeric types to simpler representations:
    // - Rational with denominator 1 → Integer
    // - Rational with numerator 0 → Integer(0)
    // - GaussianInt with imaginary 0 → Integer
    // This keeps values in their simplest form after arithmetic operations
    pub fn demote(self) -> Self {
        match &self {
            // Check Rational: demote if denominator is 1 or numerator is 0
            Value::Rational(r) if r.numer().is_zero() => {
                // 0/n → 0
                Value::Integer(BigInt::from(0))
            }
            Value::Rational(r) if r.denom().is_one() => {
                // n/1 → n
                // Extract the inner BigRational and clone its numerator
                if let Value::Rational(r) = self {
                    Value::Integer(r.numer().clone())
                } else {
                    unreachable!()
                }
            }
            // Check GaussianInt: demote if imaginary part is 0
            Value::GaussianInt(_re, im) if im.is_zero() => {
                // a+0i → a (move real part out)
                if let Value::GaussianInt(re, _im) = self {
                    Value::Integer(re)
                } else {
                    unreachable!()
                }
            }
            // All other cases: return unchanged (no deconstruct/reconstruct)
            _ => self,
        }
    }
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
            Value::Integer(i) => write!(f, "{}", i),
            // RUST CONCEPT: BigRational displays as "numerator/denominator"
            Value::Rational(r) => write!(f, "{}", r), // Shows as fraction like "3/4"
            // RUST CONCEPT: GaussianInt displays as "a+bi" with integer parts
            Value::GaussianInt(re, im) => {
                use num_traits::Zero;

                // Special case: 0+1i displays as just "i"
                if re.is_zero() && im == &BigInt::from(1) {
                    write!(f, "i")
                }
                // Special case: 0-1i displays as "-i"
                else if re.is_zero() && im == &BigInt::from(-1) {
                    write!(f, "-i")
                }
                // Special case: 0+ni displays as "ni" (pure imaginary)
                else if re.is_zero() {
                    if im >= &BigInt::from(0) {
                        write!(f, "{}i", im)
                    } else {
                        write!(f, "{}i", im)
                    }
                }
                // Special case: a+0i displays as just "a" (pure real)
                else if im.is_zero() {
                    write!(f, "{}", re)
                }
                // General case: a+bi
                else if im >= &BigInt::from(0) {
                    write!(f, "{}+{}i", re, im)
                } else {
                    write!(f, "{}{}i", re, im)
                }
            }
            // RUST CONCEPT: Complex64 displays as "a+bi" format with floating point
            Value::Complex(c) => {
                // Custom formatting for complex numbers
                if c.im >= 0.0 {
                    write!(f, "{}+{}i", c.re, c.im)
                } else {
                    write!(f, "{}{}i", c.re, c.im)
                }
            }
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
