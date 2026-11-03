//! BED-037 gates are observed at real machine entry and trusted-event boundaries.
use super::*;
use crate::trusted::{Grants, Scenario, Selection};
use std::cell::Cell;

const ORDINARY: &str = "fn demo()->bool{return true;}";
const SELECTED: &str = "fn demo()->Result<success:Reading,failure:SensorError>effects io{let sensor=sensor_open()or return;return sensor_read(&sensor);}";

fn ordinary() -> CheckedExecutable {
    prepare_execution(ORDINARY, AdmissionLimits::default()).unwrap()
}

fn selected(source: &str) -> CheckedExecutable {
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    prepare_execution_with_profile(source, AdmissionLimits::default(), &selection).unwrap()
}

#[test]
fn ordinary_success_consumes_one_admission_after_one_gate_and_starts_real_machine() {
    let e = ordinary();
    let calls = Cell::new(0);
    reset_execution_observations();
    let result = execute_with_gate(&e, e.entry("demo").unwrap(), ExecutionLimits::default(), || {
        calls.set(calls.get() + 1);
        assert_eq!(execution_observations(), (1, 0));
        Ok::<(), ()>(())
    })
    .unwrap();
    assert_eq!(e.render_result(&result, 10).unwrap(), "Bool(true)");
    assert_eq!(calls.get(), 1);
    assert_eq!(execution_observations(), (1, 1));
}

#[test]
fn ordinary_denial_and_failed_report_gate_have_no_real_machine_start() {
    let e = ordinary();
    for reason in ["deny", "warning delivery failed"] {
        reset_execution_observations();
        let result = execute_with_gate(
            &e,
            e.entry("demo").unwrap(),
            ExecutionLimits::default(),
            || Err(reason),
        );
        assert!(matches!(result, Err(GatedError::Gate(message)) if message == reason));
        assert_eq!(execution_observations(), (1, 0));
    }
    reset_execution_observations();
    execute_with_gate(&e, e.entry("demo").unwrap(), ExecutionLimits::default(), || {
        Ok::<(), &str>(())
    })
    .unwrap();
    assert_eq!(execution_observations(), (1, 1));
}

#[test]
fn ordinary_primary_limit_or_entry_failure_never_calls_gate() {
    let e = ordinary();
    for (entry, limits) in [
        (
            e.entry("demo").unwrap(),
            ExecutionLimits {
                steps: 0,
                ..ExecutionLimits::default()
            },
        ),
        (FunctionId(usize::MAX), ExecutionLimits::default()),
    ] {
        reset_execution_observations();
        let result = execute_with_gate(&e, entry, limits, || -> Result<(), ()> {
            panic!("gate reached after failed ordinary admission")
        });
        assert!(matches!(result, Err(GatedError::Primary(_))));
        assert_eq!(execution_observations(), (1, 0));
    }
}

#[test]
fn ordinary_public_result_boundary_precedes_gate_even_after_entry_resolution() {
    // Reuse the established private resource fixture, never a public approval bypass.
    let program = hir::resources::fixture_with(
        "fn demo()->Result<success:Leaf,failure:bool>{return Ok(acquire());}",
    );
    let e = prepare_ordinary_synthetic(program, AdmissionLimits::default()).unwrap();
    let entry = e.entry("demo").unwrap();
    reset_execution_observations();
    let result = execute_with_gate(&e, entry, ExecutionLimits::default(), || -> Result<(), ()> {
        panic!("lint ran before public-result admission")
    });
    assert!(matches!(
        result,
        Err(GatedError::Primary(ExecutionError {
            kind: ErrorKind::Integrity("live resource Result cannot cross public boundary"),
            ..
        }))
    ));
    assert_eq!(execution_observations(), (1, 0));
}

#[test]
fn later_machine_limit_remains_operation_failure_after_successful_gate() {
    let e = ordinary();
    let called = Cell::new(false);
    reset_execution_observations();
    let result = execute_with_gate(
        &e,
        e.entry("demo").unwrap(),
        ExecutionLimits {
            steps: 1,
            ..ExecutionLimits::default()
        },
        || {
            called.set(true);
            Ok::<(), ()>(())
        },
    );
    assert!(called.get());
    assert!(matches!(result, Err(GatedError::Operation(_))));
    assert_eq!(execution_observations(), (1, 1));
}

#[test]
fn selected_real_session_is_unstarted_at_gate_and_trusted_events_require_advance() {
    let e = selected(SELECTED);
    reset_execution_observations();
    let mut session =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    assert_eq!(execution_observations(), (1, 0));
    assert!(session.events().is_empty());
    assert_eq!(session.live_sensors(), 0);
    let pending = || -> Result<(), &str> { Err("deny or failed report") };
    assert!(pending().is_err());
    assert_eq!(execution_observations(), (1, 0));
    assert!(session.events().is_empty());
    // The positive counterpart advances this same already admitted session.
    let allowed = || Ok::<(), &str>(());
    allowed().unwrap();
    session.advance().unwrap();
    assert_eq!(execution_observations(), (1, 1));
    assert!(session
        .events()
        .iter()
        .any(|event| matches!(event, TrustedEvent::Attempt(_, _))));
    assert_eq!(session.live_sensors(), 0);
}

#[test]
fn selected_initial_reservation_and_result_boundary_fail_before_gate_or_events() {
    let e = selected(SELECTED);
    reset_execution_observations();
    let result = ExecutionSession::new(
        &e,
        e.entry("demo").unwrap(),
        ExecutionLimits {
            working_storage: 1,
            ..ExecutionLimits::default()
        },
    );
    assert!(result.is_err());
    assert_eq!(execution_observations(), (1, 0));
    let resource_result = selected(
        "fn demo()->Result<success:Sensor,failure:SensorError>effects io{return sensor_open();}",
    );
    reset_execution_observations();
    let result = ExecutionSession::new(
        &resource_result,
        resource_result.entry("demo").unwrap(),
        ExecutionLimits::default(),
    );
    assert!(matches!(
        result,
        Err(ExecutionError {
            kind: ErrorKind::Integrity("live resource Result cannot cross public boundary"),
            ..
        })
    ));
    assert_eq!(execution_observations(), (1, 0));
}

#[test]
fn audit_preflight_profile_storage_and_host_failures_leave_gate_unused() {
    let plain = ordinary();
    assert!(matches!(
        plain.render_authority_with_gate(ExecutionLimits::default(), || -> Result<(), ()> {
            panic!("missing profile did not precede gate")
        }),
        Err(GatedError::Primary(_))
    ));
    let e = selected(SELECTED);
    assert!(matches!(
        e.render_authority_with_gate(
            ExecutionLimits {
                working_storage: 1,
                ..ExecutionLimits::default()
            },
            || -> Result<(), ()> { panic!("storage did not precede gate") },
        ),
        Err(GatedError::Primary(ExecutionError {
            kind: ErrorKind::ConfiguredLimit("authority audit storage"),
            ..
        }))
    ));
    audit::with_scratch_failure(|| {
        assert!(matches!(
            e.render_authority_with_gate(ExecutionLimits::default(), || -> Result<(), ()> {
                panic!("scratch failure did not precede gate")
            }),
            Err(GatedError::Primary(ExecutionError {
                kind: ErrorKind::Host("authority reachability allocation"),
                ..
            }))
        ));
    });
}

#[test]
fn audit_consumes_same_preflight_and_preserves_exact_output_storage_work_neighbors() {
    let e = selected(SELECTED);
    let normal = e.render_authority(ExecutionLimits::default()).unwrap();
    let n = normal.len();
    let storage = e.program.functions().len() + std::mem::size_of::<render::Output>() + n;
    let limits = ExecutionLimits {
        rendered_output: n,
        working_storage: storage,
        ..ExecutionLimits::default()
    };
    reset_execution_observations();
    let calls = Cell::new(0);
    let actual = e
        .render_authority_with_gate(limits, || {
            calls.set(calls.get() + 1);
            Ok::<(), ()>(())
        })
        .unwrap();
    assert_eq!(actual, normal);
    assert_eq!(calls.get(), 1);
    assert_eq!(execution_observations(), (0, 0));
    assert!(matches!(
        e.render_authority_with_gate(
            ExecutionLimits {
                working_storage: storage - 1,
                ..limits
            },
            || -> Result<(), ()> { panic!("insufficient audit storage reached gate") },
        ),
        Err(GatedError::Primary(_))
    ));
    let warned = Cell::new(false);
    assert!(matches!(
        e.render_authority_with_gate(
            ExecutionLimits {
                rendered_output: n - 1,
                ..limits
            },
            || {
                warned.set(true);
                Ok::<(), ()>(())
            },
        ),
        Err(GatedError::Operation(ExecutionError {
            kind: ErrorKind::ConfiguredLimit("rendered output"),
            ..
        }))
    ));
    assert!(warned.get());
    let mut low = 1;
    let mut high = limits.steps;
    while low < high {
        let middle = (low + high) / 2;
        if e
            .render_authority(ExecutionLimits {
                steps: middle,
                ..limits
            })
            .is_ok()
        {
            high = middle;
        } else {
            low = middle + 1;
        }
    }
    assert!(e
        .render_authority_with_gate(
            ExecutionLimits {
                steps: low,
                ..limits
            },
            || Ok::<(), ()>(()),
        )
        .is_ok());
    let warned = Cell::new(false);
    let error = e.render_authority_with_gate(
        ExecutionLimits {
            steps: low - 1,
            ..limits
        },
        || {
            warned.set(true);
            Ok::<(), ()>(())
        },
    );
    assert!(warned.get());
    assert!(matches!(error, Err(GatedError::Operation(_))));
    assert_eq!(execution_observations(), (0, 0));
}

#[test]
fn audit_denied_or_failed_gate_never_runs_the_report_or_source() {
    let e = selected(SELECTED);
    reset_execution_observations();
    for reason in ["denied", "warning failed"] {
        let error = e.render_authority_with_gate(ExecutionLimits::default(), || Err(reason));
        assert!(matches!(error, Err(GatedError::Gate(message)) if message == reason));
    }
    assert_eq!(execution_observations(), (0, 0));
}

#[test]
fn audit_remaining_work_failure_is_after_gate_while_prefix_work_failure_is_primary() {
    let e = selected(SELECTED);
    let limits = ExecutionLimits::default();
    let prefix = limits.rendered_output + e.program.functions().len();
    let called = Cell::new(false);
    let result = e.render_authority_with_gate(
        ExecutionLimits {
            steps: prefix as u64,
            ..limits
        },
        || {
            called.set(true);
            Ok::<(), ()>(())
        },
    );
    assert!(called.get());
    assert!(matches!(result, Err(GatedError::Operation(_))));
    assert!(matches!(
        e.render_authority_with_gate(
            ExecutionLimits {
                steps: prefix as u64 - 1,
                ..limits
            },
            || -> Result<(), ()> { panic!("prefix work exhaustion reached gate") },
        ),
        Err(GatedError::Primary(_))
    ));
}

struct OnePass<'a> {
    remaining: &'a [u8],
    reads: &'a Cell<usize>,
    dropped: &'a Cell<bool>,
}
impl Read for OnePass<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        let Some((&byte, rest)) = self.remaining.split_first() else {
            return Ok(0);
        };
        out[0] = byte;
        self.remaining = rest;
        Ok(1)
    }
}
impl Drop for OnePass<'_> {
    fn drop(&mut self) {
        self.dropped.set(true);
    }
}

#[test]
fn bounded_companion_owns_facts_after_single_reader_is_dropped() {
    let reads = Cell::new(0);
    let dropped = Cell::new(false);
    let input = OnePass {
        remaining: ORDINARY.as_bytes(),
        reads: &reads,
        dropped: &dropped,
    };
    let (prepared, scans) = crate::lexer::with_scan_count(|| {
        prepare_execution_from_reader_with_facts(input, AdmissionLimits::default(), None)
    });
    let (e, facts) = prepared.unwrap().into_parts();
    assert_eq!(scans, 1);
    assert!(dropped.get());
    assert_eq!(reads.get(), ORDINARY.len() + 1);
    let (_, expected) = crate::compile_source_with_facts(ORDINARY).unwrap().into_parts();
    assert_eq!(facts, expected);
    let legacy = prepare_execution_from_reader(ORDINARY.as_bytes(), AdmissionLimits::default())
        .unwrap();
    assert_eq!(e.program, legacy.program);
    assert_eq!(e.mir, legacy.mir);
    assert_eq!(e.origins, legacy.origins);
    assert_eq!(e.limits, legacy.limits);
}

#[test]
fn bounded_and_ordinary_companions_agree_for_comments_locations_and_empty_units() {
    for source in [
        "",
        " \t\r\n",
        "// only comment",
        "\n\t type R=range 0..1;",
        "type R=range 0..1; // trailing",
        "\r\n\tfunction f(){}",
        "function demo()->bool{return true;}//! ordinary comment",
    ] {
        let (_, facts) = prepare_execution_from_reader_with_facts(
            source.as_bytes(),
            AdmissionLimits::default(),
            None,
        )
        .unwrap()
        .into_parts();
        let (_, expected) = crate::compile_source_with_facts(source).unwrap().into_parts();
        assert_eq!(facts, expected, "{source:?}");
    }
}

#[test]
fn bounded_companion_preserves_source_and_token_limit_neighbors() {
    let tokens = crate::lexer::lex(ORDINARY).unwrap().len();
    let exact = AdmissionLimits {
        source_bytes: ORDINARY.len(),
        tokens,
        ..AdmissionLimits::default()
    };
    assert!(prepare_execution_from_reader(ORDINARY.as_bytes(), exact).is_ok());
    assert!(prepare_execution_from_reader_with_facts(ORDINARY.as_bytes(), exact, None).is_ok());
    for limits in [
        AdmissionLimits {
            source_bytes: ORDINARY.len() - 1,
            ..exact
        },
        AdmissionLimits {
            tokens: tokens - 1,
            ..exact
        },
    ] {
        let old = prepare_execution_from_reader(ORDINARY.as_bytes(), limits)
            .err()
            .unwrap();
        let new = prepare_execution_from_reader_with_facts(ORDINARY.as_bytes(), limits, None)
            .err()
            .unwrap();
        assert_eq!(old, new);
    }
}

#[test]
fn bounded_companion_preserves_primary_frontend_errors_with_partial_comment_facts() {
    for source in ["@ // comment", "// comment\n@", "fn demo( {}"] {
        let old = prepare_execution_from_reader(source.as_bytes(), AdmissionLimits::default())
            .err()
            .unwrap();
        let new = prepare_execution_from_reader_with_facts(
            source.as_bytes(),
            AdmissionLimits::default(),
            None,
        )
        .err()
        .unwrap();
        assert_eq!(old, new);
    }
}

#[test]
fn selected_reader_companion_matches_legacy_approval_without_retaining_input() {
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    let (new, facts) = prepare_execution_from_reader_with_facts(
        SELECTED.as_bytes(),
        AdmissionLimits::default(),
        Some(&selection),
    )
    .unwrap()
    .into_parts();
    let old = prepare_execution_from_reader_with_profile(
        SELECTED.as_bytes(),
        AdmissionLimits::default(),
        &selection,
    )
    .unwrap();
    let (_, expected) = crate::compile_source_with_facts(ORDINARY).unwrap().into_parts();
    assert!(facts.parsed_nonempty());
    assert!(!facts.has_comment());
    assert_eq!(facts.first_code_token(), expected.first_code_token());
    assert_eq!(new.program, old.program);
    assert_eq!(new.mir, old.mir);
    assert_eq!(new.trusted, old.trusted);
    assert_eq!(new.domain, old.domain);
}
