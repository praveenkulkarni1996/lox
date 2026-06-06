use lox_parser::*;

// === Expression Statements ===

#[test]
fn test_missing_semicolon() {
    let result = parse("42");
    assert!(result.is_none());
}

// === Print Statement ===

#[test]
fn test_print_statement() {
    let ast = parse("print 42;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Print(_)))
    ));
}

#[test]
fn test_print_expression() {
    let ast = parse("print 1 + 2;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Print(
        AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Add(_, _))),
                None,
            ),
            None,
        ))),
    ))) = ast
    {
        // ok
    } else {
        panic!("expected Print(Add(...))");
    }
}

// === Assignment ===

#[test]
fn test_simple_assignment() {
    let ast = parse("x = 42;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Assign(ref name, _))
        ))) if name == "x"
    ));
}

#[test]
fn test_assignment_rhs_expression() {
    // x = 1 + 2 — the RHS should be an Add term
    let ast = parse("x = 1 + 2;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Assign(ref name, rhs),
    )))) = ast
    {
        assert_eq!(name, "x");
        assert!(matches!(
            *rhs,
            AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Add(_, _))),
                    None,
                ),
                None,
            ))
        ));
    } else {
        panic!("expected assignment");
    }
}

#[test]
fn test_chained_assignment() {
    // x = y = 42 should be right-associative: Assignment("x", Assignment("y", 42))
    let ast = parse("x = y = 42;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Assign(ref name, rhs),
    )))) = ast
    {
        assert_eq!(name, "x");
        assert!(matches!(*rhs, AstAssignment::Assign(ref inner, _) if inner == "y"));
    } else {
        panic!("expected chained assignment");
    }
}

#[test]
fn test_invalid_assignment_target() {
    // 42 = x should fail — non-identifier on the LHS
    let result = parse("42 = x;");
    assert!(result.is_none());
}

// === Block Statements ===

#[test]
fn test_empty_block() {
    let ast = parse("{}").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Block(ref decls))) if decls.is_empty()
    ));
}

#[test]
fn test_block_with_single_statement() {
    let ast = parse("{ 42; }").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Block(ref decls))) if decls.len() == 1
    ));
}

#[test]
fn test_block_with_multiple_declarations() {
    let ast = parse("{ var x = 1; 42; }").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Block(ref decls))) if decls.len() == 2
    ));
}

#[test]
fn test_nested_blocks() {
    let ast = parse("{ { 42; } }").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Block(ref decls))) if decls.len() == 1
    ));
}

#[test]
fn test_unterminated_block_is_error() {
    let result = parse("{ 42;");
    assert!(result.is_none());
}

// === If Statements ===

#[test]
fn test_if_without_else() {
    let ast = parse("if (true) 1;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::If(_, _, None)))
    ));
}

#[test]
fn test_if_with_else() {
    let ast = parse("if (true) 1; else 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::If(_, _, Some(_))))
    ));
}

#[test]
fn test_if_with_block_branches() {
    let ast = parse("if (x) { 1; } else { 2; }").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::If(
            _,
            ref then_branch,
            Some(ref else_branch),
        )))
            if matches!(**then_branch, AstStatement::Block(_))
                && matches!(**else_branch, AstStatement::Block(_))
    ));
}

#[test]
fn test_dangling_else_binds_to_nearest_if() {
    // The `else` binds to the inner `if`, so the outer `if` has no else branch.
    let ast = parse("if (a) if (b) 1; else 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::If(
            _,
            ref then_branch,
            None,
        )))
            if matches!(**then_branch, AstStatement::If(_, _, Some(_)))
    ));
}

#[test]
fn test_if_missing_open_paren_is_error() {
    let result = parse("if true) 1;");
    assert!(result.is_none());
}

#[test]
fn test_if_missing_body_is_error() {
    let result = parse("if (true)");
    assert!(result.is_none());
}

// === While Statements ===

#[test]
fn test_while_loop() {
    let ast = parse("while (true) print 1;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::While(
            _,
            ref body
        ))) if matches!(**body, AstStatement::Print(_))
    ));
}

#[test]
fn test_while_block_body() {
    let ast = parse("while (x) { print x; }").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::While(
            _,
            ref body
        ))) if matches!(**body, AstStatement::Block(_))
    ));
}

#[test]
fn test_while_missing_open_paren_is_error() {
    let result = parse("while true) print 1;");
    assert!(result.is_none());
}

#[test]
fn test_while_missing_body_is_error() {
    let result = parse("while (true)");
    assert!(result.is_none());
}

// === For Statements (desugared to Block + While) ===

#[test]
fn test_for_loop() {
    let for_ast = parse("for (var i = 0; i < 3; i = i + 1) print i;").unwrap();
    let while_ast = parse("{ var i = 0; while (i < 3) { print i; i = i + 1; } }").unwrap();
    assert_eq!(for_ast, while_ast);
}

#[test]
fn test_for_empty_initializer() {
    let for_ast = parse("for (; x < 3; x = x + 1) print x;").unwrap();
    let while_ast = parse("{ while (x < 3) { print x; x = x + 1; } }").unwrap();
    assert_eq!(for_ast, while_ast);
}

#[test]
fn test_for_empty_condition() {
    let for_ast = parse("for (var i = 0;; i = i + 1) print i;").unwrap();
    let while_ast = parse("{ var i = 0; while (true) { print i; i = i + 1; } }").unwrap();
    assert_eq!(for_ast, while_ast);
}

#[test]
fn test_for_empty_increment() {
    let for_ast = parse("for (var i = 0; i < 3;) print i;").unwrap();
    let while_ast = parse("{ var i = 0; while (i < 3) print i; }").unwrap();
    assert_eq!(for_ast, while_ast);
}

#[test]
fn test_for_all_empty() {
    let for_ast = parse("for (;;) print 1;").unwrap();
    let while_ast = parse("{ while (true) print 1; }").unwrap();
    assert_eq!(for_ast, while_ast);
}

#[test]
fn test_for_expression_initializer() {
    let for_ast = parse("for (i = 0; i < 3; i = i + 1) print i;").unwrap();
    let while_ast = parse("{ i = 0; while (i < 3) { print i; i = i + 1; } }").unwrap();
    assert_eq!(for_ast, while_ast);
}

#[test]
fn test_for_missing_open_paren_is_error() {
    let result = parse("for var i = 0; i < 3; i = i + 1) print i;");
    assert!(result.is_none());
}
