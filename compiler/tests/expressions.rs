use bowfin_compiler::{
    ast, compile_source, hir, parse_source, verify::verify_report, FrontendError,
};

const REQUIRES: &str = "amount <= battery.charge";
const ENSURES: &str = "battery.charge == old(battery.charge) - amount";
fn source(requires: &str, ensures: &str, body: &str) -> String {
    format!("type Percent = range 0..100;\nrecord Battery {{ charge: Percent }}\nverified fn consume(battery: &mut Battery, amount: Percent)\nrequires {requires}\nensures {ensures}\n{{ {body} }}")
}
fn ast_binary(expression: &ast::Expr, expected: ast::BinaryOp) -> (&ast::Expr, &ast::Expr) {
    let ast::ExprKind::Binary { op, left, right } = &expression.kind else {
        panic!("expected binary expression")
    };
    assert_eq!(*op, expected);
    (left, right)
}
fn type_error(source: &str, fragment: &str) {
    let Err(FrontendError::Type(errors)) = compile_source(source) else {
        panic!("expected type error: {source}")
    };
    assert!(
        errors.iter().any(|error| error.message.contains(fragment)),
        "{errors:?}"
    );
    for error in errors {
        let rendered = error.render("expression.bow", source);
        assert!(rendered.contains("expression.bow:"));
        assert!(!rendered.contains("RangeTypeId"));
        assert!(!rendered.contains("ParameterId"));
        assert!(error.span.end > error.span.start);
    }
}

#[test]
fn expression_precedence_and_source_structure_are_explicit() {
    use ast::BinaryOp::*;
    let text = source(
        "!false || true && amount - 1 - 2 <= battery.charge == true",
        ENSURES,
        "",
    );
    let ast = parse_source(&text).unwrap();
    let expression = &ast.functions[0].as_verified().unwrap().requires;
    let (not, and) = ast_binary(expression, Or);
    assert!(matches!(
        not.kind,
        ast::ExprKind::Unary {
            op: ast::UnaryOp::Not,
            ..
        }
    ));
    let (boolean, equality) = ast_binary(and, And);
    assert_eq!(boolean.kind, ast::ExprKind::BoolLiteral(true));
    let (comparison, truth) = ast_binary(equality, Equal);
    assert_eq!(truth.kind, ast::ExprKind::BoolLiteral(true));
    let (subtraction, field) = ast_binary(comparison, LessEqual);
    assert!(matches!(&field.kind, ast::ExprKind::FieldAccess(access) if access.field == "charge"));
    let (first, two) = ast_binary(subtraction, Subtract);
    assert_eq!(two.kind, ast::ExprKind::IntegerLiteral(2));
    let (amount, one) = ast_binary(first, Subtract);
    assert_eq!(amount.kind, ast::ExprKind::Name("amount".into()));
    assert_eq!(one.kind, ast::ExprKind::IntegerLiteral(1));
    assert_eq!(
        &text[expression.span.start..expression.span.end],
        "!false || true && amount - 1 - 2 <= battery.charge == true"
    );
    assert_eq!(
        compile_source(&text).unwrap().functions()[0]
            .as_verified()
            .unwrap()
            .requires
            .ty,
        hir::ExprType::Bool
    );
}

#[test]
fn parentheses_and_boolean_associativity_preserve_grouping() {
    for (text, op) in [
        ("true || false || true", ast::BinaryOp::Or),
        ("true && false && true", ast::BinaryOp::And),
    ] {
        let ast = parse_source(&source(text, ENSURES, "")).unwrap();
        let (left, _) = ast_binary(&ast.functions[0].as_verified().unwrap().requires, op);
        ast_binary(left, op);
    }
    let ast = parse_source(&source("(true || false) && true", ENSURES, "")).unwrap();
    let (left, _) = ast_binary(
        &ast.functions[0].as_verified().unwrap().requires,
        ast::BinaryOp::And,
    );
    ast_binary(left, ast::BinaryOp::Or);
    let program = compile_source(&source(
        "(((amount))) <= ((battery.charge))",
        "((battery.charge)) == ((old((battery.charge))) - (amount))",
        "battery.charge -= ((amount));",
    ))
    .unwrap();
    assert!(verify_report(&program).diagnostics.is_empty());
}

#[test]
fn comparisons_are_non_associative_without_parentheses() {
    for requires in [
        "amount <= battery.charge <= amount",
        "true == false == true",
    ] {
        let Err(FrontendError::Parse(error)) = parse_source(&source(requires, ENSURES, "")) else {
            panic!("expected chained comparison error")
        };
        assert!(error.message.contains("non-associative"));
    }
    assert!(compile_source(&source("(1 <= 2) == true", ENSURES, "")).is_ok());
    type_error(
        &source("1 <= (2 <= 3)", ENSURES, ""),
        "incompatible operand types",
    );
}

#[test]
fn signed_literals_remain_i64_constants_not_unary_numeric_expressions() {
    let program = compile_source(&source(
        "-9223372036854775808 <= 9223372036854775807",
        ENSURES,
        "battery.charge -= -5;",
    ))
    .unwrap();
    let hir::ExprKind::Binary { left, right, .. } =
        &program.functions()[0].as_verified().unwrap().requires.kind
    else {
        panic!("expected comparison")
    };
    assert_eq!(left.kind, hir::ExprKind::IntegerLiteral(i64::MIN));
    assert_eq!(right.kind, hir::ExprKind::IntegerLiteral(i64::MAX));
    assert_eq!(left.ty, hir::ExprType::IntegerLiteral);
    for expression in [
        "-amount <= 0",
        "9223372036854775808 <= 0",
        "-9223372036854775809 <= 0",
    ] {
        assert!(matches!(
            parse_source(&source(expression, ENSURES, "")),
            Err(FrontendError::Parse(_))
        ));
    }
}

#[test]
fn expression_references_resolve_before_type_checking() {
    for (requires, ensures, body) in [
        ("missing <= amount", ENSURES, ""),
        (REQUIRES, "battery.charge == old(other.charge) - amount", ""),
        (REQUIRES, ENSURES, "battery.charge -= missing;"),
    ] {
        assert!(matches!(
            compile_source(&source(requires, ensures, body)),
            Err(FrontendError::Resolve(_))
        ));
    }
    let program = compile_source(&source(REQUIRES, ENSURES, "battery.charge -= amount;")).unwrap();
    let function = &program.functions()[0].as_verified().unwrap();
    let hir::ExprKind::Binary { right, .. } = &function.ensures.kind else {
        panic!("expected equality")
    };
    let hir::ExprKind::Binary { left, right, .. } = &right.kind else {
        panic!("expected subtraction")
    };
    let hir::ExprKind::OldField(access) = &left.kind else {
        panic!("expected old field")
    };
    assert_eq!(access.field, program.records()[0].fields[0]);
    assert_eq!(access.parameter, function.state_param);
    assert_eq!(left.ty, hir::ExprType::Range(access.ty));
    assert_eq!(
        right.kind,
        hir::ExprKind::Parameter(function.amount_param.parameter)
    );
    assert_eq!(right.ty, left.ty);
}

#[test]
fn boolean_operators_require_boolean_operands() {
    for expression in [
        "true",
        "false",
        "!false",
        "!!true",
        "true && false",
        "false || true",
        "true == false",
        "!(amount <= battery.charge)",
    ] {
        let program = compile_source(&source(expression, "true", "")).unwrap();
        assert_eq!(
            program.functions()[0].as_verified().unwrap().requires.ty,
            hir::ExprType::Bool
        );
    }
    for expression in [
        "amount && true",
        "true && amount",
        "false || 1",
        "1 || false",
        "!amount",
        "!1",
        "amount == true",
        "true == amount",
        "true <= false",
    ] {
        type_error(
            &source(expression, ENSURES, ""),
            if expression.starts_with('!') {
                "negation requires Bool"
            } else {
                "incompatible operand types"
            },
        );
    }
}

#[test]
fn contracts_and_subtract_assignment_have_separate_type_obligations() {
    type_error(
        &source("amount", ENSURES, ""),
        "requires expression must have type Bool",
    );
    type_error(
        &source(REQUIRES, "old(battery.charge) - amount", ""),
        "ensures expression must have type Bool",
    );
    type_error(
        &source(REQUIRES, ENSURES, "battery.charge -= true;"),
        "subtract-assignment",
    );
    type_error(
        &source("battery == battery", ENSURES, ""),
        "record parameter `battery` is not a numeric expression",
    );
}

#[test]
fn same_range_and_literal_operators_compute_canonical_types() {
    for expression in [
        "amount <= battery.charge",
        "battery.charge == amount",
        "amount <= 5",
        "5 <= amount",
        "amount == 5",
        "5 == amount",
        "1 <= 2",
        "1 == 2",
        "(battery.charge - amount) <= 0",
        "(battery.charge - 5) == amount",
    ] {
        assert_eq!(
            compile_source(&source(expression, ENSURES, ""))
                .unwrap()
                .functions()[0]
                .as_verified()
                .unwrap()
                .requires
                .ty,
            hir::ExprType::Bool
        );
    }
    for rhs in ["amount", "5", "amount - 0", "battery.charge - amount"] {
        let program = compile_source(&source(
            REQUIRES,
            ENSURES,
            &format!("battery.charge -= {rhs};"),
        ))
        .unwrap();
        let operand = &program.functions()[0].as_verified().unwrap().body[0].operand;
        assert_eq!(
            operand.ty,
            if rhs == "5" {
                hir::ExprType::IntegerLiteral
            } else {
                hir::ExprType::Range(program.ranges()[0].id)
            }
        );
    }
    for expression in [
        "(5 - amount) == 0",
        "(5 - 1) == 0",
        "(amount - true) == 0",
        "(true - amount) == 0",
    ] {
        type_error(
            &source(expression, ENSURES, ""),
            "incompatible operand types",
        );
    }
}

#[test]
fn distinct_nominal_types_are_rejected_for_each_operator_and_assignment() {
    for bounds in ["0..100", "0..50"] {
        for (requires, ensures, body, operation) in [
            ("battery.charge <= amount", "true", "", "<="),
            ("battery.charge == amount", "true", "", "=="),
            ("true", "(battery.charge - amount) == 0", "", "subtraction"),
            (
                "true",
                "true",
                "battery.charge -= amount;",
                "subtract-assignment",
            ),
        ] {
            let text = source(requires, ensures, body)
                .replace(
                    "record Battery",
                    &format!("type Other = range {bounds}; record Battery"),
                )
                .replace("amount: Percent", "amount: Other");
            type_error(
                &text,
                &format!("incompatible named range types for {operation}: `Percent` and `Other`"),
            );
        }
    }
}

#[test]
fn old_is_restricted_to_the_state_field_in_ensures() {
    for (requires, body) in [
        ("old(battery.charge) <= amount", ""),
        (REQUIRES, "battery.charge -= old(battery.charge);"),
    ] {
        let Err(FrontendError::Resolve(errors)) = compile_source(&source(requires, ENSURES, body))
        else {
            panic!("expected old context restriction")
        };
        assert!(errors[0].message.contains("only in ensures"));
    }
    for old in ["old(amount - 1)", "old(true)", "old(amount)"] {
        assert!(matches!(
            parse_source(&source(REQUIRES, &format!("battery.charge == {old}"), "")),
            Err(FrontendError::Parse(_))
        ));
    }
    assert!(matches!(
        compile_source(&source(REQUIRES, "battery.charge == old(other.charge)", "")),
        Err(FrontendError::Resolve(_))
    ));
}

#[test]
fn well_typed_richer_expressions_reach_only_the_verifier_support_boundary() {
    for (requires, ensures, body, role) in [
        (
            "(amount <= battery.charge) && true",
            ENSURES,
            "battery.charge -= amount;",
            "requires expression",
        ),
        (
            "true || false",
            ENSURES,
            "battery.charge -= amount;",
            "requires expression",
        ),
        (
            REQUIRES,
            "(battery.charge == old(battery.charge) - amount) && true",
            "battery.charge -= amount;",
            "ensures expression",
        ),
        (
            REQUIRES,
            ENSURES,
            "battery.charge -= amount - 0;",
            "subtract-assignment operand",
        ),
        (
            "battery.charge <= amount",
            ENSURES,
            "battery.charge -= amount;",
            "requires expression",
        ),
        (
            REQUIRES,
            "old(battery.charge) - amount == battery.charge",
            "battery.charge -= amount;",
            "ensures expression",
        ),
    ] {
        let text = source(requires, ensures, body);
        let program = compile_source(&text).expect("well-typed source must compile to HIR");
        let report = verify_report(&program);
        assert_eq!(report.functions_proven, 0);
        assert_eq!(report.diagnostics.len(), 1);
        let error = &report.diagnostics[0];
        assert_eq!(
            error.message,
            "well-typed expression is not yet supported by the prototype verifier"
        );
        assert!(error.required.contains(role));
        let expr_span = if role == "requires expression" {
            program.functions()[0].as_verified().unwrap().requires.span
        } else if role == "ensures expression" {
            program.functions()[0].as_verified().unwrap().ensures.span
        } else {
            program.functions()[0].as_verified().unwrap().body[0]
                .operand
                .span
        };
        assert_eq!(error.span, expr_span);
        assert!(error
            .render("unsupported.bow", &text)
            .contains("unsupported.bow:"));
    }
}

#[test]
fn typed_subtraction_does_not_establish_range_or_overflow_safety() {
    let text = source(REQUIRES, ENSURES, "battery.charge -= amount;").replace("0..100", "10..100");
    let program = compile_source(&text).unwrap();
    let report = verify_report(&program);
    assert!(report.diagnostics[0]
        .message
        .contains("field range preservation"));
    let text = source(REQUIRES, ENSURES, "battery.charge -= amount;")
        .replace("0..100", "-9223372036854775808..9223372036854775807");
    let report = verify_report(&compile_source(&text).unwrap());
    assert!(report.diagnostics[0]
        .message
        .contains("subtraction overflow"));
}

#[test]
fn prohibited_operators_and_verified_expression_calls_remain_rejected() {
    for expression in [
        "amount + 1",
        "amount * 1",
        "amount / 1",
        "amount % 1",
        "amount < 1",
        "amount > 1",
        "amount >= 1",
        "amount != 1",
        "amount +% 1",
        "amount +? 1",
        "amount +^ 1",
    ] {
        assert!(
            parse_source(&source(expression, ENSURES, "")).is_err(),
            "{expression}"
        );
    }
    let Err(FrontendError::Resolve(errors)) =
        compile_source(&source("consume(amount)", ENSURES, ""))
    else {
        panic!("verified expression calls must remain rejected")
    };
    assert!(errors[0]
        .message
        .contains("only between ordinary value functions"));
}
