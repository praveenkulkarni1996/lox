//! Tree-walk interpreter for the Lox language.
//!
//! Evaluates a parsed Lox AST by recursively walking the tree and producing runtime values.
//! Follows the evaluation rules from
//! [Crafting Interpreters](https://craftinginterpreters.com/evaluating-expressions.html).

pub mod environment;
pub use environment::Environment;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(thiserror::Error, Debug)]
pub enum LoxError {
    #[error("Cannot Convert {0} to Number")]
    NumberConversionError(Value),

    #[error("Could not find variable {0}.")]
    VariableNotFound(String),

    #[error("Can only call functions and classes; '{0}' is not callable.")]
    NotCallable(Value),

    #[error("Expected {expected} arguments but got {got}.")]
    ArityMismatch { expected: usize, got: usize },

    #[error("I/O error while writing output: {0}")]
    Io(#[from] std::io::Error),
}

use lox_parser::{
    self,
    Ast::Declare,
    AstAssignment::{Assign, LogicOr},
    AstComparison::{Greater, GreaterEqual, Less, LessEqual, Term},
    AstDeclaration::{FunDeclare, Statement, VarDeclare},
    AstEquality::{Comparison, Equal, NotEqual},
    AstExpression::Assignment,
    AstFactor::{Div, Mul, Unary},
    AstLogicAnd::And,
    AstLogicOr::Or,
    AstPrimary::{False, Group, Id, Nil, Number, Str, True},
    AstStatement::{Block, Expr, If, Print, While},
    AstTerm::{Add, Factor, Sub},
    AstUnary::{Negative, Not},
};

use lox_lexer::Lexer;
use lox_parser::Parser;

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
    /// A callable value, such as a native function.
    Callable(Callable),
}

/// A built-in function implemented in Rust.
///
/// The `name` is the function's identity: it is unique across all natives, so it
/// is what we use for both [`Display`](std::fmt::Display) (how the value prints
/// and appears in error messages) and equality. It is independent of whatever
/// variable currently binds the function.
///
/// Reference: <https://craftinginterpreters.com/functions.html#native-functions>
#[derive(Debug, Clone)]
pub struct NativeFn {
    /// Display name and identity, e.g. `clock`.
    name: &'static str,
    /// Number of arguments the function expects.
    arity: usize,
    /// The implementation.
    func: fn(&[Value]) -> Result<Value, LoxError>,
}

/// Native functions are compared by `name`, which uniquely identifies each
/// built-in.
///
/// We deliberately do not compare the `func` pointers. Rust does not give
/// function pointers meaningful equality: the `unpredictable_function_pointer_comparisons`
/// lint warns that the compiler may emit one function at several addresses
/// (across codegen units) or fold distinct functions with identical bodies to a
/// single address, so `==` on `fn` pointers is unreliable in both directions.
impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

/// A user-defined function: a `fun` declaration paired with the environment it
/// was declared in (its closure).
///
/// Calls run the body in a child of `closure`, which is what gives Lox closures
/// (and recursion, since a declaration binds its own name into that environment
/// — see `eval_declaration`).
///
/// Reference: <https://craftinginterpreters.com/functions.html#function-objects>
pub struct LoxFunction {
    /// The declared name, used **only** for display (`<fn name>`). It is purely
    /// cosmetic: calls resolve a function by looking the binding up in the
    /// environment (the key there, not this field), and function equality is
    /// reference identity (`Rc::ptr_eq`, see `Callable`'s `PartialEq`) — neither
    /// path reads `name`. Renaming or rebinding the value cannot change it.
    name: String,
    /// Parameter names, bound to argument values on each call. Its length is the
    /// function's arity.
    params: Vec<String>,
    /// The body, shared from the parse tree via `Rc` so the value can outlive the
    /// parse loop and be called repeatedly.
    ///
    /// TODO(#20): replace this shared ownership with a borrowed slice once the
    /// EagerParser redesign lets a single owned AST outlive the interpreter; the
    /// `Rc` here also forms a leak-prone cycle (env → function → closure env).
    body: Rc<Vec<lox_parser::AstDeclaration>>,
    /// The environment captured at declaration time (the function's closure).
    closure: Rc<Environment>,
}

/// Debug only the identity (name and arity). We avoid printing `closure` and
/// `body`: the closure can form a cycle (env → function → closure env), and the
/// body is large, noisy AST.
impl std::fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoxFunction")
            .field("name", &self.name)
            .field("arity", &self.params.len())
            .finish_non_exhaustive()
    }
}

/// Anything that can be invoked with `(...)` in Lox.
#[derive(Debug, Clone)]
pub enum Callable {
    /// A function implemented in Rust.
    Native(NativeFn),
    /// A user-defined function.
    Function(Rc<LoxFunction>),
}

/// Native functions compare by name (see [`NativeFn`]); user functions compare
/// by reference identity (two bindings are equal only if they are the same
/// function object), matching the book's treatment of function equality.
impl PartialEq for Callable {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Callable::Native(a), Callable::Native(b)) => a == b,
            (Callable::Function(a), Callable::Function(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Callable {
    /// The number of arguments this callable expects.
    fn arity(&self) -> usize {
        match self {
            Callable::Native(native) => native.arity,
            Callable::Function(function) => function.params.len(),
        }
    }
}

impl std::fmt::Display for Callable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Callable::Native(native) => write!(f, "<native fn {}>", native.name),
            Callable::Function(function) => write!(f, "<fn {}>", function.name),
        }
    }
}

/// The `clock()` native: seconds elapsed since the Unix epoch as a number.
///
/// Reference: <https://craftinginterpreters.com/functions.html#telling-time>
fn clock(_args: &[Value]) -> Result<Value, LoxError> {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    Ok(Value::Number(secs))
}

impl From<Value> for bool {
    /// Truthiness conversion for Lox values.
    ///
    /// Lox follows Ruby's simple rule: `false` and `nil` are falsey,
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
            Value::Callable(c) => write!(f, "{}", c),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = LoxError;

    /// Lox follows Ruby's simple rule: `false` and `nil` are falsey,
    /// and everything else is truthy.
    ///
    /// Reference: <https://craftinginterpreters.com/evaluating-expressions.html#detecting-runtime-errors>
    fn try_from(v: Value) -> Result<f64, LoxError> {
        match v {
            Value::Number(num) => Ok(num),
            Value::Nil => Err(LoxError::NumberConversionError(v)),
            Value::Str(_) => Err(LoxError::NumberConversionError(v)),
            Value::Boolean(_) => Err(LoxError::NumberConversionError(v)),
            Value::Callable(_) => Err(LoxError::NumberConversionError(v)),
        }
    }
}

pub struct Interpreter<W>
where
    W: std::io::Write,
{
    /// The output writer, shared via `Rc<RefCell<W>>` so that child interpreters
    /// created for block scopes write to the same output.
    pub out: Rc<RefCell<W>>,
    /// The current environment. Child interpreters hold a child of this env.
    pub env: Rc<Environment>,
}

impl<W: std::io::Write> Interpreter<W> {
    pub fn new(out: W) -> Self {
        let env = Environment::new();
        env.declare(
            "clock",
            Value::Callable(Callable::Native(NativeFn {
                name: "clock",
                arity: 0,
                func: clock,
            })),
        );
        Interpreter {
            out: Rc::new(RefCell::new(out)),
            env,
        }
    }

    fn child(&self) -> Self {
        Interpreter {
            out: Rc::clone(&self.out),
            env: Environment::child_of(&self.env),
        }
    }
}

fn eval_primary<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstPrimary,
) -> Result<Value, LoxError> {
    match ast {
        Number(x) => Ok(Value::Number(*x)),
        Str(s) => Ok(Value::Str(s.clone())),
        True => Ok(Value::Boolean(true)),
        False => Ok(Value::Boolean(false)),
        Nil => Ok(Value::Nil),
        Group(expr) => eval_expression(interpreter, expr),
        Id(identifier) => interpreter.env.read(identifier),
    }
}

fn eval_unary<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstUnary,
) -> Result<Value, LoxError> {
    match ast {
        lox_parser::AstUnary::Call(call) => eval_call(interpreter, call),
        Not(unary) => Ok(Value::Boolean(!bool::from(eval_unary(interpreter, unary)?))),
        Negative(unary) => Ok(Value::Number(
            -(f64::try_from(eval_unary(interpreter, unary)?)?),
        )),
    }
}

/// Evaluate a `call` expression.
///
/// A bare primary evaluates directly. For an actual call, the callee and each
/// argument are evaluated left-to-right, then the callee must be a
/// [`Value::Callable`] of matching arity; otherwise a [`LoxError`] is returned.
///
/// Dispatch lives here (rather than on [`Value`]) so that the interpreter's
/// writer generic never leaks into the value type.
///
/// Reference: <https://craftinginterpreters.com/functions.html#function-calls>
fn eval_call<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstCall,
) -> Result<Value, LoxError> {
    match ast {
        lox_parser::AstCall::Primary(primary) => eval_primary(interpreter, primary),
        lox_parser::AstCall::Call(callee, args) => {
            let callee = eval_call(interpreter, callee)?;
            let args = args
                .iter()
                .map(|arg| eval_expression(interpreter, arg))
                .collect::<Result<Vec<_>, _>>()?;
            match callee {
                Value::Callable(callable) => {
                    if callable.arity() != args.len() {
                        return Err(LoxError::ArityMismatch {
                            expected: callable.arity(),
                            got: args.len(),
                        });
                    }
                    match callable {
                        // Native functions don't need interpreter context.
                        Callable::Native(native) => (native.func)(&args),
                        // User functions need the writer and AST evaluator, so
                        // they are dispatched here where the interpreter is in
                        // scope rather than on `Callable`.
                        Callable::Function(function) => {
                            call_function(interpreter, &function, &args)
                        }
                    }
                }
                other => Err(LoxError::NotCallable(other)),
            }
        }
    }
}

/// Invoke a user-defined function with already-evaluated, arity-checked arguments.
///
/// Runs the body in a fresh environment that is a child of the function's
/// *closure* (not the call site), with the parameters bound to the arguments.
/// Returns `Value::Nil` — explicit `return` is added in a later chapter (#17).
///
/// Reference: <https://craftinginterpreters.com/functions.html#function-objects>
fn call_function<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    function: &LoxFunction,
    args: &[Value],
) -> Result<Value, LoxError> {
    let env = Environment::child_of(&function.closure);
    for (param, arg) in function.params.iter().zip(args) {
        env.declare(param, arg.clone());
    }
    let call_interpreter = Interpreter {
        out: Rc::clone(&interpreter.out),
        env,
    };
    for decl in function.body.iter() {
        eval_declaration(&call_interpreter, decl)?;
    }
    Ok(Value::Nil)
}

fn eval_factor<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstFactor,
) -> Result<Value, LoxError> {
    match ast {
        Unary(unary) => eval_unary(interpreter, unary),
        Mul(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_factor(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_unary(interpreter, rhs)?)?;
            Ok(Value::Number(lhs * rhs))
        }
        Div(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_factor(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_unary(interpreter, rhs)?)?;
            Ok(Value::Number(lhs / rhs))
        }
    }
}

fn eval_term<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstTerm,
) -> Result<Value, LoxError> {
    match ast {
        Factor(factor) => eval_factor(interpreter, factor),
        Add(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_term(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_factor(interpreter, rhs)?)?;
            Ok(Value::Number(lhs + rhs))
        }
        Sub(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_term(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_factor(interpreter, rhs)?)?;
            Ok(Value::Number(lhs - rhs))
        }
    }
}

fn eval_comparison<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstComparison,
) -> Result<Value, LoxError> {
    match ast {
        Term(term) => eval_term(interpreter, term),
        GreaterEqual(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs >= rhs))
        }
        Greater(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs > rhs))
        }
        LessEqual(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs <= rhs))
        }
        Less(lhs, rhs) => {
            let lhs: f64 = f64::try_from(eval_comparison(interpreter, lhs)?)?;
            let rhs: f64 = f64::try_from(eval_term(interpreter, rhs)?)?;
            Ok(Value::Boolean(lhs < rhs))
        }
    }
}

fn eval_equality<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstEquality,
) -> Result<Value, LoxError> {
    match ast {
        Comparison(comparison) => eval_comparison(interpreter, comparison),
        Equal(lhs, rhs) => {
            let lhs: Value = eval_equality(interpreter, lhs)?;
            let rhs: Value = eval_comparison(interpreter, rhs)?;
            Ok(Value::Boolean(lhs == rhs))
        }
        NotEqual(lhs, rhs) => {
            let lhs: Value = eval_equality(interpreter, lhs)?;
            let rhs: Value = eval_comparison(interpreter, rhs)?;
            Ok(Value::Boolean(lhs != rhs))
        }
    }
}

/// Evaluate a logical `and` expression with short-circuit semantics.
///
/// Returns the first falsey operand (without evaluating the rest), or
/// the last operand if all are truthy.
///
/// Reference: <https://craftinginterpreters.com/control-flow.html#logical-operators>
fn eval_logic_and<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstLogicAnd,
) -> Result<Value, LoxError> {
    match ast {
        And(equality, None) => eval_equality(interpreter, equality),
        And(equality, Some(tail)) => {
            let left = eval_equality(interpreter, equality)?;
            if bool::from(left.clone()) {
                eval_logic_and(interpreter, tail)
            } else {
                Ok(left)
            }
        }
    }
}

/// Evaluate a logical `or` expression with short-circuit semantics.
///
/// Returns the first truthy operand (without evaluating the rest), or
/// the last operand if all are falsey.
///
/// Reference: <https://craftinginterpreters.com/control-flow.html#logical-operators>
fn eval_logic_or<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstLogicOr,
) -> Result<Value, LoxError> {
    match ast {
        Or(head, None) => eval_logic_and(interpreter, head),
        Or(head, Some(tail)) => {
            let left = eval_logic_and(interpreter, head)?;
            if bool::from(left.clone()) {
                Ok(left)
            } else {
                eval_logic_or(interpreter, tail)
            }
        }
    }
}

fn eval_assignment<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstAssignment,
) -> Result<Value, LoxError> {
    match ast {
        Assign(identifier, expr) => {
            let value = eval_assignment(interpreter, expr)?;
            interpreter.env.update(identifier, value.clone())?;
            Ok(value)
        }
        LogicOr(logic_or) => eval_logic_or(interpreter, logic_or),
    }
}

fn eval_expression<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstExpression,
) -> Result<Value, LoxError> {
    match ast {
        Assignment(assignment) => eval_assignment(interpreter, assignment),
    }
}

fn eval_block<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    decls: &[lox_parser::AstDeclaration],
) -> Result<Value, LoxError> {
    let child = interpreter.child();
    for decl in decls {
        eval_declaration(&child, decl)?;
    }
    Ok(Value::Nil)
}

fn eval_if<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    condition: &lox_parser::AstExpression,
    then_branch: &lox_parser::AstStatement,
    else_branch: &Option<Box<lox_parser::AstStatement>>,
) -> Result<Value, LoxError> {
    if bool::from(eval_expression(interpreter, condition)?) {
        eval_statement(interpreter, then_branch)
    } else if let Some(else_branch) = else_branch {
        eval_statement(interpreter, else_branch)
    } else {
        Ok(Value::Nil)
    }
}

/// Evaluate a `while` loop.
///
/// Repeatedly evaluates the condition; while truthy, evaluates the body and
/// discards its value. Returns `Value::Nil` when the condition becomes falsey.
///
/// Reference: <https://craftinginterpreters.com/control-flow.html#while-loops>
fn eval_while<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    condition: &lox_parser::AstExpression,
    body: &lox_parser::AstStatement,
) -> Result<Value, LoxError> {
    while bool::from(eval_expression(interpreter, condition)?) {
        eval_statement(interpreter, body)?;
    }
    Ok(Value::Nil)
}

fn eval_statement<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstStatement,
) -> Result<Value, LoxError> {
    match ast {
        Expr(expr) => eval_expression(interpreter, expr),
        Print(expr) => {
            let result = eval_expression(interpreter, expr)?;
            writeln!(interpreter.out.borrow_mut(), "{}", result)?;
            Ok(Value::Nil)
        }
        Block(decls) => eval_block(interpreter, decls),
        If(cond, then_branch, else_branch) => eval_if(interpreter, cond, then_branch, else_branch),
        While(condition, body) => eval_while(interpreter, condition, body),
    }
}

fn eval_declaration<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::AstDeclaration,
) -> Result<Value, LoxError> {
    match ast {
        VarDeclare(name, expr) => {
            let value = eval_expression(interpreter, expr)?;
            interpreter.env.declare(name, value);
            Ok(Value::Nil)
        }
        FunDeclare { name, params, body } => {
            let function = LoxFunction {
                name: name.clone(),
                params: params.clone(),
                body: Rc::clone(body),
                // Capture the defining environment as the closure. Declaring the
                // name into this same environment (below) is what lets the body
                // — which runs in a child of the closure — call itself.
                closure: Rc::clone(&interpreter.env),
            };
            interpreter
                .env
                .declare(name, Value::Callable(Callable::Function(Rc::new(function))));
            Ok(Value::Nil)
        }
        Statement(stmt) => eval_statement(interpreter, stmt),
    }
}

pub fn eval<W: std::io::Write>(
    interpreter: &Interpreter<W>,
    ast: &lox_parser::Ast,
) -> Result<Value, LoxError> {
    match ast {
        Declare(decl) => eval_declaration(interpreter, decl),
    }
}

/// Lex, parse, and evaluate an entire source string against `interpreter`,
/// returning the value of the last declaration (or the first error).
///
/// The interpreter is borrowed rather than owned so that callers can reuse it
/// across multiple `run` calls — this is what lets the REPL persist variables
/// between lines, and lets tests inspect a captured writer afterwards.
///
/// # Examples
///
/// ```
/// use lox_interpreter::{Interpreter, Value, run};
///
/// let interpreter = Interpreter::new(Vec::new());
/// assert_eq!(run("var x = 41; x + 1;", &interpreter).unwrap(), Value::Number(42.0));
/// ```
pub fn run<W: std::io::Write>(
    source: &str,
    interpreter: &Interpreter<W>,
) -> Result<Value, LoxError> {
    let parser = Parser::new(Lexer::new(source.chars()));
    let mut result = Value::Nil;
    // NOTE: parse errors currently truncate silently — the Parser iterator
    // yields None for both clean EOF and a parse failure. Surfacing them is
    // tracked in issue #5.
    for ast in parser {
        result = eval(interpreter, &ast)?;
    }
    Ok(result)
}
