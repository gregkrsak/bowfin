use bowfin_compiler::{
    compile_source,
    hir::{self, ExprKind, ExprType},
    mir::{self, Terminator},
    parse_source, FrontendError,
};
const DECL: &str = "type Percent = range 0..100; record Ticket { value: Percent }";
fn source(body: &str) -> String {
    format!("{DECL} {body}")
}
fn good(body: &str) -> hir::Program {
    compile_source(&source(body)).unwrap()
}
fn error(body: &str, phase: &str, wording: &str) {
    let error = compile_source(&source(body)).unwrap_err();
    let actual = match &error {
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        _ => "Lex",
    };
    assert_eq!(actual, phase, "{body}: {error}");
    assert!(error.to_string().contains(wording), "{body}: {error}");
}
#[test]
fn optional_else_nested_and_empty_branches() {
    for statements in [
        "if flag { let observed = value; }",
        "if (flag) { let first = value; } else { let second = value; }",
        "if flag { if flag { if flag { let third = value; } } else {} }",
        "if flag {}",
        "if flag {} else {}",
        "if value == value { let observed = value; }",
    ] {
        let p = good(&format!(
            "fn example(flag: bool, value: Percent) -> Percent {{ {statements} return value; }}"
        ));
        for f in mir::lower(&p).functions() {
            f.validate().unwrap();
        }
    }
}
#[test]
fn source_conditional_preserves_optional_else_and_spans() {
    let text =
        source("fn example(flag: bool) -> bool { if flag {} else {} if flag {} return flag; }");
    let ast = parse_source(&text).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    for (index, present) in [(0, true), (1, false)] {
        let bowfin_compiler::ast::ValueStatement::If {
            else_body, span, ..
        } = &f.body[index]
        else {
            panic!()
        };
        assert_eq!(else_body.is_some(), present);
        assert!(text[span.start..span.end].starts_with("if flag {"));
        assert!(text[span.start..span.end].ends_with('}'));
    }
}
#[test]
fn sibling_locals_have_distinct_canonical_ids_and_nested_visibility() {
    let p = good("fn example(flag: bool, value: Percent) -> Percent { if flag { let observed = value; if flag { let copied = observed; } } else { let observed = value; } return value; }");
    assert_eq!(p.locals()[0].name, p.locals()[2].name);
    assert_ne!(p.locals()[0].id, p.locals()[2].id);
    let f = p.functions()[0].as_ordinary().unwrap();
    let hir::ValueStatement::If {
        condition,
        then_body,
        else_body,
        ..
    } = &f.body[0]
    else {
        panic!()
    };
    assert_eq!(condition.kind, ExprKind::Parameter(f.parameters[0]));
    assert_eq!(condition.ty, ExprType::Bool);
    let hir::ValueStatement::If {
        then_body: nested, ..
    } = &then_body[1]
    else {
        panic!()
    };
    let hir::ValueStatement::Let { initializer, .. } = &nested[0] else {
        panic!()
    };
    assert_eq!(initializer.kind, ExprKind::Local(p.locals()[0].id));
    let hir::ValueStatement::Let { local, .. } = &else_body[0] else {
        panic!()
    };
    assert_eq!(*local, p.locals()[2].id);
}
#[test]
fn branch_scope_leaks_and_source_order_are_resolution_errors() {
    for statements in [
        "if flag { let inside = value; } let escaped = inside;",
        "if flag { let inside = value; } else { let escaped = inside; }",
        "if flag { let escaped = inside; } else { let inside = value; }",
        "if flag { let first = later; let later = value; }",
        "if flag { let self_ref = self_ref; }",
    ] {
        error(
            &format!(
                "fn bad(flag: bool, value: Percent) -> Percent {{ {statements} return value; }}"
            ),
            "Resolve",
            "unknown",
        );
    }
}
#[test]
fn visible_names_cannot_be_shadowed() {
    for statements in [
        "if flag { let value = value; }",
        "let outer = value; if flag { let outer = value; }",
        "if flag { let outer = value; if flag { let outer = value; } }",
    ] {
        error(
            &format!(
                "fn bad(flag: bool, value: Percent) -> Percent {{ {statements} return value; }}"
            ),
            "Resolve",
            "duplicate declaration",
        );
    }
}
#[test]
fn condition_requires_exact_bool_before_ownership() {
    for (parameters, condition) in [
        ("value: Percent", "value"),
        ("value: Ticket", "value"),
        ("value: &bool", "value"),
        ("value: &mut Percent", "value"),
        ("value: bool", "1"),
    ] {
        error(
            &format!("fn bad({parameters}) -> bool {{ if {condition} {{}} return true; }}"),
            "Type",
            "if condition must have type Bool",
        );
    }
}
#[test]
fn conditional_value_syntax_is_rejected() {
    for body in [
        "let predecessor_result = if flag {} else {}; return true;",
        "return if flag {} else {};",
        "let predecessor_result = consume(if flag {} else {}); return true;",
    ] {
        error(&format!("fn consume(value: bool) -> bool {{ return value; }} fn bad(flag: bool) -> bool {{ {body} }}"), "Parse", "expression");
    }
}
#[test]
fn unsupported_control_flow_and_terminal_suffixes_are_rejected() {
    for statements in [
        "for flag {}",
        "match flag {}",
        "if flag { return true; } else { return false; }",
        "if flag {} else if flag {}",
        "if flag { break; }",
        "if flag { continue; }",
    ] {
        error(
            &format!("fn bad(flag: bool) -> bool {{ {statements} return true; }}"),
            "Parse",
            "",
        );
    }
}
#[test]
fn branch_copy_mutation_and_local_loan_expiry() {
    good("fn update(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut owned = value; if flag { let access = &mut owned; *access = replacement; } else { let view = &owned; let observed = *view; } let access = &mut owned; *access = *access; return owned; }");
    good("fn update(flag: bool, value: Percent) -> Percent { let mut owned = value; if flag { let unused = &mut owned; } let observed = owned; return owned; }");
}
#[test]
fn conditional_moves_then_else_both_and_nested_reject_joined_use() {
    for statements in [
        "if flag { let moved = ticket; }",
        "if flag {} else { let moved = ticket; }",
        "if flag { let moved = ticket; } else { let moved = ticket; }",
        "if flag { if flag { let moved = ticket; } }",
    ] {
        error(
            &format!(
                "fn bad(flag: bool, ticket: Ticket) -> Ticket {{ {statements} return ticket; }}"
            ),
            "Ownership",
            "may have been moved on a previous control-flow path",
        );
    }
}
#[test]
fn borrowing_conditionally_moved_owner_is_rejected() {
    error("fn bad(flag: bool, ticket: Ticket) -> bool { if flag { let moved = ticket; } let view = &ticket; return true; }", "Ownership", "may have been moved");
}
#[test]
fn branches_are_independent_and_neither_move_preserves_owner() {
    good("fn sink(value: Ticket) -> bool { return true; } fn example(flag: bool, ticket: Ticket) -> bool { if flag { let done = sink(ticket); } else { let done = sink(ticket); } return true; }");
    good("fn example(flag: bool, ticket: Ticket) -> Ticket { if flag { let observed = flag; } else { let observed = flag; } return ticket; }");
}
#[test]
fn condition_call_moves_before_split() {
    for statements in [
        "if predicate(ticket) {}",
        "if predicate(ticket) { let again = ticket; }",
        "if predicate(ticket) {} else { let again = ticket; }",
    ] {
        error(&format!("fn predicate(value: Ticket) -> bool {{ return true; }} fn bad(ticket: Ticket) -> Ticket {{ {statements} return ticket; }}"), "Ownership", "use of moved value `ticket`");
    }
}
#[test]
fn mutable_reference_conditional_transfer_rejects_read_write_and_call() {
    for later in [
        "let observed = *access;",
        "*access = value;",
        "let done = sink(access);",
    ] {
        error(&format!("fn sink(access: &mut Percent) -> bool {{ return true; }} fn bad(flag: bool, access: &mut Percent, value: Percent) -> bool {{ if flag {{ let observed = *access; let done = sink(access); }} {later} return true; }}"), "Ownership", "may have been moved");
    }
}
#[test]
fn shared_reference_handles_remain_copy_across_branches() {
    good("fn sink(view: &Percent) -> bool { return true; } fn example(flag: bool, value: Percent) -> Percent { let view = &value; if flag { let alias = view; let done = sink(alias); } else { let done = sink(view); } let observed = *view; return value; }");
}
#[test]
fn incoming_exclusive_loan_protects_branch_owner_access() {
    error("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let observed = owned; } return *access; }", "Ownership", "exclusively borrowed");
}
#[test]
fn incoming_shared_loan_protects_against_branch_mutable_borrow() {
    error("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let access = &mut owned; } return *view; }", "Ownership", "shared borrow is active");
}
#[test]
fn incoming_loan_protects_owner_move_before_post_join_use() {
    error("fn sink(view: &Ticket) -> bool { return true; } fn bad(flag: bool, ticket: Ticket) -> bool { let view = &ticket; if flag { let moved = ticket; } return sink(view); }", "Ownership", "shared-borrowed");
}
#[test]
fn branch_only_reference_uses_no_longer_hold_sibling_or_later_operations() {
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let observed = *access; } else { let observed = owned; } return owned; }");
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let observed = *view; let access = &mut owned; } return owned; }");
}
#[test]
fn last_use_resumes_after_join_and_straight_line_behavior_remains() {
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let observed = *view; } let access = &mut owned; *access = value; return owned; }");
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; let observed = *view; if flag { let access = &mut owned; *access = value; } return owned; }");
}
#[test]
fn branch_call_and_write_holds_remain_active() {
    error("fn consume(view: &Ticket, ticket: Ticket) -> bool { return true; } fn bad(flag: bool, ticket: Ticket) -> bool { if flag { let done = consume(&ticket, ticket); } return true; }", "Ownership", "shared-borrowed");
    error("fn identity(value: Percent) -> Percent { return value; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; if flag { let access = &mut owned; *access = identity(owned); } return owned; }", "Ownership", "exclusively borrowed");
    error("fn sink(access: &mut bool) -> bool { return true; } fn bad(flag: bool, access: &mut bool) -> bool { if flag { *access = sink(access); } return true; }", "Ownership", "use of moved value");
}
#[test]
fn nested_loan_holds_and_branch_local_cleanup_are_sound() {
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { if flag { let observed = *view; } let access = &mut owned; } return owned; }");
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; if flag { let access = &mut owned; if flag { *access = value; } } return owned; }");
}
#[test]
fn non_copy_write_and_move_out_remain_forbidden_in_branches() {
    error("fn bad(flag: bool, access: &mut Ticket, replacement: Ticket) -> bool { if flag { *access = replacement; } return true; }", "Ownership", "before destruction semantics are defined");
    error(
        "fn bad(flag: bool, view: &Ticket) -> bool { if flag { let owned = *view; } return true; }",
        "Ownership",
        "out through borrowed reference",
    );
}
#[test]
fn simple_if_mir_has_explicit_false_edge_join_and_return() {
    let p = good("fn example(flag: bool, value: Percent) -> Percent { if flag { let observed = value; } return value; }");
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    assert_eq!(f.blocks().len(), 3);
    let Terminator::Branch {
        then_target,
        else_target: join,
        condition,
    } = f.block(f.entry()).terminator()
    else {
        panic!()
    };
    assert_eq!(
        condition.kind,
        ExprKind::Parameter(p.functions()[0].as_ordinary().unwrap().parameters[0])
    );
    assert_eq!(
        f.block(*then_target).terminator(),
        &Terminator::Goto { target: *join }
    );
    assert!(matches!(
        f.block(*join).terminator(),
        Terminator::Return { .. }
    ));
    let mir::Statement::Let { local, .. } = &f.block(*then_target).statements()[0] else {
        panic!()
    };
    assert_eq!(*local, p.locals()[0].id);
}
#[test]
fn if_else_mir_has_distinct_targets_and_common_join() {
    let p = good("fn example(flag: bool, value: Percent) -> Percent { if flag { let observed = value; } else { let observed = value; } return value; }");
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    assert_eq!(f.blocks().len(), 4);
    let Terminator::Branch {
        then_target,
        else_target,
        ..
    } = f.block(f.entry()).terminator()
    else {
        panic!()
    };
    assert_ne!(then_target, else_target);
    assert_eq!(
        f.block(*then_target).terminator(),
        f.block(*else_target).terminator()
    );
    let Terminator::Goto { target } = f.block(*then_target).terminator() else {
        panic!()
    };
    assert!(matches!(
        f.block(*target).terminator(),
        Terminator::Return { .. }
    ));
}
#[test]
fn nested_and_sequential_mir_is_deterministic_acyclic_and_has_only_one_return() {
    let p = good("fn example(flag: bool, value: Percent) -> Percent { if flag { if flag { let observed = value; } else { let observed = value; } } else { if flag {} } if flag {} return value; }");
    let m = mir::lower(&p);
    assert_eq!(m, mir::lower(&p));
    let f = &m.functions()[0];
    f.validate().unwrap();
    assert_eq!(
        f.blocks()
            .filter(|(_, b)| matches!(b.terminator(), Terminator::Branch { .. }))
            .count(),
        4
    );
    assert_eq!(
        f.blocks()
            .filter(|(_, b)| matches!(b.terminator(), Terminator::Return { .. }))
            .count(),
        1
    );
    for (_, b) in f.blocks() {
        for s in b.statements() {
            match s {
                mir::Statement::BindResult { .. } => {
                    panic!("unexpected Result binding in predecessor conditional fixture")
                }
                mir::Statement::ReplaceOwned { .. } => {
                    panic!("Copy-only fixture has no replacement")
                }
                mir::Statement::Let { .. }
                | mir::Statement::DerefAssign { .. }
                | mir::Statement::CopyFieldAssign { .. }
                | mir::Statement::CopyReferenceFieldAssign { .. }
                | mir::Statement::Assign { .. }
                | mir::Statement::CallNoValue { .. } => {}
            }
        }
    }
}
#[test]
fn verified_path_excluded_from_mir_and_proof_counts_unchanged() {
    let text = format!(
        "{} {}",
        include_str!("../../examples/battery_ok.bow"),
        source("fn example(flag: bool) -> bool { if flag {} return flag; }")
            .replace("type Percent = range 0..100;", "")
    );
    let p = compile_source(&text).unwrap();
    let m = mir::lower(&p);
    assert_eq!(m.functions().len(), 1);
    let report = bowfin_compiler::verify::verify_report(&p);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.functions_proven, 1);
}

#[test]
fn boolean_call_conditions_accept_copy_arguments_and_existing_borrow_holds() {
    good("fn predicate(value: Percent) -> bool { return value == value; } fn example(value: Percent) -> Percent { if predicate(value) { let observed = value; } return value; }");
    good("fn predicate(view: &Percent) -> bool { return true; } fn example(value: Percent) -> Percent { let mut owned = value; if predicate(&owned) { let access = &mut owned; *access = value; } return owned; }");
}
#[test]
fn mutable_reference_branches_are_independent_and_owner_recovers_without_surviving_handles() {
    good("fn sink(access: &mut Percent) -> bool { return true; } fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let done = sink(access); } else { *access = value; } return owned; }");
}
#[test]
fn branch_writes_retain_exact_nominal_typing_and_no_mutation_expansion() {
    error("fn predicate(flag: bool) -> bool { return flag; } fn bad(flag: bool) -> bool { if flag { predicate(flag); } return flag; }", "Type", "cannot be called as a statement");
    error(
        "fn bad(flag: bool) -> bool { if flag { flag = true; } return flag; }",
        "Type",
        "cannot assign parameter",
    );
    error("type Other = range 0..100; fn bad(flag: bool, access: &mut Percent, value: Other) -> bool { if flag { *access = value; } return true; }", "Type", "distinct named ranges");
    error("fn bad(flag: bool, view: &Percent, value: Percent) -> bool { if flag { *view = value; } return true; }", "Type", "exclusive mutable reference");
    for statements in ["if flag let local = flag;", "if flag { let value; }"] {
        error(&format!("fn predicate(flag: bool) -> bool {{ return flag; }} fn bad(flag: bool) -> bool {{ {statements} return flag; }}"), "Parse", "");
    }
}
#[test]
fn verified_conditionals_remain_outside_the_specialized_grammar() {
    let text = include_str!("../../examples/battery_ok.bow").replace(
        "battery.charge -= amount;",
        "if true { battery.charge -= amount; }",
    );
    assert!(matches!(
        compile_source(&text),
        Err(FrontendError::Parse(_))
    ));
}
