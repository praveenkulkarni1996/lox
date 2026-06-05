use lox_lexer::Lexer;
use lox_parser::*;

fn parse(input: &str) -> Option<Ast> {
    let tokens = Lexer::new(input.chars());
    let mut parser = Parser::new(tokens);
    parser.next()
}

// === Primary Literals ===

#[test]
fn test_number_literal() {
    let ast = parse("42;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
            AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                AstPrimary::Number(n)
            ))))
        )))))) if n == 42.0
    ));
}

#[test]
fn test_decimal_number() {
    let ast = parse("1.25;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
            AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                AstPrimary::Number(n)
            ))))
        )))))) if (n - 1.25).abs() < f64::EPSILON
    ));
}

#[test]
fn test_string_literal() {
    let ast = parse("\"hello\";").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
            AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                AstPrimary::Str(ref s)
            ))))
        )))))) if s == "hello"
    ));
}

#[test]
fn test_true_literal() {
    let ast = parse("true;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                    AstPrimary::True
                ))))
            )))
        )))
    ));
}

#[test]
fn test_false_literal() {
    let ast = parse("false;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                    AstPrimary::False
                ))))
            )))
        )))
    ));
}

#[test]
fn test_nil_literal() {
    let ast = parse("nil;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                    AstPrimary::Nil
                ))))
            )))
        )))
    ));
}

// === Unary Expressions ===

#[test]
fn test_negation() {
    let ast = parse("-5;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Negative(_))))
            )))
        )))
    ));
}

#[test]
fn test_not() {
    let ast = parse("!true;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Not(_))))
            )))
        )))
    ));
}

#[test]
fn test_double_negation() {
    let ast = parse("--1;").unwrap();
    // --1 should be Negative(Negative(Primary(1)))
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
            AstFactor::Unary(AstUnary::Negative(inner)),
        )))),
    )))) = ast
    {
        assert!(matches!(*inner, AstUnary::Negative(_)));
    } else {
        panic!("unexpected AST shape");
    }
}

#[test]
fn test_not_false() {
    let ast = parse("!false;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
            AstFactor::Unary(AstUnary::Not(inner)),
        )))),
    )))) = ast
    {
        assert!(matches!(*inner, AstUnary::Primary(AstPrimary::False)));
    } else {
        panic!("unexpected AST shape");
    }
}

// === Factor (Multiplication and Division) ===

#[test]
fn test_multiplication() {
    let ast = parse("2 * 3;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Mul(_, _)))
            )))
        )))
    ));
}

#[test]
fn test_division() {
    let ast = parse("10 / 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Div(_, _)))
            )))
        )))
    ));
}

#[test]
fn test_chained_multiplication() {
    // 2 * 3 * 4 should be left-associative: Mul(Mul(2, 3), 4)
    let ast = parse("2 * 3 * 4;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
            AstFactor::Mul(lhs, _rhs),
        )))),
    )))) = ast
    {
        assert!(matches!(*lhs, AstFactor::Mul(_, _)));
    } else {
        panic!("unexpected AST shape");
    }
}

// === Term (Addition and Subtraction) ===

#[test]
fn test_addition() {
    let ast = parse("1 + 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Add(_, _))
            )))
        )))
    ));
}

#[test]
fn test_subtraction() {
    let ast = parse("5 - 3;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Sub(_, _))
            )))
        )))
    ));
}

#[test]
fn test_chained_addition() {
    // 1 + 2 + 3 should be left-associative: Add(Add(1, 2), 3)
    let ast = parse("1 + 2 + 3;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Add(lhs, _rhs)))),
    )))) = ast
    {
        assert!(matches!(*lhs, AstTerm::Add(_, _)));
    } else {
        panic!("unexpected AST shape");
    }
}

// === Comparison ===

#[test]
fn test_less_than() {
    let ast = parse("1 < 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Less(_, _)
            )))
        )))
    ));
}

#[test]
fn test_less_equal() {
    let ast = parse("1 <= 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::LessEqual(_, _)
            )))
        )))
    ));
}

#[test]
fn test_greater_than() {
    let ast = parse("3 > 1;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Greater(_, _)
            )))
        )))
    ));
}

#[test]
fn test_greater_equal() {
    let ast = parse("3 >= 1;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::GreaterEqual(_, _)
            )))
        )))
    ));
}

// === Equality ===

#[test]
fn test_equal() {
    let ast = parse("1 == 1;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Equal(_, _)))
        )))
    ));
}

#[test]
fn test_not_equal() {
    let ast = parse("1 != 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::NotEqual(_, _)))
        )))
    ));
}

// === Grouping ===

#[test]
fn test_grouped_expression() {
    let ast = parse("(42);").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                    AstPrimary::Group(_)
                ))))
            )))
        )))
    ));
}

#[test]
fn test_grouped_addition() {
    // (1 + 2) should parse the addition inside the group
    let ast = parse("(1 + 2);").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
            AstFactor::Unary(AstUnary::Primary(AstPrimary::Group(expr))),
        )))),
    )))) = ast
    {
        assert!(matches!(
            *expr,
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Add(_, _))
            )))
        ));
    } else {
        panic!("unexpected AST shape");
    }
}

// === Precedence ===

#[test]
fn test_mul_before_add() {
    // 1 + 2 * 3 should parse as Add(1, Mul(2, 3))
    let ast = parse("1 + 2 * 3;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Add(lhs, rhs)))),
    )))) = ast
    {
        // lhs is just 1 (a Factor)
        assert!(matches!(*lhs, AstTerm::Factor(AstFactor::Unary(_))));
        // rhs is 2 * 3 (a Mul)
        assert!(matches!(rhs, AstFactor::Mul(_, _)));
    } else {
        panic!("expected Add at top level");
    }
}

#[test]
fn test_grouping_overrides_precedence() {
    // (1 + 2) * 3 should parse as Mul(Group(Add(1, 2)), 3)
    let ast = parse("(1 + 2) * 3;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(
                AstComparison::Term(AstTerm::Factor(AstFactor::Mul(_, _)))
            )))
        )))
    ));
}

#[test]
fn test_comparison_below_equality() {
    // 1 < 2 == true parses as Equal(Less(1, 2), true)
    let ast = parse("1 < 2 == true;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Eq(AstEquality::Equal(_, _)))
        )))
    ));
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
        AstExpression::Assignment(AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(
            AstTerm::Add(_, _),
        )))),
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
            AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Add(
                _,
                _
            ))))
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

// === Error Cases ===

#[test]
fn test_empty_input() {
    let result = parse("");
    assert!(result.is_none());
}

#[test]
fn test_unclosed_paren() {
    let result = parse("(1 + 2;");
    assert!(result.is_none());
}

#[test]
fn test_missing_operand() {
    let result = parse("1 +;");
    assert!(result.is_none());
}

#[test]
fn test_missing_semicolon() {
    let result = parse("42");
    assert!(result.is_none());
}

#[test]
fn test_identifier() {
    let ast = parse("foo;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
            AstAssignment::Eq(AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Unary(
                AstUnary::Primary(AstPrimary::Id(ref name))
            ))))
        ))))) if name == "foo"
    ));
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
