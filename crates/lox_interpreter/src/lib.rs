//! Tree-walk interpreter for the Lox language.
//!
//! Evaluates a parsed Lox AST by recursively walking the tree and producing runtime values.
//! Follows the evaluation rules from
//! [Crafting Interpreters](https://craftinginterpreters.com/evaluating-expressions.html).

#[derive(thiserror::Error, Debug)]
pub enum LoxError {
    #[error("Cannot Convert {0} to Number")]
    NumberConversionError(Value),
}

use std::fmt;

use lox_parser::{
    self,
    Ast::Declare,
    AstComparison::{Greater, GreaterEqual, Less, LessEqual, Term},
    AstDeclaration::{Statement, VarDeclare},
    AstEquality::{Comparison, Equal, NotEqual},
    AstExpression::Eq,
    AstFactor::{Div, Mul, Unary},
    AstPrimary::{False, Nil, Number, Str, True},
    AstStatement::{Expr, Print},
    AstTerm::{Add, Factor, Sub},
    AstUnary::{Negative, Not, Primary},
};

/// A runtime value produced by evaluating a Lox expression.
#[derive(Debug, PartialEq)]
pub enum Value {
    /// A floating-point number.
    Number(f64),
    /// A string value.
    Str(String),
    /// A boolean value (`true` or `false`).
    Boolean(bool),
    /// The nil value, representing the absence of a value.
    Nil,
}

impl From<Value> for bool {
    /// Truthiness conversion for Lox values.
    ///
    /// Lox follows Ruby’s simple rule: `false` and `nil` are falsey,
    /// and everything else is truthy.
    ///
    /// Reference: <https://craftinginterpreters.com/evaluating-expressions.html#truthiness-and-falsiness>
    fn from(v: Value) -> Self {
        match v {
            Value::Nil => false,
            Value::Boolean(b) => b,
            _ => true,
        }
    }
}

impl std::fmt::Display for Value {
    /// The textbook is quite opinionated about what constitutes a valid way to
    /// print values. I am not precisely following the letter of the law.
    ///
    /// Reference: <https://craftinginterpreters.com/evaluating-expressions.html#hooking-up-the-interpreter>
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Number(num) => write!(f, "{}", num),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Str(s) => write!(f, "{}", s),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = LoxError;

    /// Lox follows Ruby’s simple rule: `false` and `nil` are falsey,
    /// and everything else is truthy.
    ///
    /// Reference: <https://craftinginterpreters.com/evaluating-expressions.html#detecting-runtime-errors>
    fn try_from(v: Value) -> Result<f64, LoxError> {
        match v {
            Value::Number(num) => Ok(num),
            Value::Nil => Err(LoxError::NumberConversionError(v)),
            Value::Str(_) => Err(LoxError::NumberConversionError(v)),
            Value::Boolean(_) => Err(LoxError::NumberConversionError(v)),
        }
    }
}

/// Evaluates a primary expression (literal or grouped expression).
fn eval_primary(ast: lox_parser::AstPrimary) -> Result<Value, LoxError> {
    match ast {
        Number(x) => Ok(Value::Number(x)),
        Str(s) => Ok(Value::Str(s)),
        True => Ok(Value::Boolean(true)),
        False => Ok(Value::Boolean(false)),
        Nil => Ok(Value::Nil),
        _ => todo!("evaluate groups and identifier:walk"),
    }
}

/// Evaluates a unary expression (`!` or `-` prefix operators).
fn eval_unary(ast: lox_parser::AstUnary) -> Result<Value, LoxError> {
    match ast {
        Primary(primary) => Ok(eval_primary(primary)?),
        Not(unary) => Ok(Value::Boolean(!bool::from(eval_unary(*unary)?))),
        Negative(unary) => Ok(Value::Number(-(f64::try_from(eval_unary(*unary)?)?))),
    }
}

/// Evaluates a product or quotient of two numbers.
fn eval_factor(ast: lox_parser::AstFactor) -> Result<Value, LoxError> {
    match ast {
        Unary(unary) => Ok(eval_unary(unary)?),
        Mul(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_factor(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_unary(rhs)?)?;
            Ok(Value::Number(lhs * rhs))
        }
        Div(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_factor(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_unary(rhs)?)?;
            Ok(Value::Number(lhs / rhs))
        }
    }
}

/// Evaluates the sum or difference.
fn eval_term(ast: lox_parser::AstTerm) -> Result<Value, LoxError> {
    match ast {
        Factor(factor) => Ok(eval_factor(factor)?),
        Add(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_term(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_factor(rhs)?)?;
            Ok(Value::Number(lhs + rhs))
        }
        Sub(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_term(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_factor(rhs)?)?;
            Ok(Value::Number(lhs - rhs))
        }
    }
}

fn eval_comparison(ast: lox_parser::AstComparison) -> Result<Value, LoxError> {
    match ast {
        Term(term) => Ok(eval_term(term)?),
        GreaterEqual(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(rhs)?)?;
            Ok(Value::Boolean(lhs >= rhs))
        }
        Greater(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(rhs)?)?;
            Ok(Value::Boolean(lhs > rhs))
        }
        LessEqual(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(rhs)?)?;
            Ok(Value::Boolean(lhs <= rhs))
        }
        Less(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(*lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(rhs)?)?;
            Ok(Value::Boolean(lhs < rhs))
        }
    }
}

fn eval_equality(ast: lox_parser::AstEquality) -> Result<Value, LoxError> {
    match ast {
        Comparison(comparison) => Ok(eval_comparison(comparison)?),
        Equal(lhs, rhs) => {
            let lhs: Value = eval_equality(*lhs)?;
            let rhs: Value = eval_comparison(rhs)?;
            Ok(Value::Boolean(lhs == rhs))
        }
        NotEqual(lhs, rhs) => {
            let lhs: Value = eval_equality(*lhs)?;
            let rhs: Value = eval_comparison(rhs)?;
            Ok(Value::Boolean(lhs != rhs))
        }
    }
}

fn eval_expression(ast: lox_parser::AstExpression) -> Result<Value, LoxError> {
    match ast {
        Eq(eq) => eval_equality(eq),
    }
}

fn eval_statement(
    ast: lox_parser::AstStatement,
    out: &mut impl std::io::Write,
) -> Result<Value, LoxError> {
    match ast {
        Expr(expr) => eval_expression(expr),
        Print(expr) => {
            let result = eval_expression(expr)?;
            writeln!(out, "{}", result).unwrap();
            Ok(Value::Nil)
        }
    }
}

fn eval_declaration(
    ast: lox_parser::AstDeclaration,
    out: &mut impl std::io::Write,
) -> Result<Value, LoxError> {
    match ast {
        VarDeclare(_, _) => todo!("add support for variable declaration"),
        Statement(stmt) => eval_statement(stmt, out),
    }
}

pub fn eval(ast: lox_parser::Ast) -> Result<Value, LoxError> {
    eval_to(ast, &mut std::io::stdout())
}

pub fn eval_to(ast: lox_parser::Ast, out: &mut impl std::io::Write) -> Result<Value, LoxError> {
    match ast {
        Declare(decl) => eval_declaration(decl, out),
    }
}
