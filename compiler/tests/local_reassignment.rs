use bowfin_compiler::{
    compile_source,
    hir::{self, ExprKind, ValueStatement},
    mir::{self, Statement, Terminator},
    parse_source, FrontendError,
};
const DECL: &str =
    "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL} {body}")).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    p
}
fn bad(body: &str, phase: &str, wording: &str) {
    let e = compile_source(&format!("{DECL} {body}")).unwrap_err();
    let (actual, detail) = match &e {
        FrontendError::Lex(_) => ("Lex", e.to_string()),
        FrontendError::Parse(_) => ("Parse", e.to_string()),
        FrontendError::Resolve(ds) => (
            "Resolve",
            ds[0].render("case.bow", &format!("{DECL} {body}")),
        ),
        FrontendError::Type(ds) => ("Type", ds[0].render("case.bow", &format!("{DECL} {body}"))),
        FrontendError::Ownership(ds) => (
            "Ownership",
            ds[0].render("case.bow", &format!("{DECL} {body}")),
        ),
        FrontendError::Effects(ds) => (
            "Effects",
            ds[0].render("case.bow", &format!("{DECL} {body}")),
        ),
    };
    assert_eq!(actual, phase, "{body}: {detail}");
    assert!(detail.contains(wording), "{body}: {detail}");
    for private in ["LocalId", "ParameterId", "LoanId", "ConditionalMove"] {
        assert!(!detail.contains(private), "{detail}");
    }
}
macro_rules! accept {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            good($body);
        }
    };
}
macro_rules! reject {
    ($name:ident, $body:expr, $phase:expr, $wording:expr) => {
        #[test]
        fn $name() {
            bad($body, $phase, $wording);
        }
    };
}
accept!(
    bool_and_mutable_alias,
    "fn f() -> bool { let mutable enabled = false; enabled = true; return enabled; }"
);
accept!(range_repeated_self_assignment, "fn f(a: Percent, b: Percent) -> Percent { let mut current = a; current = b; current = current; current = a; return current; }");
accept!(rhs_call_reads_old_target, "fn next(a: Percent) -> Percent { return a; } fn f(a: Percent) -> Percent { let mut current = a; current = next(current); return current; }");
accept!(rhs_unary_and_binary, "fn f(a: Percent, b: Percent) -> bool { let mut current = a; current = current - b; let mut flag = false; flag = !flag || current == b; return flag; }");
accept!(conditional_initialization_then_assignment, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = if flag { a } else { b }; current = b; return current; }");
accept!(nested_statement_branch_assignment, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; if flag { current = b; if flag { current = a; } } else { current = current; } return current; }");
accept!(branch_local_targets_reuse_spelling, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { if flag { let mut current = a; current = b; } else { let mut current = b; current = a; } return a; }");
reject!(
    immutable_local,
    "fn f(a: Percent) -> Percent { let current = a; current = a; return current; }",
    "Type",
    "let mut / let mutable"
);
reject!(
    parameter_target,
    "fn f(a: Percent, b: Percent) -> Percent { a = b; return a; }",
    "Type",
    "cannot assign parameter `a`"
);
reject!(
    unknown_target,
    "fn f(a: Percent) -> Percent { missing = a; return a; }",
    "Resolve",
    "unknown assignment target `missing`"
);
reject!(
    target_before_declaration,
    "fn f(a: Percent) -> Percent { current = a; let mut current = a; return current; }",
    "Resolve",
    "unknown assignment target"
);
reject!(branch_target_after_join, "fn f(flag: bool, a: Percent) -> Percent { if flag { let mut current = a; } current = a; return a; }", "Resolve", "unknown assignment target");
reject!(sibling_target_invisible, "fn f(flag: bool, a: Percent) -> Percent { if flag { let mut current = a; } else { current = a; } return a; }", "Resolve", "unknown assignment target");
reject!(
    shared_reference_local,
    "fn f(a: Percent, b: Percent) -> bool { let view = &a; view = &b; return true; }",
    "Type",
    "cannot reassign reference local `view`"
);
reject!(mutable_reference_local, "fn f(a: Percent, b: Percent) -> bool { let mut first = a; let mut second = b; let access = &mut first; access = &mut second; return true; }", "Type", "cannot reassign reference local `access`");
accept!(
    owned_record_replacement,
    "fn f(a: Ticket, b: Ticket) -> Ticket { let mut current = a; current = b; return current; }"
);
reject!(
    nominal_range_mismatch,
    "fn f(a: Percent, b: Other) -> Percent { let mut current = a; current = b; return current; }",
    "Type",
    "expected range `Percent`, found range `Other`"
);
reject!(
    literal_outside_established_range,
    "fn f(a: Percent) -> Percent { let mut current = a; current = 101; return current; }",
    "Type",
    "literal outside named range"
);
reject!(
    bool_range_mismatch,
    "fn f(a: Percent) -> Percent { let mut current = a; current = true; return current; }",
    "Type",
    "assignment RHS type mismatch"
);
reject!(
    type_precedes_record_ownership,
    "fn f(a: Ticket) -> Ticket { let mut current = a; current = true; return current; }",
    "Type",
    "assignment RHS type mismatch"
);
accept!(dead_shared_loan, "fn f(a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; let observed = *view; current = b; return current; }");
accept!(dead_exclusive_loan, "fn f(a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; *access = b; current = a; return current; }");
accept!(rhs_final_shared_deref, "fn f(a: Percent) -> Percent { let mut current = a; let view = &current; current = *view; return current; }");
accept!(rhs_final_mutable_deref, "fn f(a: Percent) -> Percent { let mut current = a; let access = &mut current; current = *access; return current; }");
accept!(rhs_last_shared_call_hold_then_write, "fn read(a: &Percent) -> Percent { return *a; } fn f(a: Percent) -> Percent { let mut current = a; let view = &current; current = read(view); return current; }");
accept!(rhs_last_mutable_call_hold_then_write, "fn read(a: &mut Percent) -> Percent { return *a; } fn f(a: Percent) -> Percent { let mut current = a; let access = &mut current; current = read(access); return current; }");
accept!(rhs_direct_borrow_call_then_write, "fn read(a: &mut Percent) -> Percent { return *a; } fn f(a: Percent) -> Percent { let mut current = a; current = read(&mut current); return current; }");
accept!(rhs_direct_shared_call_then_write, "fn read(a: &Percent) -> Percent { return *a; } fn f(a: Percent) -> Percent { let mut current = a; current = read(&current); return current; }");
reject!(post_assignment_shared_handle, "fn f(a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; current = b; return *view; }", "Ownership", "cannot assign `current` while it is borrowed");
reject!(post_assignment_mutable_handle, "fn f(a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; current = b; return *access; }", "Ownership", "conflicting borrow began");
reject!(rhs_shared_use_with_future_continuation, "fn f(a: Percent) -> Percent { let mut current = a; let view = &current; current = *view; return *view; }", "Ownership", "cannot assign");
accept!(sibling_only_reference_use, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; if flag { let observed = *view; } else { current = b; } return current; }");
accept!(nested_sibling_only_reference_use, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; if flag { if flag { let observed = *access; } else { current = b; } } else { current = a; } return current; }");
reject!(post_join_shared_use_blocks_branch_write, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; if flag { let observed = *view; } else { current = b; } return *view; }", "Ownership", "cannot assign");
reject!(post_join_exclusive_use_blocks_branch_write, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; if flag { let observed = *access; } else { current = b; } return *access; }", "Ownership", "cannot assign");
reject!(same_branch_no_resurrection, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; if flag { current = b; let observed = *view; } return current; }", "Ownership", "cannot assign");
reject!(rhs_nested_call_holds, "fn pair(a: &mut Percent, b: Percent) -> Percent { return b; } fn id(a: Percent) -> Percent { return a; } fn f(a: Percent) -> Percent { let mut current = a; current = pair(&mut current, id(current)); return current; }", "Ownership", "exclusively borrowed");
accept!(rhs_unrelated_record_move, "fn consume(a: Ticket, value: Percent) -> Percent { return value; } fn f(a: Percent, ticket: Ticket) -> Percent { let mut current = a; current = consume(ticket, current); return current; }");
reject!(rhs_move_remains_consumed, "fn consume(a: Ticket) -> bool { return true; } fn f(ticket: Ticket) -> Ticket { let mut flag = false; flag = consume(ticket); return ticket; }", "Ownership", "moved value");
reject!(rhs_conditional_move_remains_unavailable, "fn consume(a: Ticket) -> bool { return true; } fn f(flag: bool, ticket: Ticket) -> Ticket { let mut current = false; if flag { current = consume(ticket); } return ticket; }", "Ownership", "may have been moved");
reject!(potential_joined_loan_blocks_assignment, "fn sink(a: &mut Percent) -> bool { return true; } fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; if flag { let done = sink(access); } current = b; return *access; }", "Ownership", "cannot assign");
#[test]
fn grammar_remains_statement_only_and_direct_local() {
    bad(
        "fn f() -> bool { let mut x = true; x.field = false; return x; }",
        "Type",
        "owned record local",
    );
    bad(
        "fn id(a: bool) -> bool { return a; } fn f() -> bool { id(true); return true; }",
        "Type",
        "cannot be called as a statement",
    );
    for body in [
        "let mut x = true; x = false return x;",
        "let mut x = true; x = x = false; return x;",
        "let mut x = true; x -= false; return x;",
        "let mut x = true; x *= false; return x;",
        "let mut x = true; let predecessor_result = (x = false); return x;",
        "let mut x = true; let predecessor_result = id(x = false); return x;",
        "let mut x = true; return x = false;",
        "let mut x = true; let predecessor_result = x == (x = false); return x;",
        "id(true) = false; return true;",
        "let mut x = true; *(&mut x) = false; return x;",
        "let mut x = true; x = if true { true } else { false }; return x;",
        "let mut x = true; x; return x;",
    ] {
        bad(
            &format!("fn id(a: bool) -> bool {{ return a; }} fn f() -> bool {{ {body} }}"),
            "Parse",
            "",
        );
    }
    bad(
        "fn f() -> bool { let mut x = true; x += false; return x; }",
        "Lex",
        "unexpected character",
    );
}
#[test]
fn ast_and_hir_preserve_target_identity_and_complete_span() {
    let text = format!("{DECL} fn f(a: Percent) -> Percent {{ let mut current = a; current = current; return current; }}");
    let ast = parse_source(&text).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::Assign { target, span, .. } = &f.body[1] else {
        panic!()
    };
    assert_eq!(target, "current");
    assert_eq!(&text[span.start..span.end], "current = current;");
    let p = compile_source(&text).unwrap();
    let f = p.functions()[0].as_ordinary().unwrap();
    let ValueStatement::Let { local, .. } = &f.body[0] else {
        panic!()
    };
    let ValueStatement::Assign {
        local: target,
        value,
        span: typed_span,
    } = &f.body[1]
    else {
        panic!()
    };
    assert_eq!(local, target);
    assert_eq!(*span, *typed_span);
    assert_eq!(value.kind, ExprKind::Local(*local));
    assert_eq!(p.locals().len(), 1);
}
#[test]
fn mir_straight_line_assignment_is_distinct_and_adds_no_blocks() {
    let p = good("fn f(a: Percent) -> Percent { let mut current = a; current = current; current = current; return current; }");
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    assert_eq!(f.blocks().len(), 1);
    let b = f.block(f.entry());
    let [Statement::Let { local, .. }, Statement::Assign {
        local: first,
        value,
        ..
    }, Statement::Assign { local: second, .. }] = b.statements()
    else {
        panic!()
    };
    assert_eq!(local, first);
    assert_eq!(local, second);
    assert_eq!(value.kind, ExprKind::Local(*local));
    assert!(matches!(b.terminator(), Terminator::Return { .. }));
}
#[test]
fn mir_nested_assignments_preserve_branch_paths_and_initialization() {
    let p = good("fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = if flag { a } else { b }; if flag { if flag { current = b; } } else { current = a; } return current; }");
    let m = mir::lower(&p);
    assert_eq!(m, mir::lower(&p));
    let f = &m.functions()[0];
    f.validate().unwrap();
    let mut lets = 0;
    let mut assignments = 0;
    for (_, block) in f.blocks() {
        for statement in block.statements() {
            match statement {
                Statement::Let {
                    local, initializer, ..
                } => {
                    assert_eq!(*local, p.locals()[0].id);
                    assert!(!matches!(initializer.kind, ExprKind::IfValue { .. }));
                    lets += 1;
                }
                Statement::Assign { local, value, .. } => {
                    assert_eq!(*local, p.locals()[0].id);
                    assert!(!matches!(value.kind, ExprKind::IfValue { .. }));
                    assignments += 1;
                }
                _ => panic!(),
            }
        }
    }
    assert_eq!(lets, 2);
    assert_eq!(assignments, 2);
    let simple = good("fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; if flag { current = b; } return current; }");
    let m = mir::lower(&simple);
    let f = &m.functions()[0];
    let Terminator::Branch {
        then_target,
        else_target,
        ..
    } = f.block(f.entry()).terminator()
    else {
        panic!()
    };
    assert!(matches!(
        f.block(*then_target).statements(),
        [Statement::Assign { .. }]
    ));
    assert!(f.block(*else_target).statements().is_empty());
    assert!(matches!(
        f.block(*else_target).terminator(),
        Terminator::Return { .. }
    ));
}
