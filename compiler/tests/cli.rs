use std::{path::PathBuf, process::Command};

fn example(name: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../examples")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bowfin"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn explicit_effect_contracts_check_transitively_without_changing_proof_scope() {
    for command in ["check", "verify"] {
        let good = run(&[command, &example("explicit_effects.bow")]);
        assert!(good.status.success());
        assert!(good.stderr.is_empty());
        let text = String::from_utf8(good.stdout).unwrap();
        assert!(text.contains(
            "ordinary effects: explicit upper bounds and complete callee contracts checked"
        ));
        assert!(text.contains("not effects-aware verification"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        let bad = run(&[command, &example("explicit_effects_fail.bow")]);
        assert_eq!(bad.status.code(), Some(1));
        assert!(bad.stdout.is_empty());
        let text = String::from_utf8(bad.stderr).unwrap();
        assert!(text.contains("unadmitted effect `io`"));
        assert!(text.contains("call to `middle`"));
    }
    let good = run(&[
        "run",
        &example("explicit_effects.bow"),
        "--function",
        "demo",
    ]);
    assert!(good.status.success());
    assert!(good.stderr.is_empty());
    assert_eq!(String::from_utf8(good.stdout).unwrap().trim(), "Bool(true)");
    let bad = run(&[
        "run",
        &example("explicit_effects_fail.bow"),
        "--function",
        "demo",
    ]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(bad.stdout.is_empty());
    assert!(String::from_utf8(bad.stderr)
        .unwrap()
        .contains("unadmitted effect `io`"));
}

#[test]
fn success_reports_only_supported_proofs() {
    for (command, status) in [("verify", "verified"), ("check", "checked")] {
        let output = run(&[command, &example("battery_ok.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.starts_with(&format!("{status} supported integer contracts:")));
        assert!(text.contains("function bodies proven: 1"));
        assert!(text.contains("calls checked: 1"));
        assert!(
            text.contains("general ownership, effects, and code generation are not implemented")
        );
    }
}

#[test]
fn rejected_programs_return_one_and_no_success_message() {
    for (name, message) in [
        ("battery_fail.bow", "90 <= 80 is false"),
        ("battery_sequence_fail.bow", "50 <= 30 is false"),
        (
            "battery_body_fail.bow",
            "postcondition cannot be established",
        ),
        ("effects.bow", "supported grammar"),
    ] {
        for command in ["verify", "check"] {
            let output = run(&[command, &example(name)]);
            assert_eq!(output.status.code(), Some(1), "{name}");
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains(message), "{text}");
            assert!(text.contains("-->"));
            assert!(text.contains(name));
        }
    }
}

#[test]
fn usage_and_io_errors_return_two() {
    for args in [
        vec![],
        vec!["build"],
        vec!["verify"],
        vec!["--version", "extra"],
        vec!["verify", "one.bow", "two.bow"],
        vec!["verify", "file-that-does-not-exist.bow"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
    }
    assert!(run(&["--version"]).status.success());
}

#[test]
fn ordinary_functions_report_typing_without_execution_or_proof() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("value_flow.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(
            text.starts_with("checked ordinary value functions; no verified function contracts:")
        );
        assert!(text.contains("ordinary value functions type-checked: 6"));
        assert!(text.contains("not executed or verified; constrained results are not proven"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(!text.contains("verified supported integer contracts"));
    }
}

#[test]
fn record_moves_are_checked_without_claiming_execution_or_verification() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("move_values.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary value functions type-checked: 8"));
        assert!(text.contains("core Copy/move checking passed"));
        assert!(text.contains("whole-value loans checked; Copy-safe borrowed access checked; destruction is not implemented"));
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
    }
}

#[test]
fn ownership_failure_exits_one_and_reports_both_move_sites() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("move_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("use of moved value `ticket`"));
        assert!(text.contains("move_fail.bow:7:12"));
        assert!(text.contains("into local `next` at line 6, column 16"));
        assert!(text.contains("ownership checking stopped"));
    }
}

#[test]
fn borrowing_success_reports_loans_without_execution_or_proof() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("borrow_values.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("whole-value loans checked"));
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(text.contains("destruction is not implemented"));
    }
}

#[test]
fn borrowing_failure_reports_the_conflict_and_earlier_loan_location() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("borrow_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("cannot move `ticket` while it is shared-borrowed"));
        assert!(text.contains("borrow_fail.bow:11:17"));
        assert!(text.contains("borrow began at line 10, column 16"));
        assert!(text.contains("ownership checking stopped"));
        assert!(!text.contains("LoanId"));
    }
}

#[test]
fn copy_safe_mutation_reports_frontend_acceptance_without_runtime_claims() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("deref_mutation.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Copy-safe borrowed access checked"));
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(text.contains("destruction is not implemented"));
    }
}
#[test]
fn non_copy_mutation_failure_explains_the_replacement_boundary() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("deref_mutation_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("cannot replace non-Copy value `Ticket`"));
        assert!(text.contains("before destruction semantics are defined"));
        assert!(text.contains("deref_mutation_fail.bow:6:5"));
        assert!(text.contains("ownership checking stopped"));
    }
}

#[test]
fn conditional_success_reports_mir_without_execution_or_ordinary_proof() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("if_control_flow.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 1 functions"));
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
    }
}
#[test]
fn conditional_move_failure_reports_prior_path_and_move_location() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("if_move_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("`ticket` may have been moved on a previous control-flow path"));
        assert!(text.contains("a move occurred at line 6"));
        assert!(text.contains("-->"));
    }
}

#[test]
fn path_sensitive_borrowing_reports_acceptance_and_valid_mir_without_runtime_claims() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("path_sensitive_borrowing.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 1 functions"));
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
    }
}
#[test]
fn post_join_future_reference_reports_original_borrow_conflict() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("path_sensitive_borrowing_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text
            .contains("cannot create exclusive borrow of `owned` while shared borrow is active"));
        assert!(text.contains("the conflicting borrow began at line 5"));
        assert!(text.contains("-->"));
    }
}

#[test]
fn conditional_initializer_cli_parity() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("conditional_value.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
        let output = run(&[command, &example("conditional_value_move_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("may have been moved on a previous control-flow path"));
        assert!(text.contains("conditional_value_move_fail.bow"));
    }
}

#[test]
fn local_reassignment_cli_reports_frontend_only_and_borrow_conflict() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("local_reassignment.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
        let output = run(&[command, &example("local_reassignment_borrow_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("cannot assign `current` while it is borrowed"));
        assert!(text.contains("conflicting borrow began at line 6"));
        assert!(text.contains("local_reassignment_borrow_fail.bow:7:5"));
        assert!(text.contains("ownership checking stopped"));
    }
}

#[test]
fn while_cli_reports_frontend_only_and_cyclic_ownership_boundaries() {
    for command in ["check", "verify"] {
        for file in ["while_copy.bow", "while_nested.bow"] {
            let output = run(&[command, &example(file)]);
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.contains("not executed or verified"));
            assert!(text.contains("function bodies proven: 0"));
            assert!(text.contains("ordinary MIR control-flow lowering passed: 1 functions"));
        }
        for (file, message) in [
            (
                "while_reference_fail.bow",
                "cannot move loop-carried mutable reference",
            ),
            (
                "while_move_fail.bow",
                "cannot move pre-existing non-Copy value",
            ),
        ] {
            let output = run(&[command, &example(file)]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains(message));
            assert!(
                text.contains("unchanged provenance") || text.contains("future generalized cyclic")
            );
            assert!(text.contains("ownership checking stopped"));
        }
    }
}

#[test]
fn iteration_local_move_cli_parity_and_nested_boundary_diagnostics() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("while_iteration_move.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 3 functions"));
        assert!(text.contains("not executed or verified"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("iteration-local Move"));
        for (file, owner) in [
            ("while_carried_move_fail.bow", "ticket"),
            ("while_nested_carried_move_fail.bow", "outer"),
        ] {
            let output = run(&[command, &example(file)]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains(&format!(
                "cannot move pre-existing non-Copy value `{owner}` in while loop"
            )));
            assert!(text.contains("future generalized cyclic ownership analysis"));
            assert!(text.contains("-->"));
            assert!(text.contains("ownership checking stopped"));
        }
    }
}

#[test]
fn iteration_local_borrow_cli_parity_and_relative_reference_boundaries() {
    for command in ["check", "verify"] {
        for file in [
            "while_iteration_borrow.bow",
            "while_iteration_mut_borrow.bow",
            "while_iteration_move_borrow.bow",
        ] {
            let output = run(&[command, &example(file)]);
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.contains("ordinary MIR control-flow lowering passed"));
            assert!(text.contains("not executed or verified"));
            assert!(text.contains("function bodies proven: 0"));
            assert!(text.contains("iteration-local Move/borrowing"));
        }
        for file in [
            "while_carried_reference_fail.bow",
            "while_nested_reference_fail.bow",
        ] {
            let output = run(&[command, &example(file)]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains("cannot move loop-carried mutable reference `access`"));
            assert!(text.contains("unchanged provenance"));
            assert!(text.contains("-->"));
            assert!(text.contains("ownership checking stopped"));
        }
    }
}

#[test]
fn stable_carried_reference_cli_parity_and_recurrent_conflict() {
    for command in ["check", "verify"] {
        for file in [
            "while_carried_shared.bow",
            "while_carried_mutable.bow",
            "while_carried_nested.bow",
        ] {
            let output = run(&[command, &example(file)]);
            assert!(output.status.success());
            assert!(output.stderr.is_empty());
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.contains("not executed or verified"));
            assert!(text.contains("function bodies proven: 0"));
            assert!(text.contains("ordinary MIR control-flow lowering passed"));
        }
        let output = run(&[command, &example("while_recurrent_borrow_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("cannot assign `owned` while it is borrowed"));
        assert!(text.contains("-->"));
        assert!(text.contains("ownership checking stopped"));
    }
}

#[test]
fn no_value_functions_report_honest_completion_without_proof() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("no_value_functions.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.starts_with("checked ordinary functions; no verified function contracts:"));
        assert!(
            text.contains("ordinary no-value functions type-checked: 3 (not executed or verified)")
        );
        assert!(text.contains("ordinary value functions type-checked: 1"));
        assert!(text.contains("ordinary MIR control-flow lowering passed: 4 functions"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        for (name, message) in [
            ("no_value_call_fail.bow", "moved"),
            ("no_value_recurrent_fail.bow", "borrow"),
        ] {
            let output = run(&[command, &example(name)]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains(message), "{text}");
            assert!(text.contains(name));
        }
    }
}

#[test]
fn ordinary_record_cli_checks_preserve_the_proof_boundary() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("ordinary_records.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 6 functions"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(text.contains("not executed or verified"));
        for (name, message) in [
            (
                "ordinary_record_move_fail.bow",
                "use of moved value `battery`",
            ),
            (
                "ordinary_record_condition_fail.bow",
                "record construction is unsupported in recurring while conditions",
            ),
        ] {
            let output = run(&[command, &example(name)]);
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains(message), "{text}");
            assert!(text.contains(name));
        }
    }
}

#[test]
fn multi_field_record_cli_preserves_whole_record_and_proof_boundaries() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("multiple_field_records.bow")]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 1 functions"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(text.contains("not executed or verified"));
        let output = run(&[command, &example("multiple_field_record_move_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8(output.stderr)
            .unwrap()
            .contains("use of moved value `ticket`"));
    }
}

#[test]
fn copy_field_assignment_cli_checks_and_lowers_without_execution_or_proof() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("copy_field_assignment.bow")]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 1 functions"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(text.contains("not executed or verified"));
        let output = run(&[command, &example("copy_field_assignment_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("cannot assign `working.charge` while it is borrowed"));
        assert!(text.contains("the conflicting borrow began at line 7"));
    }
}

#[test]
fn reference_field_reads_cli_preserves_last_use_and_whole_record_proof_boundaries() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("reference_field_reads.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 3 functions"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        assert!(text.contains("not executed or verified"));
        let output = run(&[command, &example("reference_field_reads_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        for context in [
            "working.health",
            "Battery.health",
            "Percent",
            "active shared whole-record loan",
            "the conflicting borrow began at line 6",
        ] {
            assert!(text.contains(context), "{text}");
        }
    }
}

#[test]
fn exclusive_reference_field_write_cli_preserves_proof_scope_and_final_gate_diagnostic() {
    for command in ["check", "verify"] {
        let output = run(&[command, &example("reference_field_assignment.bow")]);
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("ordinary MIR control-flow lowering passed: 3 functions"));
        assert!(text.contains("function bodies proven: 0"));
        assert!(text.contains("calls checked: 0"));
        let output = run(&[command, &example("reference_field_assignment_fail.bow")]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let text = String::from_utf8(output.stderr).unwrap();
        assert!(text.contains("moved value `access`"));
        assert!(text.contains("reference_field_assignment_fail.bow"));
    }
}
