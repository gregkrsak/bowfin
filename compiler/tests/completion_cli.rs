use std::process::{Command, Output};
fn run(command: &str, file: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bowfin"))
        .args([
            command,
            &format!("{}/../examples/{file}", env!("CARGO_MANIFEST_DIR")),
            "--function",
            "sample",
        ])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn defining_sample_cli_delivers_success_and_both_completion_error_scenarios() {
    for (scenario, expected) in [
        (
            "sample",
            "Ok(Sample { reading: Reading(42), finished: FinishReceipt(1) })",
        ),
        ("finish-io-failure", "Error(SensorError(3))"),
        ("finish-partial-io-failure", "Error(SensorError(3))"),
    ] {
        let output = run(
            "run",
            "sensor_finish.bow",
            &["--profile", "sensor-demo", "--scenario", scenario],
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{expected}\n")
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn completion_cli_preserves_profile_grants_and_discriminating_rejections() {
    for file in [
        "sensor_finish_live_loan_fail.bow",
        "sensor_finish_reuse_fail.bow",
    ] {
        let output = run("run", file, &["--profile", "sensor-demo"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
    for options in [
        &[][..],
        &["--profile", "sensor-demo", "--grants", "open-release"],
        &[
            "--profile",
            "sensor-demo",
            "--grants",
            "open-finish-release",
        ],
        &["--profile", "sensor-demo", "--grants", "finish-release"],
    ] {
        let output = run("run", "sensor_finish.bow", options);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn completion_cli_audit_reports_static_protocol_without_executing_a_scenario() {
    let output = run(
        "audit",
        "sensor_finish.bow",
        &[
            "--profile",
            "sensor-demo",
            "--scenario",
            "finish-partial-io-failure",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("binding=SensorFinishV1"));
    assert!(text.contains("possible finish protocol:"));
    assert!(text.contains("not proof or authority"));
    assert!(!text.ends_with("Error(SensorError(3))\n"));
}
