use bowfin_compiler::{compile_source, hir, mir, parse_source, FrontendError};
const DECL: &str = "type Percent = range 0..100; record Ticket { value: Percent } fn fresh() -> Ticket { return fresh(); } fn consume(ticket: Ticket) -> bool { return true; } fn sink(access: &mut bool) -> bool { return true; } fn read(view: &bool) -> bool { return *view; } fn both(view: &bool, observed: bool) -> bool { return observed; }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL} {body}")).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, text: &str) {
    let source = format!("{DECL} {body}");
    let e = compile_source(&source).unwrap_err();
    let actual = match &e {
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        _ => panic!("unexpected {e}"),
    };
    assert_eq!(actual, phase, "{e}");
    assert!(e.to_string().contains(text), "{e}");
    for private in ["LoanId", "LocalId", "BasicBlockId"] {
        assert!(!e.to_string().contains(private));
    }
}
macro_rules! accept {
    ($name:ident, $source:expr) => {
        #[test]
        fn $name() {
            good($source);
        }
    };
}
macro_rules! reject {
    ($name:ident, $source:expr, $phase:expr, $text:expr) => {
        #[test]
        fn $name() {
            bad($source, $phase, $text);
        }
    };
}
accept!(simple, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { continue; } return value; }");
accept!(copy_mutation, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { running = false; continue; } return value; }");
accept!(one_arm, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { if skip { continue; } running = false; } return value; }");
accept!(all_arms, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { if skip { continue; } else { continue; } } return value; }");
accept!(nested_all_arms, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { if skip { if running { continue; } else { continue; } } else { continue; } } return value; }");
accept!(fresh_move_cleanup, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let ticket = fresh(); continue; } return value; }");
accept!(fresh_move_transfer, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let ticket = fresh(); let moved = ticket; continue; } return value; }");
accept!(paired_exclusive_moves, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let ticket = fresh(); if skip { let done = consume(ticket); continue; } let done = consume(ticket); } return value; }");
accept!(paired_inverse_moves, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let ticket = fresh(); if skip { let done = consume(ticket); } else { let done = consume(ticket); continue; } } return value; }");
accept!(shared_local_cleanup, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; let seen = *view; continue; } return value; }");
accept!(mutable_local_cleanup, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let access = &mut value; *access = true; continue; } return value; }");
accept!(mutable_local_transfer, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let access = &mut value; let second = access; *second = true; continue; } return value; }");
accept!(finite_suffix_cut_before_write, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; if skip { value = true; continue; } let seen = *view; } return value; }");
accept!(finite_suffix_cut_before_borrow, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; if skip { let access = &mut value; *access = true; continue; } let seen = *view; } return value; }");
accept!(finite_exclusive_suffix_cut, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let access = &mut value; if skip { let seen = value; continue; } *access = true; } return value; }");
accept!(nested_suffix_cut, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; if skip { if running { value = true; continue; } } let seen = *view; } return value; }");
accept!(shared_carried, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let view = &value; while running { let alias = view; let seen = *alias; continue; } return value; }");
accept!(mutable_carried, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let access = &mut value; while running { *access = true; continue; } return value; }");
accept!(carried_all_continue_false_exit, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let view = &value; while running { if skip { let seen = *view; continue; } else { continue; } } value = true; return value; }");
accept!(post_loop_read, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let view = &value; while running { continue; } let seen = *view; return value; }");
accept!(dead_header_handle, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let unused = &mut value; while running { value = true; continue; } return value; }");
accept!(nested_targets, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { while skip { continue; } running = false; continue; } return value; }");
accept!(outer_local_inner_carried, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; while skip { let seen = *view; continue; } continue; } return value; }");
accept!(outer_local_mutable_inner_carried, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let access = &mut value; while skip { *access = true; continue; } continue; } return value; }");
accept!(outer_recurrent_inner_all_continue, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let view = &value; while running { while skip { if running { let seen = *view; continue; } else { continue; } } continue; } return value; }");
accept!(inner_only_frame_discharge, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; while skip { let seen = *view; continue; } value = true; continue; } return value; }");
accept!(call_holds, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let view = &value; let seen = both(view, read(view)); continue; } return value; }");
accept!(write_holds, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { let access = &mut value; *access = *access; continue; } return value; }");
accept!(sibling_spelling, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; while running { if skip { let seen = value; continue; } else { let seen = value; } } return value; }");
accept!(constant_false, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let view = &value; while false { let seen = *view; continue; } return value; }");
accept!(constant_true, "fn f(flag: bool, skip: bool) -> bool { let mut running = flag; let mut value = false; let access = &mut value; while true { *access = true; continue; } return value; }");
reject!(
    outside,
    "fn f(flag: bool) -> bool { continue; return flag; }",
    "Parse",
    "only inside"
);
reject!(
    outside_if,
    "fn f(flag: bool) -> bool { if flag { continue; } return flag; }",
    "Parse",
    "only inside"
);
reject!(
    missing_semicolon,
    "fn f(flag: bool) -> bool { while flag { continue } return flag; }",
    "Parse",
    ""
);
reject!(
    value,
    "fn f(flag: bool) -> bool { while flag { continue true; } return flag; }",
    "Parse",
    ""
);
reject!(
    label,
    "fn f(flag: bool) -> bool { while flag { continue outer; } return flag; }",
    "Parse",
    ""
);
reject!(
    direct_suffix,
    "fn f(flag: bool) -> bool { while flag { continue; let seen = flag; } return flag; }",
    "Parse",
    "after continue"
);
reject!(all_arm_suffix, "fn f(flag: bool) -> bool { while flag { if flag { continue; } else { continue; } let seen = flag; } return flag; }", "Parse", "no fallthrough");
reject!(nested_all_arm_suffix, "fn f(flag: bool) -> bool { while flag { if flag { if flag { continue; } else { continue; } } else { continue; } let seen = flag; } return flag; }", "Parse", "no fallthrough");
reject!(
    initializer,
    "fn f(flag: bool) -> bool { while flag { let seen = continue; } return flag; }",
    "Parse",
    ""
);
reject!(
    argument,
    "fn f(flag: bool) -> bool { while flag { let seen = read(continue); } return flag; }",
    "Parse",
    ""
);
reject!(
    unsupported_break,
    "fn f(flag: bool) -> bool { while flag { break outer; } return flag; }",
    "Parse",
    ""
);
accept!(
    loop_return,
    "fn f(flag: bool) -> bool { while flag { return flag; } return flag; }"
);
reject!(branch_scope, "fn f(flag: bool) -> bool { while flag { if flag { let seen = flag; continue; } let copy = seen; } return flag; }", "Resolve", "unknown");
reject!(recurrent_shared_skip, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { if flag { value = true; continue; } let seen = *view; } return flag; }", "Ownership", "cannot assign");
reject!(recurrent_exclusive_skip, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { if flag { let other = &value; continue; } *access = true; } return flag; }", "Ownership", "exclusive borrow");
reject!(recurrent_one_shot, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; let mut running = flag; while running { running = false; if flag { value = true; continue; } let seen = *view; } return flag; }", "Ownership", "cannot assign");
reject!(post_loop_suffix, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { value = true; continue; } let seen = *view; return flag; }", "Ownership", "cannot assign");
reject!(post_loop_exclusive, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { let seen = value; continue; } let later = *access; return flag; }", "Ownership", "exclusively borrowed");
reject!(inner_exit_keeps_outer, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { while flag { let seen = *view; continue; } value = true; continue; } return flag; }", "Ownership", "cannot assign");
reject!(outer_move, "fn f(flag: bool) -> bool { let ticket = fresh(); while flag { let moved = ticket; continue; } return flag; }", "Ownership", "pre-existing non-Copy");
reject!(outer_mutable_move, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { let moved = access; continue; } return flag; }", "Ownership", "loop-carried mutable");
reject!(inner_moves_outer_local, "fn f(flag: bool) -> bool { while flag { let ticket = fresh(); while flag { let moved = ticket; continue; } } return flag; }", "Ownership", "pre-existing non-Copy");
reject!(paired_fallthrough_moves, "fn f(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { let done = consume(ticket); } let done = consume(ticket); } return flag; }", "Ownership", "may have been moved");
reject!(one_survivor_actual_move, "fn f(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { continue; } else { let moved = ticket; } let done = consume(ticket); } return flag; }", "Ownership", "use of moved value");
reject!(same_path_live, "fn f(flag: bool) -> bool { let mut value = false; while flag { let view = &value; value = true; let seen = *view; continue; } return flag; }", "Ownership", "cannot assign");
reject!(call_hold_conflict, "fn f(flag: bool) -> bool { let mut value = false; while flag { let view = &value; let access = &mut value; let done = both(view, sink(access)); continue; } return flag; }", "Ownership", "shared borrow");
reject!(write_hold_conflict, "fn f(flag: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = value; continue; } return flag; }", "Ownership", "exclusively borrowed");

#[test]
fn continue_survives_source_and_canonical_hir() {
    let source = "fn f(flag: bool) -> bool { while flag { continue; } return flag; }";
    let ast = parse_source(source).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::While { body, .. } = &f.body[0] else {
        panic!()
    };
    assert!(matches!(
        body[0],
        bowfin_compiler::ast::ValueStatement::Continue { .. }
    ));
    let p = compile_source(source).unwrap();
    let hir::ValueStatement::While { body, .. } = &p.functions()[0].as_ordinary().unwrap().body[0]
    else {
        panic!()
    };
    assert!(matches!(body[0], hir::ValueStatement::Continue { .. }));
}
#[test]
fn all_continue_mir_has_no_unreachable_join_or_fabricated_bottom() {
    for (body, blocks) in [
        ("continue;", 4),
        ("if flag { continue; } else { continue; }", 6),
    ] {
        let p = compile_source(&format!(
            "fn f(flag: bool) -> bool {{ while flag {{ {body} }} return flag; }}"
        ))
        .unwrap();
        let m = mir::lower(&p);
        let f = &m.functions()[0];
        f.validate().unwrap();
        assert_eq!(f.blocks().len(), blocks);
        let mir::Terminator::Goto { target: header } = f.block(f.entry()).terminator().clone()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            else_target: exit, ..
        } = f.block(header).terminator().clone()
        else {
            panic!()
        };
        assert!(matches!(
            f.block(exit).terminator().clone(),
            mir::Terminator::Return { .. }
        ));
        let backedges = f.blocks().filter(|(_, b)| matches!(b.terminator().clone(), mir::Terminator::Goto { target } if target == header)).count();
        assert_eq!(backedges, if blocks == 4 { 2 } else { 3 });
    }
}
#[test]
fn nested_continue_targets_innermost_header_and_restores_outer() {
    let p = compile_source("fn f(flag: bool) -> bool { while flag { while flag { continue; } continue; } return flag; }").unwrap();
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    let mir::Terminator::Goto { target: outer } = f.block(f.entry()).terminator().clone() else {
        panic!()
    };
    let mir::Terminator::Branch {
        then_target: outer_body,
        ..
    } = f.block(outer).terminator().clone()
    else {
        panic!()
    };
    let mir::Terminator::Goto { target: inner } = f.block(outer_body).terminator().clone() else {
        panic!()
    };
    let mir::Terminator::Branch {
        then_target: inner_body,
        else_target: inner_exit,
        ..
    } = f.block(inner).terminator().clone()
    else {
        panic!()
    };
    assert_eq!(
        f.block(inner_body).terminator().clone(),
        mir::Terminator::Goto { target: inner }
    );
    assert_eq!(
        f.block(inner_exit).terminator().clone(),
        mir::Terminator::Goto { target: outer }
    );
}

#[test]
fn one_arm_continue_keeps_only_the_real_suffix_and_canonical_local_ids() {
    let p = compile_source("fn f(flag: bool) -> bool { while flag { if flag { let observed = flag; continue; } else { let observed = flag; } let suffix = flag; } return flag; }").unwrap();
    let function = p.functions()[0].as_ordinary().unwrap();
    let hir::ValueStatement::While { body, .. } = &function.body[0] else {
        panic!()
    };
    let hir::ValueStatement::If {
        then_body,
        else_body,
        ..
    } = &body[0]
    else {
        panic!()
    };
    let hir::ValueStatement::Let { local: yes, .. } = then_body[0] else {
        panic!()
    };
    let hir::ValueStatement::Let { local: no, .. } = else_body[0] else {
        panic!()
    };
    let hir::ValueStatement::Let { local: suffix, .. } = body[1] else {
        panic!()
    };
    assert_ne!(yes, no);
    let m = mir::lower(&p);
    let f = &m.functions()[0];
    f.validate().unwrap();
    let mir::Terminator::Goto { target: header } = f.block(f.entry()).terminator().clone() else {
        panic!()
    };
    let mir::Terminator::Branch {
        then_target: dispatch,
        ..
    } = f.block(header).terminator().clone()
    else {
        panic!()
    };
    let mir::Terminator::Branch {
        then_target,
        else_target,
        ..
    } = f.block(dispatch).terminator().clone()
    else {
        panic!()
    };
    assert_eq!(
        f.block(then_target).terminator(),
        &mir::Terminator::Goto { target: header }
    );
    let mir::Terminator::Goto { target: join } = f.block(else_target).terminator().clone() else {
        panic!()
    };
    assert!(f
        .block(join)
        .statements()
        .iter()
        .any(|s| matches!(s, mir::Statement::Let { local, .. } if *local == suffix)));
    assert_eq!(
        f.blocks()
            .filter(|(_, b)| b
                .statements()
                .iter()
                .any(|s| matches!(s, mir::Statement::Let { local, .. } if *local == suffix)))
            .count(),
        1
    );
}

#[test]
fn repeated_public_check_and_lower_are_read_only_and_deterministic() {
    let p = good("fn f(flag: bool) -> bool { let mut value = false; while flag { let view = &value; if flag { value = true; continue; } let seen = *view; } return value; }");
    let before = p.clone();
    for _ in 0..3 {
        bowfin_compiler::ownership::check(&p).unwrap();
        mir::lower(&p)
            .functions()
            .iter()
            .for_each(|f| f.validate().unwrap());
    }
    assert_eq!(p, before);
}

#[test]
fn public_ast_resolution_enforces_continue_placement_and_structural_suffix() {
    let mut ast =
        parse_source("fn f(flag: bool) -> bool { while flag { continue; } return flag; }").unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &mut ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::While { body, .. } = &f.body[0] else {
        panic!()
    };
    let continue_statement = body[0].clone();
    f.body.insert(0, continue_statement.clone());
    assert!(bowfin_compiler::resolve::resolve(&ast).unwrap_err()[0]
        .message
        .contains("only inside"));
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &mut ast.functions[0] else {
        panic!()
    };
    f.body.remove(0);
    let tail = f.body[1].clone();
    let bowfin_compiler::ast::ValueStatement::While { body, .. } = &mut f.body[0] else {
        panic!()
    };
    body.push(tail);
    assert!(bowfin_compiler::resolve::resolve(&ast).unwrap_err()[0]
        .message
        .contains("no fallthrough"));
}
reject!(direct_call_hold_survives_until_nested_argument, "fn f(flag: bool) -> bool { let mut value = false; while flag { let predecessor_result = both(&value, sink(&mut value)); continue; } return value; }", "Ownership", "shared borrow");
reject!(write_hold_blocks_rhs_borrow_after_final_target_use, "fn f(flag: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = read(&value); continue; } return value; }", "Ownership", "exclusive borrow");
reject!(write_rhs_transfer_is_not_reborrow, "fn f(flag: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = sink(access); continue; } return value; }", "Ownership", "use of moved value");
