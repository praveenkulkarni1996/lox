use lox_interpreter::{Interpreter, LoxError, Value, eval};
use lox_lexer::Lexer;
use lox_parser::Parser;

fn interpret(input: &str) -> Result<Value, LoxError> {
    let tokens = Lexer::new(input.chars());
    let mut parser = Parser::new(tokens);
    let ast = parser.next().expect("expected a parsed expression");
    let mut interpreter = Interpreter::new(std::io::stdout());
    eval(&mut interpreter, ast)
}

fn interpret_capturing(input: &str) -> (Result<Value, LoxError>, String) {
    let tokens = Lexer::new(input.chars());
    let mut parser = Parser::new(tokens);
    let ast = parser.next().expect("expected a parsed expression");
    let mut interpreter = Interpreter::new(Vec::new());
    let result = eval(&mut interpreter, ast);
    (result, String::from_utf8(interpreter.out).unwrap())
}

// === Primary Literals ===

#[test]
fn test_number() {
    let result = interpret("42;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_decimal_number() {
    let result = interpret("1.25;").unwrap();
    assert_eq!(result, Value::Number(1.25));
}

#[test]
fn test_string() {
    let result = interpret("\"hello\";").unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
}

#[test]
fn test_true() {
    let result = interpret("true;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_false() {
    let result = interpret("false;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_nil() {
    let result = interpret("nil;").unwrap();
    assert_eq!(result, Value::Nil);
}

// === Unary Expressions ===

#[test]
fn test_negation() {
    let result = interpret("-5;").unwrap();
    assert_eq!(result, Value::Number(-5.0));
}

#[test]
fn test_double_negation() {
    let result = interpret("--5;").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_not_true() {
    let result = interpret("!true;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_not_false() {
    let result = interpret("!false;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_not_nil() {
    let result = interpret("!nil;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_not_number() {
    // Numbers are truthy, so !number is false
    let result = interpret("!42;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_negate_non_number_is_error() {
    let result = interpret("-true;");
    assert!(result.is_err());
}

#[test]
fn test_negate_nil_is_error() {
    let result = interpret("-nil;");
    assert!(result.is_err());
}

#[test]
fn test_negate_string_is_error() {
    let result = interpret("-\"hello\";");
    assert!(result.is_err());
}

// === Arithmetic (Factor) ===

#[test]
fn test_multiplication() {
    let result = interpret("2 * 3;").unwrap();
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn test_division() {
    let result = interpret("10 / 2;").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_chained_multiplication() {
    let result = interpret("2 * 3 * 4;").unwrap();
    assert_eq!(result, Value::Number(24.0));
}

#[test]
fn test_division_by_zero() {
    let result = interpret("1 / 0;").unwrap();
    assert_eq!(result, Value::Number(f64::INFINITY));
}

// === Arithmetic (Term) ===

#[test]
fn test_addition() {
    let result = interpret("1 + 2;").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_subtraction() {
    let result = interpret("5 - 3;").unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_chained_addition() {
    let result = interpret("1 + 2 + 3;").unwrap();
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn test_mixed_arithmetic() {
    // 2 + 3 * 4 = 2 + 12 = 14 (multiplication first)
    let result = interpret("2 + 3 * 4;").unwrap();
    assert_eq!(result, Value::Number(14.0));
}

#[test]
fn test_subtraction_and_division() {
    // 10 - 6 / 2 = 10 - 3 = 7
    let result = interpret("10 - 6 / 2;").unwrap();
    assert_eq!(result, Value::Number(7.0));
}

// === Comparison ===

#[test]
fn test_less_than_true() {
    let result = interpret("1 < 2;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_less_than_false() {
    let result = interpret("2 < 1;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_less_equal_true() {
    let result = interpret("2 <= 2;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_less_equal_false() {
    let result = interpret("3 <= 2;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_greater_than_true() {
    let result = interpret("3 > 1;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_greater_than_false() {
    let result = interpret("1 > 3;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_greater_equal_true() {
    let result = interpret("2 >= 2;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_greater_equal_false() {
    let result = interpret("1 >= 2;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

// === Equality ===

#[test]
fn test_number_equality() {
    let result = interpret("42 == 42;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_number_inequality() {
    let result = interpret("1 == 2;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_not_equal_numbers() {
    let result = interpret("1 != 2;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_not_equal_same() {
    let result = interpret("42 != 42;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_nil_equals_nil() {
    let result = interpret("nil == nil;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_bool_equality() {
    let result = interpret("true == true;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_bool_inequality() {
    let result = interpret("true == false;").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_different_types_not_equal() {
    let result = interpret("1 != nil;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

// === Combined / Precedence ===

#[test]
fn test_arithmetic_in_comparison() {
    // 1 + 2 < 4 => 3 < 4 => true
    let result = interpret("1 + 2 < 4;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_negation_in_arithmetic() {
    // -1 + 3 = 2
    let result = interpret("-1 + 3;").unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_comparison_in_equality() {
    // (1 < 2) == true => true == true => true
    let result = interpret("1 < 2 == true;").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

// === Print Statement ===

#[test]
fn test_print_number() {
    let (result, output) = interpret_capturing("print 42;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "42\n");
}

#[test]
fn test_print_decimal() {
    let (result, output) = interpret_capturing("print 1.5;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "1.5\n");
}

#[test]
fn test_print_string() {
    let (result, output) = interpret_capturing("print \"hello\";");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "hello\n");
}

#[test]
fn test_print_true() {
    let (result, output) = interpret_capturing("print true;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "true\n");
}

#[test]
fn test_print_false() {
    let (result, output) = interpret_capturing("print false;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "false\n");
}

#[test]
fn test_print_nil() {
    let (result, output) = interpret_capturing("print nil;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "nil\n");
}

#[test]
fn test_print_expression() {
    let (result, output) = interpret_capturing("print 1 + 2;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "3\n");
}

#[test]
fn test_print_comparison() {
    let (result, output) = interpret_capturing("print 1 < 2;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "true\n");
}

#[test]
fn test_print_type_error_propagates() {
    let (result, output) = interpret_capturing("print -true;");
    assert!(result.is_err());
    assert_eq!(output, "");
}

#[test]
fn test_expr_statement_no_output() {
    let (result, output) = interpret_capturing("42;");
    assert_eq!(result.unwrap(), Value::Number(42.0));
    assert_eq!(output, "");
}

// === Type Errors ===

#[test]
fn test_add_non_numbers_is_error() {
    let result = interpret("true + 1;");
    assert!(result.is_err());
}

#[test]
fn test_subtract_non_numbers_is_error() {
    let result = interpret("nil - 1;");
    assert!(result.is_err());
}

#[test]
fn test_multiply_non_numbers_is_error() {
    let result = interpret("true * 2;");
    assert!(result.is_err());
}

#[test]
fn test_compare_non_numbers_is_error() {
    let result = interpret("true < 1;");
    assert!(result.is_err());
}
