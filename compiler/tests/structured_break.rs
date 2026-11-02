use bowfin_compiler::{compile_source, hir, mir, parse_source, FrontendError};
const DECL: &str = "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent } fn fresh() -> Ticket { return fresh(); } fn consume(ticket: Ticket) -> bool { return true; } fn sink(access: &mut bool) -> bool { return true; } fn read(view: &bool) -> bool { return *view; } fn both(view: &bool, value: bool) -> bool { return value; }";
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
    for id in ["LoanId", "LocalId", "BasicBlockId"] {
        assert!(!e.to_string().contains(id));
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
accept!(simple, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { break; } return value; }");
accept!(one_arm, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { break; } let seen = value; } return value; }");
accept!(inverse_arm, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { let seen = value; } else { break; } let after = value; } return value; }");
accept!(copy_mutation, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { value = true; break; } return value; }");
accept!(copy_false_is_not_liveness_oracle, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; let mut running = flag; while running { running = false; if stop { break; } let seen = *view; } value = true; return value; }");
accept!(both_break, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { break; } else { break; } } value = true; return value; }");
accept!(break_continue, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { break; } else { continue; } } return value; }");
accept!(continue_break, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { continue; } else { break; } } return value; }");
accept!(nested_mixed_terminal, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { if flag { break; } else { continue; } } else { break; } } return value; }");
accept!(break_continue_bottom, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { break; } if flag { continue; } value = true; } return value; }");
accept!(local_move_unused, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let ticket = fresh(); break; } return value; }");
accept!(local_move_transferred, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let ticket = fresh(); let moved = ticket; break; } return value; }");
accept!(local_move_conditional_cleanup, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let ticket = fresh(); if stop { break; } let done = consume(ticket); } return value; }");
accept!(local_move_alternate_consumptions, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let ticket = fresh(); if stop { let done = consume(ticket); break; } let done = consume(ticket); } return value; }");
accept!(local_conditional_value, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let ticket = if stop { fresh() } else { fresh() }; break; } return value; }");
accept!(local_shared_cleanup, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; let seen = *view; break; } return value; }");
accept!(local_unused_shared_cleanup, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; break; } return value; }");
accept!(local_mutable_cleanup, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = true; break; } return value; }");
accept!(local_mutable_transfer, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let access = &mut value; let second = access; *second = true; break; } return value; }");
accept!(local_alias_cleanup, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; let alias = view; let seen = *alias; break; } return value; }");
accept!(primary_skipped_suffix, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; if stop { value = true; break; } let seen = *view; } return value; }");
accept!(skipped_suffix_before_borrow, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; if stop { let access = &mut value; *access = true; break; } let seen = *view; } return value; }");
accept!(nested_skipped_suffix, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; if stop { if flag { value = true; break; } } let seen = *view; } return value; }");
accept!(local_exclusive_skipped_suffix, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let access = &mut value; if stop { let seen = value; break; } *access = true; } return value; }");
accept!(positive_removed_recurrence, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { value = true; break; } } return value; }");
accept!(recurrent_only_expiry, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { break; } let seen = *view; } value = true; return value; }");
accept!(recurrent_all_break, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { let seen = *view; break; } else { break; } } value = true; return value; }");
accept!(mutable_recurrence_ends, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let access = &mut value; while flag { if stop { break; } *access = true; } let view = &value; let seen = *view; return value; }");
accept!(carried_shared_post_use, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { if stop { break; } let seen = *view; } let later = *view; return value; }");
accept!(carried_mutable_post_use, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let access = &mut value; while flag { if stop { break; } *access = true; } *access = false; return value; }");
accept!(outside_only_shared, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { break; } let seen = *view; return value; }");
accept!(outside_only_mutable, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let access = &mut value; while flag { break; } *access = true; return value; }");
accept!(nested_inner_break, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { while stop { break; } value = true; } return value; }");
accept!(inner_break_outer_continue, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { while stop { break; } continue; } return value; }");
accept!(inner_break_outer_break, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { while stop { break; } break; } return value; }");
accept!(outer_break_after_inner_false_exit, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { while stop { let seen = value; } break; } return value; }");
accept!(outer_local_inner_recurrence, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; while stop { if flag { break; } let seen = *view; } value = true; break; } return value; }");
accept!(outer_recurrence_survives_inner, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while flag { while stop { let seen = *view; break; } let after = *view; break; } value = true; return value; }");
accept!(outer_empty_inner_empty_frames, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { while stop { break; } break; } return value; }");
accept!(dead_lexical_mutable, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let unused = &mut value; while flag { value = true; break; } return value; }");
accept!(constant_false, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let view = &value; while false { let seen = *view; break; } value = true; return value; }");
accept!(constant_true, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; let access = &mut value; while true { *access = true; break; } value = true; return value; }");
accept!(call_hold_completion, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let view = &value; let seen = both(view, read(view)); break; } return value; }");
accept!(write_hold_completion, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = *access; break; } return value; }");
accept!(sibling_spelling, "fn f(flag: bool, stop: bool) -> bool { let mut value = false; while flag { if stop { let observed = value; break; } else { let observed = value; } } return value; }");
reject!(
    outside,
    "fn f(flag: bool) -> bool { break; return flag; }",
    "Parse",
    "only inside"
);
reject!(
    outside_if,
    "fn f(flag: bool) -> bool { if flag { break; } return flag; }",
    "Parse",
    "only inside"
);
reject!(
    missing_semicolon,
    "fn f(flag: bool) -> bool { while flag { break } return flag; }",
    "Parse",
    ""
);
reject!(
    with_value,
    "fn f(flag: bool) -> bool { while flag { break flag; } return flag; }",
    "Parse",
    ""
);
reject!(
    with_label,
    "fn f(flag: bool) -> bool { while flag { break outer; } return flag; }",
    "Parse",
    ""
);
reject!(
    direct_suffix,
    "fn f(flag: bool) -> bool { while flag { break; let seen = flag; } return flag; }",
    "Parse",
    "after break"
);
reject!(
    branch_direct_suffix,
    "fn f(flag: bool) -> bool { while flag { if flag { break; let seen = flag; } } return flag; }",
    "Parse",
    "after break"
);
reject!(all_break_suffix, "fn f(flag: bool) -> bool { while flag { if flag { break; } else { break; } let seen = flag; } return flag; }", "Parse", "no fallthrough");
reject!(mixed_suffix, "fn f(flag: bool) -> bool { while flag { if flag { break; } else { continue; } let seen = flag; } return flag; }", "Parse", "no fallthrough");
reject!(nested_no_fallthrough_suffix, "fn f(flag: bool) -> bool { while flag { if flag { if flag { break; } else { continue; } } else { break; } let seen = flag; } return flag; }", "Parse", "no fallthrough");
reject!(
    initializer_position,
    "fn f(flag: bool) -> bool { while flag { let seen = break; } return flag; }",
    "Parse",
    ""
);
reject!(
    call_position,
    "fn f(flag: bool) -> bool { while flag { let seen = consume(break); } return flag; }",
    "Parse",
    ""
);
reject!(
    condition_position,
    "fn f(flag: bool) -> bool { while break { } return flag; }",
    "Parse",
    ""
);
reject!(
    return_position,
    "fn f(flag: bool) -> bool { return break; return flag; }",
    "Parse",
    ""
);
reject!(branch_scope, "fn f(flag: bool) -> bool { while flag { if flag { let inside = flag; break; } let seen = inside; } return flag; }", "Resolve", "unknown");
reject!(loop_scope, "fn f(flag: bool) -> bool { while flag { let inside = flag; break; } let seen = inside; return flag; }", "Resolve", "unknown");
reject!(recurrent_shared_witness, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { if flag { value = true; break; } let seen = *view; } return flag; }", "Ownership", "cannot assign");
reject!(recurrent_exclusive_witness, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { if flag { let other = &value; break; } *access = true; } return flag; }", "Ownership", "exclusive borrow");
reject!(outside_only_shared_protection, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { value = true; break; } let seen = *view; return flag; }", "Ownership", "cannot assign");
reject!(outside_only_exclusive_protection, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { let seen = value; break; } let later = *access; return flag; }", "Ownership", "exclusively borrowed");
reject!(empty_inner_preserves_outer_frame, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { let seen = *view; while flag { break; } value = true; } return flag; }", "Ownership", "cannot assign");
reject!(same_loan_nested_outer_frame, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while flag { let seen = *view; while flag { let inner = *view; break; } value = true; } return flag; }", "Ownership", "cannot assign");
reject!(carried_record_move, "fn f(flag: bool) -> bool { let ticket = fresh(); while flag { let moved = ticket; break; } return flag; }", "Ownership", "pre-existing non-Copy");
reject!(carried_record_argument, "fn f(flag: bool) -> bool { let ticket = fresh(); while flag { let done = consume(ticket); break; } return flag; }", "Ownership", "pre-existing non-Copy");
reject!(carried_mutable_move, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { let moved = access; break; } return flag; }", "Ownership", "loop-carried mutable");
reject!(carried_mutable_argument, "fn f(flag: bool) -> bool { let mut value = false; let access = &mut value; while flag { let done = sink(access); break; } return flag; }", "Ownership", "loop-carried mutable");
reject!(inner_move_outer_local, "fn f(flag: bool) -> bool { while flag { let ticket = fresh(); while flag { let moved = ticket; break; } } return flag; }", "Ownership", "pre-existing non-Copy");
reject!(fallthrough_survivor_effect, "fn f(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { break; } else { let moved = ticket; } let done = consume(ticket); } return flag; }", "Ownership", "use of moved value");
reject!(fallthrough_conditional_move, "fn f(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { let moved = ticket; } let done = consume(ticket); } return flag; }", "Ownership", "may have been moved");
reject!(call_hold, "fn f(flag: bool) -> bool { let mut value = false; while flag { let done = both(&value, sink(&mut value)); break; } return flag; }", "Ownership", "shared borrow");
reject!(write_hold, "fn f(flag: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = read(&value); break; } return flag; }", "Ownership", "exclusive borrow");
reject!(write_rhs_transfer, "fn f(flag: bool) -> bool { let mut value = false; while flag { let access = &mut value; *access = sink(access); break; } return flag; }", "Ownership", "use of moved value");
reject!(constant_recurrence, "fn f(flag: bool) -> bool { let mut value = false; let view = &value; while false { if flag { value = true; break; } let seen = *view; } return flag; }", "Ownership", "cannot assign");
reject!(
    reference_condition,
    "fn f(flag: bool) -> bool { let view = &flag; while *view { break; } return flag; }",
    "Ownership",
    "while conditions"
);
reject!(noncopy_replacement, "fn f(flag: bool) -> bool { let first = fresh(); while flag { let mut second = fresh(); second = first; break; } return flag; }", "Ownership", "pre-existing non-Copy");

#[test]
fn terminal_move_diagnostic_describes_the_retained_boundary() {
    let source = format!("{DECL} fn f(flag: bool, ticket: Ticket) -> bool {{ while flag {{ let moved = ticket; break; }} return flag; }}");
    let FrontendError::Ownership(errors) = compile_source(&source).unwrap_err() else {
        panic!()
    };
    let text = errors[0].render("test.bow", &source);
    assert!(text.contains("terminal break paths do not relax"));
    assert!(text.contains("canonical exits"));
    assert!(!text.contains("transfer would change pre-existing ownership state across a backedge"));
}
#[test]
fn source_and_canonical_hir_keep_break_span() {
    let source = "fn f(flag: bool) -> bool { while flag { break; } return flag; }";
    let ast = parse_source(source).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::While { body, .. } = &f.body[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::Break { span } = body[0] else {
        panic!()
    };
    assert_eq!(&source[span.start..span.end], "break;");
    let p = compile_source(source).unwrap();
    let hir::ValueStatement::While { body, .. } = &p.functions()[0].as_ordinary().unwrap().body[0]
    else {
        panic!()
    };
    assert!(matches!(body[0], hir::ValueStatement::Break { span: s } if s == span));
}
#[test]
fn mir_terminal_edges_have_correct_destinations_without_synthetic_join() {
    for (yes, no, expected_blocks) in [
        ("break;", "break;", 6),
        ("break;", "continue;", 6),
        ("continue;", "break;", 6),
    ] {
        let p = compile_source(&format!("fn f(flag: bool) -> bool {{ while flag {{ if flag {{ {yes} }} else {{ {no} }} }} return flag; }}")).unwrap();
        let m = mir::lower(&p);
        let f = &m.functions()[0];
        f.validate().unwrap();
        assert_eq!(f.blocks().len(), expected_blocks);
        let mir::Terminator::Goto { target: header } = f.block(f.entry()).terminator().clone()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target: dispatch,
            else_target: exit,
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
            &mir::Terminator::Goto {
                target: if yes == "break;" { exit } else { header }
            }
        );
        assert_eq!(
            f.block(else_target).terminator(),
            &mir::Terminator::Goto {
                target: if no == "break;" { exit } else { header }
            }
        );
        assert!(matches!(
            f.block(exit).terminator(),
            mir::Terminator::Return { .. }
        ));
    }
}
#[test]
fn nested_mir_targets_restore_outer_header_and_exit() {
    for outer_statement in ["break;", "continue;"] {
        let p = compile_source(&format!("fn f(flag: bool) -> bool {{ while flag {{ while flag {{ break; }} {outer_statement} }} return flag; }}")).unwrap();
        let m = mir::lower(&p);
        let f = &m.functions()[0];
        f.validate().unwrap();
        let mir::Terminator::Goto { target: outer } = f.block(f.entry()).terminator().clone()
        else {
            panic!()
        };
        let mir::Terminator::Branch {
            then_target: outer_body,
            else_target: outer_exit,
            ..
        } = f.block(outer).terminator().clone()
        else {
            panic!()
        };
        let mir::Terminator::Goto { target: inner } = f.block(outer_body).terminator().clone()
        else {
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
            f.block(inner_body).terminator(),
            &mir::Terminator::Goto { target: inner_exit }
        );
        assert_eq!(
            f.block(inner_exit).terminator(),
            &mir::Terminator::Goto {
                target: if outer_statement == "break;" {
                    outer_exit
                } else {
                    outer
                }
            }
        );
    }
}
#[test]
fn public_ast_resolution_rejects_bad_break_placement_and_suffix() {
    let mut ast =
        parse_source("fn f(flag: bool) -> bool { while flag { break; } return flag; }").unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &mut ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::While { body, .. } = &f.body[0] else {
        panic!()
    };
    let statement = body[0].clone();
    f.body.insert(0, statement);
    assert!(bowfin_compiler::resolve::resolve(&ast).unwrap_err()[0]
        .message
        .contains("only inside"));
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &mut ast.functions[0] else {
        panic!()
    };
    f.body.remove(0);
    let suffix = f.body[1].clone();
    let bowfin_compiler::ast::ValueStatement::While { body, .. } = &mut f.body[0] else {
        panic!()
    };
    body.push(suffix);
    assert!(bowfin_compiler::resolve::resolve(&ast).unwrap_err()[0]
        .message
        .contains("no fallthrough"));
}
