//! Lexical analysis (scanning) for the Lox language.
//!
//! This module tokenizes Lox source code into a stream of tokens. It handles all lexical elements
//! including keywords, identifiers, operators, numbers, strings, and comments.
//!
//! The implementation follows the principles from
//! [Crafting Interpreters](https://craftinginterpreters.com/scanning.html#reserved-words-and-identifiers),
//! specifically the maximal munch principle for keyword recognition.
//!
//! # Examples
//!
//! ```
//! use lox_lexer::{Lexer, Token};
//!
//! let tokens: Vec<Token> = Lexer::new("true false 42".chars()).collect();
//! assert!(matches!(tokens[0], Token::True));
//! assert!(matches!(tokens[1], Token::False));
//! assert!(matches!(tokens[2], Token::Number(42.0)));
//! ```

/// A token from the Lox source code.
///
/// Tokens represent the smallest meaningful units of source code: keywords, identifiers,
/// literals, operators, and punctuation. Whitespace is automatically skipped by the lexer.
/// Most variants are self-explanatory by name (e.g., `LBrace` for `{`, `True` for the
/// `true` keyword). `Error` represents a lexically invalid character or sequence.
#[derive(Debug, derive_more::Display, PartialEq)]
pub enum Token {
    Error,

    /// A string literal, e.g., `"hello"`.
    String(String),
    /// An identifier or variable name, e.g., `foo`, `_x`, `myVar123`.
    Identifier(String),
    /// A numeric literal, e.g., `42`, `3.14`.
    Number(f64),

    // Symbols and Operators
    LBrace,
    RBrace,
    LParens,
    RParens,
    Comma,
    Minus,
    Plus,
    Semicolon,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Slash,

    // Keywords
    And,
    Class,
    Else,
    False,
    For,
    Fun,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
}

fn try_keywordify(id: String) -> Token {
    match id.as_str() {
        "and" => Token::And,
        "class" => Token::Class,
        "else" => Token::Else,
        "false" => Token::False,
        "for" => Token::For,
        "fun" => Token::Fun,
        "if" => Token::If,
        "nil" => Token::Nil,
        "or" => Token::Or,
        "print" => Token::Print,
        "return" => Token::Return,
        "super" => Token::Super,
        "this" => Token::This,
        "true" => Token::True,
        "var" => Token::Var,
        "while" => Token::While,
        _ => Token::Identifier(id),
    }
}

fn scan_string(prefix: String, mut chars: impl Iterator<Item = char>) -> Token {
    match chars.next() {
        Some('\"') => Token::String(prefix),
        Some(x) => scan_string(prefix + &x.to_string(), chars),
        None => Token::Error,
    }
}

fn scan_identifier(
    prefix: String,
    chars: &mut std::iter::Peekable<impl Iterator<Item = char>>,
) -> Token {
    if let Some(c) = chars.peek()
        && (c.is_ascii_alphanumeric() || *c == '_')
    {
        scan_identifier(prefix + &chars.next().unwrap().to_string(), chars)
    } else {
        try_keywordify(prefix)
    }
}

fn scan_comment(mut chars: impl Iterator<Item = char>) {
    while let Some(char) = chars.next()
        && char != '\n'
    {
        continue;
    }
}

fn scan_number(start: f64, chars: &mut std::iter::Peekable<impl Iterator<Item = char>>) -> Token {
    let mut number = start;
    while let Some(c) = chars.peek()
        && '0' <= *c
        && *c <= '9'
    {
        let digit: f64 = chars.next().unwrap().to_digit(10).unwrap().into();
        number = (number * 10.0) + digit;
    }
    // if no decimal point - return early
    if let Some(c) = chars.peek()
        && *c != '.'
    {
        return Token::Number(number);
    }

    // TODO:
    // I think there is a bug here, about trailing "." at the end of a file.
    // In particular, we need to peek TWO characters in advance, which peekable does not
    // support.
    chars.next();
    if let Some(c) = chars.peek()
        && !('0' <= *c && *c <= '9')
    {
        return Token::Error;
    }

    let mut position: f64 = 1.0;
    while let Some(c) = chars.peek()
        && '0' <= *c
        && *c <= '9'
    {
        let digit: f64 = chars.next().unwrap().to_digit(10).unwrap().into();
        position *= 0.1;
        number += digit * position;
    }
    Token::Number(number)
}

/// A lexical analyzer that transforms a character stream into tokens.
///
/// `Lexer` implements the `Iterator` trait, yielding tokens one at a time as they are scanned
/// from the input. It handles all aspects of tokenization including string and number parsing,
/// keyword recognition, and operator disambiguation.
///
/// # Generic Parameters
///
/// * `I` - The underlying character iterator (typically `Chars` or a slice iterator)
///
/// # Examples
///
/// ```
/// use lox_lexer::{Lexer, Token};
///
/// let source = "var x = 42;";
/// let mut lexer = Lexer::new(source.chars());
///
/// assert!(matches!(lexer.next(), Some(Token::Var)));
/// assert!(matches!(lexer.next(), Some(Token::Identifier(ref s)) if s == "x"));
/// assert!(matches!(lexer.next(), Some(Token::Equal)));
/// ```
pub struct Lexer<I>
where
    I: Iterator<Item = char>,
{
    chars: std::iter::Peekable<I>,
}

impl<I> Lexer<I>
where
    I: Iterator<Item = char>,
{
    /// Creates a new lexer from a character iterator.
    ///
    /// # Arguments
    ///
    /// * `chars` - An iterator yielding characters from the source code
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_lexer::Lexer;
    ///
    /// let lexer = Lexer::new("true".chars());
    /// // Lexer is ready to be iterated
    /// ```
    pub fn new(chars: I) -> Self {
        Lexer {
            chars: chars.peekable(),
        }
    }
}

impl<I> Iterator for Lexer<I>
where
    I: Iterator<Item = char>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        if let Some(me) = self.chars.next() {
            return match me {
                // Whitespace - skip and continue to next token
                ' ' | '\n' | '\t' | '\r' => self.next(),
                // String Scanning
                '\"' => Some(scan_string(String::new(), &mut self.chars)),
                // Unambiguous Operators
                '(' => Some(Token::LParens),
                ')' => Some(Token::RParens),
                '{' => Some(Token::LBrace),
                '}' => Some(Token::RBrace),
                ',' => Some(Token::Comma),
                '-' => Some(Token::Minus),
                '+' => Some(Token::Plus),
                ';' => Some(Token::Semicolon),
                '*' => Some(Token::Star),
                // Ambiguous Operators !=, ==, <=, >=
                '!' => match self.chars.peek() {
                    Some('=') => {
                        self.chars.next();
                        Some(Token::BangEqual)
                    }
                    _ => Some(Token::Bang),
                },
                '=' => match self.chars.peek() {
                    Some('=') => {
                        self.chars.next();
                        Some(Token::EqualEqual)
                    }
                    _ => Some(Token::Equal),
                },
                '<' => match self.chars.peek() {
                    Some('=') => {
                        self.chars.next();
                        Some(Token::LessEqual)
                    }
                    _ => Some(Token::Less),
                },
                '>' => match self.chars.peek() {
                    Some('=') => {
                        self.chars.next();
                        Some(Token::GreaterEqual)
                    }
                    _ => Some(Token::Greater),
                },
                // comment (//) or division(/)
                '/' => match self.chars.peek() {
                    Some('/') => {
                        scan_comment(&mut self.chars);
                        self.next()
                    }
                    _ => Some(Token::Slash),
                },
                // Numbers
                '0'..='9' => Some(scan_number(
                    me.to_digit(10).unwrap().into(),
                    &mut self.chars,
                )),
                'a'..='z' | 'A'..='Z' | '_' => {
                    Some(scan_identifier(me.to_string(), &mut self.chars))
                }
                _ => Some(Token::Error),
            };
        }
        None
    }
}
