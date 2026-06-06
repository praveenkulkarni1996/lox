use lox_parser::*;

// === Primary Literals ===

#[test]
fn test_number_literal() {
    let ast = parse("42;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(AstLogicAnd::And(AstEquality::Comparison(
            AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                AstPrimary::Number(n)
            ))))
        ), None), None)))))) if n == 42.0
    ));
}

#[test]
fn test_decimal_number() {
    let ast = parse("1.25;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(AstLogicAnd::And(AstEquality::Comparison(
            AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                AstPrimary::Number(n)
            ))))
        ), None), None)))))) if (n - 1.25).abs() < f64::EPSILON
    ));
}

#[test]
fn test_string_literal() {
    let ast = parse("\"hello\";").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(AstLogicAnd::And(AstEquality::Comparison(
            AstComparison::Term(AstTerm::Factor(AstFactor::Unary(AstUnary::Primary(
                AstPrimary::Str(ref s)
            ))))
        ), None), None)))))) if s == "hello"
    ));
}

#[test]
fn test_true_literal() {
    let ast = parse("true;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Primary(AstPrimary::True))
                    ))),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Primary(AstPrimary::False))
                    ))),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Primary(AstPrimary::Nil))
                    ))),
                    None
                ),
                None
            )))
        )))
    ));
}

#[test]
fn test_identifier() {
    let ast = parse("foo;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
            AstAssignment::LogicOr(AstLogicOr::Or(AstLogicAnd::And(AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Unary(
                AstUnary::Primary(AstPrimary::Id(ref name))
            )))), None), None))
        )))) if name == "foo"
    ));
}

// === Unary Expressions ===

#[test]
fn test_negation() {
    let ast = parse("-5;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Negative(_))
                    ))),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Not(_))
                    ))),
                    None
                ),
                None
            )))
        )))
    ));
}

#[test]
fn test_double_negation() {
    let ast = parse("--1;").unwrap();
    // --1 should be Negative(Negative(Primary(1)))
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Unary(
                    AstUnary::Negative(inner),
                )))),
                None,
            ),
            None,
        )),
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
        AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Unary(
                    AstUnary::Not(inner),
                )))),
                None,
            ),
            None,
        )),
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Mul(
                        _,
                        _
                    )))),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Div(
                        _,
                        _
                    )))),
                    None
                ),
                None
            )))
        )))
    ));
}

#[test]
fn test_chained_multiplication() {
    // 2 * 3 * 4 should be left-associative: Mul(Mul(2, 3), 4)
    let ast = parse("2 * 3 * 4;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Mul(
                    lhs,
                    _rhs,
                )))),
                None,
            ),
            None,
        )),
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Add(_, _))),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Sub(_, _))),
                    None
                ),
                None
            )))
        )))
    ));
}

#[test]
fn test_chained_addition() {
    // 1 + 2 + 3 should be left-associative: Add(Add(1, 2), 3)
    let ast = parse("1 + 2 + 3;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Add(lhs, _rhs))),
                None,
            ),
            None,
        )),
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(AstEquality::Comparison(AstComparison::Less(_, _)), None),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::LessEqual(_, _)),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(AstEquality::Comparison(AstComparison::Greater(_, _)), None),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::GreaterEqual(_, _)),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(AstEquality::Equal(_, _), None),
                None
            )))
        )))
    ));
}

#[test]
fn test_not_equal() {
    let ast = parse("1 != 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(AstEquality::NotEqual(_, _), None),
                None
            )))
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(
                        AstFactor::Unary(AstUnary::Primary(AstPrimary::Group(_)))
                    ))),
                    None
                ),
                None
            )))
        )))
    ));
}

#[test]
fn test_grouped_addition() {
    // (1 + 2) should parse the addition inside the group
    let ast = parse("(1 + 2);").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Unary(
                    AstUnary::Primary(AstPrimary::Group(expr)),
                )))),
                None,
            ),
            None,
        )),
    )))) = ast
    {
        assert!(matches!(
            *expr,
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Add(_, _))),
                    None,
                ),
                None,
            )))
        ));
    } else {
        panic!("unexpected AST shape");
    }
}

#[test]
fn test_unclosed_paren() {
    let result = parse("(1 + 2;");
    assert!(result.is_none());
}

// === Precedence ===

#[test]
fn test_mul_before_add() {
    // 1 + 2 * 3 should parse as Add(1, Mul(2, 3))
    let ast = parse("1 + 2 * 3;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(
            AstLogicAnd::And(
                AstEquality::Comparison(AstComparison::Term(AstTerm::Add(lhs, rhs))),
                None,
            ),
            None,
        )),
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(
                    AstEquality::Comparison(AstComparison::Term(AstTerm::Factor(AstFactor::Mul(
                        _,
                        _
                    )))),
                    None
                ),
                None
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
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(AstEquality::Equal(_, _), None),
                None
            )))
        )))
    ));
}

// === Logical Operators ===

#[test]
fn test_logic_or() {
    let ast = parse("a or b;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(_, Some(_))))
        )))
    ));
}

#[test]
fn test_logic_and() {
    let ast = parse("a and b;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(_, Some(_)),
                None,
            )))
        )))
    ));
}

#[test]
fn test_chained_or() {
    // a or b or c → Or(<a>, Some(Or(<b>, Some(Or(<c>, None)))))
    let ast = parse("a or b or c;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(_, Some(tail))),
    )))) = ast
    {
        assert!(matches!(*tail, AstLogicOr::Or(_, Some(_))));
    } else {
        panic!("expected chained Or");
    }
}

#[test]
fn test_chained_and() {
    // a and b and c → And(<a>, Some(And(<b>, Some(And(<c>, None)))))
    let ast = parse("a and b and c;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(AstLogicAnd::And(_, Some(tail)), None)),
    )))) = ast
    {
        assert!(matches!(*tail, AstLogicAnd::And(_, Some(_))));
    } else {
        panic!("expected chained And");
    }
}

#[test]
fn test_and_binds_tighter_than_or() {
    // a or b and c → Or(And(<a>, None), Some(Or(And(<b>, Some(And(<c>, None))), None)))
    let ast = parse("a or b and c;").unwrap();
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::LogicOr(AstLogicOr::Or(AstLogicAnd::And(_, None), Some(tail))),
    )))) = ast
    {
        // The tail's head should be an And with a tail (b and c)
        if let AstLogicOr::Or(AstLogicAnd::And(_, Some(_)), None) = *tail {
            // good — `and` grouped `b` and `c` together
        } else {
            panic!("expected And(b, Some(And(c, None))) in tail");
        }
    } else {
        panic!("expected Or at top level");
    }
}

#[test]
fn test_equality_below_and() {
    // 1 == 1 and 2 → And(Equal(1, 1), Some(And(<2>, None)))
    let ast = parse("1 == 1 and 2;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::LogicOr(AstLogicOr::Or(
                AstLogicAnd::And(AstEquality::Equal(_, _), Some(_)),
                None,
            )))
        )))
    ));
}

#[test]
fn test_assignment_with_or() {
    // x = a or b → Assign("x", LogicOr(Or(_, Some(_))))
    let ast = parse("x = a or b;").unwrap();
    assert!(matches!(
        ast,
        Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(
            AstExpression::Assignment(AstAssignment::Assign(_, _))
        )))
    ));
    // Verify the RHS is an Or
    if let Ast::Declare(AstDeclaration::Statement(AstStatement::Expr(AstExpression::Assignment(
        AstAssignment::Assign(_, rhs),
    )))) = ast
    {
        assert!(matches!(
            *rhs,
            AstAssignment::LogicOr(AstLogicOr::Or(_, Some(_)))
        ));
    } else {
        panic!("expected Assign");
    }
}

// === Malformed Expressions ===

#[test]
fn test_empty_input() {
    let result = parse("");
    assert!(result.is_none());
}

#[test]
fn test_missing_operand() {
    let result = parse("1 +;");
    assert!(result.is_none());
}
