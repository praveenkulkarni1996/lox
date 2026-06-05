#![deny(clippy::all)]

use crate::{LoxError, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// <https://craftinginterpreters.com/statements-and-state.html#nesting-and-shadowing>
///
/// An environment is a "namespace" in which we can look at variables, and their namespaces.
///
/// # Design: Interior Mutability via RefCell and shared ownership via Rc
///
/// The environment uses `RefCell<HashMap<...>>` instead of a regular HashMap to enable interior
/// mutability. This is necessary because we need to modify the variable map (via `declare` and
/// `update`) while only holding an immutable reference (`&self`). Here's why:
///
/// In an interpreter, a child environment holds a reference to its parent. When we call
/// `child.update("x", value)`, we may need to search up the parent chain and modify a parent's
/// variable. With a borrowed reference setup, we can't have `update(&mut self)` because then
/// we couldn't also hold references to parent environments. RefCell gives us runtime borrow
/// checking, allowing methods like `declare(&self, ...)` and `update(&self, ...)` to mutate
/// the internal HashMap via `borrow_mut()` without requiring a mutable reference to self.
///
/// The parent is stored as `Option<Rc<Environment>>`. `Rc` (reference counting) means the
/// parent environment stays alive as long as any child holds a reference to it — no lifetime
/// parameters needed. This allows child environments (and child `Interpreter{}` instances
/// for block scopes) to be created and dropped freely without borrow-checker constraints.
///
/// This pattern is common in Rust when you need shared ownership with interior mutability,
/// such as when implementing scoped environments or symbol tables.
///
/// # Examples
///
/// Reading a variable that exists in the current scope:
/// ```
/// use lox_interpreter::{Value, Environment};
///
/// let env = Environment::new();
/// env.declare("x", Value::Number(42.0));
/// assert_eq!(env.read("x").unwrap(), Value::Number(42.0));
/// ```
///
/// Reading a variable that doesn't exist returns an error:
/// ```
/// use lox_interpreter::{Value, LoxError, Environment};
///
/// let env = Environment::new();
/// assert!(env.read("undefined").is_err());
/// ```
pub struct Environment {
    env: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    /// Creates a new root environment with no parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_interpreter::{Environment, Value};
    ///
    /// let env = Environment::new();
    /// env.declare("x", Value::Number(42.0));
    /// assert_eq!(env.read("x").unwrap(), Value::Number(42.0));
    /// ```
    pub fn new() -> Rc<Self> {
        Rc::new(Environment {
            env: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    /// Creates a child environment with the given parent.
    ///
    /// The parent is kept alive by the child via `Rc` — no lifetime constraints needed.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_interpreter::{Environment, Value};
    ///
    /// let parent = Environment::new();
    /// parent.declare("x", Value::Number(42.0));
    /// let child = Environment::child_of(&parent);
    /// // Child can access parent's variables
    /// assert_eq!(child.read("x").unwrap(), Value::Number(42.0));
    /// ```
    pub fn child_of(parent: &Rc<Environment>) -> Rc<Self> {
        Rc::new(Environment {
            env: RefCell::new(HashMap::new()),
            parent: Some(Rc::clone(parent)),
        })
    }

    /// Reads a variable from the environment.
    ///
    /// If the variable is found in the current scope, its value is returned.
    /// Otherwise, the lookup continues in the parent environment recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_interpreter::{Value, Environment};
    ///
    /// let env = Environment::new();
    /// env.declare("x", Value::Number(42.0));
    /// assert_eq!(env.read("x").unwrap(), Value::Number(42.0));
    /// ```
    pub fn read(&self, variable: &str) -> Result<Value, LoxError> {
        if let Some(v) = self.env.borrow().get(variable) {
            Ok(v.clone())
        } else if let Some(parent) = &self.parent {
            parent.read(variable)
        } else {
            Err(LoxError::VariableNotFound(String::from(variable)))
        }
    }

    /// Declares a new variable in the current environment.
    ///
    /// If a variable with the same name already exists in the current scope,
    /// it is replaced and the old value is returned. Variables in parent
    /// scopes are not affected.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_interpreter::{Value, Environment};
    ///
    /// let env = Environment::new();
    /// let result = env.declare("x", Value::Number(42.0));
    /// assert_eq!(result, None);
    ///
    /// // Declaring the same variable again returns the previous value
    /// let result = env.declare("x", Value::Number(99.0));
    /// assert_eq!(result, Some(Value::Number(42.0)));
    /// ```
    pub fn declare(&self, variable: &str, value: Value) -> Option<Value> {
        self.env.borrow_mut().insert(String::from(variable), value)
    }

    /// Updates an existing variable in the environment.
    ///
    /// If the variable exists in the current scope, it is updated with the new value
    /// and the old value is returned. If it doesn't exist locally, the lookup and
    /// update continues in the parent environment recursively.
    ///
    /// # Examples
    ///
    /// ```
    /// use lox_interpreter::{Value, Environment};
    ///
    /// let env = Environment::new();
    /// env.declare("x", Value::Number(42.0));
    /// let old = env.update("x", Value::Number(99.0)).unwrap();
    /// assert_eq!(old, Value::Number(42.0));
    /// assert_eq!(env.read("x").unwrap(), Value::Number(99.0));
    /// ```
    pub fn update(&self, variable: &str, after: Value) -> Result<Value, LoxError> {
        let mut env_mut = self.env.borrow_mut();
        if let Some(value) = env_mut.get_mut(variable) {
            let old = value.clone();
            *value = after;
            Ok(old)
        } else {
            drop(env_mut);
            if let Some(parent) = &self.parent {
                parent.update(variable, after)
            } else {
                Err(LoxError::VariableNotFound(String::from(variable)))
            }
        }
    }
}
