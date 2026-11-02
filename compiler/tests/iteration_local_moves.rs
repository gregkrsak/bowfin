use bowfin_compiler::{
    compile_source,
    hir::{self, ExprKind, ValueStatement},
    mir::{self, Statement, Terminator},
    FrontendError,
};

// Recursive factories exercise fresh call-result typing without adding ordinary
// record construction, execution, or a termination claim.
const DECLARATIONS: &str = "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent } record Different { value: Percent } fn fresh() -> Ticket { return fresh(); } fn different() -> Different { return different(); } fn identity(value: Ticket) -> Ticket { return value; } fn consume(value: Ticket) -> bool { return true; } fn consume_range(value: Ticket, amount: Percent) -> Percent { return amount; } fn pair(first: Ticket, second: Ticket) -> bool { return true; } fn inspect(view: &Ticket) -> bool { return true; }";
fn source(body: &str) -> String {
    format!("{DECLARATIONS} {body}")
}
fn good(body: &str) -> hir::Program {
    let p = compile_source(&source(body)).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    p
}
fn bad(body: &str, phase: &str, wording: &str) {
    let text = source(body);
    let error = compile_source(&text).unwrap_err();
    let (actual, diagnostic) = match error {
        FrontendError::Ownership(ds) => ("Ownership", ds[0].render("case.bow", &text)),
        FrontendError::Type(ds) => ("Type", ds[0].render("case.bow", &text)),
        FrontendError::Resolve(ds) => ("Resolve", ds[0].render("case.bow", &text)),
        FrontendError::Parse(e) => ("Parse", e.to_string()),
        other => panic!("unexpected error: {other}"),
    };
    assert_eq!(actual, phase, "{diagnostic}");
    assert!(diagnostic.contains(wording), "{diagnostic}");
    for id in ["LocalId", "ParameterId", "LoanId", "BasicBlockId"] {
        assert!(!diagnostic.contains(id));
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
    fresh_record_bound_and_scoped,
    "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); } return flag; }"
);
accept!(multiple_iteration_local_transfers, "fn run(flag: bool) -> bool { let mut running = flag; while running { let first = fresh(); let second = first; let third = second; let done = consume(third); running = false; } return running; }");
accept!(
    consume_fresh_result_directly,
    "fn run(flag: bool) -> bool { while flag { let done = consume(fresh()); } return flag; }"
);
accept!(nested_calls_forward_local_ownership, "fn run(flag: bool) -> bool { while flag { let first = fresh(); let forwarded = identity(identity(first)); let done = consume(forwarded); } return flag; }");
accept!(nested_fresh_calls, "fn run(flag: bool) -> bool { while flag { let done = pair(identity(fresh()), fresh()); } return flag; }");
accept!(consumer_copy_result_can_update_loop_state, "fn run(flag: bool, amount: Percent) -> Percent { let mut current = amount; let mut running = flag; while running { let ticket = fresh(); current = consume_range(ticket, current); running = false; } return current; }");
accept!(iteration_local_move_in_then, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { let done = consume(ticket); } } return flag; }");
accept!(mutually_exclusive_moves, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { let done = consume(ticket); } else { let done = consume(ticket); } } return flag; }");
accept!(nested_branch_ownership_is_confined, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { if flag { let moved = ticket; let done = consume(moved); } } else { let done = consume(ticket); } } return flag; }");
accept!(sibling_branch_fresh_owners, "fn run(flag: bool) -> bool { while flag { if flag { let ticket = fresh(); let done = consume(ticket); } else { let ticket = fresh(); let done = consume(ticket); } } return flag; }");
accept!(fresh_conditional_record_initializer, "fn run(flag: bool) -> bool { while flag { let chosen = if flag { fresh() } else { identity(fresh()) }; let done = consume(chosen); } return flag; }");
accept!(local_sources_in_conditional_initializer, "fn run(flag: bool) -> bool { while flag { let first = fresh(); let second = fresh(); let chosen = if flag { first } else { second }; let done = consume(chosen); } return flag; }");
accept!(same_local_source_in_both_value_branches, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let chosen = if flag { ticket } else { ticket }; let done = consume(chosen); } return flag; }");
accept!(nested_loops_independently_fresh, "fn run(flag: bool) -> bool { while flag { let outer = fresh(); while flag { let inner = fresh(); let done = consume(inner); } let done = consume(outer); } return flag; }");
accept!(three_nested_boundaries, "fn run(flag: bool) -> bool { while flag { let first = fresh(); while flag { let second = fresh(); while flag { let third = fresh(); let done = consume(third); } let done = consume(second); } let done = consume(first); } return flag; }");
accept!(new_local_after_inner_loop, "fn run(flag: bool) -> bool { while flag { while flag {} let ticket = fresh(); let done = consume(ticket); } return flag; }");
accept!(untouched_outer_parameter, "fn run(flag: bool, ticket: Ticket) -> Ticket { while flag { let local = fresh(); let done = consume(local); } return ticket; }");
accept!(untouched_outer_local, "fn run(flag: bool) -> Ticket { let ticket = fresh(); while flag { let local = fresh(); let done = consume(local); } return ticket; }");
accept!(untouched_moved_outer_place, "fn run(flag: bool, ticket: Ticket) -> Ticket { let forwarded = ticket; while flag { let local = fresh(); let done = consume(local); } return forwarded; }");
accept!(outside_record_loan_unchanged, "fn run(flag: bool, ticket: Ticket) -> bool { let view = &ticket; while flag { let local = fresh(); let done = consume(local); } return inspect(view); }");
accept!(sibling_loops_have_distinct_static_locals, "fn run(flag: bool) -> bool { if flag { while flag { let ticket = fresh(); let done = consume(ticket); } } else { while flag { let ticket = fresh(); let done = consume(ticket); } } return flag; }");
accept!(constant_false_still_classifies_iteration_local, "fn run() -> bool { while false { let ticket = fresh(); let done = consume(ticket); } return true; }");
accept!(constant_true_still_classifies_iteration_local, "fn run() -> bool { while true { let ticket = fresh(); let done = consume(ticket); } return true; }");

reject!(carried_parameter_to_binding, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { let moved = ticket; } return true; }", "Ownership", "pre-existing non-Copy value `ticket`");
reject!(carried_local_to_call, "fn run(flag: bool) -> bool { let ticket = fresh(); while flag { let done = consume(ticket); } return true; }", "Ownership", "pre-existing non-Copy value `ticket`");
reject!(carried_parameter_to_call, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { let done = consume(ticket); } return true; }", "Ownership", "pre-existing-state boundary");
reject!(carried_in_one_nested_branch, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { if flag { let done = consume(ticket); } } return true; }", "Ownership", "pre-existing non-Copy");
reject!(carried_in_both_nested_branches, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { if flag { let done = consume(ticket); } else { let done = consume(ticket); } } return true; }", "Ownership", "pre-existing non-Copy");
reject!(carried_in_nested_loop, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { while flag { let done = consume(ticket); } } return true; }", "Ownership", "pre-existing non-Copy");
reject!(outer_iteration_local_is_inner_carried, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); while flag { let done = consume(ticket); } } return true; }", "Ownership", "pre-existing non-Copy value `ticket`");
reject!(outer_branch_local_is_inner_carried, "fn run(flag: bool) -> bool { while flag { if flag { let ticket = fresh(); while flag { let moved = ticket; } } } return true; }", "Ownership", "pre-existing non-Copy");
reject!(outer_iteration_conditional_destination_is_inner_carried, "fn run(flag: bool) -> bool { while flag { let ticket = if flag { fresh() } else { fresh() }; while flag { let done = consume(ticket); } } return true; }", "Ownership", "pre-existing non-Copy");
reject!(iteration_local_use_after_move, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let moved = ticket; let done = consume(ticket); } return true; }", "Ownership", "use of moved value `ticket`");
reject!(duplicate_argument_move, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let done = pair(ticket, ticket); } return true; }", "Ownership", "use of moved value");
reject!(joined_iteration_local_use, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { let done = consume(ticket); } let later = consume(ticket); } return true; }", "Ownership", "may have been moved");
reject!(both_branch_moves_then_joined_use, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); if flag { let done = consume(ticket); } else { let done = consume(ticket); } let later = consume(ticket); } return true; }", "Ownership", "may have been moved");
reject!(conditional_source_cannot_be_reused, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let chosen = if flag { ticket } else { ticket }; let done = consume(ticket); } return true; }", "Ownership", "may have been moved");
accept!(iteration_local_replacement, "fn run(flag: bool) -> bool { while flag { let mut ticket = fresh(); ticket = fresh(); } return true; }");
reject!(pre_existing_replacement, "fn run(flag: bool) -> bool { let mut ticket = fresh(); while flag { ticket = fresh(); } return true; }", "Ownership", "cannot replace pre-existing non-Copy");
reject!(
    iteration_local_escapes,
    "fn run(flag: bool) -> Ticket { while flag { let ticket = fresh(); } return ticket; }",
    "Resolve",
    "unknown value"
);
reject!(branch_local_escapes_to_body, "fn run(flag: bool) -> bool { while flag { if flag { let ticket = fresh(); } let done = consume(ticket); } return true; }", "Resolve", "unknown value");
reject!(ordered_locals_unchanged, "fn run(flag: bool) -> bool { while flag { let first = later; let later = fresh(); } return true; }", "Resolve", "unknown value");
reject!(visible_shadowing_unchanged, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { let ticket = fresh(); } return true; }", "Resolve", "duplicate");
reject!(exact_record_identity_unchanged, "fn run(flag: bool) -> bool { while flag { let ticket = different(); let done = consume(ticket); } return true; }", "Type", "expected record `Ticket`, found record `Different`");
reject!(
    non_copy_condition_still_forbidden,
    "fn run(ticket: Ticket) -> bool { while consume(ticket) {} return true; }",
    "Ownership",
    "unsupported in while conditions"
);
reject!(
    fresh_chain_in_condition_still_forbidden,
    "fn run() -> bool { while consume(identity(fresh())) {} return true; }",
    "Ownership",
    "unsupported in while conditions"
);
reject!(
    fresh_chain_in_nested_condition_still_forbidden,
    "fn run(flag: bool) -> bool { while flag { while consume(fresh()) {} } return true; }",
    "Ownership",
    "unsupported in while conditions"
);
reject!(
    constant_false_does_not_allow_carried_move,
    "fn run(ticket: Ticket) -> bool { while false { let moved = ticket; } return true; }",
    "Ownership",
    "pre-existing non-Copy"
);
reject!(
    constant_true_does_not_allow_carried_move,
    "fn run(ticket: Ticket) -> bool { while true { let moved = ticket; } return true; }",
    "Ownership",
    "pre-existing non-Copy"
);
reject!(constant_value_branch_does_not_hide_carried_move, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { let chosen = if true { fresh() } else { ticket }; } return true; }", "Ownership", "pre-existing non-Copy");
accept!(borrow_of_iteration_local, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let view = &ticket; } return true; }");
accept!(direct_iteration_local_borrow_argument, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let done = inspect(&ticket); } return true; }");

#[test]
fn cyclic_mir_keeps_static_move_locals_and_call_ids_without_extra_blocks() {
    let p = good("fn run(flag: bool) -> bool { while flag { let first = fresh(); let second = identity(first); let done = consume(second); } return flag; }");
    let mir = mir::lower(&p);
    assert_eq!(mir, mir::lower(&p));
    let f = mir.functions().last().unwrap();
    assert_eq!(f.blocks().len(), 4); // Same preheader/header/body/exit as Copy-only loops.
    let ValueStatement::While { body, .. } =
        &p.functions().last().unwrap().as_ordinary().unwrap().body[0]
    else {
        panic!()
    };
    let locals: Vec<_> = body
        .iter()
        .map(|s| match s {
            ValueStatement::Let { local, .. } => *local,
            _ => panic!(),
        })
        .collect();
    assert_ne!(locals[0], locals[1]);
    let lets: Vec<_> = f.blocks().flat_map(|(_, b)| b.statements()).collect();
    assert_eq!(lets.len(), 3);
    for (s, expected) in lets.iter().zip(&locals) {
        let Statement::Let {
            local, initializer, ..
        } = s
        else {
            panic!()
        };
        assert_eq!(local, expected);
        let ExprKind::Call { function, .. } = initializer.kind else {
            panic!()
        };
        assert!(p.function(function).as_ordinary().is_some());
    }
    let (header, b) = f
        .blocks()
        .find(|(_, b)| matches!(b.terminator(), Terminator::Branch { .. }))
        .unwrap();
    let Terminator::Branch {
        then_target,
        else_target,
        ..
    } = b.terminator()
    else {
        panic!()
    };
    assert_eq!(
        f.block(*then_target).terminator(),
        &Terminator::Goto { target: header }
    );
    assert!(matches!(
        f.block(*else_target).terminator(),
        Terminator::Return { .. }
    ));
}

accept!(statement_if_condition_can_consume_iteration_local, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); if consume(ticket) { let next = fresh(); let done = consume(next); } } return flag; }");
reject!(while_condition_cannot_consume_outer_iteration_local, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); while consume(ticket) {} } return flag; }", "Ownership", "unsupported in while conditions");
