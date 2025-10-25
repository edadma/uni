// This module converts tokens (from the tokenizer) into Values (our AST/data structures)
//
// UNI EXECUTION MODEL:
// - Numbers (42, 3.14): Push themselves onto the stack
// - Strings ("hello"): Push themselves onto the stack
// - Lists ([1 2 +]): Push themselves onto the stack as data (code-as-data)
// - Atoms (hello, +): Execute by looking up in dictionary
// - Quoted Atoms ('hello): Push the atom onto the stack without executing
// - To execute a list: [1 2 +] exec  (push list, then exec executes it)
//
// RUST LEARNING NOTES:
// - We use 'use' statements to import types from other modules
// - The 'std::rc::Rc' is Rust's reference-counted smart pointer for shared ownership
// - 'Result<T, E>' is Rust's way of handling errors without exceptions
// - Pattern matching with 'match' is Rust's equivalent to switch statements but much more powerful

use crate::compat::{Rc, String, Vec, format, ToString};
use crate::interpreter::AsyncInterpreter;
use crate::tokenizer::{Token, TokenKind, tokenize};
use crate::value::{RuntimeError, Value};
use num_bigint::BigInt;
#[cfg(feature = "complex_numbers")]
use num_complex::Complex64;
use num_rational::BigRational;

// RUST CONCEPT: Error types
// We create our own error type for parser-specific errors
// #[derive(Debug)] automatically implements Debug trait so we can print errors
// This is separate from RuntimeError because parsing happens before execution
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String), // Got a token we didn't expect
    UnexpectedEndOfInput,    // Ran out of tokens when we needed more
    MismatchedBrackets,      // [ without matching ] or vice versa
    InvalidPipeNotation,      // Malformed [a | b] pair syntax
    InvalidNumber(String),   // Number literal that failed to parse
}

// RUST CONCEPT: Display trait implementation for better error messages
// This ensures the String field in UnexpectedToken is actually used
impl crate::compat::fmt::Display for ParseError {
    fn fmt(&self, f: &mut crate::compat::fmt::Formatter<'_>) -> crate::compat::fmt::Result {
        match self {
            ParseError::UnexpectedToken(msg) => write!(f, "{}", msg),
            ParseError::UnexpectedEndOfInput => write!(f, "Unexpected end of input"),
            ParseError::MismatchedBrackets => write!(f, "Mismatched brackets"),
            ParseError::InvalidPipeNotation => write!(f, "Invalid pipe notation"),
            ParseError::InvalidNumber(msg) => write!(f, "{}", msg),
        }
    }
}

// RUST CONCEPT: From trait
// This lets us convert ParseError into RuntimeError when needed
// The ? operator will automatically call this conversion
impl From<ParseError> for RuntimeError {
    fn from(err: ParseError) -> Self {
        RuntimeError::TypeError(format!("Parse error: {:?}", err))
    }
}

// RUST CONCEPT: Public functions
// 'pub fn' makes this function available to other modules
// This is our main entry point - takes a string, returns parsed Values
pub fn parse(input: &str, interp: &mut AsyncInterpreter) -> Result<Vec<Value>, ParseError> {
    // RUST CONCEPT: Error propagation
    // The ? operator here means "if tokenize fails, return that error immediately"
    // Otherwise, unwrap the Ok value and continue
    let tokens = tokenize(input)
        .map_err(|e| ParseError::UnexpectedToken(format!("Tokenize error: {}", e)))?;

    // RUST CONCEPT: Mutable variables
    // We need 'mut' because we'll be modifying the index as we parse
    // Rust variables are immutable by default - you must explicitly opt into mutation
    let mut index = 0;

    // RUST CONCEPT: Vec (growable array)
    // Vec::new() creates an empty vector that can grow as we add elements
    // This will hold all the top-level values we parse
    let mut results = Vec::new();

    // RUST CONCEPT: while loops and slice indexing
    // tokens.len() gives us the number of tokens
    // We continue until we've processed all tokens
    while index < tokens.len() {
        // RUST CONCEPT: Mutable references
        // We pass &mut index so parse_value can modify our index variable
        // This is how the parser keeps track of where it is in the token stream
        let value = parse_value(&tokens, &mut index, interp)?;
        results.push(value);
    }

    // RUST CONCEPT: Return values
    // Rust functions return the last expression if it doesn't end with semicolon
    // Ok(results) wraps our successful result in the Result type
    Ok(results)
}

// RUST CONCEPT: Function parameters and lifetimes
// &[Token] is a slice - a view into part of a vector
// &mut usize means we can modify the index parameter
// The lifetime is implicit here - Rust infers that the returned Value
// can't outlive the tokens slice (which is fine since we're cloning data)
fn parse_value(
    tokens: &[Token],
    index: &mut usize,
    interp: &mut AsyncInterpreter,
) -> Result<Value, ParseError> {
    // RUST CONCEPT: Bounds checking
    // .get() returns Option<T> - Some(value) if index exists, None if out of bounds
    // This is safer than direct indexing which would panic on out-of-bounds
    match tokens.get(*index) {
        // RUST CONCEPT: Pattern matching on references
        // We match on &Token because .get() returns Option<&Token>
        // The & in the pattern destructures the reference
        Some(token) if matches!(token.kind, TokenKind::Number(_)) => {
            if let TokenKind::Number(n) = token.kind {
                *index += 1; // RUST CONCEPT: Dereferencing - modify the value index points to
                Ok(Value::Number(n))
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::Integer(_)) => {
            if let TokenKind::Integer(s) = &token.kind {
                *index += 1;
                // RUST CONCEPT: Try i32 first for embedded-friendly performance
                // If it fits in i32 range, use Int32; otherwise fall back to BigInt
                if let Ok(i32_val) = s.parse::<i32>() {
                    Ok(Value::Int32(i32_val))
                } else {
                    match s.parse::<BigInt>() {
                        Ok(i) => Ok(Value::Integer(i)),
                        Err(_) => Err(ParseError::InvalidNumber(format!("Invalid integer: {}", s))),
                    }
                }
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::BigInt(_)) => {
            if let TokenKind::BigInt(s) = &token.kind {
                *index += 1;
                match s.parse::<BigInt>() {
                    Ok(i) => Ok(Value::Integer(i)),
                    Err(_) => Err(ParseError::InvalidNumber(format!("Invalid BigInt: {}", s))),
                }
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::Rational(_, _)) => {
            if let TokenKind::Rational(numer, denom) = &token.kind {
                *index += 1;
                match (numer.parse::<i64>(), denom.parse::<i64>()) {
                    (Ok(n), Ok(d)) if d != 0 => {
                        let rational = Value::Rational(BigRational::new(BigInt::from(n), BigInt::from(d)));
                        Ok(rational.demote())
                    }
                    _ => Err(ParseError::InvalidNumber(format!("Invalid rational: {}/{}", numer, denom))),
                }
            } else {
                unreachable!()
            }
        }

        #[cfg(feature = "complex_numbers")]
        Some(token) if matches!(token.kind, TokenKind::GaussianInt(_, _)) => {
            if let TokenKind::GaussianInt(re, im) = &token.kind {
                *index += 1;
                match (re.parse::<i64>(), im.parse::<i64>()) {
                    (Ok(r), Ok(i)) => Ok(Value::GaussianInt(BigInt::from(r), BigInt::from(i))),
                    _ => Err(ParseError::InvalidNumber(format!("Invalid Gaussian integer: {}+{}i", re, im))),
                }
            } else {
                unreachable!()
            }
        }

        #[cfg(feature = "complex_numbers")]
        Some(token) if matches!(token.kind, TokenKind::Complex(_, _)) => {
            if let TokenKind::Complex(re, im) = &token.kind {
                *index += 1;
                match (re.parse::<f64>(), im.parse::<f64>()) {
                    (Ok(r), Ok(i)) => Ok(Value::Complex(Complex64::new(r, i))),
                    _ => Err(ParseError::InvalidNumber(format!("Invalid complex: {}+{}i", re, im))),
                }
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::Atom(_)) => {
            if let TokenKind::Atom(atom_text) = &token.kind {
                *index += 1;

                // RUST CONCEPT: Atom interning
                // Atoms are symbols that get interned (deduplicated) for memory efficiency
                // The tokenizer has already identified all numeric literals, so anything
                // here is a true atom (identifier/symbol)
                let interned_atom = interp.intern_atom(atom_text);
                Ok(Value::Atom(interned_atom))
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::String(_)) => {
            if let TokenKind::String(string_text) = &token.kind {
                *index += 1;
                // RUST CONCEPT: Converting String to Rc<str>
                // .clone() gets an owned String, .into() converts to Rc<str>
                // Strings are NOT interned - each one gets its own Rc
                let string_rc: Rc<str> = string_text.clone().into();
                Ok(Value::String(string_rc))
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::Boolean(_)) => {
            if let TokenKind::Boolean(b) = token.kind {
                *index += 1;
                // RUST CONCEPT: Boolean literals
                // Boolean tokens directly create Boolean values
                Ok(Value::Boolean(b))
            } else {
                unreachable!()
            }
        }

        Some(token) if matches!(token.kind, TokenKind::Null) => {
            *index += 1;
            // RUST CONCEPT: Null literal
            // Null token creates a Null value
            Ok(Value::Null)
        }

        Some(token) if matches!(token.kind, TokenKind::LeftBracket) => {
            // RUST CONCEPT: Recursive parsing
            // Lists can contain other lists, so we call parse_list which may call parse_value again
            parse_list(tokens, index, interp)
        }

        Some(token) if matches!(token.kind, TokenKind::ArrayLeftBracket) => {
            parse_array(tokens, index, interp)
        }

        Some(token) if matches!(token.kind, TokenKind::Quote) => {
            *index += 1; // Skip the quote token

            // RUST CONCEPT: Validating syntax rules
            // In Uni, only atoms can be quoted. Lists, numbers, and strings
            // don't need quotes because they push themselves onto the stack by default
            match tokens.get(*index) {
                Some(token) if matches!(token.kind, TokenKind::Atom(_)) => {
                    if let TokenKind::Atom(atom_text) = &token.kind {
                        *index += 1; // Consume the atom token

                        // RUST CONCEPT: Creating quoted atoms directly
                        // Instead of (quote atom) structure, we create a QuotedAtom value
                        // This directly represents the semantics: push atom without executing
                        let interned_atom = interp.intern_atom(atom_text);
                        Ok(Value::QuotedAtom(interned_atom))
                    } else {
                        unreachable!()
                    }
                }
                Some(token) if matches!(token.kind, TokenKind::LeftBracket) => {
                    // RUST CONCEPT: Custom error types for syntax validation
                    // Lists don't need quotes - they're data by default
                    Err(ParseError::UnexpectedToken(
                        "Lists cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::String(_)) => {
                    // Strings don't need quotes - they push themselves onto the stack
                    Err(ParseError::UnexpectedToken(
                        "Strings cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::Number(_)) => {
                    // Numbers don't need quotes - they push themselves onto the stack
                    Err(ParseError::UnexpectedToken(
                        "Numbers cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::Integer(_)) => {
                    Err(ParseError::UnexpectedToken(
                        "Numbers cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::BigInt(_)) => {
                    Err(ParseError::UnexpectedToken(
                        "Numbers cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::Rational(_, _)) => {
                    Err(ParseError::UnexpectedToken(
                        "Numbers cannot be quoted - they are data by default".to_string(),
                    ))
                }
                #[cfg(feature = "complex_numbers")]
                Some(token) if matches!(token.kind, TokenKind::GaussianInt(_, _)) => {
                    Err(ParseError::UnexpectedToken(
                        "Numbers cannot be quoted - they are data by default".to_string(),
                    ))
                }
                #[cfg(feature = "complex_numbers")]
                Some(token) if matches!(token.kind, TokenKind::Complex(_, _)) => {
                    Err(ParseError::UnexpectedToken(
                        "Numbers cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::Boolean(_)) => {
                    // Booleans don't need quotes - they are data by default
                    Err(ParseError::UnexpectedToken(
                        "Booleans cannot be quoted - they are data by default".to_string(),
                    ))
                }
                Some(token) if matches!(token.kind, TokenKind::Null) => {
                    // Null doesn't need quotes - it is data by default
                    Err(ParseError::UnexpectedToken(
                        "Null cannot be quoted - it is data by default".to_string(),
                    ))
                }
                _ => {
                    // Quote without anything following, or followed by invalid token
                    Err(ParseError::UnexpectedEndOfInput)
                }
            }
        }

        Some(token) if matches!(token.kind, TokenKind::Pipe) => {
            // RUST CONCEPT: Error handling
            // A pipe by itself is invalid - it should only appear in [a | b] notation
            Err(ParseError::InvalidPipeNotation)
        }

        Some(token) if matches!(token.kind, TokenKind::RightBracket) => {
            // RUST CONCEPT: Error types
            // A closing bracket without a matching opening bracket is an error
            Err(ParseError::MismatchedBrackets)
        }

        // RUST CONCEPT: None pattern
        // This handles the case where we've run out of tokens
        None => Err(ParseError::UnexpectedEndOfInput),

        // Catch-all for any remaining token types
        Some(_) => Err(ParseError::UnexpectedToken("Unexpected token".to_string())),
    }
}

// RUST CONCEPT: Complex parsing logic
// This function handles the tricky case of parsing lists, which can be:
// - [1 2 3] (proper list)
// - [a . b] (cons pair)
// - [] (empty list)
fn parse_list(
    tokens: &[Token],
    index: &mut usize,
    interp: &mut AsyncInterpreter,
) -> Result<Value, ParseError> {
    // RUST CONCEPT: Assertions and debugging
    // debug_assert! is removed in release builds but helps catch bugs during development
    // This ensures we're starting at a LeftBracket token
    debug_assert!(
        matches!(tokens.get(*index), Some(token) if matches!(token.kind, TokenKind::LeftBracket))
    );

    *index += 1; // Skip the opening bracket

    // RUST CONCEPT: Vec for collecting elements
    // We'll collect list elements here, then convert to cons cells at the end
    let mut elements = Vec::new();

    // RUST CONCEPT: Loop with pattern matching
    // We loop until we find the closing bracket or run out of tokens
    loop {
        match tokens.get(*index) {
            Some(token) if matches!(token.kind, TokenKind::RightBracket) => {
                *index += 1; // Skip the closing bracket
                break; // RUST CONCEPT: break exits the loop
            }

            Some(token) if matches!(token.kind, TokenKind::Pipe) => {
                // RUST CONCEPT: Complex parsing - pipe notation [a | b]
                // We need at least one element before the pipe, and exactly one after
                if elements.is_empty() {
                    return Err(ParseError::InvalidPipeNotation);
                }

                *index += 1; // Skip the pipe

                // Parse the element after the pipe (the "tail" of the cons cell)
                let tail = parse_value(tokens, index, interp)?;

                // RUST CONCEPT: Expecting specific tokens
                // After [a | b], we MUST see a closing bracket
                match tokens.get(*index) {
                    Some(token) if matches!(token.kind, TokenKind::RightBracket) => {
                        *index += 1; // Skip the closing bracket

                        // RUST CONCEPT: Building cons cells manually
                        // [a b c | d] becomes Pair(a, Pair(b, Pair(c, d)))
                        // We build this right-to-left using fold
                        let cons_cell = elements
                            .into_iter()
                            .rev() // RUST CONCEPT: Iterator adaptors - reverse the order
                            .fold(tail, |acc, elem| {
                                // RUST CONCEPT: Closures (anonymous functions)
                                // |acc, elem| is the closure parameter list
                                // We build Pair(elem, acc) for each element
                                Value::Pair(Rc::new(elem), Rc::new(acc))
                            });

                        return Ok(cons_cell);
                    }
                    _ => return Err(ParseError::MismatchedBrackets),
                }
            }

            None => {
                // RUST CONCEPT: Error cases
                // We ran out of tokens while looking for the closing bracket
                return Err(ParseError::UnexpectedEndOfInput);
            }

            _ => {
                // RUST CONCEPT: Recursive parsing continues
                // Parse the next element and add it to our list
                let element = parse_value(tokens, index, interp)?;
                elements.push(element);
            }
        }
    }

    // RUST CONCEPT: Converting Vec to linked list
    // If we get here, we have a proper list [1 2 3] without pipe notation
    // Convert the Vec<Value> to a linked list of Pair nodes ending in Nil
    let list = elements
        .into_iter()
        .rev() // Reverse so we build right-to-left
        .fold(Value::Nil, |acc, elem| {
            // RUST CONCEPT: Fold (reduce) operation
            // This is like reduce in other languages
            // We start with Nil and wrap each element: Pair(elem, previous_result)
            Value::Pair(Rc::new(elem), Rc::new(acc))
        });

    Ok(list)
}

fn parse_array(
    tokens: &[Token],
    index: &mut usize,
    interp: &mut AsyncInterpreter,
) -> Result<Value, ParseError> {
    debug_assert!(
        matches!(tokens.get(*index), Some(token) if matches!(token.kind, TokenKind::ArrayLeftBracket))
    );

    *index += 1; // Skip the #[ token

    let mut elements = Vec::new();

    loop {
        match tokens.get(*index) {
            Some(token) if matches!(token.kind, TokenKind::RightBracket) => {
                *index += 1;
                break;
            }
            Some(token) if matches!(token.kind, TokenKind::Pipe) => {
                return Err(ParseError::UnexpectedToken(
                    "Arrays do not support pipe notation".to_string(),
                ));
            }
            None => {
                return Err(ParseError::UnexpectedEndOfInput);
            }
            _ => {
                let element = parse_value(tokens, index, interp)?;
                elements.push(element);
            }
        }
    }

    Ok(interp.make_array(elements))
}

// RUST CONCEPT: Conditional compilation and testing
// #[cfg(test)] means this code only compiles when running tests
// This keeps test code out of the release binary
#[cfg(test)]
mod tests {
    use super::*; // RUST CONCEPT: Import everything from parent module

    // Test-only helper method for ParseError
    impl ParseError {
        fn message(&self) -> String {
            match self {
                ParseError::UnexpectedToken(msg) => msg.clone(),
                ParseError::UnexpectedEndOfInput => "Unexpected end of input".to_string(),
                ParseError::MismatchedBrackets => "Mismatched brackets".to_string(),
                ParseError::InvalidPipeNotation => "Invalid pipe notation".to_string(),
                ParseError::InvalidNumber(msg) => msg.clone(),
            }
        }
    }
    use super::ParseError; // RUST CONCEPT: Explicit import to help compiler see usage

    // RUST CONCEPT: Test functions
    // #[test] tells Rust this function is a unit test
    #[test]
    fn test_parse_numbers() {
        // RUST CONCEPT: Creating test data
        // We need a mutable interpreter for atom interning
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Unwrap for tests
        // .unwrap() panics if Result is Err - fine for tests, bad for production
        let result = parse("42", &mut interp).unwrap();

        // RUST CONCEPT: Assertions
        // assert_eq! compares two values and panics if they're different
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Pattern matching in tests
        // We destructure the result to check it's the right type
        // Small integers now use Int32
        assert!(matches!(&result[0], Value::Int32(42)));

        // Test multiple numbers - mix of integers and floats
        let result = parse("1 2.5 -3", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        // Check mixed types
        assert!(matches!(&result[0], Value::Int32(1)));
        assert!(matches!(&result[1], Value::Number(n) if *n == 2.5));
        assert!(matches!(&result[2], Value::Int32(-3)));
    }

    #[test]
    fn test_parse_atoms() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("hello world +", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        // RUST CONCEPT: String literals and references
        // We check that atoms are properly interned
        let expected_atoms = ["hello", "world", "+"];
        for (i, expected) in expected_atoms.iter().enumerate() {
            match &result[i] {
                Value::Atom(atom) => {
                    // RUST CONCEPT: Dereferencing Rc
                    // &**atom dereferences the Rc then takes a reference to the str
                    assert_eq!(&**atom, *expected);
                }
                _ => panic!("Expected atom at position {}", i),
            }
        }

        // RUST CONCEPT: Testing interning behavior
        // Same atoms should share the same Rc (pointer equality)
        let result2 = parse("hello", &mut interp).unwrap();
        if let (Value::Atom(atom1), Value::Atom(atom2)) = (&result[0], &result2[0]) {
            assert!(Rc::ptr_eq(atom1, atom2)); // Same pointer = successful interning
        }
    }

    #[test]
    fn test_parse_strings() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("\"hello world\" \"\"", &mut interp).unwrap();
        assert_eq!(result.len(), 2);

        match &result[0] {
            Value::String(s) => assert_eq!(&**s, "hello world"),
            _ => panic!("Expected string"),
        }

        match &result[1] {
            Value::String(s) => assert_eq!(&**s, ""), // Empty string
            _ => panic!("Expected empty string"),
        }

        // RUST CONCEPT: Testing non-interning
        // Strings with same content should NOT share pointers
        let result2 = parse("\"hello world\"", &mut interp).unwrap();
        if let (Value::String(s1), Value::String(s2)) = (&result[0], &result2[0]) {
            assert_eq!(s1, s2); // Same content
            assert!(!Rc::ptr_eq(s1, s2)); // Different pointers = not interned
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("[]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Matching specific enum variants
        match &result[0] {
            Value::Nil => (), // () is the unit value - like void but it's a real value
            _ => panic!("Expected empty list (Nil)"),
        }
    }

    #[test]
    fn test_parse_simple_list() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("[1 2 3]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Nested pattern matching
        // We can destructure complex nested structures in one match
        match &result[0] {
            Value::Pair(car1, cdr1) => {
                // First element should be 1
                assert!(matches!(**car1, Value::Int32(1)));

                match cdr1.as_ref() {
                    // RUST CONCEPT: as_ref() converts &Rc<T> to &T
                    Value::Pair(car2, cdr2) => {
                        // Second element should be 2
                        assert!(matches!(**car2, Value::Int32(2)));

                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                // Third element should be 3
                                assert!(matches!(**car3, Value::Int32(3)));
                                // End should be Nil
                                assert!(matches!(**cdr3, Value::Nil));
                            }
                            _ => panic!("Expected third pair"),
                        }
                    }
                    _ => panic!("Expected second pair"),
                }
            }
            _ => panic!("Expected list pair"),
        }
    }

    #[test]
    fn test_parse_array_literal() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("#[1 2 3]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            Value::Array(array_rc) => {
                let array = array_rc.borrow();
                assert_eq!(array.len(), 3);
                assert!(matches!(array[0], Value::Int32(1)));
                assert!(matches!(array[1], Value::Int32(2)));
                assert!(matches!(array[2], Value::Int32(3)));
            }
            _ => panic!("Expected array value"),
        }
    }

    #[test]
    fn test_parse_array_disallows_pipe() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("#[1 | 2]", &mut interp);
        assert!(matches!(
            result,
            Err(ParseError::UnexpectedToken(msg)) if msg.contains("Arrays do not support pipe notation")
        ));
    }

    #[test]
    fn test_parse_pipe_notation() {
        let mut interp = AsyncInterpreter::new();

        // Test [1 | 2] - a simple cons cell
        let result = parse("[1 | 2]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            Value::Pair(car, cdr) => {
                assert!(matches!(**car, Value::Int32(1)));
                assert!(matches!(**cdr, Value::Int32(2)));
            }
            _ => panic!("Expected pair"),
        }

        // Test [1 2 | 3] - multiple elements before pipe
        let result = parse("[1 2 | 3]", &mut interp).unwrap();
        match &result[0] {
            Value::Pair(car1, cdr1) => {
                assert!(matches!(**car1, Value::Int32(1)));
                match cdr1.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(**car2, Value::Int32(2)));
                        assert!(matches!(**cdr2, Value::Int32(3)));
                    }
                    _ => panic!("Expected nested pair"),
                }
            }
            _ => panic!("Expected pair"),
        }
    }

    #[test]
    fn test_parse_quoted_values() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("'hello", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Testing QuotedAtom parsing
        // A quoted atom becomes QuotedAtom directly
        match &result[0] {
            Value::QuotedAtom(atom) => {
                assert_eq!(&**atom, "hello");
            }
            _ => panic!("Expected QuotedAtom"),
        }
    }

    #[test]
    fn test_parse_nested_lists() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("[[1 2] [3]]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Testing complex data structures
        // This tests that we can parse nested lists correctly
        match &result[0] {
            Value::Pair(first_list, cdr) => {
                // First list should be [1 2]
                match first_list.as_ref() {
                    Value::Pair(one, rest1) => {
                        assert!(matches!(**one, Value::Int32(1)));
                        match rest1.as_ref() {
                            Value::Pair(two, rest2) => {
                                assert!(matches!(**two, Value::Int32(2)));
                                assert!(matches!(**rest2, Value::Nil));
                            }
                            _ => panic!("Expected [1 2] structure"),
                        }
                    }
                    _ => panic!("Expected first list"),
                }

                // Rest should be ([3])
                match cdr.as_ref() {
                    Value::Pair(second_list, final_cdr) => {
                        // Second list should be [3]
                        match second_list.as_ref() {
                            Value::Pair(three, rest3) => {
                                assert!(matches!(**three, Value::Int32(3)));
                                assert!(matches!(**rest3, Value::Nil));
                            }
                            _ => panic!("Expected [3] structure"),
                        }
                        assert!(matches!(**final_cdr, Value::Nil));
                    }
                    _ => panic!("Expected second list pair"),
                }
            }
            _ => panic!("Expected outer list"),
        }
    }

    #[test]
    fn test_parse_errors() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing error cases
        // We use assert!(result.is_err()) to verify we get errors for bad input

        // Mismatched brackets
        assert!(parse("[1 2", &mut interp).is_err());
        assert!(parse("1 2]", &mut interp).is_err());

        // Invalid pipe notation
        assert!(parse("[|]", &mut interp).is_err());
        assert!(parse("[1 | 2 3]", &mut interp).is_err()); // Too many elements after pipe

        // Standalone pipe
        assert!(parse("1 | 2", &mut interp).is_err());
    }

    #[test]
    fn test_parse_with_comments() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing integration
        // Comments should be stripped by tokenizer, so parser never sees them
        let result = parse("42 \\ this is a comment\n37", &mut interp).unwrap();
        assert_eq!(result.len(), 2);

        match (&result[0], &result[1]) {
            (Value::Int32(42), Value::Int32(37)) => {
                // Values match expected
            }
            _ => panic!("Expected two Int32 values: 42 and 37"),
        }
    }

    #[test]
    fn test_parse_quoted_atoms_only() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing valid quote syntax - only atoms can be quoted
        let result = parse("'hello", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // Should be QuotedAtom("hello")
        match &result[0] {
            Value::QuotedAtom(atom) => {
                assert_eq!(&**atom, "hello");
            }
            _ => panic!("Expected QuotedAtom"),
        }

        // Test multiple quoted atoms
        let result = parse("'+ '- '*", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        for (i, op) in ["+", "-", "*"].iter().enumerate() {
            match &result[i] {
                Value::QuotedAtom(atom) => {
                    assert_eq!(&**atom, *op);
                }
                _ => panic!("Expected QuotedAtom at {}", i),
            }
        }
    }

    #[test]
    fn test_parse_reject_quoted_non_atoms() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing syntax restrictions
        // These should all be rejected because only atoms can be quoted

        // Quoted lists should be rejected
        assert!(parse("'[1 2 3]", &mut interp).is_err());
        assert!(parse("'[]", &mut interp).is_err());
        assert!(parse("'[[nested]]", &mut interp).is_err());

        // Quoted strings should be rejected
        assert!(parse("'\"hello\"", &mut interp).is_err());
        assert!(parse("'\"\"", &mut interp).is_err());

        // Quoted float numbers should be rejected (integers become atoms and CAN be quoted)
        assert!(parse("'3.14", &mut interp).is_err());
        assert!(parse("'-17.5", &mut interp).is_err());
        assert!(parse("'1e5", &mut interp).is_err());

        // Verify the error messages are helpful
        let result = parse("'[1 2]", &mut interp);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Lists cannot be quoted"));

        let result = parse("'\"string\"", &mut interp);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Strings cannot be quoted"));

        let result = parse("'3.14", &mut interp);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Numbers cannot be quoted"));
    }

    #[test]
    fn test_parse_mixed_types_in_list() {
        let mut interp = AsyncInterpreter::new();

        let result = parse("[42 hello \"world\" [nested]]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Testing heterogeneous data structures
        // Uni lists can contain any mix of types
        match &result[0] {
            Value::Pair(first, rest1) => {
                assert!(matches!(first.as_ref(), Value::Int32(42)));

                match rest1.as_ref() {
                    Value::Pair(second, rest2) => {
                        assert!(matches!(second.as_ref(), Value::Atom(a) if &**a == "hello"));

                        match rest2.as_ref() {
                            Value::Pair(third, rest3) => {
                                assert!(
                                    matches!(third.as_ref(), Value::String(s) if &**s == "world")
                                );

                                match rest3.as_ref() {
                                    Value::Pair(fourth, rest4) => {
                                        // Fourth should be a nested list
                                        assert!(matches!(fourth.as_ref(), Value::Pair(_, _)));
                                        assert!(matches!(rest4.as_ref(), Value::Nil));
                                    }
                                    _ => panic!("Expected fourth element"),
                                }
                            }
                            _ => panic!("Expected third element"),
                        }
                    }
                    _ => panic!("Expected second element"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing tokenizer integration
        // Various whitespace should be handled correctly
        let inputs = [
            "  1   2   3  ", // Extra spaces
            "1\n2\t3\r\n4",  // Mixed whitespace
            "[  1   2  ]",   // Spaces in lists
            "[ 1 | 2 ]",     // Spaces around pipe
            "'   hello   ",  // Spaces after quote
        ];

        let expected_lengths = [3, 4, 1, 1, 1];

        for (input, expected_len) in inputs.iter().zip(expected_lengths.iter()) {
            let result = parse(input, &mut interp).unwrap();
            assert_eq!(result.len(), *expected_len, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_deeply_nested_lists() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing recursive parsing limits
        // Deep nesting should work (until stack overflow)
        let result = parse("[[[[1]]]]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // Navigate down the nested structure
        let mut current = &result[0];
        for _level in 0..4 {
            match current {
                Value::Pair(car, cdr) => {
                    current = car.as_ref();
                    assert!(matches!(cdr.as_ref(), Value::Nil));
                }
                Value::Number(n) if *n == 1.0 => break,
                _ => panic!("Expected nested structure"),
            }
        }
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_pipe_cases() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing edge cases thoroughly

        // Multiple pipes should fail
        assert!(parse("[1 | 2 | 3]", &mut interp).is_err());

        // Pipe at beginning should fail
        assert!(parse("[| 1]", &mut interp).is_err());

        // Multiple elements after pipe should fail
        assert!(parse("[1 | 2 3]", &mut interp).is_err());

        // Pipe in nested list should work
        let result = parse("[[1 | 2]]", &mut interp).unwrap();
        match &result[0] {
            Value::Pair(inner_list, outer_cdr) => {
                match inner_list.as_ref() {
                    Value::Pair(one, two) => {
                        assert!(matches!(one.as_ref(), Value::Int32(1)));
                        assert!(matches!(two.as_ref(), Value::Int32(2)));
                    }
                    _ => panic!("Expected inner pair"),
                }
                assert!(matches!(outer_cdr.as_ref(), Value::Nil));
            }
            _ => panic!("Expected outer list"),
        }
    }

    #[test]
    fn test_parse_scientific_notation() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing tokenizer/parser integration
        // Scientific notation should parse as numbers
        let result = parse("1e5 2.5e-3 1E10", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        let expected = [1e5, 2.5e-3, 1E10];
        for (i, expected_val) in expected.iter().enumerate() {
            match &result[i] {
                Value::Number(n) => assert_eq!(*n, *expected_val),
                _ => panic!("Expected number at position {}", i),
            }
        }
    }

    #[test]
    fn test_parse_empty_input() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing boundary conditions
        let result = parse("", &mut interp).unwrap();
        assert_eq!(result.len(), 0);

        let result = parse("   ", &mut interp).unwrap(); // Just whitespace
        assert_eq!(result.len(), 0);

        let result = parse("\\ just a comment", &mut interp).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_quote_edge_cases() {
        let mut interp = AsyncInterpreter::new();

        // Quote without following value should fail
        assert!(parse("'", &mut interp).is_err());

        // RUST CONCEPT: Testing syntax consistency
        // Nested quotes like ''hello don't make sense in Uni's execution model
        // Quote can only be followed by atoms, not other quote tokens
        assert!(parse("''hello", &mut interp).is_err());

        // Quote followed by other non-atom tokens should also fail
        assert!(parse("'|", &mut interp).is_err());
        assert!(parse("']", &mut interp).is_err());

        // But quotes followed by atoms should work perfectly
        let result = parse("'quote-me", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            Value::QuotedAtom(atom) => {
                assert_eq!(&**atom, "quote-me");
            }
            _ => panic!("Expected QuotedAtom"),
        }
    }

    #[test]
    fn test_parse_error_messages() {
        let mut interp = AsyncInterpreter::new();

        // RUST CONCEPT: Testing error types and messages
        let error_cases = [
            ("[1 2", "UnexpectedEndOfInput"),
            ("1 2]", "MismatchedBrackets"),
            ("[|]", "InvalidPipeNotation"),
            ("1 | 2", "InvalidPipeNotation"),
            ("'[1 2]", "Lists cannot be quoted"),
            ("'\"hello\"", "Strings cannot be quoted"),
            ("'42.0", "Numbers cannot be quoted"),  // Floats cannot be quoted
            ("'", "UnexpectedEndOfInput"), // Quote with nothing following
        ];

        for (input, expected_error) in error_cases.iter() {
            let result = parse(input, &mut interp);
            assert!(result.is_err(), "Expected error for input: '{}'", input);

            let error_string = format!("{:?}", result.unwrap_err());
            // Just check that we get the right error type
            assert!(
                error_string.contains(expected_error),
                "Expected '{}' in error message for '{}', got: {}",
                expected_error,
                input,
                error_string
            );
        }
    }

    #[test]
    fn test_parse_booleans() {
        let mut interp = AsyncInterpreter::new();

        // Test parsing true
        let result = parse("true", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Boolean(true)));

        // Test parsing false
        let result = parse("false", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Boolean(false)));

        // Test mixed with other values
        let result = parse("true 42 \"hello\" false", &mut interp).unwrap();
        assert_eq!(result.len(), 4);
        assert!(matches!(result[0], Value::Boolean(true)));
        assert!(matches!(result[1], Value::Int32(42))); // Small integers use Int32
        assert!(matches!(result[2], Value::String(_)));
        assert!(matches!(result[3], Value::Boolean(false)));
    }

    #[test]
    fn test_parse_null() {
        let mut interp = AsyncInterpreter::new();

        // Test parsing null
        let result = parse("null", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Null));

        // Test mixed with other values
        let result = parse("null 42 true \"test\"", &mut interp).unwrap();
        assert_eq!(result.len(), 4);
        assert!(matches!(result[0], Value::Null));
        assert!(matches!(result[1], Value::Int32(42)));
        assert!(matches!(result[2], Value::Boolean(true)));
        assert!(matches!(result[3], Value::String(_)));
    }

    #[test]
    fn test_parse_boolean_null_in_lists() {
        let mut interp = AsyncInterpreter::new();

        // Test booleans in list
        let result = parse("[true false null 42]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            Value::Pair(car1, cdr1) => {
                assert!(matches!(car1.as_ref(), Value::Boolean(true)));
                match cdr1.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(car2.as_ref(), Value::Boolean(false)));
                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                assert!(matches!(car3.as_ref(), Value::Null));
                                match cdr3.as_ref() {
                                    Value::Pair(car4, cdr4) => {
                                        assert!(matches!(car4.as_ref(), Value::Int32(42)));
                                        assert!(matches!(cdr4.as_ref(), Value::Nil));
                                    }
                                    _ => panic!("Expected fourth element"),
                                }
                            }
                            _ => panic!("Expected third element"),
                        }
                    }
                    _ => panic!("Expected second element"),
                }
            }
            _ => panic!("Expected list structure"),
        }
    }

    #[test]
    fn test_parse_quoted_booleans_null_error() {
        let mut interp = AsyncInterpreter::new();

        // Quoted booleans should be errors - booleans are data by default
        let error_cases = vec![
            ("'true", "cannot be quoted"),
            ("'false", "cannot be quoted"),
            ("'null", "cannot be quoted"),
        ];

        for (input, expected_error) in error_cases.iter() {
            let result = parse(input, &mut interp);
            assert!(result.is_err(), "Expected error for input: '{}'", input);

            let error_string = format!("{:?}", result.unwrap_err());
            assert!(
                error_string
                    .to_lowercase()
                    .contains(&expected_error.to_lowercase()),
                "Expected '{}' in error message for '{}', got: {}",
                expected_error,
                input,
                error_string
            );
        }
    }

    #[test]
    fn test_boolean_null_not_atoms() {
        let mut interp = AsyncInterpreter::new();

        // Explicitly verify that true/false/null are NOT parsed as atoms
        let result = parse("true false null", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        // These should NOT be atoms - they should be their proper types
        assert!(
            matches!(result[0], Value::Boolean(true)),
            "Expected Boolean(true), got: {:?}",
            result[0]
        );
        assert!(
            !matches!(result[0], Value::Atom(_)),
            "true should NOT be parsed as an atom"
        );

        assert!(
            matches!(result[1], Value::Boolean(false)),
            "Expected Boolean(false), got: {:?}",
            result[1]
        );
        assert!(
            !matches!(result[1], Value::Atom(_)),
            "false should NOT be parsed as an atom"
        );

        assert!(
            matches!(result[2], Value::Null),
            "Expected Null, got: {:?}",
            result[2]
        );
        assert!(
            !matches!(result[2], Value::Atom(_)),
            "null should NOT be parsed as an atom"
        );

        // Verify similar-looking strings ARE still atoms
        let result = parse("TRUE True false-flag NULL nil", &mut interp).unwrap();
        assert_eq!(result.len(), 5);

        for (i, val) in result.iter().enumerate() {
            assert!(
                matches!(val, Value::Atom(_)),
                "Element {} should be an atom, got: {:?}",
                i,
                val
            );
        }
    }

    #[test]
    fn test_parse_error_message_method() {
        let mut interp = AsyncInterpreter::new();

        // Test custom error message for UnexpectedToken
        let error = ParseError::UnexpectedToken("Custom error message".to_string());
        assert_eq!(error.message(), "Custom error message");

        // Test other error types have sensible messages
        assert_eq!(
            ParseError::UnexpectedEndOfInput.message(),
            "Unexpected end of input"
        );
        assert_eq!(
            ParseError::MismatchedBrackets.message(),
            "Mismatched brackets"
        );
        assert_eq!(
            ParseError::InvalidPipeNotation.message(),
            "Invalid pipe notation"
        );

        // Test that error messages are accessible from parse results
        let result = parse("'", &mut interp);
        assert!(result.is_err());
        if let Err(error) = result {
            let message = error.message();
            assert!(!message.is_empty());
        }
    }

    // ========== TESTS FOR NEW NUMBER TYPES ==========

    #[test]
    fn test_parse_bigint_literals() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // Test simple BigInt
        let result = parse("123n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Integer(ref i) if *i == BigInt::from(123)));

        // Test negative BigInt
        let result = parse("-456n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Integer(ref i) if *i == BigInt::from(-456)));

        // Test large BigInt
        let result = parse("123456789012345678901234567890n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        let expected = BigInt::parse_bytes(b"123456789012345678901234567890", 10).unwrap();
        assert!(matches!(result[0], Value::Integer(ref i) if *i == expected));

        // Test zero BigInt
        let result = parse("0n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Integer(ref i) if *i == BigInt::from(0)));
    }

    #[test]
    fn test_parse_rational_literals() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = AsyncInterpreter::new();

        // Test simple fraction
        let result = parse("3/4", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        let expected = BigRational::new(BigInt::from(3), BigInt::from(4));
        assert!(matches!(result[0], Value::Rational(ref r) if *r == expected));

        // Test 1/2
        let result = parse("1/2", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        let expected = BigRational::new(BigInt::from(1), BigInt::from(2));
        assert!(matches!(result[0], Value::Rational(ref r) if *r == expected));

        // Test negative numerator
        let result = parse("-5/8", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        let expected = BigRational::new(BigInt::from(-5), BigInt::from(8));
        assert!(matches!(result[0], Value::Rational(ref r) if *r == expected));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_gaussian_and_complex_literals() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = AsyncInterpreter::new();

        // Test standard complex number (integers -> GaussianInt)
        let result = parse("3+4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(3) && im == &BigInt::from(4)));

        // Test negative imaginary (integers -> GaussianInt)
        let result = parse("5-2i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(5) && im == &BigInt::from(-2)));

        // Test pure imaginary (integer -> GaussianInt)
        let result = parse("5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(5)));

        // Test with negative real part (integers -> GaussianInt)
        let result = parse("-3+4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(-3) && im == &BigInt::from(4)));

        // Test with decimal parts (decimals -> Complex64)
        let result = parse("1.5+2.5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(1.5, 2.5)));
    }

    #[test]
    fn test_parse_mixed_number_types() {
        use num_bigint::BigInt;
        use num_rational::BigRational;
        let mut interp = AsyncInterpreter::new();

        // Test parsing multiple different number types
        #[cfg(feature = "complex_numbers")]
        {
            let result = parse("42 123n 3/4 2+3i", &mut interp).unwrap();
            assert_eq!(result.len(), 4);

            assert!(matches!(result[0], Value::Int32(42)));
            assert!(matches!(result[1], Value::Integer(ref i) if *i == BigInt::from(123)));
            assert!(matches!(result[2], Value::Rational(ref r) if *r == BigRational::new(BigInt::from(3), BigInt::from(4))));
            assert!(matches!(result[3], Value::GaussianInt(ref re, ref im)
                if re == &BigInt::from(2) && im == &BigInt::from(3)));
        }

        #[cfg(not(feature = "complex_numbers"))]
        {
            let result = parse("42 123n 3/4", &mut interp).unwrap();
            assert_eq!(result.len(), 3);

            assert!(matches!(result[0], Value::Int32(42)));
            assert!(matches!(result[1], Value::Integer(ref i) if *i == BigInt::from(123)));
            assert!(matches!(result[2], Value::Rational(ref r) if *r == BigRational::new(BigInt::from(3), BigInt::from(4))));
        }
    }

    #[test]
    fn test_parse_number_types_in_lists() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // Test BigInt in list
        let result = parse("[1 2n 3]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // Verify it's a list with proper elements
        match &result[0] {
            Value::Pair(_, _) => {
                // List created correctly
            }
            _ => panic!("Expected list"),
        }

        // Test GaussianInt in list (integers -> GaussianInt)
        #[cfg(feature = "complex_numbers")]
        {
            let result = parse("[1+2i 3+4i]", &mut interp).unwrap();
            assert_eq!(result.len(), 1);
            match &result[0] {
                Value::Pair(car, cdr) => {
                    assert!(matches!(**car, Value::GaussianInt(ref re, ref im)
                        if re == &BigInt::from(1) && im == &BigInt::from(2)));
                    match cdr.as_ref() {
                        Value::Pair(car2, _) => {
                            assert!(matches!(**car2, Value::GaussianInt(ref re, ref im)
                                if re == &BigInt::from(3) && im == &BigInt::from(4)));
                        }
                        _ => panic!("Expected second element"),
                    }
                }
                _ => panic!("Expected list"),
            }
        }
    }

    // ========== COMPLEX NUMBER PARSING EDGE CASES ==========

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_negative_imaginary() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // Test with negative imaginary part: 3-4i (integers -> GaussianInt)
        let result = parse("3-4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(3) && im == &BigInt::from(-4)));

        // Test with both negative: -3-4i (integers -> GaussianInt)
        let result = parse("-3-4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(-3) && im == &BigInt::from(-4)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_zero_parts() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // Test 0+5i (zero real part) - integers -> GaussianInt
        let result = parse("0+5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(5)));

        // Test 5+0i (zero imaginary part) - integers -> GaussianInt
        let result = parse("5+0i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(5) && im == &BigInt::from(0)));

        // Test 0+0i (both zero) - integers -> GaussianInt
        let result = parse("0+0i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_decimal_parts() {
        use num_complex::Complex64;
        let mut interp = AsyncInterpreter::new();

        // Test decimals in both parts
        let result = parse("1.5+2.5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(1.5, 2.5)));

        // Test decimal in real only
        let result = parse("3.14+2i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(3.14, 2.0)));

        // Test decimal in imaginary only
        let result = parse("2+3.14i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(2.0, 3.14)));

        // Test very small decimals
        let result = parse("0.1+0.2i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if (c.re - 0.1).abs() < 0.0001 && (c.im - 0.2).abs() < 0.0001));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_pure_imaginary_edge_cases() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = AsyncInterpreter::new();

        // Test simple pure imaginary (integer -> GaussianInt)
        let result = parse("5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(5)));

        // Test negative pure imaginary (integer -> GaussianInt)
        let result = parse("-5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(-5)));

        // Test decimal pure imaginary (decimal -> Complex64)
        let result = parse("3.5i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(0.0, 3.5)));

        // Test zero pure imaginary (integer -> GaussianInt)
        let result = parse("0i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(0) && im == &BigInt::from(0)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_large_numbers() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // Test large integer values (integers -> GaussianInt)
        let result = parse("1000000+2000000i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(1000000) && im == &BigInt::from(2000000)));

        // Test very large decimals (decimals -> Complex64)
        let result = parse("123456.789+987654.321i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            Value::Complex(c) => {
                assert!((c.re - 123456.789).abs() < 0.001);
                assert!((c.im - 987654.321).abs() < 0.001);
            }
            _ => panic!("Expected complex"),
        }
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_with_spaces_fails() {
        let mut interp = AsyncInterpreter::new();

        // In postfix, "3 + 4i" should parse as three separate tokens
        // (not a complex number)
        let result = parse("3 + 4i", &mut interp).unwrap();
        assert_eq!(result.len(), 3); // Three separate values

        // Only "3+4i" (no spaces) should be a complex number (GaussianInt since both integers)
        let result = parse("3+4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(_, _)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_not_confused_with_operators() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // "3+4" is not complex (no 'i' suffix)
        // It should parse as two separate tokens in postfix
        let result = parse("3 4 +", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        // But "3+4i" IS complex (has 'i' suffix, integers -> GaussianInt)
        let result = parse("3+4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(3) && im == &BigInt::from(4)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_multiple_signs() {
        use num_bigint::BigInt;
        let mut interp = AsyncInterpreter::new();

        // Test multiple complex numbers in sequence (all integers -> GaussianInt)
        let result = parse("1+2i 3-4i -5+6i -7-8i", &mut interp).unwrap();
        assert_eq!(result.len(), 4);

        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(1) && im == &BigInt::from(2)));
        assert!(matches!(result[1], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(3) && im == &BigInt::from(-4)));
        assert!(matches!(result[2], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(-5) && im == &BigInt::from(6)));
        assert!(matches!(result[3], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(-7) && im == &BigInt::from(-8)));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_scientific_notation() {
        use num_complex::Complex64;
        let mut interp = AsyncInterpreter::new();

        // Test scientific notation in complex numbers (positive exponents only)
        let result = parse("1e2+3e1i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(100.0, 30.0)));

        // Note: Scientific notation with negative exponents (e.g., "1e-2+3e-1i")
        // is not supported because the '-' is ambiguous with complex number syntax.
        // For such cases, use decimals instead: "0.01+0.3i"
    }

    #[test]
    fn test_parse_rational_edge_cases() {
        use num_rational::BigRational;
        
        let mut interp = AsyncInterpreter::new();

        // Test 1/1 (whole number as fraction) - should demote to Int32
        let result = parse("1/1", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Int32(1)));

        // Test 0/1 (zero as fraction) - should demote to Int32(0)
        let result = parse("0/1", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Int32(0)));

        // Test large numerator and denominator
        let result = parse("123456789/987654321", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        let expected = BigRational::new(BigInt::from(123456789), BigInt::from(987654321));
        assert!(matches!(result[0], Value::Rational(ref r) if *r == expected));
    }

    #[test]
    fn test_parse_bigint_edge_cases() {
        use num_bigint::BigInt;
        use num_traits::Zero;
        let mut interp = AsyncInterpreter::new();

        // Test 0n
        let result = parse("0n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Integer(ref i) if i.is_zero()));

        // Test 1n
        let result = parse("1n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Integer(ref i) if *i == BigInt::from(1)));

        // Test negative zero: -0n
        let result = parse("-0n", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Integer(ref i) if i.is_zero()));
    }

    #[test]
    fn test_parse_integer_vs_float_literals() {
        
        let mut interp = AsyncInterpreter::new();

        // Integers (no decimal point) - small integers use Int32
        let result = parse("1", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Int32(1)));

        let result = parse("42", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Int32(42)));

        let result = parse("-5", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Int32(-5)));

        let result = parse("0", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Int32(0)));

        // Floats (with decimal point)
        let result = parse("1.0", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Number(n) if n == 1.0));

        let result = parse("42.0", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Number(n) if n == 42.0));

        let result = parse("-5.0", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Number(n) if n == -5.0));

        let result = parse("3.14", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Number(n) if n == 3.14));

        // Scientific notation (always float)
        let result = parse("1e10", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Number(n) if n == 1e10));

        let result = parse("1.5e-3", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Number(n) if n == 1.5e-3));
    }

    #[test]
    #[cfg(feature = "complex_numbers")]
    fn test_parse_complex_integer_vs_float() {
        use num_bigint::BigInt;
        use num_complex::Complex64;
        let mut interp = AsyncInterpreter::new();

        // Integer complex (gaussian)
        let result = parse("1+2i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(1) && im == &BigInt::from(2)));

        let result = parse("3-4i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::GaussianInt(ref re, ref im)
            if re == &BigInt::from(3) && im == &BigInt::from(-4)));

        // Float complex
        let result = parse("1.0+2.0i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(1.0, 2.0)));

        let result = parse("3.0-4.0i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(3.0, -4.0)));

        // Mixed (one has decimal point means Complex)
        let result = parse("1.0+2i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(1.0, 2.0)));

        let result = parse("1+2.0i", &mut interp).unwrap();
        assert_eq!(result.len(), 1);
        assert!(matches!(result[0], Value::Complex(c) if c == Complex64::new(1.0, 2.0)));
    }
}
