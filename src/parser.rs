// This module converts tokens (from the tokenizer) into Values (our AST/data structures)
//
// UNI EXECUTION MODEL:
// - Numbers (42, 3.14): Push themselves onto the stack
// - Strings ("hello"): Push themselves onto the stack
// - Lists ([1 2 +]): Push themselves onto the stack as data (code-as-data)
// - Atoms (hello, +): Execute by looking up in dictionary
// - Quoted Atoms ('hello): Push the atom onto the stack without executing
// - To execute a list: [1 2 +] eval  (push list, then eval executes it)
//
// RUST LEARNING NOTES:
// - We use 'use' statements to import types from other modules
// - The 'std::rc::Rc' is Rust's reference-counted smart pointer for shared ownership
// - 'Result<T, E>' is Rust's way of handling errors without exceptions
// - Pattern matching with 'match' is Rust's equivalent to switch statements but much more powerful

use std::rc::Rc;
use crate::tokenizer::{Token, tokenize};
use crate::value::{Value, RuntimeError};
use crate::interpreter::Interpreter;

// RUST CONCEPT: Error types
// We create our own error type for parser-specific errors
// #[derive(Debug)] automatically implements Debug trait so we can print errors
// This is separate from RuntimeError because parsing happens before execution
#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),        // Got a token we didn't expect
    UnexpectedEndOfInput,           // Ran out of tokens when we needed more
    MismatchedBrackets,             // [ without matching ] or vice versa
    InvalidDotNotation,             // Malformed [a . b] pair syntax
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
pub fn parse(input: &str, interp: &mut Interpreter) -> Result<Vec<Value>, ParseError> {
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
fn parse_value(tokens: &[Token], index: &mut usize, interp: &mut Interpreter) -> Result<Value, ParseError> {
    // RUST CONCEPT: Bounds checking
    // .get() returns Option<T> - Some(value) if index exists, None if out of bounds
    // This is safer than direct indexing which would panic on out-of-bounds
    match tokens.get(*index) {
        // RUST CONCEPT: Pattern matching on references
        // We match on &Token because .get() returns Option<&Token>
        // The & in the pattern destructures the reference
        Some(&Token::Number(n)) => {
            *index += 1;  // RUST CONCEPT: Dereferencing - modify the value index points to
            Ok(Value::Number(n))
        },

        Some(Token::Atom(atom_text)) => {
            *index += 1;
            // RUST CONCEPT: Method calls and string conversion
            // We need to intern the atom through the interpreter
            // .clone() on a String creates a new owned string
            let interned_atom = interp.intern_atom(&atom_text.clone());
            Ok(Value::Atom(interned_atom))
        },

        Some(Token::String(string_text)) => {
            *index += 1;
            // RUST CONCEPT: Converting String to Rc<str>
            // .clone() gets an owned String, .into() converts to Rc<str>
            // Strings are NOT interned - each one gets its own Rc
            let string_rc: Rc<str> = string_text.clone().into();
            Ok(Value::String(string_rc))
        },

        Some(&Token::LeftBracket) => {
            // RUST CONCEPT: Recursive parsing
            // Lists can contain other lists, so we call parse_list which may call parse_value again
            parse_list(tokens, index, interp)
        },

        Some(&Token::Quote) => {
            *index += 1;  // Skip the quote token

            // RUST CONCEPT: Validating syntax rules
            // In Uni, only atoms can be quoted. Lists, numbers, and strings
            // don't need quotes because they push themselves onto the stack by default
            match tokens.get(*index) {
                Some(Token::Atom(atom_text)) => {
                    *index += 1;  // Consume the atom token

                    // RUST CONCEPT: Creating quoted atoms directly
                    // Instead of (quote atom) structure, we create a QuotedAtom value
                    // This directly represents the semantics: push atom without executing
                    let interned_atom = interp.intern_atom(&atom_text.clone());
                    Ok(Value::QuotedAtom(interned_atom))
                },
                Some(&Token::LeftBracket) => {
                    // RUST CONCEPT: Custom error types for syntax validation
                    // Lists don't need quotes - they're data by default
                    Err(ParseError::UnexpectedToken("Lists cannot be quoted - they are data by default".to_string()))
                },
                Some(Token::String(_)) => {
                    // Strings don't need quotes - they push themselves onto the stack
                    Err(ParseError::UnexpectedToken("Strings cannot be quoted - they are data by default".to_string()))
                },
                Some(&Token::Number(_)) => {
                    // Numbers don't need quotes - they push themselves onto the stack
                    Err(ParseError::UnexpectedToken("Numbers cannot be quoted - they are data by default".to_string()))
                },
                _ => {
                    // Quote without anything following, or followed by invalid token
                    Err(ParseError::UnexpectedEndOfInput)
                }
            }
        },

        Some(&Token::Dot) => {
            // RUST CONCEPT: Error handling
            // A dot by itself is invalid - it should only appear in [a . b] notation
            Err(ParseError::InvalidDotNotation)
        },

        Some(&Token::RightBracket) => {
            // RUST CONCEPT: Error types
            // A closing bracket without a matching opening bracket is an error
            Err(ParseError::MismatchedBrackets)
        },

        // RUST CONCEPT: None pattern
        // This handles the case where we've run out of tokens
        None => Err(ParseError::UnexpectedEndOfInput),
    }
}

// RUST CONCEPT: Complex parsing logic
// This function handles the tricky case of parsing lists, which can be:
// - [1 2 3] (proper list)
// - [a . b] (cons pair)
// - [] (empty list)
fn parse_list(tokens: &[Token], index: &mut usize, interp: &mut Interpreter) -> Result<Value, ParseError> {
    // RUST CONCEPT: Assertions and debugging
    // debug_assert! is removed in release builds but helps catch bugs during development
    // This ensures we're starting at a LeftBracket token
    debug_assert!(matches!(tokens.get(*index), Some(&Token::LeftBracket)));

    *index += 1; // Skip the opening bracket

    // RUST CONCEPT: Vec for collecting elements
    // We'll collect list elements here, then convert to cons cells at the end
    let mut elements = Vec::new();

    // RUST CONCEPT: Loop with pattern matching
    // We loop until we find the closing bracket or run out of tokens
    loop {
        match tokens.get(*index) {
            Some(&Token::RightBracket) => {
                *index += 1; // Skip the closing bracket
                break;       // RUST CONCEPT: break exits the loop
            },

            Some(&Token::Dot) => {
                // RUST CONCEPT: Complex parsing - dot notation [a . b]
                // We need at least one element before the dot, and exactly one after
                if elements.is_empty() {
                    return Err(ParseError::InvalidDotNotation);
                }

                *index += 1; // Skip the dot

                // Parse the element after the dot (the "tail" of the cons cell)
                let tail = parse_value(tokens, index, interp)?;

                // RUST CONCEPT: Expecting specific tokens
                // After [a . b], we MUST see a closing bracket
                match tokens.get(*index) {
                    Some(&Token::RightBracket) => {
                        *index += 1; // Skip the closing bracket

                        // RUST CONCEPT: Building cons cells manually
                        // [a b c . d] becomes Pair(a, Pair(b, Pair(c, d)))
                        // We build this right-to-left using fold
                        let cons_cell = elements.into_iter()
                            .rev()  // RUST CONCEPT: Iterator adaptors - reverse the order
                            .fold(tail, |acc, elem| {
                                // RUST CONCEPT: Closures (anonymous functions)
                                // |acc, elem| is the closure parameter list
                                // We build Pair(elem, acc) for each element
                                Value::Pair(Rc::new(elem), Rc::new(acc))
                            });

                        return Ok(cons_cell);
                    },
                    _ => return Err(ParseError::MismatchedBrackets),
                }
            },

            None => {
                // RUST CONCEPT: Error cases
                // We ran out of tokens while looking for the closing bracket
                return Err(ParseError::UnexpectedEndOfInput);
            },

            _ => {
                // RUST CONCEPT: Recursive parsing continues
                // Parse the next element and add it to our list
                let element = parse_value(tokens, index, interp)?;
                elements.push(element);
            }
        }
    }

    // RUST CONCEPT: Converting Vec to linked list
    // If we get here, we have a proper list [1 2 3] without dot notation
    // Convert the Vec<Value> to a linked list of Pair nodes ending in Nil
    let list = elements.into_iter()
        .rev()  // Reverse so we build right-to-left
        .fold(Value::Nil, |acc, elem| {
            // RUST CONCEPT: Fold (reduce) operation
            // This is like reduce in other languages
            // We start with Nil and wrap each element: Pair(elem, previous_result)
            Value::Pair(Rc::new(elem), Rc::new(acc))
        });

    Ok(list)
}

// RUST CONCEPT: Conditional compilation and testing
// #[cfg(test)] means this code only compiles when running tests
// This keeps test code out of the release binary
#[cfg(test)]
mod tests {
    use super::*;  // RUST CONCEPT: Import everything from parent module

    // RUST CONCEPT: Test functions
    // #[test] tells Rust this function is a unit test
    #[test]
    fn test_parse_numbers() {
        // RUST CONCEPT: Creating test data
        // We need a mutable interpreter for atom interning
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Unwrap for tests
        // .unwrap() panics if Result is Err - fine for tests, bad for production
        let result = parse("42", &mut interp).unwrap();

        // RUST CONCEPT: Assertions
        // assert_eq! compares two values and panics if they're different
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Pattern matching in tests
        // We destructure the result to check it's the right type
        match &result[0] {
            Value::Number(n) => assert_eq!(*n, 42.0),
            _ => panic!("Expected number"),  // RUST CONCEPT: panic! for test failures
        }

        // Test multiple numbers
        let result = parse("1 2.5 -3", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        // RUST CONCEPT: Match with guards and ranges
        // We can have multiple patterns in one match
        for (i, expected) in [1.0, 2.5, -3.0].iter().enumerate() {
            match &result[i] {
                Value::Number(n) => assert_eq!(*n, *expected),
                _ => panic!("Expected number at position {}", i),
            }
        }
    }

    #[test]
    fn test_parse_atoms() {
        let mut interp = Interpreter::new();

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
                },
                _ => panic!("Expected atom at position {}", i),
            }
        }

        // RUST CONCEPT: Testing interning behavior
        // Same atoms should share the same Rc (pointer equality)
        let result2 = parse("hello", &mut interp).unwrap();
        if let (Value::Atom(atom1), Value::Atom(atom2)) = (&result[0], &result2[0]) {
            assert!(Rc::ptr_eq(atom1, atom2));  // Same pointer = successful interning
        }
    }

    #[test]
    fn test_parse_strings() {
        let mut interp = Interpreter::new();

        let result = parse("\"hello world\" \"\"", &mut interp).unwrap();
        assert_eq!(result.len(), 2);

        match &result[0] {
            Value::String(s) => assert_eq!(&**s, "hello world"),
            _ => panic!("Expected string"),
        }

        match &result[1] {
            Value::String(s) => assert_eq!(&**s, ""),  // Empty string
            _ => panic!("Expected empty string"),
        }

        // RUST CONCEPT: Testing non-interning
        // Strings with same content should NOT share pointers
        let result2 = parse("\"hello world\"", &mut interp).unwrap();
        if let (Value::String(s1), Value::String(s2)) = (&result[0], &result2[0]) {
            assert_eq!(s1, s2);  // Same content
            assert!(!Rc::ptr_eq(s1, s2));  // Different pointers = not interned
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let mut interp = Interpreter::new();

        let result = parse("[]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Matching specific enum variants
        match &result[0] {
            Value::Nil => (),  // () is the unit value - like void but it's a real value
            _ => panic!("Expected empty list (Nil)"),
        }
    }

    #[test]
    fn test_parse_simple_list() {
        let mut interp = Interpreter::new();

        let result = parse("[1 2 3]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Nested pattern matching
        // We can destructure complex nested structures in one match
        match &result[0] {
            Value::Pair(car1, cdr1) => {
                // First element should be 1
                assert!(matches!(**car1, Value::Number(n) if n == 1.0));

                match cdr1.as_ref() {  // RUST CONCEPT: as_ref() converts &Rc<T> to &T
                    Value::Pair(car2, cdr2) => {
                        // Second element should be 2
                        assert!(matches!(**car2, Value::Number(n) if n == 2.0));

                        match cdr2.as_ref() {
                            Value::Pair(car3, cdr3) => {
                                // Third element should be 3
                                assert!(matches!(**car3, Value::Number(n) if n == 3.0));
                                // End should be Nil
                                assert!(matches!(**cdr3, Value::Nil));
                            },
                            _ => panic!("Expected third pair"),
                        }
                    },
                    _ => panic!("Expected second pair"),
                }
            },
            _ => panic!("Expected list pair"),
        }
    }

    #[test]
    fn test_parse_dot_notation() {
        let mut interp = Interpreter::new();

        // Test [1 . 2] - a simple cons cell
        let result = parse("[1 . 2]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            Value::Pair(car, cdr) => {
                assert!(matches!(**car, Value::Number(n) if n == 1.0));
                assert!(matches!(**cdr, Value::Number(n) if n == 2.0));
            },
            _ => panic!("Expected pair"),
        }

        // Test [1 2 . 3] - multiple elements before dot
        let result = parse("[1 2 . 3]", &mut interp).unwrap();
        match &result[0] {
            Value::Pair(car1, cdr1) => {
                assert!(matches!(**car1, Value::Number(n) if n == 1.0));
                match cdr1.as_ref() {
                    Value::Pair(car2, cdr2) => {
                        assert!(matches!(**car2, Value::Number(n) if n == 2.0));
                        assert!(matches!(**cdr2, Value::Number(n) if n == 3.0));
                    },
                    _ => panic!("Expected nested pair"),
                }
            },
            _ => panic!("Expected pair"),
        }
    }

    #[test]
    fn test_parse_quoted_values() {
        let mut interp = Interpreter::new();

        let result = parse("'hello", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Testing QuotedAtom parsing
        // A quoted atom becomes QuotedAtom directly
        match &result[0] {
            Value::QuotedAtom(atom) => {
                assert_eq!(&**atom, "hello");
            },
            _ => panic!("Expected QuotedAtom"),
        }
    }

    #[test]
    fn test_parse_nested_lists() {
        let mut interp = Interpreter::new();

        let result = parse("[[1 2] [3]]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Testing complex data structures
        // This tests that we can parse nested lists correctly
        match &result[0] {
            Value::Pair(first_list, cdr) => {
                // First list should be [1 2]
                match first_list.as_ref() {
                    Value::Pair(one, rest1) => {
                        assert!(matches!(**one, Value::Number(n) if n == 1.0));
                        match rest1.as_ref() {
                            Value::Pair(two, rest2) => {
                                assert!(matches!(**two, Value::Number(n) if n == 2.0));
                                assert!(matches!(**rest2, Value::Nil));
                            },
                            _ => panic!("Expected [1 2] structure"),
                        }
                    },
                    _ => panic!("Expected first list"),
                }

                // Rest should be ([3])
                match cdr.as_ref() {
                    Value::Pair(second_list, final_cdr) => {
                        // Second list should be [3]
                        match second_list.as_ref() {
                            Value::Pair(three, rest3) => {
                                assert!(matches!(**three, Value::Number(n) if n == 3.0));
                                assert!(matches!(**rest3, Value::Nil));
                            },
                            _ => panic!("Expected [3] structure"),
                        }
                        assert!(matches!(**final_cdr, Value::Nil));
                    },
                    _ => panic!("Expected second list pair"),
                }
            },
            _ => panic!("Expected outer list"),
        }
    }

    #[test]
    fn test_parse_errors() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing error cases
        // We use assert!(result.is_err()) to verify we get errors for bad input

        // Mismatched brackets
        assert!(parse("[1 2", &mut interp).is_err());
        assert!(parse("1 2]", &mut interp).is_err());

        // Invalid dot notation
        assert!(parse("[.]", &mut interp).is_err());
        assert!(parse("[1 . 2 3]", &mut interp).is_err());  // Too many elements after dot

        // Standalone dot
        assert!(parse("1 . 2", &mut interp).is_err());
    }

    #[test]
    fn test_parse_with_comments() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing integration
        // Comments should be stripped by tokenizer, so parser never sees them
        let result = parse("42 \\ this is a comment\n37", &mut interp).unwrap();
        assert_eq!(result.len(), 2);

        match (&result[0], &result[1]) {
            (Value::Number(n1), Value::Number(n2)) => {
                assert_eq!(*n1, 42.0);
                assert_eq!(*n2, 37.0);
            },
            _ => panic!("Expected two numbers"),
        }
    }

    #[test]
    fn test_parse_quoted_atoms_only() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing valid quote syntax - only atoms can be quoted
        let result = parse("'hello", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // Should be QuotedAtom("hello")
        match &result[0] {
            Value::QuotedAtom(atom) => {
                assert_eq!(&**atom, "hello");
            },
            _ => panic!("Expected QuotedAtom"),
        }

        // Test multiple quoted atoms
        let result = parse("'+ '- '*", &mut interp).unwrap();
        assert_eq!(result.len(), 3);

        for (i, op) in ["+", "-", "*"].iter().enumerate() {
            match &result[i] {
                Value::QuotedAtom(atom) => {
                    assert_eq!(&**atom, *op);
                },
                _ => panic!("Expected QuotedAtom at {}", i),
            }
        }
    }

    #[test]
    fn test_parse_reject_quoted_non_atoms() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing syntax restrictions
        // These should all be rejected because only atoms can be quoted

        // Quoted lists should be rejected
        assert!(parse("'[1 2 3]", &mut interp).is_err());
        assert!(parse("'[]", &mut interp).is_err());
        assert!(parse("'[[nested]]", &mut interp).is_err());

        // Quoted strings should be rejected
        assert!(parse("'\"hello\"", &mut interp).is_err());
        assert!(parse("'\"\"", &mut interp).is_err());

        // Quoted numbers should be rejected
        assert!(parse("'42", &mut interp).is_err());
        assert!(parse("'3.14", &mut interp).is_err());
        assert!(parse("'-17", &mut interp).is_err());
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

        let result = parse("'42", &mut interp);
        assert!(result.is_err());
        let error_msg = format!("{:?}", result.unwrap_err());
        assert!(error_msg.contains("Numbers cannot be quoted"));
    }

    #[test]
    fn test_parse_mixed_types_in_list() {
        let mut interp = Interpreter::new();

        let result = parse("[42 hello \"world\" [nested]]", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        // RUST CONCEPT: Testing heterogeneous data structures
        // Uni lists can contain any mix of types
        match &result[0] {
            Value::Pair(first, rest1) => {
                assert!(matches!(first.as_ref(), Value::Number(n) if *n == 42.0));

                match rest1.as_ref() {
                    Value::Pair(second, rest2) => {
                        assert!(matches!(second.as_ref(), Value::Atom(a) if &**a == "hello"));

                        match rest2.as_ref() {
                            Value::Pair(third, rest3) => {
                                assert!(matches!(third.as_ref(), Value::String(s) if &**s == "world"));

                                match rest3.as_ref() {
                                    Value::Pair(fourth, rest4) => {
                                        // Fourth should be a nested list
                                        assert!(matches!(fourth.as_ref(), Value::Pair(_, _)));
                                        assert!(matches!(rest4.as_ref(), Value::Nil));
                                    },
                                    _ => panic!("Expected fourth element"),
                                }
                            },
                            _ => panic!("Expected third element"),
                        }
                    },
                    _ => panic!("Expected second element"),
                }
            },
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_whitespace_handling() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing tokenizer integration
        // Various whitespace should be handled correctly
        let inputs = [
            "  1   2   3  ",           // Extra spaces
            "1\n2\t3\r\n4",            // Mixed whitespace
            "[  1   2  ]",             // Spaces in lists
            "[ 1 . 2 ]",               // Spaces around dot
            "'   hello   ",            // Spaces after quote
        ];

        let expected_lengths = [3, 4, 1, 1, 1];

        for (input, expected_len) in inputs.iter().zip(expected_lengths.iter()) {
            let result = parse(input, &mut interp).unwrap();
            assert_eq!(result.len(), *expected_len, "Failed for input: '{}'", input);
        }
    }

    #[test]
    fn test_parse_deeply_nested_lists() {
        let mut interp = Interpreter::new();

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
                },
                Value::Number(n) if *n == 1.0 => break,
                _ => panic!("Expected nested structure"),
            }
        }
    }

    #[test]
    fn test_parse_complex_dot_cases() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing edge cases thoroughly

        // Multiple dots should fail
        assert!(parse("[1 . 2 . 3]", &mut interp).is_err());

        // Dot at beginning should fail
        assert!(parse("[. 1]", &mut interp).is_err());

        // Multiple elements after dot should fail
        assert!(parse("[1 . 2 3]", &mut interp).is_err());

        // Dot in nested list should work
        let result = parse("[[1 . 2]]", &mut interp).unwrap();
        match &result[0] {
            Value::Pair(inner_list, outer_cdr) => {
                match inner_list.as_ref() {
                    Value::Pair(one, two) => {
                        assert!(matches!(one.as_ref(), Value::Number(n) if *n == 1.0));
                        assert!(matches!(two.as_ref(), Value::Number(n) if *n == 2.0));
                    },
                    _ => panic!("Expected inner pair"),
                }
                assert!(matches!(outer_cdr.as_ref(), Value::Nil));
            },
            _ => panic!("Expected outer list"),
        }
    }

    #[test]
    fn test_parse_scientific_notation() {
        let mut interp = Interpreter::new();

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
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing boundary conditions
        let result = parse("", &mut interp).unwrap();
        assert_eq!(result.len(), 0);

        let result = parse("   ", &mut interp).unwrap();  // Just whitespace
        assert_eq!(result.len(), 0);

        let result = parse("\\ just a comment", &mut interp).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_parse_quote_edge_cases() {
        let mut interp = Interpreter::new();

        // Quote without following value should fail
        assert!(parse("'", &mut interp).is_err());

        // RUST CONCEPT: Testing syntax consistency
        // Nested quotes like ''hello don't make sense in Uni's execution model
        // Quote can only be followed by atoms, not other quote tokens
        assert!(parse("''hello", &mut interp).is_err());

        // Quote followed by other non-atom tokens should also fail
        assert!(parse("'.", &mut interp).is_err());
        assert!(parse("']", &mut interp).is_err());

        // But quotes followed by atoms should work perfectly
        let result = parse("'quote-me", &mut interp).unwrap();
        assert_eq!(result.len(), 1);

        match &result[0] {
            Value::QuotedAtom(atom) => {
                assert_eq!(&**atom, "quote-me");
            },
            _ => panic!("Expected QuotedAtom"),
        }
    }

    #[test]
    fn test_parse_error_messages() {
        let mut interp = Interpreter::new();

        // RUST CONCEPT: Testing error types and messages
        let error_cases = [
            ("[1 2", "UnexpectedEndOfInput"),
            ("1 2]", "MismatchedBrackets"),
            ("[.]", "InvalidDotNotation"),
            ("1 . 2", "InvalidDotNotation"),
            ("'[1 2]", "Lists cannot be quoted"),
            ("'\"hello\"", "Strings cannot be quoted"),
            ("'42", "Numbers cannot be quoted"),
            ("'", "UnexpectedEndOfInput"),  // Quote with nothing following
        ];

        for (input, expected_error) in error_cases.iter() {
            let result = parse(input, &mut interp);
            assert!(result.is_err(), "Expected error for input: '{}'", input);

            let error_string = format!("{:?}", result.unwrap_err());
            // Just check that we get the right error type
            assert!(error_string.contains(expected_error),
                "Expected '{}' in error message for '{}', got: {}",
                expected_error, input, error_string);
        }
    }
}