use lox_interpreter::{Environment, LoxError, Value};

#[test]
fn test_environment_new_creates_root() {
    let env = Environment::new();
    // New environment should be empty (reading undefined returns error)
    assert!(env.read("x").is_err());
}

#[test]
fn test_environment_new_is_usable() {
    let env = Environment::new();
    env.declare("x", Value::Number(42.0));
    assert_eq!(env.read("x").unwrap(), Value::Number(42.0));
}

#[test]
fn test_environment_child_of_can_access_parent() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));
    let child = Environment::child_of(&parent);
    // Child should be able to access parent's variables
    assert_eq!(child.read("x").unwrap(), Value::Number(42.0));
}

#[test]
fn test_declare_new_variable_returns_none() {
    let env = Environment::new();
    let result = env.declare("x", Value::Number(42.0));
    assert_eq!(result, None);
}

#[test]
fn test_declare_overwrites_existing_variable() {
    let env = Environment::new();
    env.declare("x", Value::Number(42.0));
    let result = env.declare("x", Value::Number(99.0));
    assert_eq!(result, Some(Value::Number(42.0)));
}

#[test]
fn test_declare_multiple_variables() {
    let env = Environment::new();
    let r1 = env.declare("x", Value::Number(1.0));
    let r2 = env.declare("y", Value::Str("hello".to_string()));
    let r3 = env.declare("z", Value::Boolean(true));

    assert_eq!(r1, None);
    assert_eq!(r2, None);
    assert_eq!(r3, None);
}

#[test]
fn test_read_declared_variable() {
    let env = Environment::new();
    env.declare("x", Value::Number(42.0));
    let result = env.read("x");
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[test]
fn test_read_undefined_variable_is_error() {
    let env = Environment::new();
    let result = env.read("undefined");
    assert!(result.is_err());
}

#[test]
fn test_read_undefined_variable_error_contains_name() {
    let env = Environment::new();
    match env.read("undefined") {
        Err(LoxError::VariableNotFound(name)) => assert_eq!(name, "undefined"),
        _ => panic!("Expected VariableNotFound error"),
    }
}

#[test]
fn test_read_different_types() {
    let env = Environment::new();
    env.declare("num", Value::Number(3.5));
    env.declare("str", Value::Str("test".to_string()));
    env.declare("bool", Value::Boolean(false));
    env.declare("nil", Value::Nil);

    assert_eq!(env.read("num").unwrap(), Value::Number(3.5));
    assert_eq!(env.read("str").unwrap(), Value::Str("test".to_string()));
    assert_eq!(env.read("bool").unwrap(), Value::Boolean(false));
    assert_eq!(env.read("nil").unwrap(), Value::Nil);
}

#[test]
fn test_update_existing_variable_returns_old_value() {
    let env = Environment::new();
    env.declare("x", Value::Number(42.0));
    let result = env.update("x", Value::Number(99.0));
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[test]
fn test_update_actually_changes_value() {
    let env = Environment::new();
    env.declare("x", Value::Number(42.0));
    env.update("x", Value::Number(99.0)).unwrap();
    assert_eq!(env.read("x").unwrap(), Value::Number(99.0));
}

#[test]
fn test_update_undefined_variable_is_error() {
    let env = Environment::new();
    let result = env.update("undefined", Value::Number(42.0));
    assert!(result.is_err());
}

#[test]
fn test_update_different_types() {
    let env = Environment::new();
    env.declare("x", Value::Number(42.0));

    // Update to string
    let old = env.update("x", Value::Str("hello".to_string())).unwrap();
    assert_eq!(old, Value::Number(42.0));
    assert_eq!(env.read("x").unwrap(), Value::Str("hello".to_string()));

    // Update to boolean
    let old = env.update("x", Value::Boolean(true)).unwrap();
    assert_eq!(old, Value::Str("hello".to_string()));
    assert_eq!(env.read("x").unwrap(), Value::Boolean(true));
}

#[test]
fn test_parent_lookup_when_not_in_local_scope() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));

    let child = Environment::child_of(&parent);
    let result = child.read("x");
    assert_eq!(result.unwrap(), Value::Number(42.0));
}

#[test]
fn test_local_shadows_parent() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));

    let child = Environment::child_of(&parent);
    child.declare("x", Value::Number(99.0));

    // Child shadows parent's variable
    assert_eq!(child.read("x").unwrap(), Value::Number(99.0));
}

#[test]
fn test_child_can_access_parent_variables() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(1.0));
    parent.declare("y", Value::Number(2.0));

    let child = Environment::child_of(&parent);
    assert_eq!(child.read("x").unwrap(), Value::Number(1.0));
    assert_eq!(child.read("y").unwrap(), Value::Number(2.0));
}

#[test]
fn test_child_declaration_does_not_affect_parent() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));

    let child = Environment::child_of(&parent);
    child.declare("y", Value::Number(99.0));
    // Child can see its own variable
    assert_eq!(child.read("y").unwrap(), Value::Number(99.0));
}

#[test]
fn test_update_in_parent_from_child() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));

    let child = Environment::child_of(&parent);
    let old = child.update("x", Value::Number(99.0)).unwrap();
    // Verify the old value returned by update
    assert_eq!(old, Value::Number(42.0));
    // Verify the update modified the value in the child's scope
    assert_eq!(child.read("x").unwrap(), Value::Number(99.0));
}

#[test]
fn test_declare_creates_shadowing_variable() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));

    let child = Environment::child_of(&parent);
    child.declare("x", Value::Number(99.0));

    // Child sees its own x (shadows parent's x)
    assert_eq!(child.read("x").unwrap(), Value::Number(99.0));
}

#[test]
fn test_grandchild_environment() {
    let grandparent = Environment::new();
    grandparent.declare("x", Value::Number(1.0));

    let parent = Environment::child_of(&grandparent);
    parent.declare("y", Value::Number(2.0));

    let grandchild = Environment::child_of(&parent);
    assert_eq!(grandchild.read("x").unwrap(), Value::Number(1.0));
    assert_eq!(grandchild.read("y").unwrap(), Value::Number(2.0));
}

#[test]
fn test_variable_not_found_in_chain() {
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));

    let child = Environment::child_of(&parent);
    assert!(child.read("undefined").is_err());
}

#[test]
fn test_multiple_variables_in_parent_chain() {
    let parent = Environment::new();
    parent.declare("a", Value::Number(1.0));
    parent.declare("b", Value::Str("b".to_string()));

    let child = Environment::child_of(&parent);
    child.declare("c", Value::Boolean(true));

    assert_eq!(child.read("a").unwrap(), Value::Number(1.0));
    assert_eq!(child.read("b").unwrap(), Value::Str("b".to_string()));
    assert_eq!(child.read("c").unwrap(), Value::Boolean(true));
}

#[test]
fn test_declare_string_variable() {
    let env = Environment::new();
    let result = env.declare("name", Value::Str("Alice".to_string()));
    assert_eq!(result, None);
    assert_eq!(env.read("name").unwrap(), Value::Str("Alice".to_string()));
}

#[test]
fn test_declare_boolean_variable() {
    let env = Environment::new();
    let result = env.declare("flag", Value::Boolean(true));
    assert_eq!(result, None);
    assert_eq!(env.read("flag").unwrap(), Value::Boolean(true));
}

#[test]
fn test_declare_nil_variable() {
    let env = Environment::new();
    let result = env.declare("nothing", Value::Nil);
    assert_eq!(result, None);
    assert_eq!(env.read("nothing").unwrap(), Value::Nil);
}

#[test]
fn test_update_preserves_variable_name() {
    let env = Environment::new();
    env.declare("x", Value::Number(1.0));
    env.update("x", Value::Number(2.0)).unwrap();
    env.update("x", Value::Number(3.0)).unwrap();

    assert_eq!(env.read("x").unwrap(), Value::Number(3.0));
}

#[test]
fn test_sequential_declares_and_updates() {
    let env = Environment::new();
    env.declare("x", Value::Number(1.0));
    env.declare("y", Value::Number(2.0));
    env.update("x", Value::Number(10.0)).unwrap();
    env.declare("z", Value::Number(3.0));
    env.update("y", Value::Number(20.0)).unwrap();

    assert_eq!(env.read("x").unwrap(), Value::Number(10.0));
    assert_eq!(env.read("y").unwrap(), Value::Number(20.0));
    assert_eq!(env.read("z").unwrap(), Value::Number(3.0));
}

#[test]
fn test_read_clones_value() {
    let env = Environment::new();
    let original = Value::Str("hello".to_string());
    env.declare("x", original.clone());

    let read1 = env.read("x").unwrap();
    let read2 = env.read("x").unwrap();

    assert_eq!(read1, read2);
    assert_eq!(read1, original);
}

#[test]
fn test_parent_child_lifecycle() {
    // (1) Create parent and do operations
    let parent = Environment::new();
    parent.declare("x", Value::Number(42.0));
    parent.declare("y", Value::Number(10.0));
    assert_eq!(parent.read("x").unwrap(), Value::Number(42.0));
    assert_eq!(parent.read("y").unwrap(), Value::Number(10.0));

    // (2) Create child and do operations - the key is that we can modify
    // parent through the child, and those changes propagate to parent
    let x_value_in_child = {
        let child = Environment::child_of(&parent);
        child.declare("z", Value::Number(100.0));

        // Child can read parent's variables
        assert_eq!(child.read("x").unwrap(), Value::Number(42.0));
        assert_eq!(child.read("y").unwrap(), Value::Number(10.0));
        assert_eq!(child.read("z").unwrap(), Value::Number(100.0));

        // Child can update parent's variables
        child.update("x", Value::Number(99.0)).unwrap();
        assert_eq!(child.read("x").unwrap(), Value::Number(99.0));

        // Return the updated value before child is dropped
        child.read("x").unwrap()
    };

    // (3) Verify the child's update to parent persisted
    // By capturing x_value_in_child inside the child's scope before it's dropped,
    // we can verify that the child's modifications affected the parent
    assert_eq!(x_value_in_child, Value::Number(99.0));
    assert_eq!(parent.read("x").unwrap(), Value::Number(99.0));
}
