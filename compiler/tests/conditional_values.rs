use bowfin_compiler::{
    compile_source,
    hir::{self, ExprKind, ExprType},
    mir::{self, Statement, Terminator},
    parse_source, FrontendError,
};
const DECL: &str = "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent } record OtherTicket { value: Percent }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL} {body}")).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    p
}
fn bad(body: &str, phase: &str, wording: &str) {
    let e = compile_source(&format!("{DECL} {body}")).unwrap_err();
    let actual = match &e {
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        _ => "Lex",
    };
    assert_eq!(actual, phase, "{body}: {e}");
    assert!(format!("{e:?}").contains(wording), "{body}: {e}");
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
    bool_values,
    "fn f(flag: bool) -> bool { let chosen = if flag { true } else { false }; return chosen; }"
);
accept!(range_copy_reuse, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let chosen = if flag { a } else { b }; let again = a - b; return chosen; }");
accept!(mutable_destination, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut chosen = if flag { a } else { b }; let access = &mut chosen; *access = a; return chosen; }");
accept!(range_calls, "fn id(a: Percent) -> Percent { return a; } fn f(flag: bool, a: Percent) -> Percent { let chosen = if flag { id(a) } else { id(a) }; return chosen; }");
accept!(nested_values, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let chosen = if flag { if flag { a } else { b } } else { if flag { b } else { a } }; return chosen; }");
accept!(record_destination, "fn f(flag: bool, a: Ticket, b: Ticket) -> Ticket { let chosen = if flag { a } else { b }; return chosen; }");
accept!(same_record_both_paths, "fn f(flag: bool, a: Ticket) -> Ticket { let chosen = if flag { a } else { a }; return chosen; }");
accept!(record_call_results, "fn id(a: Ticket) -> Ticket { return a; } fn f(flag: bool, a: Ticket, b: Ticket) -> Ticket { let chosen = if flag { id(a) } else { id(b) }; return chosen; }");
accept!(statement_and_value_nesting, "fn f(flag: bool, a: Percent) -> Percent { if flag { let chosen = if flag { a } else { a }; let copy = chosen; } let chosen = if flag { a } else { a }; if chosen == a {} return chosen; }");
accept!(copy_deref, "fn f(flag: bool, a: &Percent, b: &Percent) -> Percent { let chosen = if flag { *a } else { *b }; return chosen; }");
reject!(range_nominal_identity, "fn f(flag: bool, a: Percent, b: Other) -> Percent { let chosen = if flag { a } else { b }; return chosen; }", "Type", "range `Percent` and range `Other`");
reject!(record_nominal_identity, "fn f(flag: bool, a: Ticket, b: OtherTicket) -> Ticket { let chosen = if flag { a } else { b }; return chosen; }", "Type", "record `Ticket` and record `OtherTicket`");
reject!(mixed_bool_range, "fn f(flag: bool, a: Percent) -> Percent { let chosen = if flag { a } else { true }; return chosen; }", "Type", "same type");
reject!(literal_range, "fn f(flag: bool, a: Percent) -> Percent { let chosen = if flag { a } else { 1 }; return chosen; }", "Type", "concrete owned type");
reject!(
    two_literals,
    "fn f(flag: bool) -> bool { let chosen = if flag { 1 } else { 2 }; return true; }",
    "Type",
    "concrete owned type"
);
reject!(
    shared_result,
    "fn f(flag: bool, a: &Percent) -> bool { let chosen = if flag { a } else { a }; return true; }",
    "Type",
    "cannot produce a reference"
);
reject!(mutable_result, "fn f(flag: bool, a: &mut Percent) -> bool { let chosen = if flag { a } else { a }; return true; }", "Type", "cannot produce a reference");
reject!(direct_borrow_result, "fn f(flag: bool, a: Percent) -> bool { let chosen = if flag { &a } else { &a }; return true; }", "Type", "cannot produce a reference");
reject!(nested_reference_result, "fn f(flag: bool, a: &Percent) -> bool { let chosen = if flag { if flag { a } else { a } } else { a }; return true; }", "Type", "cannot produce a reference");
reject!(
    non_bool_condition,
    "fn f(a: Percent) -> Percent { let chosen = if a { a } else { a }; return chosen; }",
    "Type",
    "type Bool"
);
reject!(
    reference_condition,
    "fn f(a: &bool) -> bool { let chosen = if a { true } else { false }; return chosen; }",
    "Type",
    "type Bool"
);
reject!(
    literal_condition,
    "fn f() -> bool { let chosen = if 1 { true } else { false }; return chosen; }",
    "Type",
    "type Bool"
);
reject!(type_before_ownership, "fn f(flag: bool, a: Ticket) -> bool { let moved = a; let chosen = if flag { a } else { true }; return true; }", "Type", "same type");
reject!(non_copy_deref, "fn f(flag: bool, a: &Ticket, b: Ticket) -> Ticket { let chosen = if flag { *a } else { b }; return chosen; }", "Ownership", "non-Copy");
reject!(first_source_after_join, "fn f(flag: bool, a: Ticket, b: Ticket) -> Ticket { let chosen = if flag { a } else { b }; return a; }", "Ownership", "may have been moved");
reject!(second_source_after_join, "fn f(flag: bool, a: Ticket, b: Ticket) -> Ticket { let chosen = if flag { a } else { b }; return b; }", "Ownership", "may have been moved");
reject!(
    same_source_after_join,
    "fn f(flag: bool, a: Ticket) -> Ticket { let chosen = if flag { a } else { a }; return a; }",
    "Ownership",
    "may have been moved"
);
reject!(nested_source_after_join, "fn f(flag: bool, a: Ticket, b: Ticket) -> Ticket { let chosen = if flag { if flag { a } else { b } } else { b }; return a; }", "Ownership", "may have been moved");
reject!(condition_moves_before_split, "fn pred(a: Ticket) -> bool { return true; } fn f(a: Ticket) -> Ticket { let chosen = if pred(a) { a } else { a }; return chosen; }", "Ownership", "moved value");
reject!(
    constant_edges_not_pruned,
    "fn f(a: Ticket, b: Ticket) -> Ticket { let chosen = if false { a } else { b }; return a; }",
    "Ownership",
    "may have been moved"
);
reject!(
    unknown_branch,
    "fn f(flag: bool) -> bool { let chosen = if flag { missing } else { true }; return chosen; }",
    "Resolve",
    "unknown"
);
reject!(
    destination_self_reference,
    "fn f(flag: bool) -> bool { let chosen = if flag { chosen } else { true }; return chosen; }",
    "Resolve",
    "unknown"
);
reject!(
    destination_shadowing,
    "fn f(flag: bool) -> bool { let flag = if flag { true } else { false }; return flag; }",
    "Resolve",
    "duplicate"
);
accept!(branch_specific_exclusive_expiry, "fn f(flag: bool, a: Percent) -> Percent { let mut owned = a; let access = &mut owned; let chosen = if flag { *access } else { owned }; return chosen; }");
accept!(nested_branch_specific_expiry, "fn f(flag: bool, a: Percent) -> Percent { let mut owned = a; let access = &mut owned; let chosen = if flag { if flag { *access } else { owned } } else { owned }; return chosen; }");
reject!(post_join_reference_keeps_loan, "fn f(flag: bool, a: Percent) -> Percent { let mut owned = a; let access = &mut owned; let chosen = if flag { *access } else { owned }; return *access; }", "Ownership", "exclusively borrowed");
reject!(condition_keeps_successor_loan, "fn pred(a: Percent) -> bool { return true; } fn f(a: Percent) -> Percent { let mut owned = a; let access = &mut owned; let chosen = if pred(owned) { *access } else { owned }; return chosen; }", "Ownership", "exclusively borrowed");
accept!(shared_copy_provenance, "fn inspect(a: &Ticket) -> bool { return true; } fn consume(a: Ticket) -> bool { return true; } fn f(flag: bool, a: Ticket) -> bool { let view = &a; let alias = view; let chosen = if flag { inspect(alias) } else { consume(a) }; return chosen; }");
accept!(mutable_call_transfer_and_recovery, "fn sink(a: &mut Percent) -> bool { return true; } fn f(flag: bool, a: Percent) -> Percent { let mut owned = a; let access = &mut owned; let chosen = if flag { sink(access) } else { true }; return owned; }");
reject!(mutable_transfer_later_use, "fn sink(a: &mut Percent) -> bool { return true; } fn f(flag: bool, a: &mut Percent) -> Percent { let chosen = if flag { sink(a) } else { true }; return *a; }", "Ownership", "may have been moved");
reject!(potential_joined_loan, "fn sink(a: &mut Percent) -> bool { return true; } fn f(flag: bool, a: Percent) -> Percent { let mut owned = a; let access = &mut owned; let chosen = if flag { sink(access) } else { true }; let observed = owned; return *access; }", "Ownership", "exclusively borrowed");
reject!(direct_shared_call_hold, "fn consume(a: Ticket) -> bool { return true; } fn pair(a: &Ticket, b: bool) -> bool { return b; } fn f(flag: bool, a: Ticket) -> bool { let chosen = if flag { pair(&a, consume(a)) } else { true }; return chosen; }", "Ownership", "shared");
reject!(direct_mutable_call_hold, "fn pred(a: Percent) -> bool { return true; } fn pair(a: &mut Percent, b: bool) -> bool { return b; } fn f(flag: bool, a: Percent) -> bool { let mut owned = a; let chosen = if flag { pair(&mut owned, pred(owned)) } else { true }; return chosen; }", "Ownership", "exclusively borrowed");
reject!(final_shared_handle_call_hold, "fn consume(a: Ticket) -> bool { return true; } fn pair(a: &Ticket, b: bool) -> bool { return b; } fn f(flag: bool, a: Ticket) -> bool { let view = &a; let chosen = if flag { pair(view, consume(a)) } else { true }; return chosen; }", "Ownership", "shared");
reject!(nested_call_hold, "fn pred(a: Percent) -> bool { return true; } fn wrap(a: bool) -> bool { return a; } fn pair(a: &mut Percent, b: bool) -> bool { return b; } fn f(flag: bool, a: Percent) -> bool { let mut owned = a; let access = &mut owned; let chosen = if flag { pair(access, wrap(pred(owned))) } else { true }; return chosen; }", "Ownership", "exclusively borrowed");
#[test]
fn initializer_only_grammar_and_single_expression_branches() {
    for init in [
        "if flag { true }",
        "if flag {} else { true }",
        "if flag { true; } else { false }",
        "if flag { true } else { false; }",
        "if flag { let x = true; x } else { false }",
        "if flag { return true; } else { false }",
        "if flag { *access = true; true } else { false }",
        "if flag { true } else if flag { true } else { false }",
        "(if flag { true } else { false })",
        "!if flag { true } else { false }",
        "true && if flag { true } else { false }",
        "if flag { true } else { false } && true",
    ] {
        let body = format!("fn f(flag: bool) -> bool {{ let chosen = {init}; return chosen; }}");
        assert!(
            matches!(
                compile_source(&format!("{DECL} {body}")),
                Err(FrontendError::Parse(_))
            ),
            "{init}"
        );
    }
    for body in [
        "fn f(flag: bool) -> bool { return if flag { true } else { false }; }",
        "fn id(a: bool) -> bool { return a; } fn f(flag: bool) -> bool { let chosen = id(if flag { true } else { false }); return chosen; }",
        "fn f(flag: bool, access: &mut bool) -> bool { *access = if flag { true } else { false }; return true; }",
        "fn f(flag: bool) -> bool { let chosen = &if flag { true } else { false }; return true; }",
        "fn f(flag: bool) -> bool { let chosen = *if flag { true } else { false }; return true; }",
    ] { assert!(matches!(compile_source(&format!("{DECL} {body}")), Err(FrontendError::Parse(_))), "{body}"); }
}
#[test]
fn ast_hir_spans_and_canonical_references() {
    let text = format!("{DECL} fn f(flag: bool, a: Percent, b: Percent) -> Percent {{ let chosen = if flag {{ a }} else {{ b }}; return chosen; }}");
    let ast = parse_source(&text).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::Let { initializer, .. } = &f.body[0] else {
        panic!()
    };
    assert_eq!(
        &text[initializer.span.start..initializer.span.end],
        "if flag { a } else { b }"
    );
    assert!(matches!(
        initializer.kind,
        bowfin_compiler::ast::ExprKind::IfValue { .. }
    ));
    let p = compile_source(&text).unwrap();
    let f = p.functions()[0].as_ordinary().unwrap();
    let hir::ValueStatement::Let {
        local, initializer, ..
    } = &f.body[0]
    else {
        panic!()
    };
    assert_eq!(*local, p.locals()[0].id);
    assert_eq!(initializer.ty, ExprType::Range(p.ranges()[0].id));
    let ExprKind::IfValue {
        condition,
        then_value,
        else_value,
    } = &initializer.kind
    else {
        panic!()
    };
    assert_eq!(condition.kind, ExprKind::Parameter(f.parameters[0]));
    assert_eq!(then_value.kind, ExprKind::Parameter(f.parameters[1]));
    assert_eq!(else_value.kind, ExprKind::Parameter(f.parameters[2]));
}
fn contains_if(e: &hir::TypedExpr) -> bool {
    match &e.kind {
        ExprKind::IfValue { .. } => true,
        ExprKind::Call { arguments, .. } => arguments.iter().any(contains_if),
        ExprKind::Unary { operand, .. } => contains_if(operand),
        ExprKind::Binary { left, right, .. } => contains_if(left) || contains_if(right),
        _ => false,
    }
}
#[test]
fn mir_all_paths_initialize_one_canonical_destination_once() {
    for nested in [false, true] {
        let init = if nested {
            "if flag { if flag { a } else { b } } else { if flag { b } else { a } }"
        } else {
            "if flag { a } else { b }"
        };
        let p = good(&format!("fn f(flag: bool, a: Ticket, b: Ticket) -> Ticket {{ let chosen = {init}; return chosen; }}"));
        let destination = p.locals()[0].id;
        let m = mir::lower(&p);
        assert_eq!(m, mir::lower(&p));
        let f = &m.functions()[0];
        f.validate().unwrap();
        let mut leaves = 0;
        for (_, block) in f.blocks() {
            for s in block.statements() {
                let Statement::Let {
                    local, initializer, ..
                } = s
                else {
                    panic!()
                };
                assert_eq!(*local, destination);
                assert!(!contains_if(initializer));
                assert!(matches!(initializer.kind, ExprKind::Parameter(_)));
                leaves += 1;
            }
            if let Terminator::Branch {
                condition,
                then_target,
                else_target,
            } = block.terminator()
            {
                assert_eq!(condition.ty, ExprType::Bool);
                assert_ne!(then_target, else_target);
            }
        }
        assert_eq!(leaves, if nested { 4 } else { 2 });
        fn paths(
            f: &mir::MirFunction,
            id: mir::BasicBlockId,
            destination: hir::LocalId,
            mut initialized: usize,
        ) -> usize {
            let b = f.block(id);
            initialized += b
                .statements()
                .iter()
                .filter(|s| matches!(s, Statement::Let { local, .. } if *local == destination))
                .count();
            match b.terminator() {
                Terminator::Propagate { .. } | Terminator::ForwardError { .. } => {
                    panic!("conditional-value predecessor contains no propagation")
                }
                mir::Terminator::Match { .. } => {
                    panic!("unexpected Result match in predecessor conditional-value fixture")
                }
                Terminator::Goto { target } => paths(f, *target, destination, initialized),
                Terminator::Branch {
                    then_target,
                    else_target,
                    ..
                } => {
                    paths(f, *then_target, destination, initialized)
                        + paths(f, *else_target, destination, initialized)
                }
                Terminator::ReturnNoValue => panic!("value-function paths return values"),
                Terminator::Return { value } => {
                    assert_eq!(initialized, 1);
                    assert_eq!(value.kind, ExprKind::Local(destination));
                    1
                }
            }
        }
        assert_eq!(paths(f, f.entry(), destination, 0), leaves);
    }
}
accept!(bool_operator_branches_and_call_condition, "fn pred(a: Percent) -> bool { return a == a; } fn f(a: Percent) -> bool { let chosen = if pred(a) { a <= a } else { true && false }; return chosen; }");
accept!(local_branch_values_and_mutable_alias_keyword, "fn f(flag: bool, a: Percent) -> Percent { let first = a; let mutable chosen = if flag { first } else { a }; let access = &mut chosen; *access = a; return chosen; }");
reject!(reference_local_result, "fn f(flag: bool, a: Percent) -> bool { let view = &a; let chosen = if flag { view } else { view }; return true; }", "Type", "cannot produce a reference");
reject!(mutable_borrow_result, "fn f(flag: bool, a: Percent) -> bool { let mut owned = a; let chosen = if flag { &mut owned } else { &mut owned }; return true; }", "Type", "cannot produce a reference");
reject!(
    record_condition,
    "fn f(a: Ticket) -> bool { let chosen = if a { true } else { false }; return chosen; }",
    "Type",
    "type Bool"
);
reject!(
    unknown_condition,
    "fn f() -> bool { let chosen = if missing { true } else { false }; return chosen; }",
    "Resolve",
    "unknown"
);
reject!(reference_error_before_moved_handle, "fn f(flag: bool, a: &mut Percent) -> bool { let previous = a; let chosen = if flag { a } else { a }; return true; }", "Type", "cannot produce a reference");
