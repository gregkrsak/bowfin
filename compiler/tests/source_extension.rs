//! Explicit M3 corpus expectations and source-path compatibility controls.
//! Filenames and fences grant no language, profile, or execution authority.
use bowfin_compiler::{compile_source, execution, mir, trusted, verify, FrontendError};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

const COMMANDS: [&str; 2] = [env!("CARGO_BIN_EXE_bowfin"), env!("CARGO_BIN_EXE_klyxr")];
const UNSELECTED_ACCEPTED: [&str; 37] = [
    "battery",
    "battery_ok",
    "borrow_values",
    "conditional_value",
    "contextual_range_literals",
    "copy_field_assignment",
    "deref_mutation",
    "early_return",
    "early_return_move",
    "explicit_effects",
    "if_control_flow",
    "local_reassignment",
    "move_values",
    "multiple_field_records",
    "no_value_functions",
    "ordinary_records",
    "ordinary_run",
    "owned_replacement",
    "path_sensitive_borrowing",
    "reference_field_assignment",
    "reference_field_reads",
    "result_handling",
    "result_propagation",
    "value_flow",
    "while_break",
    "while_break_nested",
    "while_carried_mutable",
    "while_carried_nested",
    "while_carried_shared",
    "while_continue",
    "while_continue_nested",
    "while_copy",
    "while_iteration_borrow",
    "while_iteration_move",
    "while_iteration_move_borrow",
    "while_iteration_mut_borrow",
    "while_nested",
];

// These causes come from the source witnesses and their accepted boundaries,
// not from running an old or new interpreter and copying its observed result.
const FRONTEND_REJECTED: [(&str, &str, &str); 30] = [
    (
        "borrow_fail",
        "ownership",
        "cannot move `ticket` while it is shared-borrowed",
    ),
    (
        "conditional_value_move_fail",
        "ownership",
        "may have been moved on a previous control-flow path",
    ),
    (
        "contextual_range_literal_fail",
        "type",
        "literal outside named range `Percent`",
    ),
    (
        "copy_field_assignment_fail",
        "ownership",
        "cannot assign `working.charge` while it is borrowed",
    ),
    (
        "deref_mutation_fail",
        "ownership",
        "cannot replace non-Copy value `Ticket`",
    ),
    (
        "early_return_recurrent_fail",
        "ownership",
        "cannot assign `owned` while it is borrowed",
    ),
    ("effects", "lex", "unexpected character `#`"),
    ("explicit_effects_fail", "effects", "unadmitted effect `io`"),
    (
        "if_move_fail",
        "ownership",
        "may have been moved on a previous control-flow path",
    ),
    (
        "local_reassignment_borrow_fail",
        "ownership",
        "cannot assign `current` while it is borrowed",
    ),
    ("move_fail", "ownership", "use of moved value `ticket`"),
    (
        "multiple_field_record_move_fail",
        "ownership",
        "use of moved value `ticket`",
    ),
    (
        "no_value_call_fail",
        "ownership",
        "use of moved value `ticket`",
    ),
    (
        "no_value_recurrent_fail",
        "ownership",
        "cannot assign `owned` while it is borrowed",
    ),
    (
        "ordinary_record_condition_fail",
        "ownership",
        "record construction is unsupported in recurring while conditions",
    ),
    (
        "ordinary_record_move_fail",
        "ownership",
        "use of moved value `battery`",
    ),
    (
        "path_sensitive_borrowing_fail",
        "ownership",
        "cannot create exclusive borrow of `owned` while shared borrow is active",
    ),
    (
        "reference_field_assignment_fail",
        "ownership",
        "use of moved value `access`",
    ),
    (
        "reference_field_reads_fail",
        "ownership",
        "cannot assign `working.health` while it is borrowed",
    ),
    (
        "result_propagation_mismatch",
        "type",
        "or return failure payload type mismatch",
    ),
    (
        "result_unhandled_fail",
        "ownership",
        "unresolved Result outcome `pending`",
    ),
    (
        "while_break_recurrent_fail",
        "ownership",
        "cannot assign `value` while it is borrowed",
    ),
    (
        "while_carried_move_fail",
        "ownership",
        "cannot move pre-existing non-Copy value `ticket` in while loop",
    ),
    (
        "while_carried_reference_fail",
        "ownership",
        "cannot move loop-carried mutable reference `access`",
    ),
    (
        "while_continue_recurrent_fail",
        "ownership",
        "cannot assign `value` while it is borrowed",
    ),
    (
        "while_move_fail",
        "ownership",
        "cannot move pre-existing non-Copy value `ticket` in while loop",
    ),
    (
        "while_nested_carried_move_fail",
        "ownership",
        "cannot move pre-existing non-Copy value `outer` in while loop",
    ),
    (
        "while_nested_reference_fail",
        "ownership",
        "cannot move loop-carried mutable reference `access`",
    ),
    (
        "while_recurrent_borrow_fail",
        "ownership",
        "cannot assign `owned` while it is borrowed",
    ),
    (
        "while_reference_fail",
        "ownership",
        "cannot move loop-carried mutable reference `access`",
    ),
];

const PROOF_REJECTED: [(&str, usize, usize, &str); 3] = [
    (
        "battery_body_fail",
        0,
        0,
        "postcondition cannot be established",
    ),
    ("battery_fail", 1, 0, "precondition cannot be established"),
    (
        "battery_sequence_fail",
        1,
        1,
        "precondition cannot be established",
    ),
];
const SELECTED: [(&str, &str, Option<&str>); 5] = [
    ("sensor_finish", "sample", None),
    (
        "sensor_finish_live_loan_fail",
        "sample",
        Some("cannot move `sensor` while it is shared-borrowed"),
    ),
    (
        "sensor_finish_reuse_fail",
        "sample",
        Some("use of moved value `sensor`"),
    ),
    ("sentinel", "monitor", None),
    ("trusted_sensor", "sample", None),
];

fn examples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples")
}

fn source(name: &str) -> String {
    fs::read_to_string(examples().join(format!("{name}.bow"))).unwrap()
}

fn phase(error: &FrontendError) -> &'static str {
    match error {
        FrontendError::Lex(_) => "lex",
        FrontendError::Parse(_) => "parse",
        FrontendError::Resolve(_) => "resolve",
        FrontendError::Type(_) => "type",
        FrontendError::Ownership(_) => "ownership",
        FrontendError::Effects(_) => "effects",
    }
}

fn invoke(binary: &str, command: &str, path: &Path, extra: &[&str]) -> Output {
    Command::new(binary)
        .arg(command)
        .arg(path)
        .args(extra)
        .output()
        .unwrap()
}

fn assert_result(output: &Output, expected: &str) {
    assert_eq!(output.status.code(), Some(0), "{:?}", output);
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
}

fn selected_options<'a>(root: &'a str, scenario: &'a str) -> Vec<&'a str> {
    vec![
        "--function",
        root,
        "--profile",
        "sensor-demo",
        "--scenario",
        scenario,
        "--sensor-limit",
        "1",
        "--max-tokens",
        "2048",
        "--max-frontend-work",
        "10000000000000000",
        "--max-admission-storage",
        "1073741824",
        "--max-diagnostic-storage",
        "4194304",
    ]
}

struct Scratch(PathBuf);
impl Scratch {
    fn new(label: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "bowfin-m3-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn m3_corpus_frontend_expectations_are_explicit() {
    let declared: Vec<_> = UNSELECTED_ACCEPTED
        .iter()
        .copied()
        .chain(FRONTEND_REJECTED.iter().map(|row| row.0))
        .chain(PROOF_REJECTED.iter().map(|row| row.0))
        .chain(SELECTED.iter().map(|row| row.0))
        .map(|name| format!("{name}.bow"))
        .collect();
    let expected: BTreeSet<_> = declared.iter().cloned().collect();
    assert_eq!(declared.len(), 75);
    assert_eq!(expected.len(), 75, "duplicate corpus expectation");
    let actual: BTreeSet<_> = fs::read_dir(examples())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(".bow") || name.ends_with(".klx"))
        .collect();
    assert_eq!(
        actual, expected,
        "missing, added, or unmigrated corpus source"
    );

    for name in UNSELECTED_ACCEPTED {
        let program = compile_source(&source(name)).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(
            mir::lower(&program)
                .functions()
                .iter()
                .all(|function| function.validate().is_ok()),
            "{name}"
        );
        let report = verify::verify_report(&program);
        assert!(
            report.diagnostics.is_empty(),
            "{name}: {:?}",
            report.diagnostics
        );
        let expected = match name {
            "battery" => (1, 0),
            "battery_ok" => (1, 1),
            _ => (0, 0),
        };
        assert_eq!(
            (report.functions_proven, report.calls_checked),
            expected,
            "{name}"
        );
    }
    for (name, expected_phase, cause) in FRONTEND_REJECTED {
        let error = compile_source(&source(name)).unwrap_err();
        assert_eq!(phase(&error), expected_phase, "{name}: {error}");
        assert!(error.to_string().contains(cause), "{name}: {error}");
    }
    for (name, proven, calls, cause) in PROOF_REJECTED {
        let program = compile_source(&source(name)).unwrap();
        let report = verify::verify_report(&program);
        assert_eq!(report.functions_proven, proven, "{name}");
        assert_eq!(report.calls_checked, calls, "{name}");
        assert_eq!(report.diagnostics.len(), 1, "{name}");
        assert!(report.diagnostics[0].message.contains(cause), "{name}");
    }
    for (name, root, cause) in SELECTED {
        let selection = trusted::Selection::sensor_demo(
            trusted::Scenario::Sample,
            trusted::Grants::ALL,
            &[root],
        )
        .unwrap();
        let admitted = execution::prepare_execution_with_profile(
            &source(name),
            execution::AdmissionLimits {
                tokens: 2048,
                frontend_work: 10_000_000_000_000_000,
                working_storage: 1_073_741_824,
                diagnostic_storage: 4_194_304,
                ..execution::AdmissionLimits::default()
            },
            &selection,
        );
        if let Some(cause) = cause {
            let Err(error) = admitted else {
                panic!("{name}: expected ownership rejection");
            };
            let execution::ErrorKind::Frontend { phase, message } = error.kind else {
                panic!("{name}: wrong rejection class: {error:?}");
            };
            assert_eq!(phase, "ownership", "{name}");
            assert!(message.contains(cause), "{name}: {message}");
        } else {
            let artifact = admitted.unwrap_or_else(|e| panic!("{name}: {e:?}"));
            assert!(artifact.entry(root).is_ok(), "{name}");
        }
    }
}

#[test]
fn m3_corpus_runs_and_audits_have_independent_outcomes() {
    // Parameterized-only examples are checked above, not granted invented public
    // entry arguments. These are all finite zero-parameter ordinary roots in
    // the accepted non-profile corpus; recursive factories have an explicit stop.
    for (name, root, expected) in [
        (
            "contextual_range_literals",
            "full",
            "Battery { charge: Percent(80), health: Percent(100) }\n",
        ),
        ("explicit_effects", "observe", "Bool(true)\n"),
        ("explicit_effects", "sample", "NoValue\n"),
        ("explicit_effects", "demo", "Bool(true)\n"),
        (
            "ordinary_run",
            "demo",
            "Pair { first: Percent(89), second: Percent(90) }\n",
        ),
        ("ordinary_run", "loop_demo", "Percent(0)\n"),
        (
            "owned_replacement",
            "demo",
            "Reading { value: Percent(59) }\n",
        ),
        ("result_handling", "sample", "Reading(42)\n"),
        ("result_handling", "read_error_code", "ErrorCode(1)\n"),
        ("result_handling", "demo", "Error(ErrorCode(1))\n"),
        ("result_handling", "success_demo", "Reading(42)\n"),
        ("result_propagation", "reading", "Reading(42)\n"),
        ("result_propagation", "problem", "ReadError(1)\n"),
        ("result_propagation", "demo", "Ok(Reading(42))\n"),
        (
            "result_propagation",
            "failure_demo",
            "Error(ReadError(1))\n",
        ),
        ("value_flow", "enabled", "Bool(true)\n"),
    ] {
        for binary in COMMANDS {
            let output = invoke(
                binary,
                "run",
                &examples().join(format!("{name}.bow")),
                &["--function", root],
            );
            assert_result(&output, expected);
        }
    }
    for (name, root, extra, cause) in [
        (
            "ordinary_run",
            "trap_demo",
            &[][..],
            "named-range trap: Percent, inclusive 0..100, computed -101",
        ),
        (
            "while_iteration_move",
            "make_ticket",
            &["--max-call-depth", "4"][..],
            "configured tool limit exhausted: call depth",
        ),
        (
            "while_iteration_move_borrow",
            "fresh_ticket",
            &["--max-call-depth", "4"][..],
            "configured tool limit exhausted: call depth",
        ),
    ] {
        for binary in COMMANDS {
            let mut options = vec!["--function", root];
            options.extend(extra);
            let output = invoke(
                binary,
                "run",
                &examples().join(format!("{name}.bow")),
                &options,
            );
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            assert!(String::from_utf8(output.stderr).unwrap().contains(cause));
        }
    }
    for binary in COMMANDS {
        for (name, root, scenario, expected) in [
            ("trusted_sensor", "sample", "sample", "Ok(Reading(42))\n"),
            ("trusted_sensor", "sample", "no-sensor", "Error(SensorError(1))\n"),
            ("trusted_sensor", "sample", "no-sample", "Error(SensorError(2))\n"),
            ("sensor_finish", "sample", "sample", "Ok(Sample { reading: Reading(42), finished: FinishReceipt(1) })\n"),
            ("sensor_finish", "sample", "finish-io-failure", "Error(SensorError(3))\n"),
            ("sensor_finish", "sample", "finish-partial-io-failure", "Error(SensorError(3))\n"),
            ("sentinel", "monitor", "critical", "Ok(Report { observations: Count(2), warnings: Count(1), critical: Count(1), peak: Reading(90), disposition: Disposition(2), completed: Count(2) })\n"),
        ] {
            let output = invoke(binary, "run", &examples().join(format!("{name}.bow")), &selected_options(root, scenario));
            assert_result(&output, expected);
        }
        for (name, root, finish) in [
            ("trusted_sensor", "sample", false),
            ("sensor_finish", "sample", true),
            ("sentinel", "monitor", true),
        ] {
            let output = invoke(
                binary,
                "audit",
                &examples().join(format!("{name}.bow")),
                &selected_options(root, "sample"),
            );
            assert_eq!(output.status.code(), Some(0));
            assert!(output.stderr.is_empty());
            let text = String::from_utf8(output.stdout).unwrap();
            assert!(text.starts_with("authority account v1\ntarget=bowfin.ordinary-mir.v1\ncompiler=bowfin-compiler/0.0.1\n"));
            assert!(text.contains(&format!("name={root}")));
            assert!(text.contains("cleanup-only-release="));
            assert_eq!(text.contains("possible finish protocol:"), finish);
            assert!(!text.contains("Ok(Report"));
            assert!(!text.ends_with("Error(SensorError(3))\n"));
        }
    }
}

#[test]
fn m3_legacy_and_canonical_paths_preserve_both_commands() {
    let scratch = Scratch::new("identical");
    for (name, root, expected, profile) in [
        ("ordinary_run", "loop_demo", "Percent(0)\n", false),
        ("trusted_sensor", "sample", "Ok(Reading(42))\n", true),
    ] {
        let bytes = source(name).into_bytes();
        let canonical = scratch.write(&format!("{name}.bow"), &bytes);
        let legacy = scratch.write(&format!("{name}.klx"), &bytes);
        assert_eq!(fs::read(&canonical).unwrap(), fs::read(&legacy).unwrap());
        let options = if profile {
            selected_options(root, "sample")
        } else {
            vec!["--function", root]
        };
        for binary in COMMANDS {
            let current = invoke(binary, "run", &canonical, &options);
            let old = invoke(binary, "run", &legacy, &options);
            assert_result(&current, expected);
            assert_result(&old, expected);
            assert_eq!(current.stdout, old.stdout);
            assert_eq!(current.stderr, old.stderr);
            if profile {
                let current = invoke(binary, "audit", &canonical, &options);
                let old = invoke(binary, "audit", &legacy, &options);
                assert_eq!(current.status.code(), Some(0));
                assert_eq!(old.status.code(), Some(0));
                assert_eq!(current.stdout, old.stdout);
                assert_eq!(current.stderr, old.stderr);
            }
        }
    }
    let bytes = source("battery_ok");
    for suffix in ["bow", "klx"] {
        let path = scratch.write(&format!("proof.{suffix}"), bytes.as_bytes());
        for binary in COMMANDS {
            for command in ["check", "verify"] {
                let output = invoke(binary, command, &path, &[]);
                assert_eq!(output.status.code(), Some(0));
                assert!(output.stderr.is_empty());
                let text = String::from_utf8(output.stdout).unwrap();
                assert!(text.contains(path.to_str().unwrap()));
                assert!(text.contains("function bodies proven: 1"));
                assert!(text.contains("calls checked: 1"));
            }
        }
    }
}

#[test]
fn m3_same_stem_and_other_explicit_paths_select_actual_bytes() {
    let scratch = Scratch::new("actual-path");
    let current = scratch.write("same.bow", b"fn demo()->bool{return true;}");
    let legacy = scratch.write("same.klx", b"fn demo()->bool{return false;}");
    let other = scratch.write("same.source", b"fn demo()->bool{return false;}");
    let no_suffix = scratch.write("plain", b"fn demo()->bool{return true;}");
    for binary in COMMANDS {
        for (path, expected) in [
            (&current, "Bool(true)\n"),
            (&legacy, "Bool(false)\n"),
            (&other, "Bool(false)\n"),
            (&no_suffix, "Bool(true)\n"),
        ] {
            assert_result(
                &invoke(binary, "run", path, &["--function", "demo"]),
                expected,
            );
        }
    }
    fs::remove_file(&legacy).unwrap();
    for binary in COMMANDS {
        let missing = invoke(binary, "run", &legacy, &["--function", "demo"]);
        assert_eq!(missing.status.code(), Some(2));
        assert!(missing.stdout.is_empty());
        assert_eq!(
            missing.stderr,
            b"error: detected host/tool failure: source input\n"
        );
    }
}

#[test]
fn m3_missing_paths_and_diagnostic_spans_remain_honest() {
    let scratch = Scratch::new("diagnostics");
    let invalid = b"fn bad() -> bool {\n    return missing;\n}\n";
    let current = scratch.write("invalid.bow", invalid);
    let legacy = scratch.write("invalid.klx", invalid);
    assert_eq!(current.as_os_str().len(), legacy.as_os_str().len());
    for binary in COMMANDS {
        for command in ["check", "verify"] {
            let current_output = invoke(binary, command, &current, &[]);
            let legacy_output = invoke(binary, command, &legacy, &[]);
            for (path, output) in [(&current, &current_output), (&legacy, &legacy_output)] {
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                let text = std::str::from_utf8(&output.stderr).unwrap();
                assert!(text.contains("error: unknown value `missing`"));
                assert!(text.contains(&format!("--> {}:2:12", path.display())));
                assert!(text.contains("return missing;"));
            }
            let text = String::from_utf8(current_output.stderr).unwrap();
            let from = current.to_str().unwrap();
            let to = legacy.to_str().unwrap();
            assert_eq!(text.matches(from).count(), 1);
            // Only this exact supplied-path occurrence may differ. No diagnostic
            // cause, source position, failure or authority field is normalized.
            assert_eq!(text.replacen(from, to, 1).as_bytes(), legacy_output.stderr);
        }
        for suffix in ["bow", "klx"] {
            let missing = scratch.0.join(format!("missing.{suffix}"));
            let output = invoke(binary, "verify", &missing, &[]);
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            let text = String::from_utf8(output.stderr).unwrap();
            assert!(text.contains("could not read"));
            assert!(text.contains(missing.to_str().unwrap()));
        }
    }
}

#[test]
fn m3_equal_length_suffixes_preserve_actual_byte_neighbors() {
    let scratch = Scratch::new("bytes");
    let bytes = source("ordinary_run").into_bytes();
    for suffix in ["bow", "klx"] {
        let path = scratch.write(&format!("result.{suffix}"), &bytes);
        for binary in COMMANDS {
            for (budget, status) in [("10", 0), ("9", 1)] {
                let output = invoke(
                    binary,
                    "run",
                    &path,
                    &["--function", "loop_demo", "--max-output-bytes", budget],
                );
                assert_eq!(output.status.code(), Some(status));
                if status == 0 {
                    assert_result(&output, "Percent(0)\n");
                } else {
                    assert!(output.stdout.is_empty());
                    assert_eq!(
                        output.stderr,
                        b"error: diagnostic rendering allowance exhausted [truncated]\n"
                    );
                }
            }
            let exact = bytes.len().to_string();
            let insufficient = (bytes.len() - 1).to_string();
            assert_result(
                &invoke(
                    binary,
                    "run",
                    &path,
                    &["--function", "loop_demo", "--max-source-bytes", &exact],
                ),
                "Percent(0)\n",
            );
            let short = invoke(
                binary,
                "run",
                &path,
                &[
                    "--function",
                    "loop_demo",
                    "--max-source-bytes",
                    &insufficient,
                ],
            );
            assert_eq!(short.status.code(), Some(1));
            assert!(short.stdout.is_empty());
            assert!(String::from_utf8(short.stderr)
                .unwrap()
                .contains("source bytes"));
        }
    }
}
