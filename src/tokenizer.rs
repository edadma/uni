use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    Atom(String),
    String(String),  // Quoted strings - not interned
    Boolean(bool),   // Boolean literals: true, false
    Null,           // Null literal
    LeftBracket,
    RightBracket,
    Quote,
    Dot,  // For cons pair notation like [1 . rest]
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::Atom(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            Token::Null => write!(f, "null"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::Quote => write!(f, "'"),
            Token::Dot => write!(f, "."),
        }
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }

            '[' => {
                tokens.push(Token::LeftBracket);
                chars.next();
            }

            ']' => {
                tokens.push(Token::RightBracket);
                chars.next();
            }


            '\'' => {
                tokens.push(Token::Quote);
                chars.next();
            }

            '\\' => {
                // Skip comments - consume everything until newline
                chars.next(); // consume the backslash
                while let Some(ch) = chars.next() {
                    if ch == '\n' {
                        break;
                    }
                }
                // Continue tokenizing after the comment
            }

            '.' => {
                chars.next();
                // Check if it's a standalone dot (for cons notation) or part of a number
                if chars.peek().map_or(true, |&c| c.is_whitespace() || "[]\'\"\\".contains(c)) {
                    tokens.push(Token::Dot);
                } else if chars.peek().map_or(false, |&c| c.is_ascii_digit()) {
                    // It's a decimal number like .5
                    let mut num_str = String::from("0.");
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == 'e' || ch == 'E' || ch == '-' || ch == '+' {
                            num_str.push(ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    match num_str.parse::<f64>() {
                        Ok(num) => tokens.push(Token::Number(num)),
                        Err(_) => return Err(format!("Invalid number: {}", num_str)),
                    }
                } else {
                    // It's a dot followed by non-digit, treat as atom
                    let mut atom = String::from(".");
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() || "[]\'\"\\".contains(ch) {
                            break;
                        }
                        atom.push(ch);
                        chars.next();
                    }
                    tokens.push(Token::Atom(atom));
                }
            }

            '"' => {
                chars.next();
                let mut string = String::new();
                let mut escaped = false;

                while let Some(ch) = chars.next() {
                    if escaped {
                        match ch {
                            'n' => string.push('\n'),
                            't' => string.push('\t'),
                            '\\' => string.push('\\'),
                            '"' => string.push('"'),
                            _ => {
                                string.push('\\');
                                string.push(ch);
                            }
                        }
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == '"' {
                        break;
                    } else {
                        string.push(ch);
                    }
                }

                tokens.push(Token::String(string));
            }


            '+' | '-' if chars.clone().nth(1).map_or(false, |c| c.is_ascii_digit()) => {
                // Handle signed numbers
                let mut num_str = String::new();
                num_str.push(ch);
                chars.next();

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                        num_str.push(ch);
                        chars.next();
                        // Allow + or - after e/E for scientific notation
                        if (ch == 'e' || ch == 'E') && chars.peek().map_or(false, |&c| c == '+' || c == '-') {
                            num_str.push(chars.next().unwrap());
                        }
                    } else {
                        break;
                    }
                }

                match num_str.parse::<f64>() {
                    Ok(num) => tokens.push(Token::Number(num)),
                    Err(_) => {
                        // If it's not a valid number, treat it as an atom
                        // Continue collecting non-whitespace chars
                        while let Some(&ch) = chars.peek() {
                            if ch.is_whitespace() || "[].\'\"\\".contains(ch) {
                                break;
                            }
                            num_str.push(ch);
                            chars.next();
                        }
                        tokens.push(Token::Atom(num_str));
                    }
                }
            }

            '0'..='9' => {
                let mut num_str = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                        num_str.push(ch);
                        chars.next();
                        // Allow + or - after e/E for scientific notation
                        if (ch == 'e' || ch == 'E') && chars.peek().map_or(false, |&c| c == '+' || c == '-') {
                            num_str.push(chars.next().unwrap());
                        }
                    } else {
                        break;
                    }
                }

                match num_str.parse::<f64>() {
                    Ok(num) => tokens.push(Token::Number(num)),
                    Err(_) => {
                        // If it's not a valid number, treat it as an atom
                        // Continue collecting non-whitespace chars
                        while let Some(&ch) = chars.peek() {
                            if ch.is_whitespace() || "[].\'\"\\".contains(ch) {
                                break;
                            }
                            num_str.push(ch);
                            chars.next();
                        }
                        tokens.push(Token::Atom(num_str));
                    }
                }
            }

            _ => {
                let mut atom = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || "[]\'.\'\"\\".contains(ch) {
                        break;
                    }
                    atom.push(ch);
                    chars.next();
                }

                if !atom.is_empty() {
                    // RUST CONCEPT: Pattern matching on string literals
                    // Check for special boolean and null literals
                    match atom.as_str() {
                        "true" => tokens.push(Token::Boolean(true)),
                        "false" => tokens.push(Token::Boolean(false)),
                        "null" => tokens.push(Token::Null),
                        _ => tokens.push(Token::Atom(atom)),
                    }
                }
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_numbers() {
        assert_eq!(tokenize("42").unwrap(), vec![Token::Number(42.0)]);
        assert_eq!(tokenize("3.14").unwrap(), vec![Token::Number(3.14)]);
        assert_eq!(tokenize("-17").unwrap(), vec![Token::Number(-17.0)]);
        assert_eq!(tokenize("+2.5").unwrap(), vec![Token::Number(2.5)]);
        assert_eq!(tokenize("1e10").unwrap(), vec![Token::Number(1e10)]);
    }

    #[test]
    fn test_tokenize_atoms() {
        assert_eq!(tokenize("hello").unwrap(), vec![Token::Atom("hello".to_string())]);
        assert_eq!(tokenize("+").unwrap(), vec![Token::Atom("+".to_string())]);
        assert_eq!(tokenize("-").unwrap(), vec![Token::Atom("-".to_string())]);
        assert_eq!(tokenize("foo-bar").unwrap(), vec![Token::Atom("foo-bar".to_string())]);
    }

    #[test]
    fn test_tokenize_strings() {
        assert_eq!(
            tokenize("\"hello world\"").unwrap(),
            vec![Token::String("hello world".to_string())]
        );
        assert_eq!(
            tokenize("\"hello\\nworld\"").unwrap(),
            vec![Token::String("hello\nworld".to_string())]
        );
        assert_eq!(
            tokenize("\"quote: \\\"test\\\"\"").unwrap(),
            vec![Token::String("quote: \"test\"".to_string())]
        );
    }

    #[test]
    fn test_tokenize_brackets() {
        assert_eq!(
            tokenize("[1 2 3]").unwrap(),
            vec![
                Token::LeftBracket,
                Token::Number(1.0),
                Token::Number(2.0),
                Token::Number(3.0),
                Token::RightBracket
            ]
        );
    }


    #[test]
    fn test_tokenize_quote() {
        assert_eq!(
            tokenize("'foo").unwrap(),
            vec![Token::Quote, Token::Atom("foo".to_string())]
        );
        assert_eq!(
            tokenize("'[1 2]").unwrap(),
            vec![
                Token::Quote,
                Token::LeftBracket,
                Token::Number(1.0),
                Token::Number(2.0),
                Token::RightBracket
            ]
        );
    }

    #[test]
    fn test_tokenize_mixed() {
        assert_eq!(
            tokenize("def square [dup *]").unwrap(),
            vec![
                Token::Atom("def".to_string()),
                Token::Atom("square".to_string()),
                Token::LeftBracket,
                Token::Atom("dup".to_string()),
                Token::Atom("*".to_string()),
                Token::RightBracket
            ]
        );
    }

    #[test]
    fn test_tokenize_whitespace() {
        assert_eq!(
            tokenize("  42\t\n  foo  ").unwrap(),
            vec![Token::Number(42.0), Token::Atom("foo".to_string())]
        );
    }

    #[test]
    fn test_tokenize_empty() {
        assert_eq!(tokenize("").unwrap(), vec![]);
        assert_eq!(tokenize("   \t\n  ").unwrap(), vec![]);
    }

    #[test]
    fn test_tokenize_complex_expression() {
        let input = "'square [dup *] def 5 square";
        let expected = vec![
            Token::Quote,
            Token::Atom("square".to_string()),
            Token::LeftBracket,
            Token::Atom("dup".to_string()),
            Token::Atom("*".to_string()),
            Token::RightBracket,
            Token::Atom("def".to_string()),
            Token::Number(5.0),
            Token::Atom("square".to_string()),
        ];
        assert_eq!(tokenize(input).unwrap(), expected);
    }

    #[test]
    fn test_tokenize_dot_notation() {
        // Cons pair notation
        assert_eq!(
            tokenize("[1 . rest]").unwrap(),
            vec![
                Token::LeftBracket,
                Token::Number(1.0),
                Token::Dot,
                Token::Atom("rest".to_string()),
                Token::RightBracket
            ]
        );

        // Decimal numbers
        assert_eq!(tokenize(".5").unwrap(), vec![Token::Number(0.5)]);
        assert_eq!(tokenize("3.14").unwrap(), vec![Token::Number(3.14)]);
    }

    #[test]
    fn test_tokenize_operators() {
        assert_eq!(
            tokenize("+ - * / < > <= >= == != % & | ^ ~ ?").unwrap(),
            vec![
                Token::Atom("+".to_string()),
                Token::Atom("-".to_string()),
                Token::Atom("*".to_string()),
                Token::Atom("/".to_string()),
                Token::Atom("<".to_string()),
                Token::Atom(">".to_string()),
                Token::Atom("<=".to_string()),
                Token::Atom(">=".to_string()),
                Token::Atom("==".to_string()),
                Token::Atom("!=".to_string()),
                Token::Atom("%".to_string()),
                Token::Atom("&".to_string()),
                Token::Atom("|".to_string()),
                Token::Atom("^".to_string()),
                Token::Atom("~".to_string()),
                Token::Atom("?".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_arithmetic_expression() {
        assert_eq!(
            tokenize("3 4 + 2 * 10 /").unwrap(),
            vec![
                Token::Number(3.0),
                Token::Number(4.0),
                Token::Atom("+".to_string()),
                Token::Number(2.0),
                Token::Atom("*".to_string()),
                Token::Number(10.0),
                Token::Atom("/".to_string()),
            ]
        );
    }

    #[test]
    fn test_tokenize_edge_cases() {
        // Standalone + and - should be atoms
        assert_eq!(tokenize("+ -").unwrap(), vec![
            Token::Atom("+".to_string()),
            Token::Atom("-".to_string())
        ]);

        // Numbers immediately followed by brackets
        assert_eq!(tokenize("42[").unwrap(), vec![
            Token::Number(42.0),
            Token::LeftBracket
        ]);

        // Atoms with special characters
        assert_eq!(tokenize("<=>=!=").unwrap(), vec![
            Token::Atom("<=>=!=".to_string())
        ]);

        // Mixed numbers and decimals
        assert_eq!(tokenize("1.5 .5 5.").unwrap(), vec![
            Token::Number(1.5),
            Token::Number(0.5),
            Token::Number(5.0)
        ]);
    }

    #[test]
    fn test_tokenize_scientific_notation() {
        assert_eq!(tokenize("1e5").unwrap(), vec![Token::Number(1e5)]);
        assert_eq!(tokenize("2.5e-3").unwrap(), vec![Token::Number(2.5e-3)]);
        assert_eq!(tokenize("1E10").unwrap(), vec![Token::Number(1E10)]);
    }

    #[test]
    fn test_tokenize_string_edge_cases() {
        // Empty string
        assert_eq!(tokenize("\"\"").unwrap(), vec![Token::String("".to_string())]);

        // String with tabs and escapes
        assert_eq!(
            tokenize("\"\\t\\\\\\\"\"").unwrap(),
            vec![Token::String("\t\\\"".to_string())]
        );
    }

    #[test]
    fn test_tokenize_invalid_numbers() {
        // These should be treated as atoms, not cause errors
        assert_eq!(tokenize("1.2.3").unwrap(), vec![Token::Atom("1.2.3".to_string())]);
        assert_eq!(tokenize("1e").unwrap(), vec![Token::Atom("1e".to_string())]);
    }

    #[test]
    fn test_tokenize_complex_atoms() {
        // Complex atom names that might be used in Uni
        assert_eq!(tokenize("make-list").unwrap(), vec![Token::Atom("make-list".to_string())]);
        assert_eq!(tokenize("list->vector").unwrap(), vec![Token::Atom("list->vector".to_string())]);
        assert_eq!(tokenize("car/cdr").unwrap(), vec![Token::Atom("car/cdr".to_string())]);
    }

    #[test]
    fn test_tokenize_spacing_variations() {
        // Multiple spaces
        assert_eq!(tokenize("  1    2  ").unwrap(), vec![
            Token::Number(1.0),
            Token::Number(2.0)
        ]);

        // Mixed whitespace
        assert_eq!(tokenize("1\t\n2\r\n3").unwrap(), vec![
            Token::Number(1.0),
            Token::Number(2.0),
            Token::Number(3.0)
        ]);
    }

    #[test]
    fn test_tokenize_comments() {
        // Comment at end of line
        assert_eq!(
            tokenize("5 3 + \\ This is a comment").unwrap(),
            vec![
                Token::Number(5.0),
                Token::Number(3.0),
                Token::Atom("+".to_string())
            ]
        );

        // Comment with newline
        assert_eq!(
            tokenize("42 \\ comment\n37").unwrap(),
            vec![
                Token::Number(42.0),
                Token::Number(37.0)
            ]
        );

        // Multiple comments
        assert_eq!(
            tokenize("1 \\ first comment\n2 \\ second comment\n3").unwrap(),
            vec![
                Token::Number(1.0),
                Token::Number(2.0),
                Token::Number(3.0)
            ]
        );

        // Just a comment
        assert_eq!(tokenize("\\ just a comment").unwrap(), vec![]);
    }

    #[test]
    fn test_tokenize_booleans() {
        // Test true literal
        assert_eq!(tokenize("true").unwrap(), vec![Token::Boolean(true)]);

        // Test false literal
        assert_eq!(tokenize("false").unwrap(), vec![Token::Boolean(false)]);

        // Test mixed with other tokens
        assert_eq!(
            tokenize("true false 42 'test").unwrap(),
            vec![
                Token::Boolean(true),
                Token::Boolean(false),
                Token::Number(42.0),
                Token::Quote,
                Token::Atom("test".to_string())
            ]
        );

        // Test in brackets
        assert_eq!(
            tokenize("[true false]").unwrap(),
            vec![
                Token::LeftBracket,
                Token::Boolean(true),
                Token::Boolean(false),
                Token::RightBracket
            ]
        );

        // Test boolean-like but not exact (should be atoms)
        assert_eq!(tokenize("TRUE").unwrap(), vec![Token::Atom("TRUE".to_string())]);
        assert_eq!(tokenize("True").unwrap(), vec![Token::Atom("True".to_string())]);
        assert_eq!(tokenize("false-flag").unwrap(), vec![Token::Atom("false-flag".to_string())]);
        assert_eq!(tokenize("truthy").unwrap(), vec![Token::Atom("truthy".to_string())]);
    }

    #[test]
    fn test_tokenize_null() {
        // Test null literal
        assert_eq!(tokenize("null").unwrap(), vec![Token::Null]);

        // Test mixed with other values
        assert_eq!(
            tokenize("null 42 \"hello\" true").unwrap(),
            vec![
                Token::Null,
                Token::Number(42.0),
                Token::String("hello".to_string()),
                Token::Boolean(true)
            ]
        );

        // Test in list context
        assert_eq!(
            tokenize("[1 null true \"test\"]").unwrap(),
            vec![
                Token::LeftBracket,
                Token::Number(1.0),
                Token::Null,
                Token::Boolean(true),
                Token::String("test".to_string()),
                Token::RightBracket
            ]
        );

        // Test null-like but not exact (should be atoms)
        assert_eq!(tokenize("NULL").unwrap(), vec![Token::Atom("NULL".to_string())]);
        assert_eq!(tokenize("nil").unwrap(), vec![Token::Atom("nil".to_string())]);
        assert_eq!(tokenize("null-check").unwrap(), vec![Token::Atom("null-check".to_string())]);
    }

    #[test]
    fn test_tokenize_boolean_null_edge_cases() {
        // Test with whitespace
        assert_eq!(
            tokenize("  true   false   null  ").unwrap(),
            vec![Token::Boolean(true), Token::Boolean(false), Token::Null]
        );

        // Test adjacent to special characters
        assert_eq!(
            tokenize("true[false]null").unwrap(),
            vec![
                Token::Boolean(true),
                Token::LeftBracket,
                Token::Boolean(false),
                Token::RightBracket,
                Token::Null
            ]
        );

        // Test with quotes
        assert_eq!(
            tokenize("'true 'false 'null").unwrap(),
            vec![
                Token::Quote,
                Token::Boolean(true),
                Token::Quote,
                Token::Boolean(false),
                Token::Quote,
                Token::Null
            ]
        );
    }
}