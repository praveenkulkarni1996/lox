use lox_interpreter::{Interpreter, LoxError, Value, run};

fn interpret(input: &str) -> Result<Value, LoxError> {
    run(input, &Interpreter::new(std::io::stdout()))
}

fn interpret_capturing(input: &str) -> (Result<Value, LoxError>, String) {
    let interpreter = Interpreter::new(Vec::new());
    let result = run(input, &interpreter);
    let output = String::from_utf8(interpreter.out.borrow().clone()).unwrap();
    (result, output)
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

// === Grouping ===

#[test]
fn test_group_literal() {
    let result = interpret("(42);").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_group_expression() {
    let result = interpret("(1 + 2);").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_group_overrides_precedence() {
    // without grouping: 1 + 2 * 3 = 7; with grouping: (1 + 2) * 3 = 9
    let result = interpret("(1 + 2) * 3;").unwrap();
    assert_eq!(result, Value::Number(9.0));
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

// === Variable Declaration ===

#[test]
fn test_var_declaration() {
    let result = interpret("var x = 42; x;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_var_string() {
    let result = interpret("var s = \"hello\"; s;").unwrap();
    assert_eq!(result, Value::Str("hello".to_string()));
}

#[test]
fn test_var_in_expression() {
    let result = interpret("var x = 2; x * 3;").unwrap();
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn test_undefined_variable_is_error() {
    let result = interpret("x;");
    assert!(result.is_err());
}

// === Variable Assignment ===

#[test]
fn test_assignment() {
    let result = interpret("var x = 1; x = 42; x;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_assignment_returns_value() {
    // Assignment is an expression; it returns the assigned value.
    let result = interpret("var x = 1; x = 42;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_assignment_overwrites_previous_value() {
    let result = interpret("var x = 10; x = 20; x = 30; x;").unwrap();
    assert_eq!(result, Value::Number(30.0));
}

#[test]
fn test_chained_assignment() {
    // Right-associative: x = (y = 42)
    let result = interpret("var x = 0; var y = 0; x = y = 42; x;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_chained_assignment_sets_both_vars() {
    let result = interpret("var x = 0; var y = 0; x = y = 42; y;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_assignment_to_undeclared_is_error() {
    let result = interpret("x = 42;");
    assert!(result.is_err());
}

// === Block Scoping ===

#[test]
fn test_block_outer_variable_readable_inside() {
    // y = x inside the block — proves x is readable from the inner scope.
    let result = interpret("var x = 42; var y = 0; { y = x; } y;").unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_block_inner_variable_not_visible_outside() {
    let result = interpret("{ var x = 42; } x;");
    assert!(result.is_err());
}

#[test]
fn test_block_inner_shadows_outer() {
    // The inner x shadows the outer; outer is unchanged after block exits.
    let result = interpret("var x = 1; { var x = 99; } x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_block_assignment_to_outer_persists() {
    // Assigning to an outer variable from inside a block updates it.
    let result = interpret("var x = 1; { x = 99; } x;").unwrap();
    assert_eq!(result, Value::Number(99.0));
}

#[test]
fn test_nested_blocks_with_shadowing() {
    let result = interpret("var x = 1; { var x = 2; { var x = 3; } } x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

// === If Statements ===

#[test]
fn test_if_then_branch_taken() {
    let result = interpret("var x = 0; if (true) x = 1; x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_if_then_branch_skipped_without_else() {
    let result = interpret("var x = 0; if (false) x = 1; x;").unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_if_else_branch_taken() {
    let result = interpret("var x = 0; if (false) x = 1; else x = 2; x;").unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_if_truthy_non_boolean_condition() {
    // Numbers are truthy, so the then-branch runs.
    let result = interpret("var x = 0; if (1) x = 1; x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_if_nil_condition_is_falsey() {
    let result = interpret("var x = 0; if (nil) x = 1; else x = 2; x;").unwrap();
    assert_eq!(result, Value::Number(2.0));
}

#[test]
fn test_if_block_body_opens_scope() {
    // A block branch declares its own x; the outer x is untouched.
    let result = interpret("var x = 1; if (true) { var x = 9; } x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_if_print_in_branch() {
    let (result, output) = interpret_capturing("if (true) print 42;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "42\n");
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

// === Output Errors ===

/// A writer that always fails, simulating a closed/broken stdout (e.g. piping
/// `lox script.lox | head`).
struct FailingWriter;

impl std::io::Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "broken pipe",
        ))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_print_to_broken_writer_is_error_not_panic() {
    let interpreter = Interpreter::new(FailingWriter);
    let result = run("print 1;", &interpreter);
    assert!(matches!(result, Err(LoxError::Io(_))));
}

// === Logical Operators ===

// --- Operand-value return ---

#[test]
fn test_or_returns_first_truthy() {
    assert_eq!(interpret("1 or 2;").unwrap(), Value::Number(1.0));
}

#[test]
fn test_or_returns_last_when_all_falsey() {
    assert_eq!(interpret("false or nil;").unwrap(), Value::Nil);
}

#[test]
fn test_or_returns_truthy_after_falsey() {
    assert_eq!(
        interpret("nil or \"x\";").unwrap(),
        Value::Str("x".to_string())
    );
}

#[test]
fn test_and_returns_first_falsey() {
    assert_eq!(interpret("nil and 1;").unwrap(), Value::Nil);
}

#[test]
fn test_and_returns_last_when_all_truthy() {
    assert_eq!(interpret("1 and 2;").unwrap(), Value::Number(2.0));
}

// --- Short-circuit via assignment side effects ---

#[test]
fn test_or_short_circuits_on_truthy() {
    // true or (x = 1) should NOT evaluate the rhs
    let result = interpret("var x = 0; true or (x = 1); x;").unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_or_evaluates_rhs_on_falsey() {
    let result = interpret("var z = 0; false or (z = 1); z;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_and_short_circuits_on_falsey() {
    // false and (y = 1) should NOT evaluate the rhs
    let result = interpret("var y = 0; false and (y = 1); y;").unwrap();
    assert_eq!(result, Value::Number(0.0));
}

#[test]
fn test_and_evaluates_rhs_on_truthy() {
    let result = interpret("var w = 0; true and (w = 1); w;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

// --- Short-circuit avoids errors ---

#[test]
fn test_or_short_circuit_avoids_error() {
    // The rhs would error (nil < 1), but it's never evaluated
    assert!(interpret("true or (nil < 1);").is_ok());
}

#[test]
fn test_and_short_circuit_avoids_error() {
    // The rhs would error (nil < 1), but it's never evaluated
    assert!(interpret("false and (nil < 1);").is_ok());
}

// --- Precedence ---

#[test]
fn test_or_in_assignment() {
    // a = false or 5  ⟹  a = (false or 5)  ⟹  a = 5
    let result = interpret("var a = true; a = false or 5; a;").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

// === While Loops ===

#[test]
fn test_while_counter_loop() {
    let result =
        interpret("var i = 0; var s = 0; while (i < 3) { s = s + i; i = i + 1; } s;").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_while_false_from_start() {
    let result = interpret("var x = 1; while (false) { x = 99; } x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_while_single_iteration() {
    let result = interpret("var x = 0; while (x < 1) { x = x + 1; } x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
}

#[test]
fn test_while_block_scope_per_iteration() {
    // A var declared inside the loop body is not visible after the loop.
    let result = interpret("var x = 0; while (x < 1) { var y = 42; x = x + 1; } x;").unwrap();
    assert_eq!(result, Value::Number(1.0));
    assert!(interpret("var x = 0; while (x < 1) { var y = 42; x = x + 1; } y;").is_err());
}

#[test]
fn test_while_with_print() {
    let (result, output) = interpret_capturing("var i = 0; while (i < 3) { print i; i = i + 1; }");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "0\n1\n2\n");
}

#[test]
fn test_while_with_logical_condition() {
    let result = interpret("var x = 0; while (x < 5 and x < 3) { x = x + 1; } x;").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

// === For Loops ===

#[test]
fn test_for_classic_counter() {
    let result = interpret("var s = 0; for (var i = 0; i < 4; i = i + 1) s = s + i; s;").unwrap();
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn test_for_loop_var_scoped() {
    // The loop variable is scoped to the for block; accessing it after is an error.
    let result = interpret("for (var i = 0; i < 1; i = i + 1) i; i;");
    assert!(result.is_err());
}

#[test]
fn test_for_expression_initializer() {
    let result =
        interpret("var i = 0; var s = 0; for (i = 1; i < 4; i = i + 1) s = s + i; s;").unwrap();
    assert_eq!(result, Value::Number(6.0));
}

#[test]
fn test_for_empty_initializer() {
    let result = interpret("var i = 0; for (; i < 3; i = i + 1) i; i;").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_for_empty_increment() {
    let result = interpret("var i = 0; for (; i < 3;) { i = i + 1; } i;").unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn test_for_with_print() {
    let (result, output) = interpret_capturing("for (var i = 0; i < 3; i = i + 1) print i;");
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "0\n1\n2\n");
}

#[test]
fn test_for_fibonacci() {
    let (result, output) = interpret_capturing(
        "var a = 0; var temp = 0; for (var b = 1; a < 10000; b = temp + b) { print a; temp = a; a = b; }",
    );
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(
        output,
        "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n89\n144\n233\n377\n610\n987\n1597\n2584\n4181\n6765\n"
    );
}

// === Function Calls (native functions) ===

#[test]
fn test_clock_returns_number() {
    let result = interpret("clock();").unwrap();
    assert!(matches!(result, Value::Number(_)));
}

#[test]
fn test_clock_value_displays_as_native_fn() {
    let (result, output) = interpret_capturing("print clock;");
    assert!(result.is_ok());
    assert_eq!(output, "<native fn clock>\n");
}

#[test]
fn test_call_arity_mismatch() {
    let err = interpret("clock(1);").unwrap_err();
    assert!(matches!(
        err,
        LoxError::ArityMismatch {
            expected: 0,
            got: 1
        }
    ));
}

#[test]
fn test_call_non_callable_literal() {
    let err = interpret("42();").unwrap_err();
    assert!(matches!(err, LoxError::NotCallable(_)));
}

#[test]
fn test_call_non_callable_variable() {
    let err = interpret("var x = 1; x();").unwrap_err();
    assert!(matches!(err, LoxError::NotCallable(_)));
}

// === Function Declarations ===

#[test]
fn test_function_define_and_call() {
    let (result, output) = interpret_capturing(
        r#"
        fun f() { print 1; }
        f();
        "#,
    );
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "1\n");
}

#[test]
fn test_function_params_bind() {
    let (result, output) = interpret_capturing(
        r#"
        fun add(a, b) { print a + b; }
        add(1, 2);
        "#,
    );
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "3\n");
}

#[test]
fn test_function_implicit_nil_return() {
    let result = interpret(
        r#"
        fun f() {}
        f();
        "#,
    );
    assert_eq!(result.unwrap(), Value::Nil);
}

#[test]
fn test_function_recursion_via_side_effects() {
    // No `return` yet (that is issue #17), so recursion is observed via prints.
    let (result, output) = interpret_capturing(
        r#"
        fun count(n) {
            if (n > 0) {
                print n;
                count(n - 1);
            }
        }
        count(3);
        "#,
    );
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "3\n2\n1\n");
}

#[test]
fn test_function_arity_mismatch() {
    let err = interpret(
        r#"
        fun f(a) {}
        f();
        "#,
    )
    .unwrap_err();
    assert!(matches!(
        err,
        LoxError::ArityMismatch {
            expected: 1,
            got: 0
        }
    ));
}

#[test]
fn test_function_value_displays_as_fn() {
    let (result, output) = interpret_capturing(
        r#"
        fun f() {}
        print f;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(output, "<fn f>\n");
}

#[test]
fn test_function_closure_captures_enclosing_scope() {
    // Full closure coverage is issue #18; this checks the basic capture.
    let (result, output) = interpret_capturing(
        r#"
        fun outer() {
            var x = 10;
            fun inner() { print x; }
            inner();
        }
        outer();
        "#,
    );
    assert_eq!(result.unwrap(), Value::Nil);
    assert_eq!(output, "10\n");
}

// === Cross-run persistence (eager parse + borrowed AST, issue #20) ===

#[test]
fn test_function_persists_across_separate_runs() {
    // A function defined in one run() call must remain callable in a later call
    // on the same interpreter (the REPL case): its borrowed body outlives the
    // first parse because the program is leaked to 'static.
    let interpreter = Interpreter::new(Vec::new());
    run("fun greet() { print 42; }", &interpreter).unwrap();
    run("greet();", &interpreter).unwrap();
    let output = String::from_utf8(interpreter.out.borrow().clone()).unwrap();
    assert_eq!(output, "42\n");
}

#[test]
fn test_closure_persists_across_separate_runs() {
    // A closure created in one run() still sees its captured scope when invoked
    // from a later run() — its borrowed body and captured environment both
    // outlive the first parse. (`return` is not available until #17, so the
    // inner closure is exposed by assigning it to a global.)
    let interpreter = Interpreter::new(Vec::new());
    run(
        r#"
        var saved = nil;
        fun make() {
            var x = 7;
            fun inner() { print x; }
            saved = inner;
        }
        make();
        "#,
        &interpreter,
    )
    .unwrap();
    run("saved();", &interpreter).unwrap();
    let output = String::from_utf8(interpreter.out.borrow().clone()).unwrap();
    assert_eq!(output, "7\n");
}
