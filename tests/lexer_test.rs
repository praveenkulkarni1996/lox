use lox::{lexer::{Lexer, Token}};

fn lex(input: &str) -> Vec<Token> {
    Lexer::new(input.chars())
        .filter(|t| !matches!(t, Token::NoOp))
        .collect()
}

#[test]
fn test_true_keyword() {
    let tokens = lex("true");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::True));
}

#[test]
fn test_false_keyword() {
    let tokens = lex("false");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::False));
}

#[test]
fn test_true_and_false() {
    let tokens = lex("true false");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_true_not_identifier() {
    let tokens = lex("truee");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::Identifier(ref s) if s == "truee"));
}
