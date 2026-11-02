use bowfin_compiler::{compile_source, hir, mir, FrontendError};
const DECLARATIONS: &str = "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent } fn fresh() -> Ticket { return fresh(); } fn consume(ticket: Ticket) -> bool { return true; } fn inspect(view: &Ticket) -> bool { return true; } fn read(view: &Percent) -> Percent { return *view; } fn read_bool(view: &bool) -> bool { return *view; } fn sink(access: &mut bool) -> bool { return true; } fn consume_access(access: &mut bool, value: bool) -> bool { return value; } fn shared_call(view: &bool, value: bool) -> bool { return value; } fn owned(value: bool) -> bool { return value; }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECLARATIONS} {body}")).unwrap();
    let lowered = mir::lower(&p);
    for f in lowered.functions() {
        f.validate().unwrap();
    }
    assert_eq!(lowered, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, text: &str) {
    let source = format!("{DECLARATIONS} {body}");
    let error = compile_source(&source).unwrap_err();
    let (actual, rendered) = match &error {
        FrontendError::Ownership(ds) => ("Ownership", ds[0].render("test.bow", &source)),
        FrontendError::Type(ds) => ("Type", ds[0].render("test.bow", &source)),
        FrontendError::Resolve(ds) => ("Resolve", ds[0].render("test.bow", &source)),
        _ => panic!("unexpected error {error}"),
    };
    assert_eq!(actual, phase, "{rendered}");
    assert!(rendered.contains(text), "{rendered}");
    assert!(rendered.contains("-->"));
    for id in ["LoanId", "LocalId", "ParameterId", "BasicBlockId"] {
        assert!(!rendered.contains(id));
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
accept!(stable_shared_dereference, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let seen = *view; } return value; }");
accept!(shared_call, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let seen = read(view); } return value; }");
accept!(repeated_shared_uses, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let a = *view; let b = read(view); let c = *view; } return value; }");
accept!(shared_aliases, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let alias = view; let second = alias; let a = *view; let b = *second; } return value; }");
accept!(mutable_header_read, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let seen = *access; } return owned; }");
accept!(mutable_boolean_write, "fn f(flag: bool, replacement: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = replacement; } return owned; }");
accept!(mutable_range_write, "fn f(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; while flag { *access = value; let seen = *access; } return owned; }");
accept!(mutable_same_handle_write_rhs, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = *access; } return owned; }");
accept!(
    parameter_shared,
    "fn f(flag: bool, view: &bool) -> bool { while flag { let seen = *view; } return true; }"
);
accept!(parameter_mutable, "fn f(flag: bool, access: &mut bool) -> bool { while flag { *access = false; let seen = *access; } return true; }");
accept!(branch_use_one_side, "fn f(flag: bool, choice: bool, value: Percent) -> Percent { let view = &value; while flag { if choice { let seen = *view; } } return value; }");
accept!(branch_use_both_sides, "fn f(flag: bool, choice: bool, value: Percent) -> Percent { let view = &value; while flag { if choice { let a = read(view); } else { let b = *view; } } return value; }");
accept!(conditional_initializer, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let chosen = if flag { *view } else { read(view) }; } return value; }");
accept!(carried_and_fresh_loans, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let a = *view; let temporary = &value; let b = *temporary; } return value; }");
accept!(iteration_move_alongside_carried, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let ticket = fresh(); let moved = ticket; let done = consume(moved); let seen = *view; } return value; }");
accept!(outer_local_inner_carried, "fn f(flag: bool, value: Percent) -> Percent { while flag { let view = &value; while flag { let seen = *view; } } return value; }");
accept!(preexisting_both_nested, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { while flag { let seen = *view; } } return value; }");
accept!(outer_mutable_inner_write, "fn f(flag: bool) -> bool { while flag { let mut owned = false; let access = &mut owned; while flag { *access = true; } let recovered = owned; } return true; }");
accept!(nested_shared_alias, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let alias = view; while flag { let second = alias; let seen = *second; } } return value; }");
accept!(false_exit_record_move, "fn f(flag: bool, ticket: Ticket) -> bool { let view = &ticket; while flag { let seen = inspect(view); } let done = consume(ticket); return done; }");
accept!(false_exit_copy_assignment, "fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let seen = *view; } owned = true; return owned; }");
accept!(false_exit_exclusive_owner_recovery, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = true; } let view = &owned; return *view; }");
accept!(post_loop_shared_use, "fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let seen = *view; } return *view; }");
accept!(post_loop_mutable_use, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = true; } *access = false; return owned; }");
accept!(dead_unused_header_exclusive_not_pinned, "fn f(flag: bool) -> bool { let mut owned = false; let unused = &mut owned; while flag { owned = true; } return owned; }");
accept!(body_shared_call_holds, "fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = shared_call(view, read_bool(view)); } return owned; }");
accept!(inner_only_obligation_expires_on_exit, "fn f(flag: bool) -> bool { while flag { let mut owned = false; let view = &owned; while flag { let seen = *view; } owned = true; } return true; }");
accept!(constant_false_stable_body_checked, "fn f(value: Percent) -> Percent { let view = &value; while false { let seen = *view; } return value; }");
accept!(constant_true_stable_body_checked, "fn f(value: Percent) -> Percent { let view = &value; while true { let seen = *view; } return value; }");
accept!(one_shot_referent_mutation, "fn f(flag: bool) -> bool { let mut running = flag; let mut owned = false; let access = &mut owned; while running { *access = true; running = false; } return owned; }");
reject!(shared_read_then_conflicting_assignment, "fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let seen = *view; owned = true; } return owned; }", "Ownership", "cannot assign `owned` while it is borrowed");
reject!(skip_path_still_requires_shared, "fn f(flag: bool, choice: bool) -> bool { let mut owned = false; let view = &owned; while flag { if choice { let seen = *view; } else { owned = true; } } return owned; }", "Ownership", "cannot assign `owned` while it is borrowed");
reject!(skip_path_still_requires_exclusive, "fn f(flag: bool, choice: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { if choice { *access = true; } else { let seen = owned; } } return owned; }", "Ownership", "exclusively borrowed");
reject!(exclusive_conflict_before_write, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let other = &owned; *access = true; } return owned; }", "Ownership", "cannot create shared borrow");
reject!(exclusive_conflict_after_write, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = true; let other = &owned; } return owned; }", "Ownership", "cannot create shared borrow");
reject!(shared_conflicting_fresh_exclusive, "fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let seen = *view; let other = &mut owned; } return owned; }", "Ownership", "cannot create exclusive borrow");
reject!(inner_exit_cannot_release_outer_shared, "fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { while flag { let seen = *view; } owned = true; } return owned; }", "Ownership", "cannot assign `owned`");
reject!(inner_exit_cannot_release_outer_exclusive, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { while flag { *access = true; } let other = &owned; } return owned; }", "Ownership", "cannot create shared borrow");
reject!(nested_skip_path_retention, "fn f(flag: bool, choice: bool) -> bool { let mut owned = false; let view = &owned; while flag { if choice { while flag { let seen = *view; } } else { owned = true; } } return owned; }", "Ownership", "cannot assign `owned`");
reject!(post_loop_reference_protects_owner, "fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let seen = *view; } owned = true; return *view; }", "Ownership", "cannot assign `owned`");
reject!(post_loop_mutable_protects_owner, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = true; } let seen = owned; return *access; }", "Ownership", "exclusively borrowed");
reject!(carried_mutable_transfer, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let moved = access; } return owned; }", "Ownership", "cannot move loop-carried mutable reference");
reject!(carried_mutable_call, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let done = sink(access); } return owned; }", "Ownership", "cannot move loop-carried mutable reference");
reject!(carried_mutable_transfer_in_write_rhs, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = consume_access(access, false); } return owned; }", "Ownership", "cannot move loop-carried mutable reference");
reject!(outer_local_mutable_inner_transfer, "fn f(flag: bool) -> bool { let mut owned = false; while flag { let access = &mut owned; while flag { let moved = access; } } return owned; }", "Ownership", "cannot move loop-carried mutable reference");
reject!(
    condition_stored_reference,
    "fn f(flag: bool) -> bool { let view = &flag; while read_bool(view) {} return flag; }",
    "Ownership",
    "while conditions"
);
reject!(
    condition_dereference,
    "fn f(flag: bool) -> bool { let view = &flag; while *view {} return flag; }",
    "Ownership",
    "while conditions"
);
reject!(
    condition_fresh_borrow,
    "fn f(flag: bool) -> bool { while read_bool(&flag) {} return flag; }",
    "Ownership",
    "while conditions"
);
reject!(
    reference_assignment,
    "fn f(flag: bool) -> bool { let mut view = &flag; while flag { view = &flag; } return flag; }",
    "Type",
    "reference"
);
reject!(mutable_to_shared_coercion, "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let seen = read_bool(access); } return owned; }", "Type", "expected &bool, found &mut bool");
reject!(non_copy_dereference, "fn f(flag: bool, ticket: Ticket) -> bool { let view = &ticket; while flag { let moved = *view; } return true; }", "Ownership", "out through borrowed reference");
reject!(carried_owner_still_cannot_move, "fn f(flag: bool, ticket: Ticket) -> bool { let view = &ticket; while flag { let checked = inspect(view); let moved = ticket; } return true; }", "Ownership", "pre-existing non-Copy");
reject!(constant_false_not_retention_exemption, "fn f() -> bool { let mut owned = false; let view = &owned; while false { let seen = *view; owned = true; } return owned; }", "Ownership", "cannot assign `owned`");
reject!(constant_true_not_retention_exemption, "fn f() -> bool { let mut owned = false; let view = &owned; while true { let seen = *view; owned = true; } return owned; }", "Ownership", "cannot assign `owned`");
reject!(one_shot_not_retention_exemption, "fn f(flag: bool) -> bool { let mut running = flag; let mut owned = false; let view = &owned; while running { let seen = *view; running = false; owned = true; } return owned; }", "Ownership", "cannot assign `owned`");
reject!(nominal_reference_types, "fn other(view: &Other) -> bool { return true; } fn f(flag: bool, value: Percent) -> bool { let view = &value; while flag { let seen = other(view); } return true; }", "Type", "expected &Other, found &Percent");
