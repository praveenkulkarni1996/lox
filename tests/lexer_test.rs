use lox::lexer::{Lexer, Token};

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

#[test]
fn test_complex_input() {
    let input = "++// hello
        _AbC()<=<> =>!!===23.123if\"Hello+;-*,\"+;-*,abd_";
    let tokens = lex(input);

    assert_eq!(tokens.len(), 22);
    assert!(matches!(tokens[0], Token::Plus));
    assert!(matches!(tokens[1], Token::Plus));
    assert!(matches!(tokens[2], Token::Identifier(ref s) if s == "_AbC"));
    assert!(matches!(tokens[3], Token::LParens));
    assert!(matches!(tokens[4], Token::RParens));
    assert!(matches!(tokens[5], Token::LessEqual));
    assert!(matches!(tokens[6], Token::Less));
    assert!(matches!(tokens[7], Token::Greater));
    assert!(matches!(tokens[8], Token::Equal));
    assert!(matches!(tokens[9], Token::Greater));
    assert!(matches!(tokens[10], Token::Bang));
    assert!(matches!(tokens[11], Token::BangEqual));
    assert!(matches!(tokens[12], Token::EqualEqual));
    assert!(matches!(tokens[13], Token::Number(23.123)));
    assert!(matches!(tokens[14], Token::If));
    assert!(matches!(tokens[15], Token::String(ref s) if s == "Hello+;-*,"));
    assert!(matches!(tokens[16], Token::Plus));
    assert!(matches!(tokens[17], Token::Semicolon));
    assert!(matches!(tokens[18], Token::Minus));
    assert!(matches!(tokens[19], Token::Star));
    assert!(matches!(tokens[20], Token::Comma));
    assert!(matches!(tokens[21], Token::Identifier(ref s) if s == "abd_"));
}
