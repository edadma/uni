use crate::compat::{Box, Rc, String, Vec, fmt};
use crate::tokenizer::SourcePos;

#[cfg(not(target_os = "none"))]
use std::cell::RefCell;
#[cfg(target_os = "none")]
use core::cell::RefCell;

use num_bigint::BigInt;
#[cfg(feature = "complex_numbers")]
use num_complex::Complex64;
use num_rational::BigRational;
use num_traits::{One, Zero};

// ASYNC CONCEPT: Async primitive function type
// Primitives can now be async, allowing them to await I/O operations
use core::future::Future;
use core::pin::Pin;

pub type AsyncPrimitiveFn = fn(&mut crate::interpreter::AsyncInterpreter)
    -> Pin<Box<dyn Future<Output = Result<(), RuntimeError>> + '_>>;

// RUST CONCEPT: Using the num ecosystem for arbitrary precision and special number types
// BigInt: Arbitrary precision integers (unlimited size)
// BigRational: Exact rational numbers (fractions)
// GaussianInt: Gaussian integers (a + bi where a, b are integers)
// Complex64: Complex numbers with f64 components

#[derive(Clone)]
pub enum Value {
    Number(f64),                    // Floating point number (default)
    Int32(i32),                     // 32-bit signed integer (embedded-friendly)
    Integer(BigInt),                // Arbitrary precision integer
    Rational(BigRational),          // Exact rational number (fraction)
    #[cfg(feature = "complex_numbers")]
    GaussianInt(BigInt, BigInt),    // Gaussian integer (real, imaginary) - both integers
    #[cfg(feature = "complex_numbers")]
    Complex(Complex64),             // Complex number (a + bi) - floating point components
    Atom(Rc<str>),                  // Interned atoms for efficiency
    QuotedAtom(Rc<str>),            // Quoted atoms - push without executing
    String(Rc<str>),                // Literal strings - ref counted but not interned
    Boolean(bool),                  // True/false boolean values
    Null,                           // Null/undefined value (distinct from Nil empty list)
    Pair(Rc<Value>, Rc<Value>),    // Cons cell for lists
    Array(Rc<RefCell<Vec<Value>>>), // Mutable array/vector
    Variable(Rc<RefCell<Value>>),   // Mutable variable (Forth-style)
    Nil,                            // Empty list marker

    // ASYNC CONCEPT: Async builtin primitive
    // This is the key change - builtins are now async functions
    AsyncBuiltin(AsyncPrimitiveFn),

    // RUST CONCEPT: Records (Scheme-style record types)
    // Records are named product types with labeled fields
    // type_name: The record type name (e.g., "person")
    // fields: The field values stored in a mutable vector
    // Uses Rc<RefCell<...>> for shared ownership with interior mutability
    Record {
        type_name: Rc<str>,
        fields: Rc<RefCell<Vec<Value>>>,
    },
    // RUST CONCEPT: Record type descriptors
    // Stores metadata about record types (field names, field count)
    // Used to validate and access record instances
    RecordType {
        type_name: Rc<str>,
        field_names: Rc<Vec<Rc<str>>>,
    },
    // RUST CONCEPT: I32 buffer for integer data and DSP
    // Stores 32-bit signed integers
    // Dynamic size Vec for flexibility - can grow/shrink as needed
    I32Buffer(Rc<RefCell<Vec<i32>>>),

    // RUST CONCEPT: F32 buffer for single-precision floating point data
    // Stores 32-bit floats (standard for audio/DSP, GPU compute)
    // More memory-efficient than f64 for large datasets
    F32Buffer(Rc<RefCell<Vec<f32>>>),
}

// Implement Debug manually since AsyncPrimitiveFn doesn't implement Debug
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::AsyncBuiltin(_) => write!(f, "AsyncBuiltin(<function>)"),
            Value::Number(n) => write!(f, "Number({})", n),
            Value::Int32(i) => write!(f, "Int32({})", i),
            Value::Integer(i) => write!(f, "Integer({})", i),
            Value::Rational(r) => write!(f, "Rational({})", r),
            #[cfg(feature = "complex_numbers")]
            Value::GaussianInt(re, im) => write!(f, "GaussianInt({}, {})", re, im),
            #[cfg(feature = "complex_numbers")]
            Value::Complex(c) => write!(f, "Complex({:?})", c),
            Value::Atom(a) => write!(f, "Atom({})", a),
            Value::QuotedAtom(a) => write!(f, "QuotedAtom({})", a),
            Value::String(s) => write!(f, "String({})", s),
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::Null => write!(f, "Null"),
            Value::Pair(car, cdr) => write!(f, "Pair({:?}, {:?})", car, cdr),
            Value::Array(arr) => write!(f, "Array({:?})", arr),
            Value::Variable(v) => write!(f, "Variable({:?})", v),
            Value::Nil => write!(f, "Nil"),
            Value::Record { type_name, fields } => write!(f, "Record({}:{:?})", type_name, fields),
            Value::RecordType { type_name, field_names } => write!(f, "RecordType({}:{:?})", type_name, field_names),
            Value::I32Buffer(buf) => write!(f, "I32Buffer({:?})", buf),
            Value::F32Buffer(buf) => write!(f, "F32Buffer({:?})", buf),
        }
    }
}

impl Value {
    // RUST CONCEPT: Get the type name of a value
    // Returns a string describing the type for display and debugging
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::Int32(_) => "int32",
            Value::Integer(_) => "integer",
            Value::Rational(_) => "rational",
            #[cfg(feature = "complex_numbers")]
            Value::GaussianInt(_, _) => "gaussian",
            #[cfg(feature = "complex_numbers")]
            Value::Complex(_) => "complex",
            Value::Atom(_) => "atom",
            Value::QuotedAtom(_) => "quoted-atom",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Null => "null",
            Value::Pair(_, _) => "list",
            Value::Array(_) => "vector",
            Value::Variable(_) => "variable",
            Value::Nil => "nil",
            Value::AsyncBuiltin(_) => "builtin",
            Value::Record { .. } => "record",
            Value::RecordType { .. } => "record-type",
            Value::I32Buffer(_) => "i32-buffer",
            Value::F32Buffer(_) => "f32-buffer",
        }
    }

    // RUST CONCEPT: Automatic numeric type demotion for cleaner results
    // This function attempts to demote numeric types to simpler representations:
    // - Rational with denominator 1 → Integer or Int32
    // - Rational with numerator 0 → Int32(0)
    // - GaussianInt with imaginary 0 → Integer or Int32
    // - Integer that fits in i32 → Int32
    // This keeps values in their simplest form after arithmetic operations
    pub fn demote(self) -> Self {
        use num_traits::ToPrimitive;
        match &self {
            // Check Rational: demote if denominator is 1 or numerator is 0
            Value::Rational(r) if r.numer().is_zero() => {
                // 0/n → Int32(0)
                Value::Int32(0)
            }
            Value::Rational(r) if r.denom().is_one() => {
                // n/1 → Integer or Int32
                // Extract the inner BigRational and clone its numerator
                if let Value::Rational(r) = self {
                    let big_int = r.numer().clone();
                    // Try to fit in i32 for embedded systems
                    if let Some(i32_val) = big_int.to_i32() {
                        Value::Int32(i32_val)
                    } else {
                        Value::Integer(big_int)
                    }
                } else {
                    unreachable!()
                }
            }
            // Check GaussianInt: demote if imaginary part is 0
            #[cfg(feature = "complex_numbers")]
            Value::GaussianInt(_re, im) if im.is_zero() => {
                // a+0i → Integer or Int32 (move real part out)
                if let Value::GaussianInt(re, _im) = self {
                    // Try to fit in i32
                    if let Some(i32_val) = re.to_i32() {
                        Value::Int32(i32_val)
                    } else {
                        Value::Integer(re)
                    }
                } else {
                    unreachable!()
                }
            }
            // Check Integer: demote to Int32 if it fits
            Value::Integer(i) => {
                if let Some(i32_val) = i.to_i32() {
                    Value::Int32(i32_val)
                } else {
                    self
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
    QuitRequested, // Special error to signal clean exit from REPL/script
}

// RUST CONCEPT: Implementing traits for custom error types
// The Display trait allows us to convert errors to strings
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            RuntimeError::QuitRequested => write!(f, "Quit requested"),
        }
    }
}

// RUST CONCEPT: Implementing Display for Value types
// This is the "data display" mode - strings WITH quotes for data structures
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Int32(i) => write!(f, "{}", i),
            Value::Integer(i) => write!(f, "{}", i),
            // RUST CONCEPT: BigRational displays as "numerator/denominator"
            Value::Rational(r) => write!(f, "{}", r), // Shows as fraction like "3/4"
            // RUST CONCEPT: GaussianInt displays as "a+bi" with integer parts
            #[cfg(feature = "complex_numbers")]
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
                    write!(f, "{}i", im)
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
            #[cfg(feature = "complex_numbers")]
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
                            write!(f, " | {}", other)?;
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
            Value::Variable(cell) => {
                write!(f, "<variable:{}>", cell.borrow())
            }
            Value::Nil => write!(f, "[]"),
            Value::AsyncBuiltin(_) => write!(f, "<builtin>"),
            // RUST CONCEPT: Display for record instances
            // Shows the type name and field values
            Value::Record { type_name, fields } => {
                let fields_ref = fields.borrow();
                write!(f, "#<record:{}", type_name)?;
                for field in fields_ref.iter() {
                    write!(f, " {}", field)?;
                }
                write!(f, ">")
            }
            // RUST CONCEPT: Display for record type descriptors
            // Shows the type name and field names
            Value::RecordType {
                type_name,
                field_names,
            } => {
                write!(f, "#<record-type:{}", type_name)?;
                for field_name in field_names.iter() {
                    write!(f, " {}", field_name)?;
                }
                write!(f, ">")
            }
            // RUST CONCEPT: Display for i32 buffers
            // Shows buffer length and first few samples for debugging
            Value::I32Buffer(buffer) => {
                let buffer_ref = buffer.borrow();
                let len = buffer_ref.len();
                write!(f, "#<i32-buffer:{}:[", len)?;

                // Show first 8 samples (or fewer if buffer is smaller)
                let preview_count = len.min(8);
                for i in 0..preview_count {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", buffer_ref[i])?;
                }

                if len > preview_count {
                    write!(f, " ...")?;
                }
                write!(f, "]>")
            }
            // RUST CONCEPT: Display for f32 buffers
            // Shows buffer length and first few samples for debugging
            Value::F32Buffer(buffer) => {
                let buffer_ref = buffer.borrow();
                let len = buffer_ref.len();
                write!(f, "#<f32-buffer:{}:[", len)?;

                // Show first 8 samples (or fewer if buffer is smaller)
                let preview_count = len.min(8);
                for i in 0..preview_count {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", buffer_ref[i])?;
                }

                if len > preview_count {
                    write!(f, " ...")?;
                }
                write!(f, "]>")
            }
        }
    }
}
