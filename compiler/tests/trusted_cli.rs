use std::process::{Command, Output};
fn run(command: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bowfin"))
        .args([
            command,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../examples/trusted_sensor.bow"
            ),
            "--function",
            "sample",
        ])
        .args(extra)
        .output()
        .unwrap()
}
#[test]
fn explicit_profile_executes_all_five_fixed_outcomes_as_source_results() {
    for (scenario, result) in [
        ("sample", "Ok(Reading(42))"),
        ("no-sensor", "Error(SensorError(1))"),
        ("open-io-failure", "Error(SensorError(3))"),
        ("no-sample", "Error(SensorError(2))"),
        ("read-io-failure", "Error(SensorError(3))"),
    ] {
        let output = run("run", &["--profile", "sensor-demo", "--scenario", scenario]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{result}\n")
        );
    }
}
#[test]
fn profile_is_never_implicit_and_io_does_not_grant_denied_read() {
    for options in [
        &[][..],
        &["--profile", "sensor-demo", "--grants", "open-release"],
    ] {
        let output = run("run", options);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
    }
}
#[test]
fn audit_reports_roots_grants_pairing_and_origins_without_execution() {
    let output = run(
        "audit",
        &[
            "--profile",
            "sensor-demo",
            "--scenario",
            "no-sample",
            "--sensor-limit",
            "2",
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.starts_with(&format!(
        "authority account v1\ntarget=bowfin.ordinary-mir.v1\ncompiler=bowfin-compiler/{}\ncatalog=bowfin.sensor-demo.catalog.v1\nprofile=bowfin.sensor-demo.profile.v1\nassumptions=bowfin.sensor-demo.assumptions.v1\n",
        env!("CARGO_PKG_VERSION")
    )));
    assert!(!text.contains("klyxr.sensor-demo."));
    assert!(!text.contains("klyxr.ordinary-mir."));
    assert!(!text.contains("klyxr-compiler/"));
    for required in [
        "root: function=0 name=sample",
        "scenario=NoSample; live-sensor-limit=2",
        "binding=SensorOpenV1",
        "binding=SensorReadV1",
        "cleanup-only-release=",
        "possible cleanup:",
        "source=Span",
        "only enumerated binding outcomes",
    ] {
        assert!(text.contains(required), "{required}");
    }
    assert!(!text.contains("Error(SensorError(2))"));
}
#[test]
fn invalid_profile_configuration_is_class_two_and_limits_are_terminal_class_one() {
    for options in [
        &["--profile", "unknown"][..],
        &["--scenario", "sample"],
        &["--profile", "sensor-demo", "--scenario", "unknown"],
        &["--profile", "sensor-demo", "--grants", "unknown"],
        &["--profile", "sensor-demo", "--sensor-limit", "1025"],
    ] {
        let output = run("run", options);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    for options in [
        &["--profile", "sensor-demo", "--max-steps", "1"][..],
        &["--profile", "sensor-demo", "--max-working-storage", "1"],
    ] {
        let output = run("run", options);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8(output.stderr)
            .unwrap()
            .contains("Error(SensorError"));
    }
    let output = run(
        "audit",
        &["--profile", "sensor-demo", "--max-output-bytes", "1"],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}
