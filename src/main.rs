use lox::lexer;

fn main() {
    let input = String::from(
        "++// hello
        _AbC()<=<> =>!!===23.123if\"Hello+;-*,\"+;-*,abd_",
    );

    dbg!(&input);
    let lexer = lexer::Lexer::new(input.chars());
    dbg!(lexer.collect::<Vec<lexer::Token>>());
}
