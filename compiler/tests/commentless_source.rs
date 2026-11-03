//! BED-037 CLI conformance: explicit policy, primary precedence and real commands.
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

const ORDINARY: &str = "fn demo()->bool{return true;}";
const SELECTED: &str = "fn demo()->Result<success:Reading,failure:SensorError>effects io{let sensor=sensor_open()or return;let reading=sensor_read(&sensor)or return;return Ok(reading);}";
const BINARIES: [&str; 2] = [env!("CARGO_BIN_EXE_bowfin"), env!("CARGO_BIN_EXE_klyxr")];

struct Input(PathBuf);
impl Input {
    fn new(source: &str, suffix: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "bowfin-bed037-{}-{}-{suffix}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::write(&path, source).unwrap();
        Self(path)
    }
    fn path(&self) -> &str {
        self.0.to_str().unwrap()
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
fn invoke(binary: &str, command: &str, input: &Input, options: &[&str]) -> Output {
    Command::new(binary)
        .args([command, input.path()])
        .args(options)
        .output()
        .unwrap()
}
fn warning(path: &str, line: usize, column: usize) -> Vec<u8> {
    format!("warning[commentless-source]: this source file contains no comments\n  --> {path}:{line}:{column}\nhelp: explain the file's purpose, domain meaning, or important assumptions\n").into_bytes()
}
fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

#[test]
fn default_and_explicit_levels_cover_both_executables_and_static_ordinary_commands() {
    let input = Input::new(ORDINARY, "generated.vendor.klx");
    for binary in BINARIES {
        for command in ["check", "verify", "run"] {
            let entry = if command == "run" { vec!["--function", "demo"] } else { vec![] };
            for setting in [vec![], vec!["--warn", "commentless-source"], vec!["--allow", "commentless-source"], vec!["--deny", "commentless-source"]] {
                let mut options = entry.clone();
                options.extend(&setting);
                let output = invoke(binary, command, &input, &options);
                if setting.first() == Some(&"--deny") {
                    assert_eq!(output.status.code(), Some(1));
                    assert!(output.stdout.is_empty());
                    assert!(stderr(&output).starts_with("error[commentless-source]: documentation policy"));
                    assert!(!stderr(&output).contains("verification stopped"));
                } else {
                    assert!(output.status.success(), "{}", stderr(&output));
                    if setting.first() == Some(&"--allow") {
                        assert!(output.stderr.is_empty());
                    } else {
                        assert_eq!(output.stderr, warning(input.path(), 1, 1));
                    }
                }
            }
        }
    }
}

#[test]
fn selected_run_and_audit_enforce_levels_on_both_executables() {
    let input = Input::new(SELECTED, "selected.bow");
    for binary in BINARIES {
        for command in ["run", "audit"] {
            for level in ["--allow", "--warn", "--deny"] {
                let output = invoke(binary, command, &input, &["--function", "demo", "--profile", "sensor-demo", level, "commentless-source"]);
                if level == "--deny" {
                    assert_eq!(output.status.code(), Some(1));
                    assert!(output.stdout.is_empty());
                    assert!(stderr(&output).starts_with("error[commentless-source]"));
                } else {
                    assert!(output.status.success(), "{}", stderr(&output));
                    if level == "--allow" {
                        assert!(output.stderr.is_empty());
                    } else {
                        assert_eq!(output.stderr, warning(input.path(), 1, 1));
                    }
                    let text = String::from_utf8(output.stdout).unwrap();
                    if command == "run" {
                        assert_eq!(text, "Ok(Reading(42))\n");
                    } else {
                        assert!(text.starts_with("authority account v1\n"));
                    }
                }
            }
        }
    }
}

#[test]
fn parsed_unit_forms_are_eligible_without_requiring_execution() {
    for source in [
        "type Percent=range 0..100;",
        "type Percent=range 0..100;record Battery{charge:Percent}",
        ORDINARY,
        include_str!("../../examples/battery_ok.bow"),
    ] {
        let input = Input::new(source, "forms.bow");
        for command in ["check", "verify"] {
            let output = invoke(BINARIES[0], command, &input, &[]);
            assert!(output.status.success(), "{}", stderr(&output));
            assert_eq!(output.stderr, warning(input.path(), 1, 1));
        }
    }
}

#[test]
fn empty_whitespace_comment_only_and_every_recognized_comment_have_no_finding() {
    for source in ["", " \t\r\n", "//", "///", "//!", "// license\n", "//\nfn demo()->bool{return true;}", "fn demo()->bool{// inline\nreturn true;}", "fn demo()->bool{return true;}// eof", "fn demo()->bool{return true;}//   "] {
        let input = Input::new(source, "comments.bow");
        for binary in BINARIES {
            let output = invoke(binary, "check", &input, &["--deny", "commentless-source", "--max-lint-report-bytes", "1"]);
            assert!(output.status.success(), "{}", stderr(&output));
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn authoritative_locations_preserve_tabs_crlf_and_alias_spelling() {
    for source in ["\n\t  fn demo()->bool{return true;}", "\r\n\t  function demo()->bool{return true;}"] {
        let input = Input::new(source, "location.bow");
        let output = invoke(BINARIES[0], "check", &input, &[]);
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(output.stderr, warning(input.path(), 2, 4));
    }
}

#[test]
fn usage_rejects_duplicates_conflicts_missing_unknown_and_capacity_errors() {
    let input = Input::new(ORDINARY, "options.bow");
    let cases: &[&[&str]] = &[
        &["--warn"], &["--warn", "unknown"], &["--warn", "--function"],
        &["--warn", "commentless-source", "--warn", "commentless-source"],
        &["--allow", "commentless-source", "--deny", "commentless-source"],
        &["--warn=commentless-source"], &["--max-lint-report-bytes"],
        &["--max-lint-report-bytes", "0"], &["--max-lint-report-bytes", "1048577"],
        &["--max-lint-report-bytes", "184467440737095516160"],
        &["--max-lint-report-bytes", "+1"], &["--max-lint-report-bytes", "1", "--max-lint-report-bytes", "1"],
    ];
    for binary in BINARIES {
        for command in ["check", "verify", "run", "audit"] {
            for case in cases {
                let mut options = if command == "run" || command == "audit" { vec!["--function", "demo"] } else { vec![] };
                options.extend_from_slice(case);
                let output = invoke(binary, command, &input, &options);
                assert_eq!(output.status.code(), Some(2), "{command} {case:?}: {}", stderr(&output));
                assert!(output.stdout.is_empty());
                assert!(!stderr(&output).contains("warning[commentless-source]"));
            }
        }
    }
}

#[test]
fn exact_complete_record_neighbors_include_final_lf_and_large_ceiling() {
    let input = Input::new(ORDINARY, "neighbors.bow");
    let expected = warning(input.path(), 1, 1);
    for limit in [expected.len(), 65536, 1048576] {
        let n = limit.to_string();
        let output = invoke(BINARIES[0], "run", &input, &["--function", "demo", "--max-lint-report-bytes", &n]);
        assert!(output.status.success(), "{}", stderr(&output));
        assert_eq!(output.stderr, expected);
        assert_eq!(output.stdout, b"Bool(true)\n");
    }
    for limit in [1, expected.len() - 1] {
        let n = limit.to_string();
        let output = invoke(BINARIES[0], "run", &input, &["--function", "demo", "--max-lint-report-bytes", &n]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(output.stderr, b"\nerror: commentless-source reporting limit exhausted; command stopped\n");
    }
}

#[test]
fn primary_frontend_and_proof_errors_override_deny_and_insufficient_lint_capacity() {
    for source in ["@", "fn", "/* unsupported */", "fn demo()->bool{return missing;}", "fn demo()->bool{return 2;}", include_str!("../../examples/battery_fail.bow")] {
        let input = Input::new(source, "primary.bow");
        for command in ["check", "verify"] {
            let output = invoke(BINARIES[0], command, &input, &["--deny", "commentless-source", "--max-lint-report-bytes", "1"]);
            assert_eq!(output.status.code(), Some(1));
            assert!(!stderr(&output).contains("commentless-source"));
            assert!(stderr(&output).contains("error:"));
        }
    }
}

#[test]
fn bounded_source_entry_profile_and_grant_failures_precede_lint() {
    let ordinary = Input::new(ORDINARY, "admission.bow");
    for extra in [&["--function", "missing"][..], &["--function", "demo", "--max-source-bytes", "1"], &["--function", "demo", "--max-admission-storage", "1"]] {
        let mut options = extra.to_vec();
        options.extend(["--deny", "commentless-source", "--max-lint-report-bytes", "1"]);
        let output = invoke(BINARIES[0], "run", &ordinary, &options);
        assert_eq!(output.status.code(), Some(1));
        assert!(!stderr(&output).contains("commentless-source"));
    }
    let selected = Input::new(SELECTED, "grants.bow");
    for command in ["run", "audit"] {
        let output = invoke(BINARIES[0], command, &selected, &["--function", "demo", "--profile", "sensor-demo", "--grants", "none", "--deny", "commentless-source"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(!stderr(&output).contains("commentless-source"));
    }
}

#[test]
fn warning_precedes_later_runtime_and_audit_report_failures() {
    let input = Input::new("fn demo()->bool{while true{}return true;}", "runtime.bow");
    let output = invoke(BINARIES[0], "run", &input, &["--function", "demo", "--max-steps", "1"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).starts_with(std::str::from_utf8(&warning(input.path(), 1, 1)).unwrap()));
    assert!(stderr(&output).contains("error:"));
    let selected = Input::new(SELECTED, "audit.bow");
    let output = invoke(BINARIES[0], "audit", &selected, &["--function", "demo", "--profile", "sensor-demo", "--max-output-bytes", "1"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).starts_with("warning[commentless-source]"));
    assert!(stderr(&output).contains("error:"));
}

#[test]
fn independent_lint_account_preserves_ten_byte_result_boundary() {
    let input = Input::new(ORDINARY, "result.bow");
    let good = invoke(BINARIES[0], "run", &input, &["--function", "demo", "--max-output-bytes", "10"]);
    assert!(good.status.success(), "{}", stderr(&good));
    assert_eq!(good.stdout, b"Bool(true)\n");
    assert_eq!(good.stderr, warning(input.path(), 1, 1));
    let bad = invoke(BINARIES[0], "run", &input, &["--function", "demo", "--max-output-bytes", "9"]);
    assert_eq!(bad.status.code(), Some(1));
    assert!(stderr(&bad).starts_with("warning[commentless-source]"));
}

#[test]
fn check_verify_and_run_retain_distinct_existing_contracts() {
    let input = Input::new(include_str!("../../examples/battery_ok.bow"), "claims.bow");
    for (command, prefix) in [("check", "checked supported integer contracts:"), ("verify", "verified supported integer contracts:")] {
        let output = invoke(BINARIES[0], command, &input, &[]);
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().starts_with(prefix));
    }
    let input = Input::new(&format!("{}\nfn demo()->bool{{return true;}}", include_str!("../../examples/battery_fail.bow")), "harness.bow");
    let run = invoke(BINARIES[0], "run", &input, &["--function", "demo"]);
    assert!(run.status.success(), "{}", stderr(&run));
    assert_eq!(run.stdout, b"Bool(true)\n");
    let verify = invoke(BINARIES[0], "verify", &input, &[]);
    assert_eq!(verify.status.code(), Some(1));
    assert!(!stderr(&verify).contains("commentless-source"));
}

#[test]
fn ownership_and_effect_primary_failures_suppress_the_secondary_policy() {
    for source in [include_str!("../../examples/borrow_fail.bow"), include_str!("../../examples/explicit_effects_fail.bow")] {
        let commentless = source.lines().filter(|line| !line.trim_start().starts_with("//")).collect::<Vec<_>>().join("\n");
        let input = Input::new(&commentless, "semantic-primary.bow");
        for command in ["check", "verify"] {
            let output = invoke(BINARIES[0], command, &input, &["--deny", "commentless-source", "--max-lint-report-bytes", "1"]);
            assert_eq!(output.status.code(), Some(1));
            assert!(stderr(&output).contains("error:"));
            assert!(!stderr(&output).contains("commentless-source"));
        }
    }
}
