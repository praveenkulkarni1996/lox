use std::iter::Iterator;
use std::rc::Rc;

type AstIdentifier = String;

#[derive(Debug, PartialEq)]
pub enum AstPrimary {
    Number(f64),
    Str(String),
    True,
    False,
    Nil,
    Group(Box<AstExpression>),
    Id(AstIdentifier),
}

#[derive(Debug, PartialEq)]
pub enum AstCall {
    Primary(AstPrimary),
    /// A call expression: `callee(arguments)`. Calls chain left-associatively,
    /// so `f()()` nests as `Call(Call(Primary(f), []), [])`.
    Call(Box<AstCall>, Vec<AstExpression>),
}

#[derive(Debug, PartialEq)]
pub enum AstUnary {
    Call(AstCall),
    Not(Box<AstUnary>),
    Negative(Box<AstUnary>),
}

#[derive(Debug, PartialEq)]
pub enum AstFactor {
    Unary(AstUnary),
    Mul(Box<AstFactor>, AstUnary),
    Div(Box<AstFactor>, AstUnary),
}

#[derive(Debug, PartialEq)]
pub enum AstTerm {
    Factor(AstFactor),
    Add(Box<AstTerm>, AstFactor),
    Sub(Box<AstTerm>, AstFactor),
}
#[derive(Debug, PartialEq)]
pub enum AstComparison {
    Term(AstTerm),
    GreaterEqual(Box<AstComparison>, AstTerm),
    Greater(Box<AstComparison>, AstTerm),
    LessEqual(Box<AstComparison>, AstTerm),
    Less(Box<AstComparison>, AstTerm),
}

#[derive(Debug, PartialEq)]
pub enum AstEquality {
    Comparison(AstComparison),
    Equal(Box<AstEquality>, AstComparison),
    NotEqual(Box<AstEquality>, AstComparison),
}

#[derive(Debug, PartialEq)]
pub enum AstLogicAnd {
    And(AstEquality, Option<Box<AstLogicAnd>>),
}

#[derive(Debug, PartialEq)]
pub enum AstLogicOr {
    Or(AstLogicAnd, Option<Box<AstLogicOr>>),
}

#[derive(Debug, PartialEq)]
pub enum AstAssignment {
    Assign(AstIdentifier, Box<AstAssignment>),
    LogicOr(AstLogicOr),
}

#[derive(Debug, PartialEq)]
pub enum AstExpression {
    Assignment(AstAssignment),
}

#[derive(Debug, PartialEq)]
pub enum AstStatement {
    Expr(AstExpression),
    Print(AstExpression),
    Block(Vec<AstDeclaration>),
    If(AstExpression, Box<AstStatement>, Option<Box<AstStatement>>),
    While(AstExpression, Box<AstStatement>),
}

#[derive(Debug, PartialEq)]
pub enum AstDeclaration {
    VarDeclare(AstIdentifier, AstExpression),
    /// A function declaration: `fun name(params) { body }`. The body is shared
    /// via `Rc` so a runtime function value can own it independently of the
    /// parse tree it came from (the interpreter only reads it).
    FunDeclare {
        name: AstIdentifier,
        params: Vec<AstIdentifier>,
        body: Rc<Vec<AstDeclaration>>,
    },
    Statement(AstStatement),
}

#[derive(Debug, PartialEq)]
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
            let call = parse_call(head, p)?;
            Some(AstUnary::Call(call))
        }
    }
}

/// Maximum number of arguments in a call — and, equivalently, parameters in a
/// function declaration. The book uses the same bound for both, since arguments
/// must match parameters.
///
/// Reference: <https://craftinginterpreters.com/functions.html#maximum-argument-counts>
const MAX_ARGUMENTS: usize = 255;

/// Parse a `call` expression: a primary followed by zero or more argument lists.
///
/// Grammar: `call → primary ( "(" arguments? ")" )*`. Calls are left-associative,
/// so `f()()` parses as `Call(Call(Primary(f), []), [])`.
///
/// Reference: <https://craftinginterpreters.com/functions.html#function-calls>
fn parse_call<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstCall>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut expr = AstCall::Primary(parse_primary(head, p)?);
    while p.tokens.next_if_eq(&lox_lexer::Token::LParens).is_some() {
        let args = parse_arguments(p)?;
        if args.len() > MAX_ARGUMENTS {
            return None;
        }
        expr = AstCall::Call(Box::new(expr), args);
    }
    Some(expr)
}

/// Parse a comma-separated argument list, assuming the opening `(` is consumed.
///
/// Consumes through the closing `)`. The [`MAX_ARGUMENTS`] limit is enforced by
/// the caller ([`parse_call`]); this function only parses.
///
/// Reference: <https://craftinginterpreters.com/functions.html#function-calls>
fn parse_arguments<I>(p: &mut Parser<I>) -> Option<Vec<AstExpression>>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut args = Vec::new();
    if p.tokens.next_if_eq(&lox_lexer::Token::RParens).is_some() {
        return Some(args);
    }
    loop {
        args.push(parse_expr(p.tokens.next()?, p)?);
        if p.tokens.next_if_eq(&lox_lexer::Token::Comma).is_none() {
            break;
        }
    }
    p.tokens.next_if_eq(&lox_lexer::Token::RParens)?;
    Some(args)
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
                        AstFactor::Unary(AstUnary::Call(AstCall::Primary(AstPrimary::Id(
                            identifier,
                        )))),
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

/// Parse a `for` statement and desugar it into existing AST nodes.
///
/// Lox's `for` loop has no dedicated AST node. Instead, this function parses
/// `for (init; cond; inc) body` and rewrites it as a block containing an
/// optional initializer followed by a `while` loop:
///
/// ```text
/// { init; while (cond) { body; inc; } }
/// ```
///
/// This is the one place where the parser's output diverges from the source
/// syntax — the resulting tree contains only `AstStatement::Block{}` and
/// `AstStatement::While{}`, both of which the interpreter already handles.
///
/// Each of the three clauses is optional:
/// - **Initializer**: a `var` declaration, an expression statement, or empty.
/// - **Condition**: an expression, or empty (defaults to `true`).
/// - **Increment**: an expression, or empty.
///
/// Reference: <https://craftinginterpreters.com/control-flow.html#for-loops>
fn parse_for_and_desugar<I>(p: &mut Parser<I>) -> Option<AstStatement>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    p.tokens.next_if_eq(&lox_lexer::Token::LParens)?;

    let initializer = match p.tokens.peek()? {
        lox_lexer::Token::Semicolon => {
            p.tokens.next();
            None
        }
        lox_lexer::Token::Var => {
            let head = p.tokens.next()?;
            Some(parse_declaration(head, p)?)
        }
        _ => {
            let head = p.tokens.next()?;
            let expr = parse_expr(head, p)?;
            p.tokens.next_if_eq(&lox_lexer::Token::Semicolon)?;
            Some(AstDeclaration::Statement(AstStatement::Expr(expr)))
        }
    };

    let condition = match p.tokens.peek()? {
        lox_lexer::Token::Semicolon => {
            p.tokens.next();
            make_true_expr()
        }
        _ => {
            let head = p.tokens.next()?;
            let expr = parse_expr(head, p)?;
            p.tokens.next_if_eq(&lox_lexer::Token::Semicolon)?;
            expr
        }
    };

    let increment = match p.tokens.peek()? {
        lox_lexer::Token::RParens => {
            p.tokens.next();
            None
        }
        _ => {
            let head = p.tokens.next()?;
            let expr = parse_expr(head, p)?;
            p.tokens.next_if_eq(&lox_lexer::Token::RParens)?;
            Some(expr)
        }
    };

    let body = parse_statement(p.tokens.next()?, p)?;

    let while_body = match increment {
        Some(inc) => AstStatement::Block(vec![
            AstDeclaration::Statement(body),
            AstDeclaration::Statement(AstStatement::Expr(inc)),
        ]),
        None => body,
    };

    let while_stmt = AstStatement::While(condition, Box::new(while_body));

    let mut decls = Vec::new();
    if let Some(init) = initializer {
        decls.push(init);
    }
    decls.push(AstDeclaration::Statement(while_stmt));
    Some(AstStatement::Block(decls))
}

/// Build a synthetic `true` expression for omitted `for`-loop conditions.
///
/// The condition clause of a `for` statement is optional; when absent, the loop
/// runs unconditionally. This helper constructs the full `AstExpression{}` chain
/// for the literal `true`, threading it through every precedence layer.
///
/// Reference: <https://craftinginterpreters.com/control-flow.html#for-loops>
fn make_true_expr() -> AstExpression {
    AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
        AstLogicAnd::And(
            AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Unary(
                AstUnary::Call(AstCall::Primary(AstPrimary::True)),
            )))),
            None,
        ),
        None,
    )))
}

/// Parse a brace-delimited block of declarations, assuming the opening `{` is
/// already consumed. Consumes through the closing `}`.
fn parse_block_decls<I>(p: &mut Parser<I>) -> Option<Vec<AstDeclaration>>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut decls = vec![];
    while let Some(tok) = p.tokens.peek() {
        if tok == &lox_lexer::Token::RBrace {
            break;
        }
        let head = p.tokens.next()?;
        decls.push(parse_declaration(head, p)?);
    }
    p.tokens.next_if_eq(&lox_lexer::Token::RBrace)?;
    Some(decls)
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
        lox_lexer::Token::LBrace => Some(AstStatement::Block(parse_block_decls(p)?)),
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
        lox_lexer::Token::While => {
            p.tokens.next_if_eq(&lox_lexer::Token::LParens)?;
            let condition = parse_expr(p.tokens.next()?, p)?;
            p.tokens.next_if_eq(&lox_lexer::Token::RParens)?;
            let body = parse_statement(p.tokens.next()?, p)?;
            Some(AstStatement::While(condition, Box::new(body)))
        }
        lox_lexer::Token::For => parse_for_and_desugar(p),
        _ => {
            let expr = parse_expr(head, p)?;
            match p.tokens.next()? {
                lox_lexer::Token::Semicolon => Some(AstStatement::Expr(expr)),
                _ => None,
            }
        }
    }
}

/// Parse a comma-separated parameter list, assuming the opening `(` is consumed.
///
/// Consumes through the closing `)`. The [`MAX_ARGUMENTS`] limit is enforced by
/// the caller (`parse_declaration`); this function only parses.
fn parse_parameters<I>(p: &mut Parser<I>) -> Option<Vec<AstIdentifier>>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    let mut params = Vec::new();
    if p.tokens.next_if_eq(&lox_lexer::Token::RParens).is_some() {
        return Some(params);
    }
    loop {
        params.push(parse_id(p.tokens.next()?)?);
        if p.tokens.next_if_eq(&lox_lexer::Token::Comma).is_none() {
            break;
        }
    }
    p.tokens.next_if_eq(&lox_lexer::Token::RParens)?;
    Some(params)
}

fn parse_declaration<I>(head: lox_lexer::Token, p: &mut Parser<I>) -> Option<AstDeclaration>
where
    I: Iterator<Item = lox_lexer::Token>,
{
    match head {
        lox_lexer::Token::Fun => {
            let name = parse_id(p.tokens.next()?)?;
            p.tokens.next_if_eq(&lox_lexer::Token::LParens)?;
            let params = parse_parameters(p)?;
            if params.len() > MAX_ARGUMENTS {
                return None;
            }
            p.tokens.next_if_eq(&lox_lexer::Token::LBrace)?;
            let body = parse_block_decls(p)?;
            Some(AstDeclaration::FunDeclare {
                name,
                params,
                body: Rc::new(body),
            })
        }
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
