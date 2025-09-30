// Temporary new tokenizer implementation with complete position tracking
use std::fmt;

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
    Number(f64),
    Atom(String),
    String(String), // Quoted strings - not interned
    Boolean(bool),  // Boolean literals: true, false
    Null,           // Null literal
    LeftBracket,
    ArrayLeftBracket,
    RightBracket,
    Quote,
    Dot, // For cons pair notation like [1 . rest]
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Atom(s) => write!(f, "{}", s),
            TokenKind::String(s) => write!(f, "\"{}\"", s),
            TokenKind::Boolean(b) => write!(f, "{}", if *b { "true" } else { "false" }),
            TokenKind::Null => write!(f, "null"),
            TokenKind::LeftBracket => write!(f, "["),
            TokenKind::ArrayLeftBracket => write!(f, "#["),
            TokenKind::RightBracket => write!(f, "]"),
            TokenKind::Quote => write!(f, "'"),
            TokenKind::Dot => write!(f, "."),
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
                            if ch.is_whitespace() || "[].\'\"\\\\".contains(ch) {
                                break;
                            }
                            atom.push(ch);
                            let consumed = chars.next().unwrap();
                            advance_pos(consumed, &mut line, &mut column, &mut offset);
                        }
                        tokens.push(Token::new(
                            TokenKind::Atom(atom),
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

            '.' => {
                let consumed = chars.next().unwrap();
                advance_pos(consumed, &mut line, &mut column, &mut offset);

                // Check if it's a standalone dot (for cons notation) or part of a number
                if chars
                    .peek()
                    .is_none_or(|&c| c.is_whitespace() || "[]\'\"\\\\".contains(c))
                {
                    tokens.push(Token::new(
                        TokenKind::Dot,
                        SourcePos::new(start_line, start_column, start_offset),
                        SourcePos::new(line, column, offset),
                    ));
                } else if chars.peek().is_some_and(|&c| c.is_ascii_digit()) {
                    // It's a decimal number like .5
                    let mut num_str = String::from("0.");
                    while let Some(&ch) = chars.peek() {
                        if ch.is_ascii_digit() || ch == 'e' || ch == 'E' || ch == '-' || ch == '+' {
                            num_str.push(ch);
                            let consumed = chars.next().unwrap();
                            advance_pos(consumed, &mut line, &mut column, &mut offset);
                        } else {
                            break;
                        }
                    }
                    match num_str.parse::<f64>() {
                        Ok(num) => tokens.push(Token::new(
                            TokenKind::Number(num),
                            SourcePos::new(start_line, start_column, start_offset),
                            SourcePos::new(line, column, offset),
                        )),
                        Err(_) => {
                            return Err(format!(
                                "Invalid number: {} at line {}, column {}",
                                num_str, start_line, start_column
                            ));
                        }
                    }
                } else {
                    // It's a dot followed by non-digit, treat as atom
                    let mut atom = String::from(".");
                    while let Some(&ch) = chars.peek() {
                        if ch.is_whitespace() || "[]\'\"\\\\".contains(ch) {
                            break;
                        }
                        atom.push(ch);
                        let consumed = chars.next().unwrap();
                        advance_pos(consumed, &mut line, &mut column, &mut offset);
                    }
                    tokens.push(Token::new(
                        TokenKind::Atom(atom),
                        SourcePos::new(start_line, start_column, start_offset),
                        SourcePos::new(line, column, offset),
                    ));
                }
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
                            if ch.is_whitespace() || "[].\'\"\\\\".contains(ch) {
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
                            if ch.is_whitespace() || "[].\'\"\\\\".contains(ch) {
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

            _ => {
                let mut atom = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_whitespace() || "[].\\'\"\\\\".contains(ch) {
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
        let tokens = tokenize("42").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Number(n) if n == 42.0));
        assert_eq!(tokens[0].pos.line, 1);
        assert_eq!(tokens[0].pos.column, 1);

        let tokens = tokenize("3.14").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Number(n) if n == 3.14));

        let tokens = tokenize("-17").unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0].kind, TokenKind::Number(n) if n == -17.0));
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
        assert!(matches!(tokens[1].kind, TokenKind::Number(n) if n == 1.0));
        assert!(matches!(tokens[2].kind, TokenKind::Number(n) if n == 2.0));
        assert!(matches!(tokens[3].kind, TokenKind::Number(n) if n == 3.0));
        assert!(matches!(tokens[4].kind, TokenKind::RightBracket));
    }

    #[test]
    fn test_tokenize_array_literals() {
        let tokens = tokenize("#[1 2]").unwrap();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0].kind, TokenKind::ArrayLeftBracket));
        assert!(matches!(tokens[1].kind, TokenKind::Number(n) if n == 1.0));
        assert!(matches!(tokens[2].kind, TokenKind::Number(n) if n == 2.0));
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
