use std::process::{Command, Output};

fn invoke(command: &str, scenario: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bowfin"))
        .args([
            command,
            concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/sentinel.bow"),
            "--function",
            "monitor",
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
        ])
        .args(extra)
        .output()
        .unwrap()
}

#[test]
fn sentinel_cli_success_matrix_matches_the_application_report_and_host_rendering() {
    for (scenario, warnings, critical, peak, disposition) in [
        ("sample", 0, 0, 42, 0),
        ("warning", 1, 0, 60, 1),
        ("critical", 1, 1, 90, 2),
    ] {
        let output = invoke("run", scenario, &[]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("Ok(Report {{ observations: Count(2), warnings: Count({warnings}), critical: Count({critical}), peak: Reading({peak}), disposition: Disposition({disposition}), completed: Count(2) }})\n"));
    }
}

#[test]
fn sentinel_cli_source_failure_is_typed_outer_error_even_with_exit_zero() {
    for (scenario, phase, problem, read, completion, observations, peak, completed) in [
        ("no-sensor", 1, 1, 0, 0, 0, 0, 0),
        ("open-io-failure", 1, 3, 0, 0, 0, 0, 0),
        ("no-sample", 2, 2, 2, 1, 0, 0, 1),
        ("read-io-failure", 2, 3, 3, 1, 0, 0, 1),
        ("finish-io-failure", 3, 3, 0, 2, 1, 42, 0),
        ("finish-partial-io-failure", 3, 3, 0, 2, 1, 42, 0),
    ] {
        let output = invoke("run", scenario, &[]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), format!("Error(Failure {{ phase: Stage({phase}), problem: SensorError({problem}), read_status: ReadStatus({read}), completion: CompletionStatus({completion}), observations: Count({observations}), warnings: Count(0), critical: Count(0), peak: Reading({peak}), disposition: Disposition(0), completed: Count({completed}) }})\n"));
    }
}

#[test]
fn sentinel_cli_limits_and_reporting_are_terminal_not_application_failures() {
    for extra in [
        &["--max-steps", "1"][..],
        &["--max-output-bytes", "1"],
        &["--max-working-storage", "1"],
    ] {
        let output = invoke("run", "warning", extra);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.starts_with("error:"));
        assert!(!error.contains("Error(Failure"));
    }
}

#[test]
fn sentinel_cli_audit_is_static_selected_authority_and_repeated_runs_are_deterministic() {
    for scenario in ["warning", "critical"] {
        let first = invoke("run", scenario, &[]);
        let second = invoke("run", scenario, &[]);
        assert!(first.status.success() && second.status.success());
        assert_eq!(first.stdout, second.stdout);
        let a = invoke("audit", scenario, &[]);
        let b = invoke("audit", scenario, &[]);
        assert!(a.status.success() && b.status.success());
        assert_eq!(a.stdout, b.stdout);
        let text = String::from_utf8(a.stdout).unwrap();
        assert!(text.contains(if scenario == "warning" {
            "scenario=Warning"
        } else {
            "scenario=Critical"
        }));
        assert!(text.contains("live-sensor-limit=1"));
        assert!(text.contains("possible finish protocol:"));
        assert!(text.contains("binding=SensorFinishV1"));
        assert!(!text.contains("Ok(Report"));
        assert!(!text.contains("Error(Failure"));
    }
}
