use bowfin_compiler::{ast, compile_source, hir, mir, parse_source, resolve, types, FrontendError};
const DECL: &str = "type Percent = range 0..100; type OtherRange = range 0..100; record Ticket { value: Percent } record Different { value: Percent } fn fresh() -> Ticket { return fresh(); } fn take(ticket: Ticket) -> Ticket { return ticket; } fn consume(ticket: Ticket) -> bool { return true; } fn inspect(view: &Ticket) -> bool { return true; } fn holding(view: &Ticket, ticket: Ticket) -> Ticket { return ticket; } fn holding_mut(view: &mut Ticket, ticket: Ticket) -> Ticket { return ticket; } fn after(observed: bool, ticket: Ticket) -> Ticket { return ticket; } fn pair(first: Ticket, second: Ticket) -> Ticket { return first; } fn combine(first: Ticket, view: &bool, second: Ticket) -> Ticket { return second; } fn sink(access: &mut bool) -> bool { return true; } fn sink_ticket(access: &mut Ticket) -> bool { return true; } fn read(view: &bool) -> bool { return *view; }";
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
    let text = format!("{DECL} {body}");
    let e = compile_source(&text).unwrap_err();
    let actual = match &e {
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        _ => panic!("unexpected {e}"),
    };
    assert_eq!(actual, phase, "{e}");
    assert!(e.to_string().contains(wording), "{e}");
    for id in ["LoanId", "LocalId", "BasicBlockId"] {
        assert!(!e.to_string().contains(id));
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
    final_return_unchanged,
    "fn f(value: Percent) -> Percent { return value; }"
);
accept!(
    top_level_after_locals,
    "fn f(value: Percent) -> Percent { let copied = value; return copied; }"
);
accept!(
    then_return_with_suffix,
    "fn f(flag: bool) -> bool { if flag { return true; } return false; }"
);
accept!(
    else_return_with_suffix,
    "fn f(flag: bool) -> bool { if flag { let seen = flag; } else { return true; } return false; }"
);
accept!(
    both_return,
    "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } }"
);
accept!(nested_all_return, "fn f(first: bool, second: bool) -> bool { if first { if second { return true; } else { return false; } } else { if second { return false; } return true; } }");
accept!(
    body_return_and_false_exit,
    "fn f(flag: bool) -> bool { while flag { return true; } return false; }"
);
accept!(
    constant_true_continue_keeps_false_exit,
    "fn f() -> bool { while true { continue; } return false; }"
);
accept!(
    constant_true_return_keeps_false_exit,
    "fn f() -> bool { while true { return true; } return false; }"
);
accept!(
    constant_false_return_is_still_checked,
    "fn f() -> bool { while false { return true; } return false; }"
);
accept!(return_continue, "fn f(flag: bool, stop: bool) -> bool { while flag { if stop { return true; } else { continue; } } return false; }");
accept!(continue_return, "fn f(flag: bool, stop: bool) -> bool { while flag { if stop { continue; } else { return true; } } return false; }");
accept!(return_break, "fn f(flag: bool, stop: bool) -> bool { while flag { if stop { return true; } else { break; } } return false; }");
accept!(break_return, "fn f(flag: bool, stop: bool) -> bool { while flag { if stop { break; } else { return true; } } return false; }");
accept!(mixed_three_terminal_edges, "fn f(flag: bool, stop: bool) -> bool { while flag { if stop { return true; } else { if flag { break; } else { continue; } } } return false; }");
accept!(nested_return_outer_continue, "fn f(flag: bool, stop: bool, ticket: Ticket) -> Ticket { while flag { while stop { return ticket; } continue; } return fresh(); }");
accept!(nested_return_outer_break, "fn f(flag: bool, stop: bool, ticket: Ticket) -> Ticket { while flag { while stop { if flag { return ticket; } break; } break; } return ticket; }");
accept!(outside_only_return_bypasses_reference_suffix, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { value = true; return false; } return *view; }");
accept!(nested_return_bypasses_all_outside_continuations, "fn f(flag: bool, inner: bool) -> bool { let mut value = false; let view = &value; while flag { while inner { value = true; return false; } } return *view; }");
accept!(skipped_iteration_local_reference_suffix, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; if stop { value = true; return false; } let seen = *view; } return value; }");
accept!(removed_recurrent_use_positive_control, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { value = true; return false; } } return value; }");
accept!(
    owned_record_direct_loop_return,
    "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return ticket; } return fresh(); }"
);
accept!(owned_record_return_argument, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return take(ticket); } return fresh(); }");
accept!(owned_record_deep_return_arguments, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return take(take(ticket)); } return fresh(); }");
accept!(owned_record_return_consuming_bool_call, "fn f(flag: bool, ticket: Ticket) -> bool { while flag { return consume(ticket); } return false; }");
accept!(iteration_local_record_return, "fn f(flag: bool) -> Ticket { while flag { let ticket = fresh(); return ticket; } return fresh(); }");
accept!(outer_iteration_record_returned_from_inner, "fn f(flag: bool, inner: bool) -> Ticket { while flag { let ticket = fresh(); while inner { return ticket; } let done = consume(ticket); } return fresh(); }");
accept!(completed_nested_borrow_before_terminal_move, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return after(inspect(&ticket), ticket); } return fresh(); }");
accept!(iteration_local_handle_participates_in_return, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let view = &ticket; return after(inspect(view), ticket); } return fresh(); }");
accept!(iteration_local_mutable_handle_argument, "fn f(flag: bool) -> bool { let mut value = false; while flag { let access = &mut value; return sink(access); } return value; }");
accept!(direct_borrow_return_call, "fn f(flag: bool, ticket: Ticket) -> bool { while flag { return inspect(&ticket); } return false; }");
accept!(carried_shared_reference_return_operand, "fn f(flag: bool) -> bool { let value = false; let view = &value; while flag { return *view; } return value; }");
accept!(carried_mutable_reference_read_return_operand, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { return *access; } return value; }");
accept!(returned_move_never_contaminates_sibling, "fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { return ticket; } let moved = ticket; return moved; }");
accept!(returned_reference_path_never_pins_sibling, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; if flag { return *view; } else { let access = &mut value; *access = true; } return value; }");
accept!(returned_mutable_path_never_pins_sibling, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; if flag { return *access; } let seen = value; return value; }");
accept!(all_return_owned_moves_are_independent, "fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { return ticket; } else { return take(ticket); } }");
accept!(nested_condition_early_returns, "fn f(flag: bool, stop: bool, ticket: Ticket) -> Ticket { if flag { if stop { return take(ticket); } } return ticket; }");
accept!(return_value_uses_loan_through_completed_call, "fn f(flag: bool) -> bool { let value = false; let view = &value; while flag { return read(view); } return value; }");
reject!(
    missing_false_path_return,
    "fn f(flag: bool) -> bool { if flag { return true; } }",
    "Type",
    "closing brace"
);
reject!(empty_body, "fn f() -> bool {}", "Type", "closing brace");
reject!(
    while_true_return_is_not_complete,
    "fn f() -> bool { while true { return true; } }",
    "Type",
    "closing brace"
);
reject!(
    while_false_return_is_not_complete,
    "fn f() -> bool { while false { return true; } }",
    "Type",
    "closing brace"
);
reject!(
    while_all_loop_transfers_is_not_complete,
    "fn f(flag: bool) -> bool { while flag { if flag { break; } else { continue; } } }",
    "Type",
    "closing brace"
);
reject!(implicit_tail, "fn f() -> bool { true }", "Parse", "return");
reject!(
    return_semicolon_required,
    "fn f(flag: bool) -> bool { if flag { return true } return false; }",
    "Parse",
    "semicolon"
);
reject!(
    suffix_after_direct_return,
    "fn f() -> bool { return true; let later = false; }",
    "Parse",
    "statements after return"
);
reject!(
    suffix_after_all_return,
    "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } return true; }",
    "Parse",
    "no fallthrough"
);
reject!(suffix_after_nested_all_return, "fn f(flag: bool) -> bool { if flag { if flag { return true; } else { return false; } } else { return false; } return true; }", "Parse", "no fallthrough");
reject!(suffix_after_mixed_return_continue, "fn f(flag: bool) -> bool { while flag { if flag { return true; } else { continue; } let seen = flag; } return false; }", "Parse", "no fallthrough");
reject!(suffix_after_mixed_return_break, "fn f(flag: bool) -> bool { while flag { if flag { return true; } else { break; } let seen = flag; } return false; }", "Parse", "no fallthrough");
reject!(paired_outside_only_break_still_protects, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { value = true; break; } return *view; }", "Ownership", "while it is borrowed");
reject!(operand_use_is_not_discarded, "fn f(flag: bool) -> bool { let mut value = false; while flag { let view = &value; value = true; return *view; } return value; }", "Ownership", "while it is borrowed");
reject!(recurrent_protection_before_return, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { value = true; return false; } let seen = *view; } return value; }", "Ownership", "while it is borrowed");
reject!(recurrent_owner_move_during_return, "fn f(flag: bool, stop: bool, ticket: Ticket) -> Ticket { let view = &ticket; while flag { if stop { return ticket; } let seen = inspect(view); } return ticket; }", "Ownership", "shared-borrowed");
reject!(outer_recurrence_protects_empty_inner_return, "fn f(flag: bool, inner: bool, ticket: Ticket) -> Ticket { let view = &ticket; while flag { while inner { return ticket; } let seen = inspect(view); } return ticket; }", "Ownership", "shared-borrowed");
reject!(operand_only_uses_establish_recurrence, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { value = true; return false; } else { return *view; } } return value; }", "Ownership", "while it is borrowed");
reject!(pre_return_binding_move_not_relaxed, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let moved = ticket; return moved; } return fresh(); }", "Ownership", "pre-existing non-Copy");
reject!(pre_return_call_move_not_relaxed, "fn f(flag: bool, ticket: Ticket) -> bool { while flag { let done = consume(ticket); return false; } return false; }", "Ownership", "pre-existing non-Copy");
reject!(pre_return_conditional_move_not_relaxed, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let moved = if flag { ticket } else { fresh() }; return moved; } return fresh(); }", "Ownership", "pre-existing non-Copy");
reject!(held_shared_borrow_blocks_return_move, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return holding(&ticket, ticket); } return fresh(); }", "Ownership", "shared-borrowed");
reject!(held_exclusive_borrow_blocks_return_move, "fn f(flag: bool, ticket: Ticket) -> Ticket { let mut owned = ticket; while flag { return holding_mut(&mut owned, owned); } return fresh(); }", "Ownership", "exclusively borrowed");
reject!(recurrent_exclusive_borrow_blocks_return_move, "fn f(flag: bool, stop: bool, ticket: Ticket) -> Ticket { let mut owned = ticket; let access = &mut owned; while flag { if stop { return owned; } let seen = sink_ticket(access); } return fresh(); }", "Ownership", "exclusively borrowed");
reject!(duplicate_return_use, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return pair(ticket, ticket); } return fresh(); }", "Ownership", "use of moved value");
reject!(late_argument_failure, "fn f(flag: bool, ticket: Ticket, value: bool) -> Ticket { while flag { return combine(ticket, &value, ticket); } return fresh(); }", "Ownership", "use of moved value");
reject!(carried_mutable_reference_transfer_not_relaxed, "fn f(flag: bool, access: &mut bool) -> bool { while flag { return sink(access); } return false; }", "Ownership", "loop-carried mutable reference");
reject!(
    noncopy_deref_move_not_relaxed,
    "fn f(flag: bool, view: &Ticket) -> Ticket { while flag { return *view; } return fresh(); }",
    "Ownership",
    "out through borrowed reference"
);
reject!(
    reference_returns_still_forbidden,
    "fn f(flag: bool, view: &bool) -> &bool { if flag { return view; } return view; }",
    "Type",
    "reference return types"
);
reject!(
    early_return_exact_type,
    "fn f(flag: bool, value: Percent) -> Percent { if flag { return true; } return value; }",
    "Type",
    "return"
);
reject!(
    last_return_exact_type,
    "fn f(flag: bool, value: Percent) -> Percent { if flag { return value; } return true; }",
    "Type",
    "return"
);
reject!(early_return_nominal_range, "fn f(flag: bool, value: Percent, other: OtherRange) -> Percent { if flag { return other; } return value; }", "Type", "distinct named ranges");
reject!(early_return_nominal_record, "fn f(flag: bool, value: Ticket, other: Different) -> Ticket { if flag { return other; } return value; }", "Type", "return");
reject!(
    out_of_range_literal_early_return_rejected,
    "fn f(flag: bool, value: Percent) -> Percent { if flag { return 101; } return value; }",
    "Type",
    "return"
);
reject!(
    conditional_value_return_still_parse_error,
    "fn f(flag: bool) -> bool { return if flag { true } else { false }; }",
    "Parse",
    "expression"
);
reject!(conditional_value_call_argument_still_parse_error, "fn f(flag: bool, ticket: Ticket) -> Ticket { return take(if flag { ticket } else { fresh() }); }", "Parse", "expression");
reject!(condition_does_not_gain_terminal_move, "fn f(flag: bool, ticket: Ticket) -> Ticket { while consume(ticket) { return fresh(); } return ticket; }", "Ownership", "while conditions");
reject!(survivor_move_effect_is_not_repaired, "fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { return fresh(); } else { let moved = ticket; } return ticket; }", "Ownership", "use of moved value");
reject!(
    conditional_move_availability_unchanged,
    "fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { let moved = ticket; } return ticket; }",
    "Ownership",
    "may have been moved"
);
reject!(surviving_sibling_loan_is_not_lost, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; if flag { return false; } else { let access = &mut value; } return *view; }", "Ownership", "shared borrow is active");
reject!(constant_return_does_not_prune_bad_sibling, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { if true { return ticket; } else { let moved = ticket; } } return fresh(); }", "Ownership", "pre-existing non-Copy");
reject!(constant_recurrence_not_pruned, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while false { if flag { value = true; return false; } return *view; } return value; }", "Ownership", "while it is borrowed");
reject!(noncopy_replacement_still_forbidden, "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let mut owned = fresh(); owned = ticket; return owned; } return fresh(); }", "Ownership", "pre-existing non-Copy");
reject!(
    returned_branch_local_does_not_escape,
    "fn f(flag: bool) -> bool { if flag { let inside = false; return inside; } return inside; }",
    "Resolve",
    "unknown"
);

#[test]
fn every_return_is_typed_before_any_ownership_check() {
    // First branch would double-move if ownership ran before typing the later return.
    bad("fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { return pair(ticket, ticket); } return true; }", "Type", "return");
}
fn ordinary_mut(p: &mut ast::Program) -> &mut ast::ValueFunction {
    let ast::FunctionDecl::Ordinary(f) = &mut p.functions[0] else {
        panic!()
    };
    f
}
#[test]
fn public_ast_direct_return_suffix_is_diagnostic_not_panic() {
    let mut p = parse_source("fn f() -> bool { return true; }").unwrap();
    let f = ordinary_mut(&mut p);
    f.body.push(f.body[0].clone());
    let e = resolve::resolve(&p).unwrap_err();
    assert!(e[0].message.contains("no fallthrough"));
}
#[test]
fn public_ast_all_return_completeness_matches_source() {
    let p = parse_source(
        "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } }",
    )
    .unwrap();
    let mut manual = parse_source("fn f(flag: bool) -> bool { return true; }").unwrap();
    let ast::FunctionDecl::Ordinary(f) = &p.functions[0] else {
        panic!()
    };
    ordinary_mut(&mut manual).body = f.body.clone();
    let typed = types::check(resolve::resolve(&manual).unwrap()).unwrap();
    bowfin_compiler::ownership::check(&typed).unwrap();
    let direct = compile_source(
        "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } }",
    )
    .unwrap();
    assert_eq!(
        typed.functions()[0].as_ordinary().unwrap().body,
        direct.functions()[0].as_ordinary().unwrap().body
    );
    for f in mir::lower(&typed).functions() {
        f.validate().unwrap();
    }
}
#[test]
fn public_ast_real_fallthrough_is_rejected_before_ownership_or_mir() {
    for remove_else in [false, true] {
        let mut p = parse_source(
            "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } }",
        )
        .unwrap();
        let f = ordinary_mut(&mut p);
        if remove_else {
            let ast::ValueStatement::If { else_body, .. } = &mut f.body[0] else {
                panic!()
            };
            *else_body = None;
        } else {
            f.body.clear();
        }
        let e = types::check(resolve::resolve(&p).unwrap()).unwrap_err();
        assert!(e[0].message.contains("closing brace"));
    }
}
#[test]
fn public_ast_all_terminal_suffix_and_loop_transfer_boundaries_match_source() {
    let mut p = parse_source(
        "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } }",
    )
    .unwrap();
    let f = ordinary_mut(&mut p);
    f.body.push(f.body[0].clone());
    assert!(resolve::resolve(&p).unwrap_err()[0]
        .message
        .contains("no fallthrough"));
    let mut p =
        parse_source("fn f(flag: bool) -> bool { while flag { break; } return true; }").unwrap();
    ordinary_mut(&mut p).body.pop();
    assert!(types::check(resolve::resolve(&p).unwrap()).unwrap_err()[0]
        .message
        .contains("closing brace"));
}
#[test]
fn ast_hir_and_mir_preserve_early_return_operand_span_type_and_parameter_id() {
    let text = "fn f(flag: bool, value: bool) -> bool { if flag { return value; } return false; }";
    let ast = parse_source(text).unwrap();
    let ast::FunctionDecl::Ordinary(a) = &ast.functions[0] else {
        panic!()
    };
    let ast::ValueStatement::If { then_body, .. } = &a.body[0] else {
        panic!()
    };
    let ast::ValueStatement::Return { span, .. } = then_body[0] else {
        panic!()
    };
    assert_eq!(&text[span.start..span.end], "return value;");
    let p = compile_source(text).unwrap();
    let f = p.functions()[0].as_ordinary().unwrap();
    let hir::ValueStatement::If { then_body, .. } = &f.body[0] else {
        panic!()
    };
    let hir::ValueStatement::Return { value, span: s } = &then_body[0] else {
        panic!()
    };
    assert_eq!(*s, span);
    assert_eq!(value.ty, hir::ExprType::Bool);
    assert_eq!(value.kind, hir::ExprKind::Parameter(f.parameters[1]));
    assert!(mir::lower(&p).functions()[0]
        .blocks()
        .any(|(_, b)| matches!(b.terminator(), mir::Terminator::Return {value: v} if v == value)));
}
#[test]
fn mir_all_return_branches_have_only_source_returns_and_no_join() {
    let p = compile_source(
        "fn f(flag: bool) -> bool { if flag { return true; } else { return false; } }",
    )
    .unwrap();
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    assert_eq!(f.blocks().len(), 3);
    let mir::Terminator::Branch {
        then_target,
        else_target,
        ..
    } = f.block(f.entry()).terminator()
    else {
        panic!()
    };
    for target in [then_target, else_target] {
        assert!(matches!(
            f.block(*target).terminator(),
            mir::Terminator::Return { .. }
        ));
    }
    assert_eq!(
        f.blocks()
            .filter(|(_, b)| matches!(b.terminator(), mir::Terminator::Return { .. }))
            .count(),
        2
    );
    assert!(f
        .blocks()
        .all(|(_, b)| !matches!(b.terminator(), mir::Terminator::Goto { .. })));
}
#[test]
fn mir_single_survivor_keeps_its_real_statements_and_return() {
    let p = compile_source("fn f(flag: bool) -> bool { let mut value = false; if flag { return true; } else { value = true; } return value; }").unwrap();
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    let mir::Terminator::Branch {
        then_target,
        else_target,
        ..
    } = f.block(f.entry()).terminator()
    else {
        panic!()
    };
    assert!(matches!(
        f.block(*then_target).terminator(),
        mir::Terminator::Return { .. }
    ));
    assert!(matches!(
        f.block(*else_target).statements()[0],
        mir::Statement::Assign { .. }
    ));
    let mir::Terminator::Goto { target: join } = f.block(*else_target).terminator() else {
        panic!()
    };
    assert!(
        matches!(f.block(*join).terminator(), mir::Terminator::Return {value} if matches!(value.kind, hir::ExprKind::Local(id) if id == p.locals()[0].id))
    );
}
#[test]
fn mir_return_and_loop_transfer_destinations_are_distinct_and_correct() {
    for transfer in ["break;", "continue;"] {
        for inverse in [false, true] {
            let (yes, no) = if inverse {
                (transfer, "return true;")
            } else {
                ("return true;", transfer)
            };
            let p = compile_source(&format!("fn f(flag: bool) -> bool {{ while flag {{ if flag {{ {yes} }} else {{ {no} }} }} return false; }}")).unwrap();
            let m = mir::lower(&p);
            let f = &m.functions()[0];
            f.validate().unwrap();
            assert_eq!(f.blocks().len(), 6);
            let mir::Terminator::Goto { target: header } = f.block(f.entry()).terminator() else {
                panic!()
            };
            let mir::Terminator::Branch {
                then_target: dispatch,
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
            } = f.block(*dispatch).terminator()
            else {
                panic!()
            };
            let (ret, jump) = if inverse {
                (else_target, then_target)
            } else {
                (then_target, else_target)
            };
            assert!(matches!(
                f.block(*ret).terminator(),
                mir::Terminator::Return { .. }
            ));
            assert_eq!(
                f.block(*jump).terminator(),
                &mir::Terminator::Goto {
                    target: if transfer == "break;" { *exit } else { *header }
                }
            );
            assert!(matches!(
                f.block(*exit).terminator(),
                mir::Terminator::Return { .. }
            ));
        }
    }
}
#[test]
fn mir_nested_return_is_function_exit_and_outer_loop_targets_restore() {
    for outer_jump in ["break;", "continue;"] {
        let p = compile_source(&format!("fn f(flag: bool) -> bool {{ while flag {{ while flag {{ if flag {{ return true; }} break; }} {outer_jump} }} return false; }}")).unwrap();
        let m = mir::lower(&p);
        let f = &m.functions()[0];
        f.validate().unwrap();
        let mir::Terminator::Goto {
            target: outer_header,
        } = f.block(f.entry()).terminator()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target: outer_body,
            else_target: outer_exit,
            ..
        } = f.block(*outer_header).terminator()
        else {
            panic!()
        };
        let mir::Terminator::Goto {
            target: inner_header,
        } = f.block(*outer_body).terminator()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target: inner_body,
            else_target: inner_exit,
            ..
        } = f.block(*inner_header).terminator()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target: early_return,
            else_target: inner_break,
            ..
        } = f.block(*inner_body).terminator()
        else {
            panic!()
        };
        assert!(matches!(
            f.block(*early_return).terminator(),
            mir::Terminator::Return { .. }
        ));
        assert_eq!(
            f.block(*inner_break).terminator(),
            &mir::Terminator::Goto {
                target: *inner_exit
            }
        );
        assert_eq!(
            f.block(*inner_exit).terminator(),
            &mir::Terminator::Goto {
                target: if outer_jump == "break;" {
                    *outer_exit
                } else {
                    *outer_header
                }
            }
        );
        assert!(matches!(
            f.block(*outer_exit).terminator(),
            mir::Terminator::Return { .. }
        ));
    }
}
#[test]
fn mir_direct_return_needs_no_synthetic_final_return() {
    let p = compile_source("fn f() -> bool { return true; }").unwrap();
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    assert_eq!(f.blocks().len(), 1);
    assert!(matches!(
        f.block(f.entry()).terminator(),
        mir::Terminator::Return { .. }
    ));
}
#[test]
fn verified_return_boundary_and_proof_counts_are_unchanged() {
    let text = format!("{} fn ordinary(flag: bool) -> bool {{ if flag {{ return true; }} else {{ return false; }} }}", include_str!("../../examples/battery_ok.bow"));
    let p = compile_source(&text).unwrap();
    let report = bowfin_compiler::verify::verify_report(&p);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.functions_proven, 1);
    assert_eq!(mir::lower(&p).functions().len(), 1);
    let bad = include_str!("../../examples/battery_ok.bow")
        .replace("battery.charge -= amount;", "return amount;");
    assert!(matches!(compile_source(&bad), Err(FrontendError::Parse(_))));
}
