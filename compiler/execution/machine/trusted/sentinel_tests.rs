//! Sentinel application evidence and the two finite authenticated fixtures.
use super::*;
use crate::execution::ResultPayload;
use crate::trusted::{Grants, Selection};
use std::collections::BTreeMap;

const SENTINEL: &str = include_str!("../../../../examples/sentinel.bow");

fn admission() -> AdmissionLimits {
    AdmissionLimits {
        tokens: 2048,
        frontend_work: 10_000_000_000_000_000,
        working_storage: 1_073_741_824,
        diagnostic_storage: 4_194_304,
        ..AdmissionLimits::default()
    }
}

fn artifact(source: &str, root: &str, scenario: Scenario, live: usize) -> CheckedExecutable {
    let selection = Selection::sensor_demo(scenario, Grants::ALL, &[root])
        .unwrap()
        .with_sensor_limit(live)
        .unwrap();
    prepare_execution_with_profile(source, admission(), &selection).unwrap()
}

// Validate the typed outer Result, exact record identity, and every nominal field.
// Rendering is tested separately; process success is never this assertion.
fn fields(
    e: &CheckedExecutable,
    value: &ExecutionResult,
    tag: hir::ResultTag,
) -> BTreeMap<String, i64> {
    let ExecutionResult::Value(RuntimeValue::Result(layers)) = value else {
        panic!("expected typed Result: {value:?}");
    };
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].tag, tag);
    let Some(ResultPayload::Record { record, fields }) = &layers[0].payload else {
        panic!("expected record payload");
    };
    let declaration = &e.program.records()[record.0];
    assert_eq!(
        declaration.name,
        if tag == hir::ResultTag::Ok {
            "Report"
        } else {
            "Failure"
        }
    );
    assert_eq!(fields.len(), declaration.fields.len());
    fields
        .iter()
        .map(|(id, value)| {
            let field = &e.program.fields()[id.0];
            assert_eq!(field.record, *record);
            assert_eq!(field.range_type(), Some(value.ty));
            let range = &e.program.ranges()[value.ty.0];
            assert!((range.min..=range.max).contains(&value.value));
            (field.name.clone(), value.value)
        })
        .collect()
}

fn expected(values: &[(&str, i64)]) -> BTreeMap<String, i64> {
    values.iter().map(|(k, v)| ((*k).to_owned(), *v)).collect()
}

fn lifecycle(s: &ExecutionSession<'_>, readings: &[i64]) {
    let state = s.machine.trusted.as_ref().unwrap();
    let approved = s.machine.executable.trusted.as_ref().unwrap();
    let open = approved.catalog.operations[0].id;
    let read = approved.catalog.operations[1].id;
    let finish = approved.catalog.operations[3].id;
    let release = approved.catalog.resource.release;
    let mut wanted = Vec::new();
    for (ordinal, value) in readings.iter().enumerate() {
        let id = ordinal as u64;
        let invocation = id * 3;
        wanted.extend([
            Event::Attempt(invocation, open),
            Event::Received(invocation, 0),
            Event::Published(invocation, hir::ResultTag::Ok),
            Event::Attempt(invocation + 1, read),
            Event::Received(invocation + 1, 0),
            Event::Published(invocation + 1, hir::ResultTag::Ok),
            Event::Attempt(invocation + 2, finish),
            Event::Received(invocation + 2, 0),
            Event::FinishProtected(invocation + 2, hir::ResultTag::Ok),
            Event::ReleaseAttempt(id, release),
            Event::ReleaseComplete(id),
            Event::Published(invocation + 2, hir::ResultTag::Ok),
        ]);
        assert_eq!(state.productions[ordinal * 3].sensor, Some(id));
        assert_eq!(
            state.productions[ordinal * 3 + 1]
                .range
                .as_ref()
                .unwrap()
                .value,
            *value
        );
    }
    assert_eq!(s.events(), wanted);
    assert_eq!(s.live_sensors(), 0);
    // The second fresh identity reused one vacant physical token slot.
    assert_eq!(state.entries.len(), 1);
}

#[test]
fn sentinel_warning_and_critical_typed_reports_have_identity_associated_lifecycles() {
    for (scenario, readings, warning, critical, peak, disposition) in [
        (Scenario::Warning, [60, 42], 1, 0, 60, 1),
        (Scenario::Critical, [90, 60], 1, 1, 90, 2),
        (Scenario::Sample, [42, 42], 0, 0, 42, 0),
    ] {
        let e = artifact(SENTINEL, "monitor", scenario, 1);
        let audit = e.render_authority(ExecutionLimits::default()).unwrap();
        assert!(audit.contains(&format!("scenario={scenario:?}")));
        let mut s =
            ExecutionSession::new(&e, e.entry("monitor").unwrap(), ExecutionLimits::default())
                .unwrap();
        let result = fields(&e, s.advance().unwrap(), hir::ResultTag::Ok);
        assert_eq!(
            result,
            expected(&[
                ("observations", 2),
                ("warnings", warning),
                ("critical", critical),
                ("peak", peak),
                ("disposition", disposition),
                ("completed", 2)
            ])
        );
        lifecycle(&s, &readings);
        let events = s.events().to_vec();
        let steps = s.machine.steps;
        assert_eq!(fields(&e, s.advance().unwrap(), hir::ResultTag::Ok), result);
        assert_eq!(s.events(), events);
        assert_eq!(s.machine.steps, steps);
        assert_eq!(
            e.render_authority(ExecutionLimits::default()).unwrap(),
            audit
        );
    }
}

#[test]
fn sentinel_existing_failure_matrix_preserves_partial_reads_and_completed_resources() {
    for (scenario, phase, problem, read, completion, observations, completed) in [
        (Scenario::NoSensor, 1, 1, 0, 0, 0, 0),
        (Scenario::OpenIoFailure, 1, 3, 0, 0, 0, 0),
        (Scenario::NoSample, 2, 2, 2, 1, 0, 1),
        (Scenario::ReadIoFailure, 2, 3, 3, 1, 0, 1),
        (Scenario::FinishIoFailure, 3, 3, 0, 2, 1, 0),
        (Scenario::FinishPartialIoFailure, 3, 3, 0, 2, 1, 0),
    ] {
        let e = artifact(SENTINEL, "monitor", scenario, 1);
        let mut s =
            ExecutionSession::new(&e, e.entry("monitor").unwrap(), ExecutionLimits::default())
                .unwrap();
        assert_eq!(
            fields(&e, s.advance().unwrap(), hir::ResultTag::Error),
            expected(&[
                ("phase", phase),
                ("problem", problem),
                ("read_status", read),
                ("completion", completion),
                ("observations", observations),
                ("warnings", 0),
                ("critical", 0),
                ("peak", if observations == 0 { 0 } else { 42 }),
                ("disposition", 0),
                ("completed", completed)
            ])
        );
        assert_eq!(s.live_sensors(), 0);
        assert_eq!(
            s.events()
                .iter()
                .filter(|e| matches!(e, Event::ReleaseComplete(_)))
                .count(),
            usize::from(phase != 1)
        );
        assert_eq!(
            s.events()
                .iter()
                .any(|e| matches!(e, Event::FinishPrefix(_, 1))),
            scenario == Scenario::FinishPartialIoFailure
        );
    }
}

#[test]
fn sentinel_sequences_reset_only_for_fresh_sessions_and_old_slots_do_not_reset_them() {
    for scenario in [Scenario::Warning, Scenario::Critical] {
        for _ in 0..2 {
            let e = artifact(SENTINEL, "monitor", scenario, 1);
            for _ in 0..2 {
                let mut s = ExecutionSession::new(
                    &e,
                    e.entry("monitor").unwrap(),
                    ExecutionLimits::default(),
                )
                .unwrap();
                s.advance().unwrap();
                lifecycle(
                    &s,
                    if scenario == Scenario::Warning {
                        &[60, 42]
                    } else {
                        &[90, 60]
                    },
                );
            }
        }
    }
}

#[test]
fn sentinel_repeated_reads_follow_actual_sensor_not_slot_or_operation_counter() {
    let source = "record Report{first:Reading,again:Reading,second:Reading,last:Reading}fn demo()->Result<success:Report,failure:SensorError>effects io{let first=sensor_open()or return;let second=sensor_open()or return;let b=sensor_read(&second)or return;let a=sensor_read(&first)or return;let again=sensor_read(&first)or return;let last=sensor_read(&second)or return;let done_b=sensor_finish(second)or return;let done_a=sensor_finish(first)or return;return Ok(Report{first:a,again:again,second:b,last:last});}";
    for (scenario, a, b) in [(Scenario::Warning, 60, 42), (Scenario::Critical, 90, 60)] {
        let e = artifact(source, "demo", scenario, 2);
        let mut s = ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
            .unwrap();
        assert_eq!(
            fields(&e, s.advance().unwrap(), hir::ResultTag::Ok),
            expected(&[("first", a), ("again", a), ("second", b), ("last", b)])
        );
        let state = s.machine.trusted.as_ref().unwrap();
        assert_eq!(state.productions[0].sensor, Some(0));
        assert_eq!(state.productions[1].sensor, Some(1));
        assert_eq!(state.entries.len(), 2);
        assert_eq!(
            s.events()
                .iter()
                .filter_map(|e| if let Event::ReleaseComplete(id) = e {
                    Some(*id)
                } else {
                    None
                })
                .collect::<Vec<_>>(),
            [1, 0]
        );
    }
}

#[test]
fn sentinel_exhaustion_never_wraps_after_release_or_repeated_failed_open() {
    let source = "fn demo()->Result<success:bool,failure:SensorError>effects io{let first=sensor_open()or return;let a=sensor_read(&first)or return;let done=sensor_finish(first)or return;let second=sensor_open()or return;let b=sensor_read(&second)or return;let done_second=sensor_finish(second)or return;match sensor_open(){Ok(extra)=>{let closed=sensor_finish(extra)or return;return Ok(false);}Error(problem)=>{match sensor_open(){Ok(extra)=>{let closed=sensor_finish(extra)or return;return Ok(false);}Error(again)=>{return Ok((problem==1)&&(again==1));}}}}}";
    for scenario in [Scenario::Warning, Scenario::Critical] {
        let e = artifact(source, "demo", scenario, 1);
        let mut s = ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
            .unwrap();
        assert_eq!(
            e.render_result(s.advance().unwrap(), 4096).unwrap(),
            "Ok(Bool(true))"
        );
        let state = s.machine.trusted.as_ref().unwrap();
        assert_eq!(state.next_sensor, 2);
        assert_eq!(state.entries.len(), 1);
        assert_eq!(
            state
                .productions
                .iter()
                .filter_map(|p| p.sensor)
                .collect::<Vec<_>>(),
            [0, 1]
        );
        assert_eq!(s.live_sensors(), 0);
    }
}

#[test]
fn sentinel_live_limit_precedes_sequence_exhaustion_without_excess_attempt() {
    let source = "fn demo()->Result<success:bool,failure:SensorError>effects io{let first=sensor_open()or return;let second=sensor_open()or return;let third=sensor_open()or return;return Ok(true);}";
    for scenario in [Scenario::Warning, Scenario::Critical] {
        for live in [1, 2] {
            let e = artifact(source, "demo", scenario, live);
            let mut s =
                ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
                    .unwrap();
            assert_eq!(
                s.advance().unwrap_err().kind,
                ErrorKind::ConfiguredLimit("live Sensor count")
            );
            assert_eq!(
                s.events()
                    .iter()
                    .filter(|e| matches!(e, Event::Attempt(..)))
                    .count(),
                live
            );
            assert_eq!(s.live_sensors(), live);
        }
    }
}

#[test]
fn sentinel_scenario_substitution_cannot_authenticate() {
    for (from, to) in [
        (Scenario::Warning, Scenario::Critical),
        (Scenario::Critical, Scenario::Sample),
    ] {
        let mut e = artifact(SENTINEL, "monitor", from, 1);
        e.trusted.as_mut().unwrap().catalog.selection =
            Selection::sensor_demo(to, Grants::ALL, &["monitor"])
                .unwrap()
                .with_sensor_limit(1)
                .unwrap();
        assert!(matches!(
            e.validate().unwrap_err().kind,
            ErrorKind::Integrity(_)
        ));
        assert!(
            ExecutionSession::new(&e, e.entry("monitor").unwrap(), ExecutionLimits::default())
                .is_err()
        );
    }
}

#[test]
fn sentinel_all_fuel_cuts_preserve_custody_and_terminal_non_delivery() {
    let e = artifact(SENTINEL, "monitor", Scenario::Warning, 1);
    let mut full =
        ExecutionSession::new(&e, e.entry("monitor").unwrap(), ExecutionLimits::default()).unwrap();
    full.advance().unwrap();
    let mut after_acquisition = false;
    let mut after_release_before_delivery = false;
    for steps in 1..full.machine.steps {
        let mut s = ExecutionSession::new(
            &e,
            e.entry("monitor").unwrap(),
            ExecutionLimits {
                steps,
                ..ExecutionLimits::default()
            },
        )
        .unwrap();
        assert_eq!(
            s.advance().unwrap_err().kind,
            ErrorKind::ConfiguredLimit("steps")
        );
        super::completion_bound_tests::conserved(&s.machine);
        after_acquisition |= s.live_sensors() == 1;
        after_release_before_delivery |= s
            .machine
            .trusted
            .as_ref()
            .unwrap()
            .completion
            .as_ref()
            .is_some_and(|p| p.phase == crate::trusted::approval::CompletionPhase::Released);
        let before = s.events().to_vec();
        let spent = s.machine.steps;
        assert!(s.advance().is_err());
        assert_eq!(s.events(), before);
        assert_eq!(s.machine.steps, spent);
    }
    assert!(after_acquisition && after_release_before_delivery);
}

#[test]
fn sentinel_threshold_neighbors_use_the_actual_application_through_private_receipts() {
    let e = artifact(SENTINEL, "monitor", Scenario::Sample, 1);
    let reading = e.trusted.as_ref().unwrap().catalog.reading;
    for value in [0, 49, 50, 79, 80, 4095] {
        let mut s =
            ExecutionSession::new(&e, e.entry("monitor").unwrap(), ExecutionLimits::default())
                .unwrap();
        // Existing private receipt control: still validates nominal identity and
        // bounds, executes Sentinel, and retains real Sensor custody/events.
        let result = with_conformance_response(1, reading, value, || s.advance().unwrap().clone());
        let actual = fields(&e, &result, hir::ResultTag::Ok);
        let warning = i64::from((50..80).contains(&value));
        let critical = i64::from(value >= 80);
        assert_eq!(actual["warnings"], warning);
        assert_eq!(actual["critical"], critical);
        assert_eq!(actual["peak"], value.max(42));
        assert_eq!(
            actual["disposition"],
            if critical == 1 { 2 } else { warning }
        );
        assert_eq!(s.live_sensors(), 0);
    }
}
