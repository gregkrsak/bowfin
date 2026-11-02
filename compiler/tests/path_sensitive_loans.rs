use bowfin_compiler::{compile_source, hir::Program, mir, FrontendError};
const DECL: &str = "type Percent = range 0..100; record Ticket { value: Percent }";
fn good(functions: &str) -> Program {
    let p = compile_source(&format!("{DECL} {functions}")).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn reject(functions: &str, message: &str) {
    let text = format!("{DECL} {functions}");
    let Err(FrontendError::Ownership(errors)) = compile_source(&text) else {
        panic!("expected Ownership: {text}")
    };
    assert_eq!(errors.len(), 1);
    assert!(
        errors[0].message.contains(message),
        "{text}: {:?}",
        errors[0]
    );
}
#[test]
fn newly_valid_shared_then_exclusive_else() {
    good("fn update(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let observed = *view; } else { let access = &mut owned; *access = replacement; } return owned; }");
}
#[test]
fn newly_valid_shared_last_use_then_exclusive_in_same_branch() {
    good("fn update(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let observed = *view; let access = &mut owned; *access = replacement; } return owned; }");
}
#[test]
fn newly_valid_exclusive_then_owner_else() {
    good("fn update(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { *access = replacement; } else { let observed = owned; } return owned; }");
}
#[test]
fn newly_valid_different_exclusive_borrow_on_else_edge() {
    good("fn update(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut owned = value; let first = &mut owned; if flag { *first = replacement; } else { let second = &mut owned; *second = replacement; } return owned; }");
}
#[test]
fn newly_valid_shared_reference_vs_non_copy_owner_move() {
    good("fn inspect(value: &Ticket) -> bool { return true; } fn consume(value: Ticket) -> bool { return true; } fn example(flag: bool, ticket: Ticket) -> bool { let view = &ticket; if flag { let predecessor_result = inspect(view); } else { let predecessor_result = consume(ticket); } return true; }");
}

#[test]
fn post_join_shared_use_requires_loan_on_the_else_path() {
    reject("fn bad(flag: bool, value: Percent, replacement: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let observed = *view; } else { let access = &mut owned; *access = replacement; } return *view; }", "shared borrow is active");
}
#[test]
fn post_join_mutable_use_blocks_sibling_owner_access() {
    reject("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let observed = *access; } else { let observed = owned; } return *access; }", "exclusively borrowed");
}
#[test]
fn same_path_future_shared_use_still_blocks_exclusive_borrow() {
    reject("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let access = &mut owned; let observed = *view; } return owned; }", "shared borrow is active");
}
#[test]
fn condition_evaluation_keeps_loans_needed_on_any_successor() {
    for condition in [
        "predicate(owned)",
        "predicate(owned) && flag",
        "flag || predicate(owned)",
    ] {
        reject(&format!("fn predicate(value: Percent) -> bool {{ return true; }} fn bad(flag: bool, value: Percent, replacement: Percent) -> Percent {{ let mut owned = value; let access = &mut owned; if {condition} {{ *access = replacement; }} return owned; }}"), "exclusively borrowed");
    }
}
#[test]
fn never_used_reference_is_dead_before_condition_evaluation() {
    good("fn predicate(value: Percent) -> bool { return true; } fn example(value: Percent) -> Percent { let mut owned = value; let unused = &mut owned; if predicate(owned) {} return owned; }");
}
#[test]
fn condition_final_dereference_allows_expiry_before_dispatch() {
    good("fn example(value: bool) -> bool { let mut owned = value; let access = &mut owned; if *access { let observed = owned; } else { let second = &mut owned; *second = false; } return owned; }");
}
#[test]
fn possible_joined_loan_is_not_removed_by_conditional_move() {
    reject("fn sink(access: &mut Percent) -> bool { return true; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let done = sink(access); } let observed = owned; return *access; }", "exclusively borrowed");
    reject("fn sink(access: &mut Percent) -> bool { return true; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag {} else { let done = sink(access); } let second = &mut owned; return *access; }", "second exclusive borrow");
}
#[test]
fn dead_conditional_handle_allows_owner_recovery_but_stays_unavailable() {
    good("fn sink(access: &mut Percent) -> bool { return true; } fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let done = sink(access); } else { let observed = *access; } return owned; }");
    reject("fn sink(access: &mut Percent) -> bool { return true; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let done = sink(access); } return *access; }", "may have been moved");
}
#[test]
fn nested_edge_expiry_composes_with_outer_continuation() {
    good("fn example(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { if second { let observed = *view; } else { let access = &mut owned; *access = value; } } else { let access = &mut owned; *access = value; } return owned; }");
    good("fn example(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { if second { let observed = *view; } let access = &mut owned; *access = value; } else { let access = &mut owned; *access = value; } return owned; }");
}
#[test]
fn nested_suffix_future_use_blocks_conflicting_inner_path() {
    reject("fn bad(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { if second { let observed = *view; } else { let access = &mut owned; } let later = *view; } return owned; }", "shared borrow is active");
    reject("fn bad(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { if second { let observed = *view; } else { let access = &mut owned; } } return *view; }", "shared borrow is active");
}
#[test]
fn shared_branch_alias_extends_only_its_own_path() {
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let alias = view; let observed = *alias; let access = &mut owned; *access = value; } else { let access = &mut owned; *access = value; } return owned; }");
    reject("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let alias = view; let access = &mut owned; let observed = *alias; } return owned; }", "shared borrow is active");
}
#[test]
fn multiple_outer_shared_aliases_contribute_independently_to_branch_future() {
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; let alias = view; if flag { let observed = *view; } else { let observed = *alias; let access = &mut owned; *access = value; } return owned; }");
    reject("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; let alias = view; if flag { let observed = *view; } else { let access = &mut owned; } return *alias; }", "shared borrow is active");
}
#[test]
fn mutable_branch_local_transfer_preserves_provenance_without_reborrowing() {
    good("fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let first = &mut owned; if flag { let second = first; *second = value; } else { let observed = owned; } return owned; }");
    reject("fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let first = &mut owned; if flag { let second = first; let observed = owned; *second = value; } return owned; }", "exclusively borrowed");
    reject("fn bad(flag: bool, first: &mut Percent) -> Percent { if flag { let second = first; let observed = *second; } return *first; }", "may have been moved");
}
#[test]
fn sibling_created_loans_do_not_escape_or_collide_with_later_provenance() {
    good("fn example(flag: bool, left: Percent, right: Percent) -> Percent { let mut first = left; let mut second = right; if flag { let access = &mut first; *access = right; } else { let access = &mut second; *access = left; } let access = &mut first; *access = left; return second; }");
}
#[test]
fn sequential_conditions_preserve_continuation_and_do_not_resurrect_loans() {
    reject("fn bad(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { let observed = *view; } if second { let access = &mut owned; } return *view; }", "shared borrow is active");
    reject("fn bad(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { let observed = *view; } let access = &mut owned; if second { let observed = *view; } return owned; }", "shared borrow is active");
    good("fn example(first: bool, second: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if first { let observed = *view; } if second { let access = &mut owned; *access = value; } return owned; }");
}
#[test]
fn constant_conditions_do_not_prune_possible_edges() {
    for condition in ["true", "false"] {
        reject(&format!("fn bad(ticket: Ticket) -> Ticket {{ if {condition} {{ let moved = ticket; }} return ticket; }}"), "may have been moved");
        reject(&format!("fn bad(value: Percent) -> Percent {{ let mut owned = value; let view = &owned; if {condition} {{ let observed = *view; }} else {{ let access = &mut owned; }} return *view; }}"), "shared borrow is active");
    }
}
#[test]
fn conditional_move_join_rule_and_condition_side_moves_are_unchanged() {
    for branch in [
        "if flag { let moved = ticket; }",
        "if flag {} else { let moved = ticket; }",
        "if flag { let moved = ticket; } else { let moved = ticket; }",
        "if flag { if flag { let moved = ticket; } }",
    ] {
        reject(
            &format!("fn bad(flag: bool, ticket: Ticket) -> Ticket {{ {branch} return ticket; }}"),
            "may have been moved",
        );
    }
    reject("fn predicate(ticket: Ticket) -> bool { return true; } fn bad(ticket: Ticket) -> Ticket { if predicate(ticket) {} return ticket; }", "use of moved value");
}
#[test]
fn direct_shared_call_hold_blocks_later_move_inside_branch() {
    reject("fn receive(view: &Ticket, ticket: Ticket) -> bool { return true; } fn bad(flag: bool, ticket: Ticket) -> bool { if flag { let predecessor_result = receive(&ticket, ticket); } return true; }", "shared-borrowed");
}
#[test]
fn direct_mutable_call_hold_blocks_later_owner_access_inside_branch() {
    reject("fn receive(access: &mut Percent, value: Percent) -> bool { return true; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; if flag { let predecessor_result = receive(&mut owned, owned); } return owned; }", "exclusively borrowed");
}
#[test]
fn final_shared_reference_argument_remains_call_held_through_nested_arguments() {
    reject("fn receive(view: &Ticket, ticket: Ticket) -> bool { return true; } fn identity(ticket: Ticket) -> Ticket { return ticket; } fn bad(flag: bool, ticket: Ticket) -> bool { let view = &ticket; if flag { let predecessor_result = receive(view, identity(ticket)); } return true; }", "shared-borrowed");
}
#[test]
fn final_mutable_argument_remains_call_held_and_inner_calls_release_only_their_holds() {
    reject("fn receive(access: &mut Percent, value: Percent) -> bool { return true; } fn identity(value: Percent) -> Percent { return value; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let predecessor_result = receive(access, identity(owned)); } return owned; }", "exclusively borrowed");
    good("fn inspect(view: &Percent) -> bool { return true; } fn receive(first: bool, second: bool) -> bool { return true; } fn exclusive(access: &mut Percent) -> bool { return true; } fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; if flag { let predecessor_result = receive(inspect(&owned), exclusive(&mut owned)); } return owned; }");
}
#[test]
fn writes_hold_loans_through_rhs_and_do_not_allow_target_handle_transfer() {
    reject("fn identity(value: Percent) -> Percent { return value; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { *access = identity(owned); } return owned; }", "exclusively borrowed");
    reject("fn replace(access: &mut Percent, replacement: Percent) -> Percent { return replacement; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { *access = replace(&mut owned, value); } return owned; }", "second exclusive borrow");
    reject("fn sink(access: &mut bool) -> bool { return true; } fn bad(flag: bool, access: &mut bool) -> bool { if flag { *access = sink(access); } return true; }", "use of moved value");
}
#[test]
fn branch_direct_borrow_calls_do_not_become_lexical_loans() {
    good("fn inspect(view: &Percent) -> bool { return true; } fn exclusive(access: &mut Percent) -> bool { return true; } fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; if flag { let first = inspect(&owned); let second = exclusive(&mut owned); let observed = owned; } return owned; }");
}
#[test]
fn straight_line_last_use_alias_and_transfer_behavior_is_preserved() {
    good("fn example(value: Percent) -> Percent { let mut owned = value; let view = &owned; let alias = view; let observed = *view; let other = *alias; let first = &mut owned; let second = first; *second = value; let unused = &mut owned; return owned; }");
}
#[test]
fn diagnostic_selection_remains_in_source_branch_order() {
    for (first, second, expected) in [
        (
            "let then_access = &mut owned;",
            "let else_access = &mut owned;",
            "let then_access",
        ),
        ("", "let else_access = &mut owned;", "let else_access"),
    ] {
        let text = format!("{DECL} fn bad(flag: bool, value: Percent) -> Percent {{ let mut owned = value; let view = &owned; if flag {{ {first} }} else {{ {second} }} return *view; }}");
        for _ in 0..3 {
            let Err(FrontendError::Ownership(errors)) = compile_source(&text) else {
                panic!()
            };
            let start = text.find(expected).unwrap();
            assert_eq!(
                errors[0].span.start,
                start + text[start..].find("&mut owned").unwrap()
            );
        }
    }
}
