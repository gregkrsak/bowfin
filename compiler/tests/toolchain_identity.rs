//! M2 command compatibility: independent outcomes plus exact stream comparison.
use std::process::{Command, Output};

const COMMANDS: [&str; 2] = [env!("CARGO_BIN_EXE_bowfin"), env!("CARGO_BIN_EXE_klyxr")];

fn invoke(binary: &str, args: &[&str]) -> Output {
    Command::new(binary).args(args).output().unwrap()
}

fn equivalent(args: &[&str], status: i32) -> Output {
    let canonical = invoke(COMMANDS[0], args);
    let compatibility = invoke(COMMANDS[1], args);
    assert_eq!(canonical.status.code(), Some(status), "{args:?}");
    assert_eq!(compatibility.status.code(), Some(status), "{args:?}");
    assert_eq!(canonical.stdout, compatibility.stdout, "{args:?}");
    assert_eq!(canonical.stderr, compatibility.stderr, "{args:?}");
    canonical
}

fn example(name: &str) -> String {
    format!("{}/../examples/{name}", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn m2_version_and_usage_identify_canonical_and_compatibility_commands() {
    for (binary, name, version, role) in [
        (
            COMMANDS[0],
            "bowfin",
            "bowfin 0.0.1 (vertical-slice prototype)\n",
            "",
        ),
        (
            COMMANDS[1],
            "klyxr",
            "klyxr 0.0.1 (Bowfin compatibility command; vertical-slice prototype)\n",
            "klyxr is the compatibility command for bowfin\n",
        ),
    ] {
        for flag in ["--version", "-V"] {
            let output = invoke(binary, &[flag]);
            assert_eq!(output.status.code(), Some(0));
            assert_eq!(output.stdout, version.as_bytes());
            assert!(output.stderr.is_empty());
        }
        for (args, prefix, usage) in [
            (
                vec![],
                "",
                format!("usage: {name} <check|verify> <file.bow>\n"),
            ),
            (
                vec!["verify"],
                "error: missing .bow input path\n",
                format!("usage: {name} <check|verify> <file.bow>\n"),
            ),
            (
                vec!["--help"],
                "error: unknown command `--help`\n",
                format!("usage: {name} <check|verify> <file.bow>\n"),
            ),
            (
                vec!["run"],
                "",
                format!("usage: {name} run <file.bow> --function <name> [limit overrides]\n"),
            ),
        ] {
            let output = invoke(binary, &args);
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert_eq!(output.stderr, format!("{prefix}{usage}{role}").as_bytes());
        }
    }
    // Version argument rejection does not emit the compatibility-role banner.
    let output = equivalent(&["--version", "extra"], 2);
    assert_eq!(output.stderr, b"error: too many arguments\n");
}

#[test]
fn m2_normal_check_verify_and_run_streams_are_identical() {
    let path = example("battery_ok.bow");
    for (command, verb) in [("check", "checked"), ("verify", "verified")] {
        let output = equivalent(&[command, &path], 0);
        assert!(output.stderr.is_empty());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("{verb} supported integer contracts: {path}\n  function bodies proven: 1 (subtraction safety, field ranges, postconditions)\n  calls checked: 1 (argument ranges, mutable access, preconditions)\n  scope: ordinary control flow with loop-stable Copy state and iteration-local Move/borrowing plus stable carried references and specialized signed i64 contract prototype; general ownership, effects, and code generation are not implemented\n"));
    }
    let path = example("ordinary_run.bow");
    let output = equivalent(&["run", &path, "--function", "loop_demo"], 0);
    assert_eq!(output.stdout, b"Percent(0)\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn m2_selected_results_and_audit_preserve_m1_authority() {
    let path = example("trusted_sensor.bow");
    for (scenario, expected) in [
        ("sample", "Ok(Reading(42))\n"),
        ("no-sensor", "Error(SensorError(1))\n"),
        ("no-sample", "Error(SensorError(2))\n"),
        ("read-io-failure", "Error(SensorError(3))\n"),
    ] {
        let output = equivalent(
            &[
                "run",
                &path,
                "--function",
                "sample",
                "--profile",
                "sensor-demo",
                "--scenario",
                scenario,
            ],
            0,
        );
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    let output = equivalent(
        &[
            "audit",
            &path,
            "--function",
            "sample",
            "--profile",
            "sensor-demo",
        ],
        0,
    );
    assert!(output.stderr.is_empty());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with("authority account v1\ntarget=bowfin.ordinary-mir.v1\ncompiler=bowfin-compiler/0.0.1\ncatalog=bowfin.sensor-demo.catalog.v1\nprofile=bowfin.sensor-demo.profile.v1\nassumptions=bowfin.sensor-demo.assumptions.v1\n"));
    assert!(text.contains("root: function=0 name=sample"));
    assert!(text.contains("cleanup-only-release="));
    assert!(!text.contains("klyxr."));
    assert!(!text.contains("compatibility command"));
    for extra in [
        &[][..],
        &["--profile", "sensor-demo", "--grants", "open-release"],
    ] {
        let mut args = vec!["run", &path, "--function", "sample"];
        args.extend(extra);
        let output = equivalent(&args, 1);
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[test]
fn m2_failures_and_option_validation_preserve_results() {
    let path = example("ordinary_run.bow");
    for (args, status, message) in [
        (
            vec!["run", &path, "--function", "trap_demo"],
            1,
            "computed -101",
        ),
        (
            vec!["run", &path, "--function", "demo", "--max-steps", "1"],
            1,
            "configured tool limit exhausted",
        ),
        (
            vec!["run", &path, "--function", "demo", "--max-steps", "0"],
            2,
            "configured run limit is zero or exceeds supported maximum",
        ),
        (
            vec![
                "run",
                &path,
                "--function",
                "demo",
                "--max-steps",
                "1",
                "--max-steps",
                "2",
            ],
            2,
            "invalid or repeated run option",
        ),
        (
            vec!["run", &path, "--function", "demo", "--profile", "unknown"],
            2,
            "unknown profile",
        ),
        (
            vec!["run", &path, "--function", "demo", "--scenario", "sample"],
            2,
            "profile options and audit require --profile sensor-demo",
        ),
        (
            vec!["run", "/a/nonexistent/bowfin-input", "--function", "demo"],
            2,
            "detected host/tool failure: source input",
        ),
    ] {
        let output = equivalent(&args, status);
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr).unwrap().contains(message));
    }
    let bad = example("battery_fail.bow");
    let output = equivalent(&["verify", &bad], 1);
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("90 <= 80 is false"));
}

#[test]
fn m2_actual_output_byte_neighbors_are_command_independent() {
    let path = example("ordinary_run.bow");
    let expected = b"Percent(0)\n";
    // The existing allowance counts rendered text; write_output appends a newline.
    let sufficient = (expected.len() - 1).to_string();
    let insufficient = (expected.len() - 2).to_string();
    let output = equivalent(
        &[
            "run",
            &path,
            "--function",
            "loop_demo",
            "--max-output-bytes",
            &sufficient,
        ],
        0,
    );
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
    let output = equivalent(
        &[
            "run",
            &path,
            "--function",
            "loop_demo",
            "--max-output-bytes",
            &insufficient,
        ],
        1,
    );
    assert!(output.stdout.is_empty());
    assert_eq!(
        output.stderr,
        b"error: diagnostic rendering allowance exhausted [truncated]\n"
    );

    let path = example("trusted_sensor.bow");
    let args = [
        "audit",
        &path,
        "--function",
        "sample",
        "--profile",
        "sensor-demo",
    ];
    let complete = equivalent(&args, 0);
    assert!(complete.stderr.is_empty());
    assert!(complete
        .stdout
        .starts_with(b"authority account v1\ntarget=bowfin.ordinary-mir.v1\n"));
    let sufficient = (complete.stdout.len() - 1).to_string();
    let insufficient = (complete.stdout.len() - 2).to_string();
    let mut bounded = args.to_vec();
    bounded.extend(["--max-output-bytes", &sufficient]);
    let exact = equivalent(&bounded, 0);
    assert_eq!(exact.stdout, complete.stdout);
    assert!(exact.stderr.is_empty());
    *bounded.last_mut().unwrap() = &insufficient;
    let short = equivalent(&bounded, 1);
    assert!(short.stdout.is_empty());
    assert!(String::from_utf8(short.stderr)
        .unwrap()
        .contains("configured tool limit exhausted: rendered output"));
}

#[test]
fn m2_cargo_metadata_selects_canonical_and_retains_alias() {
    assert_eq!(env!("CARGO_PKG_NAME"), "bowfin-compiler");
    assert_eq!(env!("CARGO_CRATE_NAME"), "toolchain_identity");
    assert_eq!(env!("CARGO_PKG_VERSION"), "0.0.1");
    assert_ne!(COMMANDS[0], COMMANDS[1]);
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("publish = false\ndefault-run = \"bowfin\""));
    assert!(manifest.contains("name = \"bowfin_compiler\"\npath = \"lib.rs\""));
    assert!(manifest.contains("name = \"bowfin\"\npath = \"main.rs\""));
    assert!(manifest.contains("name = \"klyxr\"\npath = \"compat_main.rs\""));
    assert_eq!(include_str!("../compat_main.rs"), "// The retained command compiles the same CLI, including reporting and dispatch.\ninclude!(\"main.rs\");\n");
    // The verify workflow also invokes Cargo's actual unqualified selection.
    // No old Cargo package or Rust-crate compatibility facade is promised.
    assert!(!manifest.contains("name = \"klyxr-compiler\""));
    assert!(!manifest.contains("name = \"klyxr_compiler\""));
}
