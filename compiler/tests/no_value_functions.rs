use bowfin_compiler::{ast, compile_source, hir, mir, parse_source, resolve, types, FrontendError};
const DECL: &str = "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent } fn work() {} fn observe(view: &bool) {} fn reset(access: &mut bool) { *access = false; } fn take_bool(value: bool) {} fn take_range(value: Percent) {} fn consume(ticket: Ticket) {} fn identity(value: bool) -> bool { return value; } fn fresh() -> Ticket { return fresh(); } fn take(ticket: Ticket) -> Ticket { return ticket; } fn inspect(view: &Ticket) -> bool { return true; } fn bool_ticket(seen: bool, ticket: Ticket) {} fn receive(first: Ticket, held: &bool, last: Ticket) {} fn refs(view: &bool, access: &mut bool) {} fn mut_refs(access: &mut bool, view: &bool) {} fn shared_bool(view: &bool, seen: bool) {} fn touch(access: &mut bool) -> bool { return true; } fn held(view: &Ticket, ticket: Ticket) {} fn read(view: &bool) -> bool { return *view; }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL} {body}")).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, wording: &str) {
    let e = compile_source(&format!("{DECL} {body}")).unwrap_err();
    let actual = match &e {
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        _ => panic!("{e}"),
    };
    assert_eq!(actual, phase, "{e}");
    assert!(e.to_string().contains(wording), "{e}");
}
#[test]
fn empty() {
    good("fn f() {}");
}
#[test]
fn ordinary_completion() {
    good("fn f(flag: bool) { let seen = flag; }");
}
#[test]
fn bare_return() {
    good("fn f() { return; }");
}
#[test]
fn nested_bare_return() {
    good("fn f(flag: bool) { if flag { if flag { return; } } }");
}
#[test]
fn all_bare_return() {
    good("fn f(flag: bool) { if flag { return; } else { return; } }");
}
#[test]
fn no_value_calls() {
    good("fn f() { work(); work(); }");
}
#[test]
fn forward_calls() {
    good("fn f() { later(); } fn later() {}");
}
#[test]
fn recursive_calls() {
    good("fn f() { f(); }");
}
#[test]
fn indirect_recursion() {
    good("fn f() { later(); } fn later() { f(); }");
}
#[test]
fn value_function_calls_no_value() {
    good("fn f() -> bool { work(); return true; }");
}
#[test]
fn branch_close_is_local() {
    good("fn f(flag: bool, value: bool) { if flag { work(); } observe(&value); }");
}
#[test]
fn while_fallthrough() {
    good("fn f(flag: bool) { while flag { work(); } }");
}
#[test]
fn while_bare_return() {
    good("fn f(flag: bool) { while flag { return; } }");
}
#[test]
fn constant_true_false_exit() {
    good("fn f() { while true { continue; } }");
}
#[test]
fn constant_false_still_checked() {
    good("fn f() { while false { work(); } }");
}
#[test]
fn return_continue() {
    good("fn f(flag: bool) { while flag { if flag { return; } else { continue; } } }");
}
#[test]
fn continue_return() {
    good("fn f(flag: bool) { while flag { if flag { continue; } else { return; } } }");
}
#[test]
fn return_break() {
    good("fn f(flag: bool) { while flag { if flag { return; } else { break; } } }");
}
#[test]
fn break_return() {
    good("fn f(flag: bool) { while flag { if flag { break; } else { return; } } }");
}
#[test]
fn mixed_three() {
    good("fn f(flag: bool) { while flag { if flag { return; } else { if flag { break; } else { continue; } } } }");
}
#[test]
fn nested_loops() {
    good("fn f(flag: bool) { while flag { while flag { if flag { return; } break; } continue; } }");
}
#[test]
fn copy_args() {
    good("fn f(value: bool) { take_bool(value); take_bool(value); }");
}
#[test]
fn move_argument() {
    good("fn f(ticket: Ticket) { consume(ticket); }");
}
#[test]
fn bind_unused_value() {
    good("fn f(value: bool) { let unused = identity(value); }");
}
#[test]
fn fresh_records_in_loop() {
    good("fn f(flag: bool) { while flag { consume(fresh()); } }");
}
#[test]
fn local_records_in_loop() {
    good("fn f(flag: bool) { while flag { let ticket = fresh(); consume(ticket); } }");
}
#[test]
fn fresh_mutable_handle() {
    good("fn f(flag: bool) { let mut owned = false; while flag { let access = &mut owned; reset(access); } }");
}
#[test]
fn shared_copy_handles() {
    good("fn f(value: bool) { let view = &value; observe(view); observe(view); }");
}
#[test]
fn mutable_move_then_owner_recovery() {
    good("fn f(value: bool) { let mut owned = value; let access = &mut owned; reset(access); let seen = owned; }");
}
#[test]
fn direct_mutable_args() {
    good("fn f(value: bool) { let mut owned = value; reset(&mut owned); let seen = owned; }");
}
#[test]
fn last_shared_then_exclusive() {
    good("fn f(value: bool) { let mut owned = value; let view = &owned; observe(view); reset(&mut owned); }");
}
#[test]
fn completed_nested_borrow() {
    good("fn f(ticket: Ticket) { bool_ticket(inspect(&ticket), ticket); }");
}
#[test]
fn nested_value_argument() {
    good("fn f(value: bool) { take_bool(identity(identity(value))); }");
}
#[test]
fn ending_states_need_no_exit_convergence() {
    good("fn f(choice: bool, ticket: Ticket) { if choice { consume(ticket); } }");
}
#[test]
fn returned_move_is_not_joined() {
    good("fn f(choice: bool, ticket: Ticket) { if choice { consume(ticket); return; } consume(ticket); }");
}
#[test]
fn return_bypasses_outside() {
    good("fn f(flag: bool) { let mut value = false; let view = &value; while flag { value = true; return; } observe(view); }");
}
#[test]
fn nested_return_bypasses_all_outside() {
    good("fn f(flag: bool) { let mut value = false; let view = &value; while flag { while flag { value = true; return; } } observe(view); }");
}
#[test]
fn skipped_iteration_suffix() {
    good("fn f(flag: bool) { let mut value = false; while flag { let view = &value; if flag { value = true; return; } observe(view); } }");
}
#[test]
fn removed_recurrence_positive() {
    good("fn f(flag: bool) { let mut value = false; let view = &value; while flag { if flag { value = true; return; } } }");
}
#[test]
fn shared_recurrent_call() {
    good(
        "fn f(flag: bool) { let value = false; let view = &value; while flag { observe(view); } }",
    );
}
#[test]
fn nested_shared_recurrent() {
    good("fn f(flag: bool) { let value = false; let view = &value; while flag { while flag { observe(view); break; } observe(view); } }");
}
#[test]
fn alias_recurrent_call() {
    good("fn f(flag: bool) { let value = false; let view = &value; while flag { let alias = view; observe(alias); } }");
}
#[test]
fn branch_sibling_spelling() {
    good("fn f(flag: bool) { if flag { let seen = flag; } else { let seen = flag; } work(); }");
}
#[test]
fn function_alias() {
    good("function f() { return; }");
}
#[test]
fn no_value_return_does_not_pin_survivor() {
    good("fn f(flag: bool) { let mut owned = false; let view = &owned; if flag { observe(view); return; } reset(&mut owned); }");
}
#[test]
fn value_return_semantics_preserved() {
    good("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return take(ticket); } return ticket; }");
}
#[test]
fn bare_return_reference_parameter() {
    good("fn f(access: &mut bool) { return; }");
}
#[test]
fn body_copy_write() {
    good("fn f(flag: bool) { let mut value = false; let access = &mut value; while flag { *access = false; if flag { return; } } }");
}
#[test]
fn value_return_in_no_value() {
    bad("fn f() { return true; }", "Type", "value return");
}
#[test]
fn bare_in_value() {
    bad("fn f() -> bool { return; }", "Type", "bare return");
}
#[test]
fn nested_wrong_bare() {
    bad(
        "fn f(flag: bool) -> bool { if flag { return; } return true; }",
        "Type",
        "bare return",
    );
}
#[test]
fn nested_wrong_value() {
    bad(
        "fn f(flag: bool) { if flag { return true; } }",
        "Type",
        "value return",
    );
}
#[test]
fn absence_never_inferrs() {
    bad(
        "fn f(ticket: Ticket) { return ticket; }",
        "Type",
        "no-value",
    );
}
#[test]
fn value_function_incomplete() {
    bad("fn f() -> bool {}", "Type", "closing brace");
}
#[test]
fn recursive_call_cannot_complete_value() {
    bad(
        "fn recurse() { recurse(); } fn f() -> bool { recurse(); }",
        "Type",
        "closing brace",
    );
}
#[test]
fn value_while_return_incomplete() {
    bad(
        "fn f(flag: bool) -> bool { while flag { return true; } }",
        "Type",
        "closing brace",
    );
}
#[test]
fn bare_suffix() {
    bad("fn f() { return; work(); }", "Parse", "after return");
}
#[test]
fn all_return_suffix() {
    bad(
        "fn f(flag: bool) { if flag { return; } else { return; } work(); }",
        "Parse",
        "no fallthrough",
    );
}
#[test]
fn mixed_return_continue_suffix() {
    bad(
        "fn f(flag: bool) { while flag { if flag { return; } else { continue; } work(); } }",
        "Parse",
        "no fallthrough",
    );
}
#[test]
fn bare_semicolon() {
    bad("fn f() { return }", "Parse", "expression");
}
#[test]
fn call_semicolon() {
    bad("fn f() { work() }", "Parse", "Semicolon");
}
#[test]
fn no_value_local() {
    bad(
        "fn f() { let x = work(); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_value_return() {
    bad(
        "fn f() -> bool { return work(); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_nested_argument() {
    bad(
        "fn f() { take_bool(work()); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_if_condition() {
    bad(
        "fn f() { if work() {} }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_while_condition() {
    bad(
        "fn f() { while work() {} }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_unary() {
    bad(
        "fn f() { let x = !work(); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_binary() {
    bad(
        "fn f() { let x = work() == work(); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_conditional() {
    bad(
        "fn f(flag: bool) { let x = if flag { work() } else { work() }; }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_nested_conditional() {
    bad("fn f(flag: bool) { let x = if flag { if flag { work() } else { work() } } else { true }; }", "Type", "cannot be used as an expression");
}
#[test]
fn no_value_assignment() {
    bad(
        "fn f() { let mut value = false; value = work(); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn no_value_write() {
    bad(
        "fn f(access: &mut bool) { *access = work(); }",
        "Type",
        "cannot be used as an expression",
    );
}
#[test]
fn value_call_statement() {
    bad(
        "fn f() { identity(true); }",
        "Type",
        "cannot be called as a statement",
    );
}
#[test]
fn unknown_statement_callee() {
    bad("fn f() { missing(); }", "Resolve", "unknown function");
}
#[test]
fn bare_expression_statement() {
    bad("fn f() { true; }", "Parse", "bare expression statements");
}
#[test]
fn operator_statement() {
    bad(
        "fn f() { false == true; }",
        "Parse",
        "bare expression statements",
    );
}
#[test]
fn call_operator_statement() {
    bad("fn f() { identity(true) == true; }", "Parse", "Semicolon");
}
#[test]
fn parenthesized_expression_statement() {
    bad(
        "fn f() { (work()); }",
        "Parse",
        "bare expression statements",
    );
}
#[test]
fn unit_type() {
    bad("fn f() -> () {}", "Parse", "identifier");
}
#[test]
fn void_type() {
    bad("fn f() -> void {}", "Resolve", "unknown value type");
}
#[test]
fn no_value_wrong_arity() {
    bad("fn f() { take_bool(); }", "Type", "wrong argument count");
}
#[test]
fn no_value_wrong_exact_type() {
    bad(
        "fn f(value: Percent) { take_bool(value); }",
        "Type",
        "call argument type mismatch",
    );
}
#[test]
fn no_value_nominal_args() {
    bad(
        "fn f(value: Other) { take_range(value); }",
        "Type",
        "call argument type mismatch",
    );
}
#[test]
fn statement_call_literal_outside_parameter_range() {
    bad(
        "fn f() { take_range(101); }",
        "Type",
        "literal outside named range",
    );
}
#[test]
fn move_used_twice() {
    bad(
        "fn f(ticket: Ticket) { consume(ticket); consume(ticket); }",
        "Ownership",
        "use of moved value",
    );
}
#[test]
fn late_failure() {
    bad(
        "fn f(ticket: Ticket, flag: bool) { receive(ticket, &flag, ticket); }",
        "Ownership",
        "use of moved value",
    );
}
#[test]
fn shared_call_hold() {
    bad(
        "fn f() { let mut value = false; refs(&value, &mut value); }",
        "Ownership",
        "shared borrow is active",
    );
}
#[test]
fn exclusive_call_hold() {
    bad(
        "fn f() { let mut value = false; mut_refs(&mut value, &value); }",
        "Ownership",
        "exclusive borrow is active",
    );
}
#[test]
fn nested_call_hold() {
    bad(
        "fn f() { let mut value = false; shared_bool(&value, touch(&mut value)); }",
        "Ownership",
        "shared borrow is active",
    );
}
#[test]
fn direct_borrow_nested_hold() {
    bad(
        "fn f(ticket: Ticket) { held(&ticket, take(ticket)); }",
        "Ownership",
        "shared-borrowed",
    );
}
#[test]
fn final_handle_nested_hold() {
    bad(
        "fn f(ticket: Ticket) { let view = &ticket; held(view, take(ticket)); }",
        "Ownership",
        "shared-borrowed",
    );
}
#[test]
fn mutable_transfer_no_restore() {
    bad(
        "fn f(access: &mut bool) { reset(access); reset(access); }",
        "Ownership",
        "use of moved value",
    );
}
#[test]
fn loop_move_before_bare() {
    bad(
        "fn f(flag: bool, ticket: Ticket) { while flag { consume(ticket); return; } }",
        "Ownership",
        "pre-existing non-Copy",
    );
}
#[test]
fn loop_move_before_completion() {
    bad(
        "fn f(flag: bool, ticket: Ticket) { while flag { consume(ticket); } }",
        "Ownership",
        "pre-existing non-Copy",
    );
}
#[test]
fn nested_value_argument_no_terminal() {
    bad(
        "fn f(flag: bool, ticket: Ticket) { while flag { consume(take(ticket)); return; } }",
        "Ownership",
        "pre-existing non-Copy",
    );
}
#[test]
fn carried_mutable_stmt_transfer() {
    bad(
        "fn f(flag: bool, access: &mut bool) { while flag { reset(access); return; } }",
        "Ownership",
        "loop-carried mutable reference",
    );
}
#[test]
fn break_reaches_outside() {
    bad("fn f(flag: bool) { let mut value = false; let view = &value; while flag { value = true; break; } observe(view); }", "Ownership", "while it is borrowed");
}
#[test]
fn recurrent_call_arg_protects() {
    bad("fn f(flag: bool) { let mut value = false; let view = &value; while flag { if flag { value = true; return; } observe(view); } }", "Ownership", "while it is borrowed");
}
#[test]
fn outer_recurrent_empty_inner() {
    bad("fn f(flag: bool) { let mut value = false; let view = &value; while flag { while flag { value = true; return; } observe(view); } }", "Ownership", "while it is borrowed");
}
#[test]
fn nested_recurrent_call_args() {
    bad("fn f(flag: bool) { let mut value = false; let view = &value; while flag { if flag { value = true; return; } while flag { observe(view); } } }", "Ownership", "while it is borrowed");
}
#[test]
fn local_fallthrough_must_join() {
    bad(
        "fn f(flag: bool, ticket: Ticket) { if flag { consume(ticket); } consume(ticket); }",
        "Ownership",
        "may have been moved",
    );
}
#[test]
fn post_join_reference_needed() {
    bad("fn f(flag: bool) { let mut value = false; let view = &value; if flag { reset(&mut value); } observe(view); }", "Ownership", "shared borrow is active");
}
#[test]
fn while_condition_reference_still_rejected() {
    bad(
        "fn f(view: &bool) { while read(view) {} }",
        "Ownership",
        "while conditions",
    );
}
#[test]
fn constant_branch_wrong_kind_checked() {
    bad(
        "fn f() { if false { return true; } }",
        "Type",
        "value return",
    );
}
#[test]
fn constant_loop_wrong_kind_checked() {
    bad(
        "fn f() { while false { return true; } }",
        "Type",
        "value return",
    );
}
#[test]
fn noncopy_deref_still_rejected() {
    bad(
        "fn f(view: &Ticket) { consume(*view); }",
        "Ownership",
        "out through borrowed reference",
    );
}
#[test]
fn reference_return_still_rejected() {
    bad("fn f() -> &bool {}", "Type", "reference return types");
}
#[test]
fn implicit_value_tail_still_rejected() {
    bad(
        "fn f() -> bool { true }",
        "Parse",
        "bare expression statements",
    );
}
#[test]
fn owned_replacement_before_implicit_completion() {
    good("fn f(a: Ticket,b: Ticket) { let mut owned = a; owned = b; }");
}
#[test]
fn bare_return_branch_scope() {
    bad(
        "fn f(flag: bool) { if flag { let inner = flag; return; } take_bool(inner); }",
        "Resolve",
        "unknown value",
    );
}

fn ordinary_mut(p: &mut ast::Program, index: usize) -> &mut ast::ValueFunction {
    let ast::FunctionDecl::Ordinary(f) = &mut p.functions[index] else {
        panic!()
    };
    f
}
#[test]
fn source_ast_hir_result_kind_and_distinct_forms_preserve_identity_and_spans() {
    let source = "fn target(value: bool) {} fn caller(value: bool) { target(value); return; } fn value() -> bool { return true; }";
    let a = parse_source(source).unwrap();
    let ast::FunctionDecl::Ordinary(f) = &a.functions[1] else {
        panic!()
    };
    assert!(f.return_type.is_none());
    let ast::ValueStatement::CallNoValue { span, .. } = f.body[0] else {
        panic!()
    };
    let p = compile_source(source).unwrap();
    let f = p.functions()[1].as_ordinary().unwrap();
    assert!(f.return_type.is_none());
    assert_eq!(
        p.functions()[2].as_ordinary().unwrap().return_type,
        Some(hir::ValueType::Bool)
    );
    let hir::ValueStatement::CallNoValue {
        function,
        arguments,
        span: hspan,
    } = &f.body[0]
    else {
        panic!()
    };
    assert_eq!(*function, p.functions()[0].id());
    assert_eq!(*hspan, span);
    assert_eq!(arguments[0].kind, hir::ExprKind::Parameter(f.parameters[0]));
    assert_eq!(arguments[0].ty, hir::ExprType::Bool);
    assert!(matches!(
        f.body[1],
        hir::ValueStatement::ReturnNoValue { .. }
    ));
    let m = mir::lower(&p);
    let b = m.functions()[1].block(m.functions()[1].entry());
    assert!(
        matches!(&b.statements()[0], mir::Statement::CallNoValue { function: id, span: mspan, .. } if id == function && *mspan == span)
    );
    assert_eq!(b.terminator(), &mir::Terminator::ReturnNoValue);
}
#[test]
fn public_ast_wrong_return_kinds_reject_before_ownership() {
    let mut a = parse_source("fn f() { return; }").unwrap();
    ordinary_mut(&mut a, 0).return_type = Some(ast::ValueType {
        result_nodes: Vec::new(),
        name: "bool".into(),
        references: vec![],
    });
    let r = resolve::resolve(&a).unwrap();
    assert!(types::check(r).unwrap_err()[0]
        .message
        .contains("bare return"));
    let mut a = parse_source("fn f() -> bool { return true; }").unwrap();
    ordinary_mut(&mut a, 0).return_type = None;
    let r = resolve::resolve(&a).unwrap();
    assert!(types::check(r).unwrap_err()[0]
        .message
        .contains("value return"));
}
#[test]
fn public_ast_call_result_kind_uses_signatures_not_parser_history() {
    let mut a = parse_source(
        "fn target() -> bool { return true; } fn f() { let predecessor_result = target(); }",
    )
    .unwrap();
    let target = ordinary_mut(&mut a, 0);
    target.return_type = None;
    target.body.clear();
    let r = resolve::resolve(&a).unwrap();
    assert!(types::check(r).unwrap_err()[0]
        .message
        .contains("cannot be used as an expression"));
    let mut a = parse_source("fn target() {} fn f() { target(); }").unwrap();
    let value = parse_source("fn value() -> bool { return true; }").unwrap();
    let ast::FunctionDecl::Ordinary(value) = &value.functions[0] else {
        panic!()
    };
    let target = ordinary_mut(&mut a, 0);
    target.return_type = value.return_type.clone();
    target.body = value.body.clone();
    let r = resolve::resolve(&a).unwrap();
    assert!(types::check(r).unwrap_err()[0]
        .message
        .contains("cannot be called as a statement"));
}
#[test]
fn public_ast_bare_return_suffix_rejects_diagnostically() {
    let mut a = parse_source("fn f() { return; }").unwrap();
    let f = ordinary_mut(&mut a, 0);
    f.body.push(f.body[0].clone());
    assert!(resolve::resolve(&a).unwrap_err()[0]
        .message
        .contains("no fallthrough"));
}
#[test]
fn public_ast_nested_no_value_expression_and_conditional_positions_reject() {
    for text in ["fn target() -> bool { return true; } fn f() { let predecessor_result = if true { target() } else { target() }; }", "fn target() -> bool { return true; } fn sink(value: bool) {} fn f() { sink(target()); }"] {
  let mut a=parse_source(text).unwrap(); let target=ordinary_mut(&mut a,0);target.return_type=None;target.body.clear();
  let r=resolve::resolve(&a).unwrap(); assert!(types::check(r).unwrap_err()[0].message.contains("cannot be used as an expression"));
 }
}
#[test]
fn mir_empty_and_normal_completion_have_no_dummy_value_or_result_local() {
    let p = compile_source("fn empty() {} fn f() { empty(); }").unwrap();
    assert!(p.locals().is_empty());
    let m = mir::lower(&p);
    for f in m.functions() {
        assert_eq!(f.blocks().len(), 1);
        assert_eq!(
            f.block(f.entry()).terminator(),
            &mir::Terminator::ReturnNoValue
        );
        f.validate().unwrap();
    }
    assert!(m.functions()[0]
        .block(m.functions()[0].entry())
        .statements()
        .is_empty());
    assert_eq!(
        m.functions()[1]
            .block(m.functions()[1].entry())
            .statements()
            .len(),
        1
    );
}
#[test]
fn mir_all_return_arms_have_no_unreachable_completion_join() {
    let p = compile_source("fn f(flag: bool) { if flag { return; } else { return; } }").unwrap();
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    assert_eq!(f.blocks().len(), 3);
    assert_eq!(
        f.blocks()
            .filter(|(_, b)| matches!(b.terminator(), mir::Terminator::ReturnNoValue))
            .count(),
        2
    );
    f.validate().unwrap();
}
#[test]
fn mir_branch_close_remains_local_and_only_live_top_level_tail_completes() {
    let p =
        compile_source("fn work() {} fn f(flag: bool) { if flag { return; } work(); }").unwrap();
    let m = mir::lower(&p);
    let f = &m.functions()[1];
    assert_eq!(f.blocks().len(), 3);
    let mir::Terminator::Branch {
        then_target,
        else_target,
        ..
    } = f.block(f.entry()).terminator()
    else {
        panic!()
    };
    assert_eq!(
        f.block(*then_target).terminator(),
        &mir::Terminator::ReturnNoValue
    );
    assert_eq!(f.block(*else_target).statements().len(), 1);
    assert_eq!(
        f.block(*else_target).terminator(),
        &mir::Terminator::ReturnNoValue
    );
}
#[test]
fn mir_mixed_return_break_continue_and_real_false_edge() {
    for transfer in ["break", "continue"] {
        let p = compile_source(&format!(
            "fn f(flag: bool) {{ while flag {{ if flag {{ return; }} else {{ {transfer}; }} }} }}"
        ))
        .unwrap();
        let m = mir::lower(&p);
        let f = &m.functions()[0];
        assert_eq!(f.blocks().len(), 6);
        f.validate().unwrap();
        let mir::Terminator::Goto { target: header } = f.block(f.entry()).terminator() else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target: body,
            else_target: exit,
            ..
        } = f.block(*header).terminator()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target,
            else_target,
            ..
        } = f.block(*body).terminator()
        else {
            panic!()
        };
        assert_eq!(
            f.block(*then_target).terminator(),
            &mir::Terminator::ReturnNoValue
        );
        assert_eq!(
            f.block(*else_target).terminator(),
            &mir::Terminator::Goto {
                target: if transfer == "break" { *exit } else { *header }
            }
        );
        assert_eq!(f.block(*exit).terminator(), &mir::Terminator::ReturnNoValue);
    }
}
#[test]
fn verified_and_harness_boundary_is_unchanged() {
    let battery = include_str!("../../examples/battery_ok.bow");
    let p = compile_source(&format!("fn nothing() {{}} {battery}")).unwrap();
    let r = bowfin_compiler::verify::verify_report(&p);
    assert!(r.diagnostics.is_empty());
    assert_eq!(r.functions_proven, 1);
    assert_eq!(r.calls_checked, 1);
    assert_eq!(mir::lower(&p).functions().len(), 1);
    let text = battery.replace("battery.charge -= amount;", "nothing();");
    assert!(compile_source(&format!("fn nothing() {{}} {text}")).is_err());
    let text = battery.replace("consume(&mutable battery,", "nothing(&mutable battery,");
    assert!(compile_source(&format!("fn nothing() {{}} {text}")).is_err());
}

#[test]
fn public_ast_statement_call_cannot_target_verified_function_or_harness() {
    let mut source = parse_source(include_str!("../../examples/battery_ok.bow")).unwrap();
    let mut caller = parse_source("fn nothing() {} fn caller() { nothing(); }").unwrap();
    let ast::ValueStatement::CallNoValue { callee, .. } = &mut ordinary_mut(&mut caller, 1).body[0]
    else {
        panic!()
    };
    *callee = "consume".into();
    source.functions.extend(caller.functions);
    assert!(resolve::resolve(&source).unwrap_err()[0]
        .message
        .contains("ordinary"));
}
