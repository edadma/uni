// Temporary new tokenizer implementation with complete position tracking
use crate::compat::{fmt, String, ToString, Vec};

// RUST CONCEPT: Source position for rich error messages
#[derive(Debug, Clone, PartialEq)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
    pub offset: usize, // Byte offset from start of input
}

impl SourcePos {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

// RUST CONCEPT: Token with embedded source position
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: SourcePos,
    pub end_pos: SourcePos,
}

impl Token {
    pub fn new(kind: TokenKind, pos: SourcePos, end_pos: SourcePos) -> Self {
        Self { kind, pos, end_pos }
    }

    // TODO: Method for calculating token span length - uncomment when implementing syntax highlighting or error spans
    // pub fn span_len(&self) -> usize {
    //     self.end_pos.offset - self.pos.offset
    // }

    // TODO: Simple token factory for tests - uncomment when needed for simplified test token creation
    // pub fn simple(kind: TokenKind) -> Self {
    //     Self::new(kind, SourcePos::new(1, 1, 0), SourcePos::new(1, 1, 0))
    // }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Number(f64),           // Float literal (has decimal point or scientific notation)
    Integer(String),       // Integer literal (no decimal point)
    BigInt(String),        // Explicit BigInt with 'n' suffix (e.g., 123n)
    Rational(String, String), // Rational literal (e.g., 3/4 -> ("3", "4"))
    #[cfg(feature = "complex_numbers")]
    GaussianInt(String, String), // Gaussian integer (e.g., 3+4i -> ("3", "4"))
    #[cfg(feature = "complex_numbers")]
    Complex(String, String),     // Complex float (e.g., 3.0+4.0i -> ("3.0", "4.0"))
    Atom(String),
    String(String), // Quoted strings - not interned
    Boolean(bool),  // Boolean literals: true, false
    Null,           // Null literal
    LeftBracket,
    ArrayLeftBracket,
    RightBracket,
    Quote,
    Pipe, // For cons pair notation like [1 | rest]
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Integer(s) => write!(f, "{}", s),
            TokenKind::BigInt(s) => write!(f, "{}n", s),
            TokenKind::Rational(n, d) => write!(f, "{}/{}", n, d),
            #[cfg(feature = "complex_numbers")]
            TokenKind::GaussianInt(re, im) => write!(f, "{}+{}i", re, im),
            #[cfg(feature = "complex_numbers")]
            TokenKind::Complex(re, im) => write!(f, "{}+{}i", re, im),
            TokenKind::Atom(s) => write!(f, "{}", s),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            TokenKind::Null => write!(f, "null"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::ArrayLeftBracket => write!(f, "#["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Quote => write!(f, "'"),
            TokenKind::Pipe => write!(f, "|"),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut line = 1;
    let mut column = 1;
    let mut offset = 0;

    // Helper function to advance position tracking
    fn advance_pos(ch: char, line: &mut usize, column: &mut usize, offset: &mut usize) {
        if ch == '\n' {
            *line += 1;
            *column = 1;
        } else {
            *column += 1;
        }
        *offset += ch.len_utf8();
    }

    // Helper function to classify atom-like strings into appropriate token types
    fn classify_atom(s: String) -> TokenKind {
        // Check for BigInt suffix (e.g., 123n, -456n, 123456789012345678901234567890n)
        if s.ends_with('n') && s.len() > 1 {
            let num_part = &s[..s.len() - 1];
            // Check if it looks like a valid integer (all digits, optionally with leading -)
            let is_integer = if let Some(stripped) = num_part.strip_prefix('-') {
                num_part.len() > 1 && stripped.chars().all(|c| c.is_ascii_digit())
            } else {
                num_part.chars().all(|c| c.is_ascii_digit())
            };

            if is_integer {
                return TokenKind::BigInt(num_part.to_string());
            }
        }

        // Check for rational (e.g., 3/4)
        if s.contains('/') {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2
                && let (Ok(_), Ok(_)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>())
            {
                return TokenKind::Rational(parts[0].to_string(), parts[1].to_string());
            }
        }

        // Check for complex/gaussian (e.g., 3+4i, 3.0+4.0i, 5i)
        #[cfg(feature = "complex_numbers")]
        if s.ends_with('i') && s.len() > 1 {
            let num_part = &s[..s.len() - 1];
            // Find the last + or - that's not at the start
            if let Some(op_pos) = num_part.char_indices().skip(1).find(|(_, c)| *c == '+' || *c == '-').map(|(pos, _)| pos) {
                let real_part = &num_part[..op_pos];
                let imag_part = &num_part[op_pos..];

                // Check if both parts are integers (Gaussian)
                if let (Ok(_), Ok(_)) = (real_part.parse::<i64>(), imag_part.parse::<i64>()) {
                    return TokenKind::GaussianInt(real_part.to_string(), imag_part.to_string());
                }

                // Check if either part is a float (Complex)
                if real_part.parse::<f64>().is_ok() && imag_part.parse::<f64>().is_ok() {
                    return TokenKind::Complex(real_part.to_string(), imag_part.to_string());
                }
            } else {
                // Pure imaginary (e.g., 5i, -5i, 3.5i)
                // Try integer first
                if num_part.parse::<i64>().is_ok() {
                    return TokenKind::GaussianInt("0".to_string(), num_part.to_string());
                }
                // Try float
                if num_part.parse::<f64>().is_ok() {
                    return TokenKind::Complex("0".to_string(), num_part.to_string());
                }
            }
        }

        // Default: it's just an atom
        TokenKind::Atom(s)
    }

    while let Some(&ch) = chars.peek() {
        let start_line = line;
        let start_column = column;
        let start_offset = offset;

        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);
            }

            '[' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);
                tokens.push(Token::new(
                    TokenKind::LeftBracket,
                    SourcePos::new(start_line, start_column, start_offset),
                    SourcePos::new(line, column, offset),
                ));
            }

            '#' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);

                match chars.peek() {
                    Some(&'[') => {
                        let consumed_bracket = chars.next().unwrap();
                        advance_pos(consumed_bracket, &mut line, &mut column, &mut offset);
                        tokens.push(Token::new(
                            TokenKind::ArrayLeftBracket,
                            SourcePos::new(start_line, start_column, start_offset),
                            SourcePos::new(line, column, offset),
                        ));
                    }
                    Some(_) => {
                        let mut atom = String::from("#");
                        while let Some(&ch) = chars.peek() {
                            if ch.is_whitespace() || "[]|\'\"\\\\".contains(ch) {
                                break;
                            }
                            atom.push(ch);
                            let consumed = chars.next().unwrap();
                            advance_pos(consumed, &mut line, &mut column, &mut offset);
                        }
                        tokens.push(Token::new(
                            classify_atom(atom),
                            SourcePos::new(start_line, start_column, start_offset),
                            SourcePos::new(line, column, offset),
                        ));
                    }
                    None => {
                        tokens.push(Token::new(
                            TokenKind::Atom("#".to_string()),
                            SourcePos::new(start_line, start_column, start_offset),
                            SourcePos::new(line, column, offset),
                        ));
                    }
                }
            }

            ']' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);
                tokens.push(Token::new(
                    TokenKind::RightBracket,
                    SourcePos::new(start_line, start_column, start_offset),
                    SourcePos::new(line, column, offset),
                ));
            }

            '\'' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);
                tokens.push(Token::new(
                    TokenKind::Quote,
                    SourcePos::new(start_line, start_column, start_offset),
                    SourcePos::new(line, column, offset),
                ));
            }

            '\\' => {
                // Skip comments - consume everything until newline
                let consumed = chars.next().unwrap(); // consume the backslash
                advance_pos(consumed, &mut line, &mut column, &mut offset);
                for ch in chars.by_ref() {
                    advance_pos(ch, &mut line, &mut column, &mut offset);
                    if ch == '\n' {
                        break;
                    }
                }
                // Continue tokenizing after the comment
            }

            '|' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);
                tokens.push(Token::new(
                    TokenKind::Pipe,
                    SourcePos::new(start_line, start_column, start_offset),
                    SourcePos::new(line, column, offset),
                ));
            }

            '"' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);
                let mut string = String::new();
                let mut escaped = false;

                for ch in chars.by_ref() {
                    advance_pos(ch, &mut line, &mut column, &mut offset);
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

                tokens.push(Token::new(
                    TokenKind::String(string),
                    SourcePos::new(start_line, start_column, start_offset),
                    SourcePos::new(line, column, offset),
                ));
            }

            '+' | '-' if chars.clone().nth(1).is_some_and(|c| c.is_ascii_digit()) => {
                // Handle signed numbers
                let mut num_str = String::new();
                num_str.push(ch);
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                        num_str.push(ch);
                        let consumed = chars.next().unwrap();
                        advance_pos(consumed, &mut line, &mut column, &mut offset);
                        // Allow + or - after e/E for scientific notation
                        if (ch == 'e' || ch == 'E')
                            && chars.peek().is_some_and(|&c| c == '+' || c == '-')
                        {
                            let sign = chars.next().unwrap();
                            num_str.push(sign);
                            advance_pos(sign, &mut line, &mut column, &mut offset);
                        }
                    } else {
                        break;
                    }
                }

                // RUST CONCEPT: Check for extended number suffixes (same as unsigned numbers)
                let has_suffix = chars.peek().is_some_and(|&c| {
                    c == 'n' // BigInt suffix
                        || c == 'i' // Complex imaginary unit
                        || c == '/' // Rational fraction
                        || c == '+' || c == '-' // Complex with separate real/imaginary
                });

                if has_suffix {
                    // Continue collecting as an atom-like string (extended number literal)
                    // Don't break on '.' since we might have decimal complex numbers like "-1.5+2.5i"
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() || "[]|\'\"\\\\".contains(ch) {
                            break;
                        }
                        num_str.push(ch);
                        let consumed = chars.next().unwrap();
                        advance_pos(consumed, &mut line, &mut column, &mut offset);
                    }
                    tokens.push(Token::new(
                        classify_atom(num_str),
                        SourcePos::new(start_line, start_column, start_offset),
                        SourcePos::new(line, column, offset),
                    ));
                } else {
                    // Regular signed number - check if it's an integer or float based on syntax
                    let has_decimal = num_str.contains('.') || num_str.contains('e') || num_str.contains('E');

                    if !has_decimal {
                        // Integer literal - use dedicated Integer token type
                        tokens.push(Token::new(
                            TokenKind::Integer(num_str),
                            SourcePos::new(start_line, start_column, start_offset),
                            SourcePos::new(line, column, offset),
                        ));
                    } else {
                        // Floating point number
                        match num_str.parse::<f64>() {
                            Ok(num) => tokens.push(Token::new(
                                TokenKind::Number(num),
                                SourcePos::new(start_line, start_column, start_offset),
                                SourcePos::new(line, column, offset),
                            )),
                            Err(_) => {
                                // If it's not a valid number, treat it as an atom
                                // Continue collecting non-whitespace chars
                                while let Some(&ch) = chars.peek() {
                                    if ch.is_whitespace() || "[]|\'\"\\\\".contains(ch) {
                                        break;
                                    }
                                    num_str.push(ch);
                                    let consumed = chars.next().unwrap();
                                    advance_pos(consumed, &mut line, &mut column, &mut offset);
                                }
                                tokens.push(Token::new(
                                    TokenKind::Atom(num_str),
                                    SourcePos::new(start_line, start_column, start_offset),
                                    SourcePos::new(line, column, offset),
                                ));
                            }
                        }
                    }
                }
            }

            '0'..='9' => {
                let mut num_str = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' {
                        num_str.push(ch);
                        let consumed = chars.next().unwrap();
                        advance_pos(consumed, &mut line, &mut column, &mut offset);
                        // Allow + or - after e/E for scientific notation
                        if (ch == 'e' || ch == 'E')
                            && chars.peek().is_some_and(|&c| c == '+' || c == '-')
                        {
                            let sign = chars.next().unwrap();
                            num_str.push(sign);
                            advance_pos(sign, &mut line, &mut column, &mut offset);
                        }
                    } else {
                        break;
                    }
                }

                // RUST CONCEPT: Check for extended number suffixes (n, i, /, +, -)
                // In postfix languages, operators need spaces, so "+"/"-" immediately
                // after a number can only mean complex number syntax (e.g., 1.5+2.5i)
                let has_suffix = chars.peek().is_some_and(|&c| {
                    c == 'n' // BigInt suffix
                        || c == 'i' // Complex imaginary unit
                        || c == '/' // Rational fraction
                        || c == '+' || c == '-' // Complex with separate real/imaginary
                });

                if has_suffix {
                    // Continue collecting as an atom-like string (extended number literal)
                    // Don't break on '.' since we might have decimal complex numbers like "1.5+2.5i"
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() || "[]|\'\"\\\\".contains(ch) {
                            break;
                        }
                        num_str.push(ch);
                        let consumed = chars.next().unwrap();
                        advance_pos(consumed, &mut line, &mut column, &mut offset);
                    }
                    tokens.push(Token::new(
                        classify_atom(num_str),
                        SourcePos::new(start_line, start_column, start_offset),
                        SourcePos::new(line, column, offset),
                    ));
                } else {
                    // Regular number - check if it's an integer or float based on syntax
                    // If no decimal point and no scientific notation, treat as integer
                    let has_decimal = num_str.contains('.') || num_str.contains('e') || num_str.contains('E');

                    if !has_decimal {
                        // Integer literal - use dedicated Integer token type
                        tokens.push(Token::new(
                            TokenKind::Integer(num_str),
                            SourcePos::new(start_line, start_column, start_offset),
                            SourcePos::new(line, column, offset),
                        ));
                    } else {
                        // Floating point number
                        match num_str.parse::<f64>() {
                            Ok(num) => tokens.push(Token::new(
                                TokenKind::Number(num),
                                SourcePos::new(start_line, start_column, start_offset),
                                SourcePos::new(line, column, offset),
                            )),
                            Err(_) => {
                                // If it's not a valid number, treat it as an atom
                                // Continue collecting non-whitespace chars
                                while let Some(&ch) = chars.peek() {
                                    if ch.is_whitespace() || "[]|\'\"\\\\".contains(ch) {
                                        break;
                                    }
                                    num_str.push(ch);
                                    let consumed = chars.next().unwrap();
                                    advance_pos(consumed, &mut line, &mut column, &mut offset);
                                }
                                tokens.push(Token::new(
                                    TokenKind::Atom(num_str),
                                    SourcePos::new(start_line, start_column, start_offset),
                                    SourcePos::new(line, column, offset),
                                ));
                            }
                        }
                    }
                }
            }

            _ => {
                let mut atom = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || "[]|\\'\"\\\\".contains(ch) {
                        break;
                    }
                    atom.push(ch);
                    let consumed = chars.next().unwrap();
                    advance_pos(consumed, &mut line, &mut column, &mut offset);
                }

                if !atom.is_empty() {
                    // RUST CONCEPT: Pattern matching on string literals
                    // Check for special boolean and null literals
                    let token_kind = match atom.as_str() {
                        "true" => TokenKind::Boolean(true),
                        "false" => TokenKind::Boolean(false),
                        "null" => TokenKind::Null,
                        _ => TokenKind::Atom(atom),
                    };
                    tokens.push(Token::new(
                        token_kind,
                        SourcePos::new(start_line, start_column, start_offset),
                        SourcePos::new(line, column, offset),
                    ));
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
        // Integer literal
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, TokenKind::Integer(s) if s == "42"));
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[0].pos.column, 1);

        // Float literal
        let tokens = tokenize("3.14").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Number(n) if n == 3.14));

        // Negative integer
        let tokens = tokenize("-17").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, TokenKind::Integer(s) if s == "-17"));
    }

    #[test]
    fn test_tokenize_atoms() {
        let tokens = tokenize("hello").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, TokenKind::Atom(s) if s == "hello"));

        let tokens = tokenize("+").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, TokenKind::Atom(s) if s == "+"));
    }

    #[test]
    fn test_tokenize_strings() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0].kind, TokenKind::String(s) if s == "hello world"));
    }

    #[test]
    fn test_tokenize_brackets() {
        let tokens = tokenize("[1 2 3]").unwrap();
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0].kind, TokenKind::LeftBracket));
        assert!(matches!(&tokens[1].kind, TokenKind::Integer(s) if s == "1"));
        assert!(matches!(&tokens[2].kind, TokenKind::Integer(s) if s == "2"));
        assert!(matches!(&tokens[3].kind, TokenKind::Integer(s) if s == "3"));
        assert!(matches!(tokens[4].kind, TokenKind::RightBracket));
    }

    #[test]
    fn test_tokenize_array_literals() {
        let tokens = tokenize("#[1 2]").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].kind, TokenKind::ArrayLeftBracket));
        assert!(matches!(&tokens[1].kind, TokenKind::Integer(s) if s == "1"));
        assert!(matches!(&tokens[2].kind, TokenKind::Integer(s) if s == "2"));
        assert!(matches!(tokens[3].kind, TokenKind::RightBracket));
    }

    #[test]
    fn test_tokenize_position_tracking() {
        let tokens = tokenize("hello\nworld").unwrap();
        assert_eq!(tokens.len(), 2);

        // First token "hello" at line 1, column 1
        assert!(matches!(&tokens[0].kind, TokenKind::Atom(s) if s == "hello"));
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[0].pos.column, 1);

        // Second token "world" at line 2, column 1
        assert!(matches!(&tokens[1].kind, TokenKind::Atom(s) if s == "world"));
        assert_eq!(tokens[1].pos.line, 2);
        assert_eq!(tokens[1].pos.column, 1);
    }

    #[test]
    fn test_tokenize_booleans_and_null() {
        let tokens = tokenize("true false null").unwrap();
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0].kind, TokenKind::Boolean(true)));
        assert!(matches!(tokens[1].kind, TokenKind::Boolean(false)));
        assert!(matches!(tokens[2].kind, TokenKind::Null));
    }

    #[test]
    fn test_simple_token_helper() {
        let pos = SourcePos::new(1, 1, 0);
        let token = Token::new(TokenKind::Number(42.0), pos.clone(), pos);
        assert!(matches!(token.kind, TokenKind::Number(n) if n == 42.0));
        assert_eq!(token.pos.line, 1);
        assert_eq!(token.pos.column, 1);
    }
}
