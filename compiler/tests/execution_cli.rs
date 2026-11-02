use std::{fs, process::Command};
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bowfin"))
        .args(args)
        .output()
        .unwrap()
}
fn example() -> String {
    format!(
        "{}/../examples/ordinary_run.bow",
        env!("CARGO_MANIFEST_DIR")
    )
}
#[test]
fn run_record_loop_and_trap() {
    let p = example();
    for (entry, want) in [
        ("demo", "Pair { first: Percent(89), second: Percent(90) }\n"),
        ("loop_demo", "Percent(0)\n"),
    ] {
        let o = run(&["run", &p, "--function", entry]);
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert_eq!(String::from_utf8(o.stdout).unwrap(), want);
    }
    let o = run(&["run", &p, "--function", "trap_demo"]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8(o.stderr)
        .unwrap()
        .contains("computed -101"));
}
#[test]
fn cli_invalid_limits_and_entry_selection() {
    let p = example();
    for flags in [
        vec!["--max-steps", "0"],
        vec!["--max-steps", "18446744073709551616"],
        vec!["--max-call-depth", "257"],
        vec!["--max-steps", "1", "--max-steps", "2"],
    ] {
        let mut a = vec!["run", p.as_str(), "--function", "demo"];
        a.extend(flags);
        assert_eq!(run(&a).status.code(), Some(2));
    }
    assert_eq!(
        run(&["run", &p, "--function", "missing"]).status.code(),
        Some(1)
    );
    assert_eq!(run(&["run", &p]).status.code(), Some(2));
    assert_eq!(
        run(&["run", &p, "--function", "demo", "--max-steps", "1"])
            .status
            .code(),
        Some(1)
    );
}
#[test]
fn cli_detected_input_failure_is_host_failure() {
    assert_eq!(
        run(&["run", "/a/nonexistent/bowfin-input", "--function", "demo"])
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn cli_bounded_input_and_output() {
    let p = example();
    for flags in [
        ["--max-source-bytes", "1"],
        ["--max-output-bytes", "4"],
        ["--max-tokens", "2"],
    ] {
        let o = run(&["run", &p, "--function", "demo", flags[0], flags[1]]);
        assert_eq!(o.status.code(), Some(1));
        assert!(o.stderr.len() < 256);
    }
}
#[test]
fn run_does_not_evaluate_failing_specialized_harness() {
    let p = std::env::temp_dir().join(format!("bowfin-execution-cli-{}.bow", std::process::id()));
    let s = format!(
        "{}\nfn demo()->Percent{{return 80;}}",
        include_str!("../../examples/battery_fail.bow")
    );
    fs::write(&p, s).unwrap();
    let path = p.to_str().unwrap();
    let o = run(&["run", path, "--function", "demo"]);
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert_eq!(o.stdout, b"Percent(80)\n");
    let o = run(&["verify", path]);
    assert_eq!(o.status.code(), Some(1));
    assert!(String::from_utf8(o.stderr)
        .unwrap()
        .contains("90 <= 80 is false"));
    fs::remove_file(p).unwrap();
}

#[test]
fn owned_replacement_example_runs_without_claiming_contract_proof() {
    let path = format!(
        "{}/../examples/owned_replacement.bow",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = run(&["run", &path, "--function", "demo"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"Reading { value: Percent(59) }\n");
    for mode in ["check", "verify"] {
        let output = run(&[mode, &path]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("no verified function contracts"));
        assert!(text.contains("not executed or verified; constrained results are not proven"));
    }
}

#[test]
fn propagation_cli_examples_select_success_and_failure() {
    let path = format!(
        "{}/../examples/result_propagation.bow",
        env!("CARGO_MANIFEST_DIR")
    );
    for (entry, expected) in [
        ("demo", "Ok(Reading(42))\n"),
        ("failure_demo", "Error(ReadError(1))\n"),
    ] {
        let output = run(&["run", &path, "--function", entry]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}
#[test]
fn propagation_cli_nominal_failure_mismatch_is_frontend_class_one() {
    let path = format!(
        "{}/../examples/result_propagation_mismatch.bow",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = run(&["run", &path, "--function", "demo"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("or return failure payload type mismatch"));
    assert!(output.stdout.is_empty());
}
