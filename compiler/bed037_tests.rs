//! Cross-module BED-037 evidence at the actual reporter and operation gates.
use crate::execution::{self, AdmissionLimits, ExecutionLimits, GatedError};
use crate::lint_reporting::{self, Fault, Outcome, Settings, Severity};
use crate::source_facts::SourceFacts;
use crate::trusted::{Grants, Scenario, Selection};
use std::io::{self, Write};

const ORDINARY: &str = "fn demo()->bool{return true;}";
const SELECTED: &str = "fn demo()->Result<success:Reading,failure:SensorError>effects io{let sensor=sensor_open()or return;let reading=sensor_read(&sensor)or return;return Ok(reading);}";
fn gate(facts: &SourceFacts, settings: Settings, writer: impl Write) -> Result<(), Outcome> {
    let outcome = lint_reporting::report(facts, "example.bow", settings, writer);
    if outcome.exit_code().is_some() {
        Err(outcome)
    } else {
        Ok(())
    }
}
fn prepared(source: &str, selection: Option<&Selection>) -> (execution::CheckedExecutable, SourceFacts) {
    execution::prepare_execution_from_reader_with_facts(
        source.as_bytes(),
        AdmissionLimits::default(),
        selection,
    )
    .unwrap()
    .into_parts()
}

#[test]
fn denied_and_faulted_warning_gate_never_start_an_otherwise_executable_machine() {
    let (artifact, facts) = prepared(ORDINARY, None);
    let entry = artifact.entry("demo").unwrap();
    for fault in [None, Some(Fault::Sizing), Some(Fault::Allocation), Some(Fault::Rendering), Some(Fault::Mismatch)] {
        execution::reset_execution_observations();
        let action = || {
            execution::execute_with_gate(&artifact, entry, ExecutionLimits::default(), || {
                let settings = Settings {
                    severity: if fault.is_none() { Severity::Deny } else { Severity::Warn },
                    ..Settings::default()
                };
                gate(&facts, settings, Vec::new())
            })
        };
        let result = match fault {
            Some(fault) => lint_reporting::with_fault(fault, action),
            None => action(),
        };
        assert!(matches!(result, Err(GatedError::Gate(_))));
        assert_eq!(execution::execution_observations(), (1, 0));
    }
    for severity in [Severity::Allow, Severity::Warn] {
        execution::reset_execution_observations();
        let result = execution::execute_with_gate(&artifact, entry, ExecutionLimits::default(), || {
            gate(&facts, Settings { severity, ..Settings::default() }, Vec::new())
        });
        assert!(result.is_ok());
        assert_eq!(execution::execution_observations(), (1, 1));
    }
}

#[test]
fn selected_real_session_retains_zero_events_at_denial_and_report_failure() {
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    let (artifact, facts) = prepared(SELECTED, Some(&selection));
    for fault in [None, Some(Fault::Sizing), Some(Fault::Allocation), Some(Fault::Rendering), Some(Fault::Mismatch)] {
        execution::reset_execution_observations();
        let session = execution::ExecutionSession::new(&artifact, artifact.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
        let action = || gate(&facts, Settings { severity: if fault.is_none() { Severity::Deny } else { Severity::Warn }, ..Settings::default() }, Vec::new());
        let result = match fault {
            Some(fault) => lint_reporting::with_fault(fault, action),
            None => action(),
        };
        assert!(result.is_err());
        assert!(session.events().is_empty());
        assert_eq!(session.live_sensors(), 0);
        assert_eq!(execution::execution_observations(), (1, 0));
    }
    for severity in [Severity::Allow, Severity::Warn] {
        execution::reset_execution_observations();
        let mut session = execution::ExecutionSession::new(&artifact, artifact.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
        gate(&facts, Settings { severity, ..Settings::default() }, Vec::new()).unwrap();
        assert!(session.advance().is_ok());
        assert!(!session.events().is_empty());
        assert!(session.events().iter().any(|event| format!("{event:?}").starts_with("Attempt")));
        assert_eq!(session.live_sensors(), 0);
        assert_eq!(execution::execution_observations(), (1, 1));
    }
}

struct BrokenWriter {
    accepted: usize,
    fail_at: usize,
    fail_flush: bool,
}
impl Write for BrokenWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.accepted >= self.fail_at {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "test writer"));
        }
        let n = bytes.len().min(self.fail_at - self.accepted);
        self.accepted += n;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "test flush"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn partial_final_lf_and_flush_failures_prevent_real_ordinary_and_trusted_activity() {
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    for selected in [false, true] {
        let (artifact, facts) = prepared(if selected { SELECTED } else { ORDINARY }, if selected { Some(&selection) } else { None });
        let mut expected = Vec::new();
        gate(&facts, Settings::default(), &mut expected).unwrap();
        for (fail_at, fail_flush) in [(0, false), (3, false), (expected.len() - 1, false), (usize::MAX, true)] {
            execution::reset_execution_observations();
            let writer = BrokenWriter { accepted: 0, fail_at, fail_flush };
            if selected {
                let session = execution::ExecutionSession::new(&artifact, artifact.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
                assert_eq!(gate(&facts, Settings::default(), writer), Err(Outcome::HostReportingFailure));
                assert!(session.events().is_empty());
                assert_eq!(session.live_sensors(), 0);
            } else {
                let result = execution::execute_with_gate(&artifact, artifact.entry("demo").unwrap(), ExecutionLimits::default(), || gate(&facts, Settings::default(), writer));
                assert!(matches!(result, Err(GatedError::Gate(Outcome::HostReportingFailure))));
            }
            assert_eq!(execution::execution_observations(), (1, 0));
        }
    }
}

#[test]
fn allow_and_no_finding_leave_all_faulted_reporting_seams_unused_and_execute() {
    for (source, severity) in [(ORDINARY, Severity::Allow), ("// comment\nfn demo()->bool{return true;}", Severity::Deny)] {
        let (artifact, facts) = prepared(source, None);
        for fault in [Fault::Sizing, Fault::Allocation, Fault::Rendering, Fault::Mismatch] {
            execution::reset_execution_observations();
            let result = lint_reporting::with_fault(fault, || {
                execution::execute_with_gate(&artifact, artifact.entry("demo").unwrap(), ExecutionLimits::default(), || gate(&facts, Settings { severity, report_bytes: 1 }, BrokenWriter { accepted: 0, fail_at: 0, fail_flush: true }))
            });
            assert!(result.is_ok());
            assert_eq!(execution::execution_observations(), (1, 1));
        }
    }
}

#[test]
fn audit_preflight_gate_and_later_reporting_remain_distinct_and_nonexecuting() {
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    let (artifact, facts) = prepared(SELECTED, Some(&selection));
    execution::reset_execution_observations();
    let early = artifact.render_authority_with_gate(ExecutionLimits { working_storage: 1, ..ExecutionLimits::default() }, || -> Result<(), Outcome> { panic!("primary failure must not invoke reporter") });
    assert!(matches!(early, Err(GatedError::Primary(_))));
    let denied = artifact.render_authority_with_gate(ExecutionLimits::default(), || gate(&facts, Settings { severity: Severity::Deny, ..Settings::default() }, Vec::new()));
    assert!(matches!(denied, Err(GatedError::Gate(Outcome::ReportedDenial))));
    let mut warning = Vec::new();
    let late = artifact.render_authority_with_gate(ExecutionLimits { rendered_output: 1, ..ExecutionLimits::default() }, || gate(&facts, Settings::default(), &mut warning));
    assert!(matches!(late, Err(GatedError::Operation(_))));
    assert!(warning.starts_with(b"warning[commentless-source]"));
    let report = artifact.render_authority_with_gate(ExecutionLimits::default(), || gate(&facts, Settings::default(), Vec::new())).unwrap();
    assert!(report.starts_with("authority account v1\n"));
    assert_eq!(execution::execution_observations(), (0, 0));
}
