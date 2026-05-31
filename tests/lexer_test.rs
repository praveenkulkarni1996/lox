use lox::{lexer::{Lexer, Token}};

fn lex(input: &str) -> Vec<Token> {
    Lexer::new(input.chars()).collect()
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

#[test]
fn test_whitespace_between_tokens() {
    let tokens = lex("true false");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_multiple_spaces() {
    let tokens = lex("true    false");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_tabs_between_tokens() {
    let tokens = lex("true\tfalse");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_newlines_between_tokens() {
    let tokens = lex("true\nfalse");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_carriage_returns() {
    let tokens = lex("true\rfalse");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_mixed_whitespace() {
    let tokens = lex("true \t\n\r false");
    assert_eq!(tokens.len(), 2);
    assert!(matches!(tokens[0], Token::True));
    assert!(matches!(tokens[1], Token::False));
}

#[test]
fn test_leading_whitespace() {
    let tokens = lex("  true");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::True));
}

#[test]
fn test_trailing_whitespace() {
    let tokens = lex("true  ");
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0], Token::True));
}

#[test]
fn test_only_whitespace() {
    let tokens = lex("   \t\n\r  ");
    assert_eq!(tokens.len(), 0);
}
