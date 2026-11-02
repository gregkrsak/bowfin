use bowfin_compiler::{compile_source, hir, mir, verify, FrontendError};
const DECL: &str = "type P = range 0..100; type Q = range 0..100; record Ticket { value: P } record Triple { a: P, b: Q, c: P } fn id(p: P) -> P { return p; } fn other(q: Q) -> Q { return q; } fn take(a: P,b: P,c: P) {} fn sum(a: P,b: P,c: P) -> P { return a; } fn consume(t: Ticket) -> P { return t.value; } fn sink(r: &mut P) {} fn inspect(r: &P) -> P { return *r; } fn predicate(t: Triple) -> bool { return true; }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL}\n{body}")).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    p
}
fn bad(body: &str, phase: &str, message: &str) -> (String, bowfin_compiler::lexer::Span) {
    let s = format!("{DECL}\n{body}");
    let (actual, text, span) = match compile_source(&s).unwrap_err() {
        FrontendError::Type(e) => (
            "Type",
            format!("{} {}", e[0].message, e[0].required),
            e[0].span,
        ),
        FrontendError::Ownership(e) => ("Ownership", e[0].message.clone(), e[0].span),
        FrontendError::Effects(e) => ("Effects", e[0].message.clone(), e[0].span),
        FrontendError::Resolve(e) => ("Resolve", e[0].message.clone(), e[0].span),
        FrontendError::Parse(e) => ("Parse", e.message, e.span),
        FrontendError::Lex(e) => ("Lex", e.message, e.span),
    };
    assert_eq!(actual, phase, "{text}");
    assert!(text.contains(message), "{text}");
    (s, span)
}
macro_rules! accepted { ($($name:ident: $s:literal;)*) => { $(#[test] fn $name() { good($s); })* }; }
accepted! {
    return_endpoints_and_grouped_literal: "fn f(flag: bool) -> P { if flag { return ((0)); } return 100; }";
    forward_and_recursive_calls: "fn f() -> P { return later(80); } fn later(p: P) -> P { return later(100); }";
    assignment_established_local: "fn f(p: P) -> P { let mut x = p; x = 80; return x; }";
    write_established_referent: "fn f(r: &mut P) { *r = 75; }";
    nested_independent_call_boundaries: "fn f() -> P { let x = id(id(100)); return id(x) - 1; }";
    construction_independent_field_boundaries: "fn f() -> Triple { return Triple { c: id(100), b: 80, a: 0 }; }";
    inner_boundaries_in_conditional: "fn f(flag: bool) -> P { let x = if flag { id(80) } else { id(100) }; return x; }";
    loop_body_and_early_return: "fn f(flag: bool) -> P { let mut x = id(0); while flag { x = 80; let r = &mut x; *r = 75; if flag { return 100; } continue; } return x; }";
    raw_constants_unchanged: "fn f(p: P) -> P { let compared = p <= 101; let raw = 1 <= 2; return p - 101; }";
}
#[test]
fn each_boundary_rejects_bounds_at_exact_literal_span() {
    for n in [-1, 101] {
        for body in [
            format!("fn f() -> P {{ return {n}; }}"),
            format!("fn f() -> P {{ return id({n}); }}"),
            format!("fn f() {{ take(0,{n},100); }}"),
            format!("fn f(p: P) {{ let mut x = p; x = {n}; }}"),
            format!("fn f(r: &mut P) {{ *r = {n}; }}"),
            format!("fn f() -> Triple {{ return Triple {{ c: 100,b: 80,a: {n} }}; }}"),
        ] {
            let (s, span) = bad(&body, "Type", "inclusive bounds 0..100");
            assert_eq!(&s[span.start..span.end], n.to_string());
        }
    }
    let (s, span) = bad(
        "fn f() -> P { return ((101)); }",
        "Type",
        "literal outside named range `P`",
    );
    assert_eq!(&s[span.start..span.end], "((101))");
}
#[test]
fn first_middle_final_calls_and_fields_preserve_written_order() {
    good("fn f() -> P { take(0,80,100); return sum(100,80,0); }");
    for position in 0..3 {
        let args = (0..3)
            .map(|i| if i == position { "101" } else { "80" })
            .collect::<Vec<_>>()
            .join(",");
        for body in [
            format!("fn f() {{ take({args}); }}"),
            format!("fn f() -> P {{ return sum({args}); }}"),
        ] {
            bad(&body, "Type", "literal outside named range");
        }
        let entries = ["c", "b", "a"]
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{n}: {}", if i == position { 101 } else { 80 }))
            .collect::<Vec<_>>()
            .join(",");
        let (s, span) = bad(
            &format!("fn f() -> Triple {{ return Triple {{ {entries} }}; }}"),
            "Type",
            "literal outside named range",
        );
        assert_eq!(&s[span.start..span.end], "101");
    }
    let p = good("fn f() -> Triple { return Triple { c: 80,b: 80,a: 80 }; }");
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let hir::ValueStatement::Return { value, .. } = &f.body[0] else {
        panic!()
    };
    let hir::ExprKind::RecordConstruct { fields, .. } = &value.kind else {
        panic!()
    };
    assert_eq!(
        fields
            .iter()
            .map(|e| p.field(e.field).name.as_str())
            .collect::<Vec<_>>(),
        ["c", "b", "a"]
    );
    for e in fields {
        assert!(matches!(
            e.value.kind,
            hir::ExprKind::FormedRangeLiteral(80)
        ));
        assert_eq!(
            e.value.ty,
            hir::ExprType::Range(p.field(e.field).range_type().unwrap())
        );
    }
    assert_ne!(fields[0].value.ty, fields[1].value.ty);
}
#[test]
fn numeric_extremes_signed_singleton_zero_and_earlier_phase_errors() {
    for (bounds, values) in [
        (
            "-9223372036854775808..9223372036854775807",
            vec!["-9223372036854775808", "9223372036854775807"],
        ),
        ("-10..-1", vec!["-10", "-1"]),
        ("0..0", vec!["0", "-0", "000"]),
    ] {
        for value in values {
            compile_source(&format!(
                "type Wide = range {bounds}; fn f() -> Wide {{ return {value}; }}"
            ))
            .unwrap();
        }
    }
    bad(
        "fn f() -> P { return 9223372036854775808; }",
        "Parse",
        "signed",
    );
    bad(
        "fn f() -> P { return 18446744073709551616; }",
        "Lex",
        "out of range",
    );
    for expr in ["-p", "-(80)", "--80"] {
        bad(
            &format!("fn f(p: P) -> P {{ return {expr}; }}"),
            "Parse",
            "integer",
        );
    }
}
#[test]
fn no_inference_or_operator_propagation() {
    for body in [
        "fn f() { let x = 80; }",
        "fn f() { let x = 80; take(x,x,x); }",
    ] {
        bad(body, "Type", "cannot materialize");
    }
    for branches in ["p } else { 80", "80 } else { p"] {
        bad(
            &format!(
                "fn f(flag: bool,p: P) -> P {{ let x = if flag {{ {branches} }}; return x; }}"
            ),
            "Type",
            "concrete owned type",
        );
    }
    for expr in ["81 - 1", "80 <= 81", "!80"] {
        bad(
            &format!("fn f() -> P {{ return {expr}; }}"),
            "Type",
            if expr.starts_with('!') {
                "Bool"
            } else if expr.contains("<=") {
                "return type mismatch"
            } else {
                "incompatible operand types"
            },
        );
    }
    bad("fn f() { let x: P = 80; }", "Parse", "expected");
    bad(
        "fn f(flag: bool) -> P { return if flag { 80 } else { 100 }; }",
        "Parse",
        "unsupported or malformed expression",
    );
}
#[test]
fn existing_nominal_values_are_never_retargeted() {
    for expr in ["q", "r.value", "*view", "other(80)"] {
        bad(&format!("record Other {{ value: Q }} fn f(q: Q,r: Other,view: &Q) -> P {{ return {expr}; }}"),"Type","distinct named ranges");
    }
}
#[test]
fn target_capability_call_kind_and_arity_precedence_remain() {
    for (body, message) in [
        ("fn f(p: P) { p = 101; }", "cannot assign parameter"),
        ("fn f(p: P) { let x = p; x = 101; }", "immutable"),
        ("fn f(r: &P) { *r = 101; }", "exclusive"),
        ("fn f() { id(101); }", "cannot be called as a statement"),
        ("fn f() -> P { return take(101,101,101); }", "no-value"),
        ("fn f() -> P { return id(101,101); }", "argument count"),
    ] {
        bad(body, "Type", message);
    }
}
#[test]
fn type_failure_precedes_ownership_and_resolve_precedes_type() {
    bad(
        "fn f(t: Ticket) -> Triple { return Triple { a: consume(t),b: 101,c: consume(t) }; }",
        "Type",
        "literal outside named range",
    );
    bad(
        "fn f(t: Ticket) -> Triple { return Triple { a: consume(t),b: 80,c: consume(t) }; }",
        "Ownership",
        "moved value",
    );
    bad(
        "fn f() -> Triple { return Triple { a: 101,b: missing,c: 80 }; }",
        "Resolve",
        "unknown",
    );
}
#[test]
fn loans_holds_recurrence_and_moved_handles_still_reject() {
    bad(
        "fn f(p: P) { let mut x = p; let r = &x; x = 80; let y = *r; }",
        "Ownership",
        "borrow",
    );
    bad(
        "fn held(r: &mut P,p: P) {} fn f(p: P) { let mut x = p; held(&mut x,id(x)); }",
        "Ownership",
        "exclusively borrowed",
    );
    bad(
        "fn f(p: P,flag: bool) { let mut x = p; let r = &x; while flag { x = 80; let y = *r; } }",
        "Ownership",
        "borrow",
    );
    bad(
        "fn f(p: P,flag: bool) { let mut x = p; let r = &mut x; if flag { sink(r); } *r = 80; }",
        "Ownership",
        "may have been moved",
    );
    bad(
        "fn f() { while predicate(Triple { a: 80,b: 80,c: 80 }) {} }",
        "Ownership",
        "construction is unsupported in recurring",
    );
}
#[test]
fn verified_harness_literals_remain_isolated() {
    let source = format!(
        "{}\nfn ordinary() -> Percent {{ return 80; }}",
        include_str!("../../examples/battery_ok.bow")
    );
    let p = compile_source(&source).unwrap();
    let report = verify::verify_report(&p);
    assert_eq!(report.functions_proven, 1);
    assert_eq!(report.calls_checked, 1);
    assert!(!p.bindings().is_empty());
    let fail = compile_source(include_str!("../../examples/battery_fail.bow")).unwrap();
    assert!(!verify::verify_report(&fail).diagnostics.is_empty());
}

#[test]
fn negative_singleton_bounds_reject_neighbors_with_type_phase_and_signed_span() {
    for (bounds, values) in [("-10..-1", vec![-11, 0]), ("-5..-5", vec![-6, -4])] {
        for value in values {
            let s =
                format!("type Negative = range {bounds}; fn f() -> Negative {{ return {value}; }}");
            let Err(FrontendError::Type(errors)) = compile_source(&s) else {
                panic!("expected Type")
            };
            assert_eq!(
                &s[errors[0].span.start..errors[0].span.end],
                value.to_string()
            );
            assert!(errors[0].required.contains(bounds));
        }
    }
}
#[test]
fn verified_call_literals_preserve_specialized_eligibility_bounds_and_proof_counts() {
    for amount in [0, 100, -1, 101] {
        let s = include_str!("../../examples/battery_ok.bow")
            .replace("charge: 80", "charge: 100")
            .replace(", 50)", &format!(", {amount})"));
        let p = compile_source(&s).unwrap();
        let report = verify::verify_report(&p);
        assert_eq!(report.functions_proven, 1);
        if (0..=100).contains(&amount) {
            assert!(report.diagnostics.is_empty());
            assert_eq!(report.calls_checked, 1);
        } else {
            assert!(!report.diagnostics.is_empty());
            assert_eq!(report.calls_checked, 0);
        }
        let hir::Statement::Call(call) = p.statements().last().unwrap() else {
            panic!()
        };
        assert_eq!(call.amount, amount); // Scalar harness path, never ordinary formed HIR.
    }
    let p = compile_source(include_str!("../../examples/battery_ok.bow")).unwrap();
    let hir::Function::Verified(v) = &p.functions()[0] else {
        panic!()
    };
    assert!(matches!(
        v.body[0].operand.kind,
        hir::ExprKind::Parameter(_)
    ));
}
#[test]
fn enclosing_call_holds_survive_formed_middle_argument() {
    bad(
        "fn held(r: &mut P,p: P,other: P) {} fn f(p: P) { let mut x=p; held(&mut x,80,x); }",
        "Ownership",
        "exclusively borrowed",
    );
    good("fn held(r: &mut P,p: P,other: P) {} fn f(p: P) { let mut x=p; held(&mut x,80,100); }");
}
