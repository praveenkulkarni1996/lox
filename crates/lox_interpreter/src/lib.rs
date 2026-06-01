//! Tree-walk interpreter for the Lox language.
//!
//! Evaluates a parsed Lox AST by recursively walking the tree and producing runtime values.
//! Follows the evaluation rules from
//! [Crafting Interpreters](https://craftinginterpreters.com/evaluating-expressions.html).

#[derive(thiserror::Error, Debug)]
pub enum LoxError {
    #[error("Cannot Convert {0} to Number")]
    NumberConversionError(Value),

    #[error("Could not find variable {0}.")]
    VariableNotFound(String),
}

use lox_parser::{
    self,
    Ast::Declare,
    AstAssignment::{Assign, Eq},
    AstComparison::{Greater, GreaterEqual, Less, LessEqual, Term},
    AstDeclaration::{Statement, VarDeclare},
    AstEquality::{Comparison, Equal, NotEqual},
    AstExpression::Assignment,
    AstFactor::{Div, Mul, Unary},
    AstPrimary::{False, Group, Id, Nil, Number, Str, True},
    AstStatement::{Expr, Print},
    AstTerm::{Add, Factor, Sub},
    AstUnary::{Negative, Not, Primary},
};

/// A runtime value produced by evaluating a Lox expression.
#[derive(Debug, PartialEq, Clone)]
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

pub struct Interpreter<W>
where
    W: std::io::Write,
{
    pub out: W,
    pub env: std::collections::HashMap<String, Value>,
}

impl<W: std::io::Write> Interpreter<W> {
    pub fn new(out: W) -> Self {
        Interpreter {
            out,
            env: std::collections::HashMap::new(),
        }
    }
}

fn eval_primary<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstPrimary,
) -> Result<Value, LoxError> {
    match ast {
        Number(x) => Ok(Value::Number(x)),
        Str(s) => Ok(Value::Str(s)),
        True => Ok(Value::Boolean(true)),
        False => Ok(Value::Boolean(false)),
        Nil => Ok(Value::Nil),
        Group(expr) => Ok(eval_expression(interpreter, *expr)?),
        Id(identifier) => match interpreter.env.get(&identifier) {
            Some(value) => Ok(value.clone()),
            None => Err(LoxError::VariableNotFound(identifier)),
        },
    }
}

fn eval_unary<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstUnary,
) -> Result<Value, LoxError> {
    match ast {
        Primary(primary) => Ok(eval_primary(interpreter, primary)?),
        Not(unary) => Ok(Value::Boolean(!bool::from(eval_unary(
            interpreter,
            *unary,
        )?))),
        Negative(unary) => Ok(Value::Number(
            -(f64::try_from(eval_unary(interpreter, *unary)?)?),
        )),
    }
}

fn eval_factor<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstFactor,
) -> Result<Value, LoxError> {
    match ast {
        Unary(unary) => Ok(eval_unary(interpreter, unary)?),
        Mul(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_factor(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_unary(interpreter, rhs)?)?;
            Ok(Value::Number(lhs * rhs))
        }
        Div(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_factor(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_unary(interpreter, rhs)?)?;
            Ok(Value::Number(lhs / rhs))
        }
    }
}

fn eval_term<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstTerm,
) -> Result<Value, LoxError> {
    match ast {
        Factor(factor) => Ok(eval_factor(interpreter, factor)?),
        Add(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_term(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_factor(interpreter, rhs)?)?;
            Ok(Value::Number(lhs + rhs))
        }
        Sub(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_term(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_factor(interpreter, rhs)?)?;
            Ok(Value::Number(lhs - rhs))
        }
    }
}

fn eval_comparison<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstComparison,
) -> Result<Value, LoxError> {
    match ast {
        Term(term) => Ok(eval_term(interpreter, term)?),
        GreaterEqual(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs >= rhs))
        }
        Greater(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs > rhs))
        }
        LessEqual(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs <= rhs))
        }
        Less(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, *lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs < rhs))
        }
    }
}

fn eval_equality<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstEquality,
) -> Result<Value, LoxError> {
    match ast {
        Comparison(comparison) => Ok(eval_comparison(interpreter, comparison)?),
        Equal(lhs, rhs) => {
            let lhs: Value = eval_equality(interpreter, *lhs)?;
            let rhs: Value = eval_comparison(interpreter, rhs)?;
            Ok(Value::Boolean(lhs == rhs))
        }
        NotEqual(lhs, rhs) => {
            let lhs: Value = eval_equality(interpreter, *lhs)?;
            let rhs: Value = eval_comparison(interpreter, rhs)?;
            Ok(Value::Boolean(lhs != rhs))
        }
    }
}

fn eval_assignment<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstAssignment,
) -> Result<Value, LoxError> {
    match ast {
        Assign(identifier, expr) => match interpreter.env.get(&identifier) {
            Some(_existing) => {
                let value = eval_assignment(interpreter, *expr)?;
                interpreter.env.insert(identifier, value.clone());
                Ok(value)
            }
            None => Err(LoxError::VariableNotFound(identifier)),
        },
        Eq(eq) => eval_equality(interpreter, eq),
    }
}

fn eval_expression<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstExpression,
) -> Result<Value, LoxError> {
    match ast {
        Assignment(assignment) => eval_assignment(interpreter, assignment),
    }
}

fn eval_statement<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstStatement,
) -> Result<Value, LoxError> {
    match ast {
        Expr(expr) => eval_expression(interpreter, expr),
        Print(expr) => {
            let result = eval_expression(interpreter, expr)?;
            writeln!(interpreter.out, "{}", result).unwrap();
            Ok(Value::Nil)
        }
    }
}

fn eval_declaration<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::AstDeclaration,
) -> Result<Value, LoxError> {
    match ast {
        VarDeclare(name, expr) => {
            let value = eval_expression(interpreter, expr)?;
            interpreter.env.insert(name, value);
            Ok(Value::Nil)
        }
        Statement(stmt) => eval_statement(interpreter, stmt),
    }
}

pub fn eval<W: std::io::Write>(
    interpreter: &mut Interpreter<W>,
    ast: lox_parser::Ast,
) -> Result<Value, LoxError> {
    match ast {
        Declare(decl) => eval_declaration(interpreter, decl),
    }
}
