use bowfin_compiler::{
    compile_source,
    hir::{self, ExprKind, ValueStatement},
    mir, FrontendError,
};
const DECLARATIONS: &str = "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent } record Different { value: Percent } fn fresh() -> Ticket { return fresh(); } fn consume(ticket: Ticket) -> bool { return true; } fn inspect(view: &Ticket) -> bool { return true; } fn inspect_mut(view: &mut Ticket) -> bool { return true; } fn read(view: &Percent) -> Percent { return *view; } fn read_bool(view: &bool) -> bool { return *view; } fn sink(access: &mut Percent) -> bool { return true; } fn write(access: &mut Percent, replacement: Percent) -> bool { *access = replacement; return true; } fn shared_pair(first: &Percent, second: &Percent) -> bool { return true; } fn shared_call(first: &Percent, second: bool) -> bool { return second; } fn exclusive_call(first: &mut Percent, second: bool) -> bool { return second; } fn owner(value: Percent) -> bool { return true; } fn identity(value: Percent) -> Percent { return value; } fn consume_access(access: &mut Percent, value: Percent) -> Percent { return value; }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECLARATIONS} {body}")).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    p
}
fn bad(body: &str, phase: &str, wording: &str) {
    let text = format!("{DECLARATIONS} {body}");
    let e = compile_source(&text).unwrap_err();
    let (actual, diagnostic) = match &e {
        FrontendError::Ownership(ds) => ("Ownership", ds[0].render("case.bow", &text)),
        FrontendError::Type(ds) => ("Type", ds[0].render("case.bow", &text)),
        FrontendError::Resolve(ds) => ("Resolve", ds[0].render("case.bow", &text)),
        FrontendError::Parse(_) => ("Parse", e.to_string()),
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
accept!(shared_owner_borrow_then_assignment, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; let mut running = flag; while running { let view = &current; let observed = *view; current = observed; running = false; } return current; }");
accept!(shared_pre_existing_record_borrow, "fn run(flag: bool, ticket: Ticket) -> Ticket { while flag { let view = &ticket; let checked = inspect(view); } return ticket; }");
accept!(iteration_record_borrow_then_move, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let view = &ticket; let checked = inspect(view); let done = consume(ticket); } return flag; }");
accept!(direct_shared_borrow_argument, "fn run(flag: bool, value: Percent) -> Percent { while flag { let observed = read(&value); } return value; }");
accept!(direct_mutable_borrow_argument, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let done = write(&mut current, value); let observed = current; } return current; }");
accept!(stored_shared_reference_call, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; let observed = read(view); } return value; }");
accept!(overlapping_shared_loans, "fn run(flag: bool, value: Percent) -> Percent { while flag { let first = &value; let second = &value; let done = shared_pair(first, second); } return value; }");
accept!(copied_shared_alias_extends_provenance, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; let alias = view; let before = *view; let after = *alias; current = after; } return current; }");
accept!(mutable_write_read_recovery, "fn run(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; *access = replacement; let observed = *access; current = observed; } return current; }");
accept!(mutable_handle_transfer, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let first = &mut current; let second = first; *second = value; let observed = *second; current = observed; } return current; }");
accept!(mutable_handle_consumed_by_call, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; let done = sink(access); let observed = current; } return current; }");
accept!(nested_calls_compatible_shared_holds, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; let done = shared_call(view, owner(read(&value))); } return value; }");
accept!(copy_deref_argument_does_not_call_hold_reference, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; let done = owner(*access); let view = &current; let observed = *view; } return current; }");
accept!(write_hold_allows_same_handle_rhs_read, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; *access = *access; *access = identity(*access); } return current; }");
accept!(branch_created_loans_do_not_escape, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { if flag { let access = &mut current; *access = value; } else { let view = &current; let observed = *view; } current = value; } return current; }");
accept!(if_condition_direct_iteration_borrow, "fn run(flag: bool, ticket: Ticket) -> Ticket { while flag { if inspect(&ticket) { let another = &ticket; let done = inspect(another); } } return ticket; }");
accept!(if_condition_stored_iteration_reference, "fn run(flag: bool, ticket: Ticket) -> Ticket { while flag { let view = &ticket; if inspect(view) {} } return ticket; }");
accept!(path_sensitive_shared_then_exclusive_else, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; if flag { let observed = *view; } else { let access = &mut current; *access = value; } } return current; }");
accept!(same_branch_final_shared_use_then_exclusive, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; if flag { let observed = *view; let access = &mut current; *access = value; } } return current; }");
accept!(conditional_copy_initializer_deref, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; let selected = if flag { *view } else { value }; } return value; }");
accept!(nested_loop_fresh_inner_borrow, "fn run(flag: bool, value: Percent) -> Percent { while flag { while flag { let view = &value; let observed = *view; } } return value; }");
accept!(outer_reference_untouched_through_inner_loop, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; while flag {} let observed = *view; } return value; }");
accept!(outer_shared_loan_and_compatible_inner_borrow, "fn run(flag: bool, value: Percent) -> Percent { while flag { let outer = &value; while flag { let inner = &value; let observed = *inner; } let later = *outer; } return value; }");
accept!(nested_mutable_loans_after_outer_last_use, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let outer = &mut current; let observed = *outer; while flag { let inner = &mut current; *inner = value; } } return current; }");
accept!(live_header_shared_loan_allows_compatible_body_loan, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; let outer = &current; while flag { let inner = &current; let observed = *inner; } return *outer; }");
accept!(live_header_exclusive_loan_of_unrelated_owner, "fn run(flag: bool, value: Percent) -> Percent { let mut first = value; let mut second = value; let outer = &mut first; while flag { let inner = &mut second; *inner = value; } return *outer; }");
accept!(dead_header_loan_stays_dead, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; let prior = &current; let observed = *prior; while flag { let access = &mut current; *access = value; } return current; }");
accept!(post_loop_borrow_after_iteration_cleanup, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; let alias = view; let observed = *alias; } let access = &mut current; *access = value; return current; }");
accept!(conditional_mutable_move_no_later_handle_use, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; if flag { let done = sink(access); } let observed = current; } return current; }");
accept!(mutable_iteration_record_borrow_then_move, "fn run(flag: bool) -> bool { while flag { let mut ticket = fresh(); let access = &mut ticket; let checked = inspect_mut(access); let done = consume(ticket); } return flag; }");
accept!(constant_false_body_borrow_checked_normally, "fn run(value: Percent) -> Percent { while false { let view = &value; let observed = *view; } return value; }");
accept!(constant_true_body_borrow_checked_normally, "fn run(value: Percent) -> Percent { while true { let view = &value; let observed = *view; } return value; }");

accept!(header_reference_parameter_deref, "fn run(flag: bool, view: &Percent) -> Percent { while flag { let observed = *view; } return *view; }");
accept!(header_reference_local_deref, "fn run(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let observed = *view; } return value; }");
accept!(header_shared_call_argument, "fn run(flag: bool, view: &Percent) -> Percent { while flag { let observed = read(view); } return *view; }");
accept!(header_shared_alias, "fn run(flag: bool, view: &Percent) -> Percent { while flag { let alias = view; } return *view; }");
reject!(header_mutable_transfer, "fn run(flag: bool, access: &mut Percent) -> Percent { while flag { let next = access; } return *access; }", "Ownership", "loop-carried mutable reference");
reject!(header_mutable_call_argument, "fn run(flag: bool, access: &mut Percent, value: Percent) -> Percent { while flag { let done = sink(access); } return value; }", "Ownership", "loop-carried mutable reference");
accept!(header_mutable_write_through, "fn run(flag: bool, access: &mut Percent, value: Percent) -> Percent { while flag { *access = value; } return value; }");
reject!(
    while_condition_direct_borrow,
    "fn run(ticket: Ticket) -> bool { while inspect(&ticket) {} return true; }",
    "Ownership",
    "reference activity is unsupported in while conditions"
);
reject!(
    while_condition_handle,
    "fn run(view: &Ticket) -> bool { while inspect(view) {} return true; }",
    "Ownership",
    "reference activity is unsupported in while conditions"
);
reject!(
    while_condition_deref,
    "fn run(view: &bool) -> bool { while *view {} return true; }",
    "Ownership",
    "reference activity is unsupported in while conditions"
);
accept!(inner_loop_uses_outer_created_reference, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; while flag { let observed = *view; } } return value; }");
reject!(inner_loop_transfers_outer_mutable_reference, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; while flag { let next = access; } } return current; }", "Ownership", "loop-carried mutable reference `access`");
reject!(inner_condition_outer_handle_rejected_as_condition, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; while owner(*view) {} } return value; }", "Ownership", "unsupported in while conditions");
reject!(live_shared_header_loan_blocks_new_mutable_borrow, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; let view = &current; while flag { let access = &mut current; } return *view; }", "Ownership", "while shared borrow is active");
reject!(live_exclusive_header_loan_blocks_new_shared_borrow, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; let access = &mut current; while flag { let view = &current; } return *access; }", "Ownership", "while exclusive borrow is active");
reject!(live_exclusive_header_loan_blocks_new_mutable_borrow, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; let access = &mut current; while flag { let next = &mut current; } return *access; }", "Ownership", "second exclusive borrow");
reject!(iteration_record_move_while_borrowed, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let view = &ticket; let done = consume(ticket); let checked = inspect(view); } return flag; }", "Ownership", "shared-borrowed");
reject!(new_borrow_does_not_allow_carried_record_move, "fn run(flag: bool, ticket: Ticket) -> bool { while flag { let view = &ticket; let checked = inspect(view); let done = consume(ticket); } return flag; }", "Ownership", "pre-existing non-Copy value");
reject!(non_copy_deref_still_rejected, "fn run(flag: bool) -> bool { while flag { let ticket = fresh(); let view = &ticket; let moved = *view; } return flag; }", "Ownership", "cannot move non-Copy value");
reject!(non_copy_write_through_still_rejected, "fn run(flag: bool) -> bool { while flag { let mut ticket = fresh(); let access = &mut ticket; *access = fresh(); } return flag; }", "Ownership", "cannot replace non-Copy value");
reject!(call_hold_shared_blocks_nested_exclusive, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; let done = shared_call(view, sink(&mut current)); } return current; }", "Ownership", "while shared borrow is active");
reject!(call_hold_direct_shared_blocks_nested_exclusive, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let done = shared_call(&current, sink(&mut current)); } return current; }", "Ownership", "while shared borrow is active");
reject!(call_hold_direct_mutable_blocks_nested_owner_access, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let done = exclusive_call(&mut current, owner(current)); } return current; }", "Ownership", "exclusively borrowed");
reject!(call_hold_stored_mutable_blocks_nested_owner_access, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; let done = exclusive_call(access, owner(current)); } return current; }", "Ownership", "exclusively borrowed");
reject!(write_hold_blocks_rhs_owner_read, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; *access = current; } return current; }", "Ownership", "exclusively borrowed");
reject!(write_hold_blocks_rhs_borrow, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; *access = read(&current); } return current; }", "Ownership", "while exclusive borrow is active");
reject!(write_hold_cannot_transfer_target_handle, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; *access = consume_access(access, value); } return current; }", "Ownership", "use of moved value");
reject!(moved_mutable_handle_not_restored, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let first = &mut current; let second = first; let observed = *first; } return current; }", "Ownership", "use of moved value");
reject!(conditionally_moved_mutable_handle_unavailable, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; if flag { let done = sink(access); } let observed = *access; } return current; }", "Ownership", "may have been moved");
reject!(post_join_use_keeps_body_loan_live_on_sibling, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; if flag { let observed = *view; } else { let access = &mut current; } let later = *view; } return current; }", "Ownership", "while shared borrow is active");
reject!(same_path_overlap_not_relaxed, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; let access = &mut current; let observed = *view; } return current; }", "Ownership", "while shared borrow is active");
reject!(reference_local_cannot_escape_body, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; } return *view; }", "Resolve", "unknown value");
reject!(
    reference_return_remains_type_error,
    "fn run(value: Percent) -> &Percent { while false {} return &value; }",
    "Type",
    "reference return"
);
reject!(reference_conditional_results_remain_type_error, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = if flag { &value } else { &value }; } return value; }", "Type", "reference");
reject!(reference_reassignment_remains_type_error, "fn run(flag: bool, value: Percent) -> Percent { while flag { let mut view = &value; view = &value; } return value; }", "Type", "reference");
reject!(reborrowing_not_added, "fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; let next = &view; } return value; }", "Type", "reference");
reject!(immutable_mutable_borrow_rule_preserved, "fn run(flag: bool, value: Percent) -> Percent { while flag { let access = &mut value; } return value; }", "Ownership", "immutable value");
reject!(nominal_referent_identity_preserved, "fn run(flag: bool, value: Other) -> Other { while flag { let observed = read(&value); } return value; }", "Type", "expected &Percent, found &Other");
reject!(mutable_to_shared_coercion_not_added, "fn run(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let access = &mut current; let observed = read(access); } return current; }", "Type", "argument type mismatch");
accept!(
    constant_false_checks_stable_header_handle_use,
    "fn run(view: &Percent) -> bool { while false { let observed = *view; } return true; }"
);
accept!(
    constant_true_checks_stable_header_handle_use,
    "fn run(view: &Percent) -> bool { while true { let observed = *view; } return true; }"
);

#[test]
fn borrowing_does_not_add_mir_blocks_or_change_loop_identity() {
    let p = good("fn run(flag: bool, value: Percent) -> Percent { while flag { let view = &value; let observed = *view; } return value; }");
    let m = mir::lower(&p);
    assert_eq!(m, mir::lower(&p));
    let f = m.functions().last().unwrap();
    assert_eq!(f.blocks().len(), 4);
    let function = p.functions().last().unwrap().as_ordinary().unwrap();
    assert_eq!(f.function(), function.id);
    let ValueStatement::While { body, .. } = &function.body[0] else {
        panic!()
    };
    let ValueStatement::Let {
        local, initializer, ..
    } = &body[0]
    else {
        panic!()
    };
    let ExprKind::Borrow { place, .. } = initializer.kind else {
        panic!()
    };
    assert_eq!(place, hir::Place::Parameter(function.parameters[1]));
    assert!(f.blocks().flat_map(|(_, b)| b.statements()).any(|s| matches!(s, mir::Statement::Let { local: l, initializer: e, .. } if l == local && e == initializer)));
}
