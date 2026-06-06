use std::iter::Iterator;

type AstIdentifier = String;

pub enum AstPrimary {
    Number(f64),
    Str(String),
    True,
    False,
    Nil,
    Group(Box<AstExpression>),
    Id(AstIdentifier),
}

pub enum AstUnary {
    Primary(AstPrimary),
    Not(Box<AstUnary>),
    Negative(Box<AstUnary>),
}

pub enum AstFactor {
    Unary(AstUnary),
    Mul(Box<AstFactor>, AstUnary),
    Div(Box<AstFactor>, AstUnary),
}

pub enum AstTerm {
    Factor(AstFactor),
    Add(Box<AstTerm>, AstFactor),
    Sub(Box<AstTerm>, AstFactor),
}
pub enum AstComparison {
    Term(AstTerm),
    GreaterEqual(Box<AstComparison>, AstTerm),
    Greater(Box<AstComparison>, AstTerm),
    LessEqual(Box<AstComparison>, AstTerm),
    Less(Box<AstComparison>, AstTerm),
}

pub enum AstEquality {
    Comparison(AstComparison),
    Equal(Box<AstEquality>, AstComparison),
    NotEqual(Box<AstEquality>, AstComparison),
}

pub enum AstLogicAnd {
    And(AstEquality, Option<Box<AstLogicAnd>>),
}

pub enum AstLogicOr {
    Or(AstLogicAnd, Option<Box<AstLogicOr>>),
}

pub enum AstAssignment {
    Assign(AstIdentifier, Box<AstAssignment>),
    LogicOr(AstLogicOr),
}

pub enum AstExpression {
    Assignment(AstAssignment),
}

pub enum AstStatement {
    Expr(AstExpression),
    Print(AstExpression),
    Block(Vec<AstDeclaration>),
    If(AstExpression, Box<AstStatement>, Option<Box<AstStatement>>),
}

pub enum AstDeclaration {
    VarDeclare(AstIdentifier, AstExpression),
    Statement(AstStatement),
}

pub enum Ast {
    Declare(AstDeclaration),
}

pub struct Parser<I>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    tokens: std::iter::Peekable<I>,
}

impl<I> Parser<I>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    pub fn new(tokens: I) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }
}

/// Parse a source string into its first top-level AST node.
///
/// Returns `None` on a parse error or empty input. This is a convenience
/// wrapper that lexes `input` and pulls a single declaration from the parser.
///
// TODO: This only returns the first declaration. Rename it (e.g.
// `parse_first`) or extend it to parse a whole program once multi-statement
// programs need a single entry point.
pub fn parse(input: &str) -> Option<Ast> {
    let mut parser = Parser::new(lox_lexer::Lexer::new(input.chars()));
    parser.next()
}

fn parse_id(head: lox_lexer::Token) -> Option<AstIdentifier> {
    match head {
        lox_lexer::Token::Identifier(var) => Some(var),
        _ => None,
    }
}

fn parse_primary<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstPrimary>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    match head {
        lox_lexer::Token::Number(num) => Some(AstPrimary::Number(num)),
        lox_lexer::Token::String(s) => Some(AstPrimary::Str(s)),
        lox_lexer::Token::True => Some(AstPrimary::True),
        lox_lexer::Token::False => Some(AstPrimary::False),
        lox_lexer::Token::Nil => Some(AstPrimary::Nil),
        lox_lexer::Token::Identifier(_) => Some(AstPrimary::Id(parse_id(head)?)),
        lox_lexer::Token::LParens => {
            let expr = parse_expr(p.tokens.next()?, p)?;
            match p.tokens.next()? {
                lox_lexer::Token::RParens => Some(AstPrimary::Group(Box::new(expr))),
                _ => None,
            }
        }
        _ => None,
    }
}

fn parse_unary<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstUnary>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    match head {
        lox_lexer::Token::Bang => {
            let next = p.tokens.next()?;
            let unary = parse_unary(next, p)?;
            Some(AstUnary::Not(Box::new(unary)))
        }
        lox_lexer::Token::Minus => {
            let next = p.tokens.next()?;
            let unary = parse_unary(next, p)?;
            Some(AstUnary::Negative(Box::new(unary)))
        }
        _ => {
            let primary = parse_primary(head, p)?;
            Some(AstUnary::Primary(primary))
        }
    }
}

fn parse_factor<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstFactor>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut lhs = {
        let unary = parse_unary(head, p)?;
        AstFactor::Unary(unary)
    };

    while let Some(token) = p.tokens.peek() {
        match token {
            lox_lexer::Token::Slash => {
                let _operator = p.tokens.next();
                let rhs = parse_unary(p.tokens.next()?, p)?;
                lhs = AstFactor::Div(Box::new(lhs), rhs);
            }
            lox_lexer::Token::Star => {
                let _operator = p.tokens.next();
                let rhs = parse_unary(p.tokens.next()?, p)?;
                lhs = AstFactor::Mul(Box::new(lhs), rhs);
            }
            _ => return Some(lhs),
        }
    }
    Some(lhs)
}

fn parse_term<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstTerm>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut lhs = AstTerm::Factor(parse_factor(head, p)?);
    while let Some(token) = p.tokens.peek() {
        match token {
            lox_lexer::Token::Minus => {
                let _operator = p.tokens.next();
                let rhs = parse_factor(p.tokens.next()?, p)?;
                lhs = AstTerm::Sub(Box::new(lhs), rhs);
            }
            lox_lexer::Token::Plus => {
                let _operator = p.tokens.next();
                let rhs = parse_factor(p.tokens.next()?, p)?;
                lhs = AstTerm::Add(Box::new(lhs), rhs);
            }
            _ => return Some(lhs),
        }
    }
    Some(lhs)
}

fn parse_comparison<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstComparison>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut lhs = AstComparison::Term(parse_term(head, p)?);
    while let Some(token) = p.tokens.peek() {
        match token {
            lox_lexer::Token::Greater => {
                let _operator = p.tokens.next();
                let rhs = parse_term(p.tokens.next()?, p)?;
                lhs = AstComparison::Greater(Box::new(lhs), rhs);
            }
            lox_lexer::Token::GreaterEqual => {
                let _operator = p.tokens.next();
                let rhs = parse_term(p.tokens.next()?, p)?;
                lhs = AstComparison::GreaterEqual(Box::new(lhs), rhs);
            }
            lox_lexer::Token::Less => {
                let _operator = p.tokens.next();
                let rhs = parse_term(p.tokens.next()?, p)?;
                lhs = AstComparison::Less(Box::new(lhs), rhs);
            }
            lox_lexer::Token::LessEqual => {
                let _operator = p.tokens.next();
                let rhs = parse_term(p.tokens.next()?, p)?;
                lhs = AstComparison::LessEqual(Box::new(lhs), rhs);
            }
            _ => return Some(lhs),
        }
    }
    Some(lhs)
}

fn parse_equality<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstEquality>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut lhs: AstEquality = AstEquality::Comparison(parse_comparison(head, p)?);
    while let Some(token) = p.tokens.peek() {
        match token {
            lox_lexer::Token::EqualEqual => {
                let _operator = p.tokens.next();
                let rhs = parse_comparison(p.tokens.next()?, p)?;
                lhs = AstEquality::Equal(Box::new(lhs), rhs);
            }
            lox_lexer::Token::BangEqual => {
                let _operator = p.tokens.next();
                let rhs = parse_comparison(p.tokens.next()?, p)?;
                lhs = AstEquality::NotEqual(Box::new(lhs), rhs);
            }
            _ => return Some(lhs),
        }
    }
    Some(lhs)
}

fn parse_logic_and<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstLogicAnd>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let equality = parse_equality(head, p)?;
    let tail = if p.tokens.next_if_eq(&lox_lexer::Token::And).is_some() {
        Some(Box::new(parse_logic_and(p.tokens.next()?, p)?))
    } else {
        None
    };
    Some(AstLogicAnd::And(equality, tail))
}

fn parse_logic_or<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstLogicOr>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let logic_and = parse_logic_and(head, p)?;
    let tail = if p.tokens.next_if_eq(&lox_lexer::Token::Or).is_some() {
        Some(Box::new(parse_logic_or(p.tokens.next()?, p)?))
    } else {
        None
    };
    Some(AstLogicOr::Or(logic_and, tail))
}

/// See https://craftinginterpreters.com/statements-and-state.html#assignment-syntax
fn parse_assignment<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstAssignment>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let logic_or = parse_logic_or(head, p);
    if let Some(_unused) = p.tokens.next_if_eq(&lox_lexer::Token::Equal) {
        match logic_or {
            Some(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Primary(AstPrimary::Id(identifier))),
                    ))),
                    None,
                ),
                None,
            )) => {
                let value = parse_assignment(p.tokens.next()?, p)?;
                Some(AstAssignment::Assign(identifier, Box::new(value)))
            }
            // ERROR: Invalid assignment target
            // Convert to an error.
            _ => None,
        }
    } else {
        Some(AstAssignment::LogicOr(logic_or?))
    }
}

fn parse_expr<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstExpression>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    Some(AstExpression::Assignment(parse_assignment(head, p)?))
}

fn parse_statement<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstStatement>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    match head {
        lox_lexer::Token::Print => {
            let expr = parse_expr(p.tokens.next()?, p)?;
            match p.tokens.next()? {
                lox_lexer::Token::Semicolon => Some(AstStatement::Print(expr)),
                _ => None,
            }
        }
        lox_lexer::Token::LBrace => {
            let mut decls = vec![];
            while let Some(tok) = p.tokens.peek() {
                if tok == &lox_lexer::Token::RBrace {
                    break;
                }
                let head = p.tokens.next()?;
                decls.push(parse_declaration(head, p)?);
            }
            p.tokens.next_if_eq(&lox_lexer::Token::RBrace)?;
            Some(AstStatement::Block(decls))
        }
        lox_lexer::Token::If => {
            p.tokens.next_if_eq(&lox_lexer::Token::LParens)?;
            let condition = parse_expr(p.tokens.next()?, p)?;
            p.tokens.next_if_eq(&lox_lexer::Token::RParens)?;
            let then_branch = parse_statement(p.tokens.next()?, p)?;
            let else_branch = match p.tokens.next_if_eq(&lox_lexer::Token::Else) {
                Some(_) => Some(Box::new(parse_statement(p.tokens.next()?, p)?)),
                None => None,
            };
            Some(AstStatement::If(
                condition,
                Box::new(then_branch),
                else_branch,
            ))
        }
        _ => {
            let expr = parse_expr(head, p)?;
            match p.tokens.next()? {
                lox_lexer::Token::Semicolon => Some(AstStatement::Expr(expr)),
                _ => None,
            }
        }
    }
}

fn parse_declaration<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstDeclaration>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    match head {
        lox_lexer::Token::Var => {
            let identifier: AstIdentifier = parse_id(p.tokens.next()?)?;
            match p.tokens.next()? {
                lox_lexer::Token::Equal => {}
                _ => {
                    return None;
                }
            }
            let initializer = parse_expr(p.tokens.next()?, p)?;
            match p.tokens.next()? {
                lox_lexer::Token::Semicolon => {
                    Some(AstDeclaration::VarDeclare(identifier, initializer))
                }
                _ => None,
            }
        }
        _ => Some(AstDeclaration::Statement(parse_statement(head, p)?)),
    }
}

impl<I> Iterator for Parser<I>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    type Item = Ast;

    fn next(&mut self) -> Option<Ast> {
        let declaration = parse_declaration(self.tokens.next()?, self)?;
        Some(Ast::Declare(declaration))
    }
}
