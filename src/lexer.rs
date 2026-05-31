/// This is solving the lexing (== scanning) part of Crafting Interpreters.
/// The implementation is derived from:
/// * https://craftinginterpreters.com/scanning.html#reserved-words-and-identifiers
use derive_more; // 2.1.1
#[derive(Debug, derive_more::Display)]
pub enum Token {
    // NoOp is a special enum that I've used, but it is not part of the original specification.
    NoOp,
    Error,

    String(String),
    Identifier(String),
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
                // Whitespace
                ' ' => Some(Token::NoOp),
                '\n' => Some(Token::NoOp),
                '\t' => Some(Token::NoOp),
                '\r' => Some(Token::NoOp),
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
