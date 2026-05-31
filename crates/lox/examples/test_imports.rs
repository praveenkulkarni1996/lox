use lox::{Lexer, Token};

fn main() {
    let tokens: Vec<Token> = Lexer::new("true false".chars()).collect();
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
    println!("Backwards-compatible imports work!");
}
