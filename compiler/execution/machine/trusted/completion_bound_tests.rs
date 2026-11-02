//! Numeric neighbors measured at real preparation/session/audit/render seams.
use super::*;
use crate::trusted::{Grants, Selection};

const SOURCE: &str = "fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;return sensor_finish(sensor);}";

fn selection(scenario: Scenario) -> Selection {
    Selection::sensor_demo(scenario, Grants::ALL, &["demo"]).unwrap()
}
fn artifact(scenario: Scenario) -> CheckedExecutable {
    prepare_execution_with_profile(SOURCE, AdmissionLimits::default(), &selection(scenario))
        .unwrap()
}
fn minimum(mut upper: u64, mut accepts: impl FnMut(u64) -> bool) -> u64 {
    assert!(accepts(upper));
    let mut lower = 1;
    while lower < upper {
        let middle = lower + (upper - lower) / 2;
        if accepts(middle) {
            upper = middle;
        } else {
            lower = middle + 1;
        }
    }
    assert!(accepts(lower));
    if lower > 1 {
        assert!(!accepts(lower - 1));
    }
    lower
}
fn metric(name: &str, text: &str) {
    if let Ok(directory) = std::env::var("B34_EVIDENCE_DIR") {
        let path = std::path::Path::new(&directory);
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(path.join(format!("{name}.json")), text).unwrap();
    }
}

pub(super) fn conserved(machine: &Machine<'_>) {
    let state = machine.trusted.as_ref().unwrap();
    let mut physical: Vec<_> = state
        .entries
        .iter()
        .filter_map(|entry| entry.token.as_ref().map(|token| token.identity))
        .collect();
    let mut tickets = Vec::new();
    let mut visit = |value: &Value| {
        if let Value::Resource { identity, .. } = value {
            tickets.push(*identity);
        }
    };
    for frame in &machine.frames {
        for (_, slot) in &frame.slots {
            if let Slot::Available(value) = slot {
                visit(value);
            }
        }
    }
    for value in &machine.values {
        visit(value);
    }
    if let Some(temporaries) = &machine.temporaries {
        temporaries.visit_values(&mut visit);
    }
    if let Some(results) = &machine.results {
        for cell in &results.cells {
            if let Some(value) = &cell.payload {
                visit(value);
            }
        }
    }
    if let Some(resources) = &machine.resources {
        resources.visit_completion_custody(&mut visit);
    }
    if let Some(pending) = &state.completion {
        if let Some(value) = &pending.value {
            visit(value);
        }
    }
    if let Some(Receipt {
        payload: Some(Payload::Sensor(token)),
        ..
    }) = &state.receipt
    {
        physical.push(token.identity);
        tickets.push(token.identity);
    }
    physical.sort_unstable();
    tickets.sort_unstable();
    assert!(physical.windows(2).all(|p| p[0] != p[1]));
    assert!(tickets.windows(2).all(|p| p[0] != p[1]));
    assert_eq!(tickets, physical);
}

#[test]
fn m7_completion_catalog_has_explicit_terms_and_exact_combined_neighbors() {
    let defaults = AdmissionLimits::default();
    let per_site = size_of::<crate::trusted::approval::CompletionPlan>() * 3
        + size_of::<crate::trusted::approval::Invocation>();
    let storage = (defaults.traversal_depth * 16 + 32) * (defaults.identifier_bytes + 512)
        + 65_536
        + 4096
        + 4 * 16
        + 8192
        + defaults.tokens * per_site;
    let work = 128 + 65_536 + 4096 + 128 + defaults.tokens as u64 * 32;
    let diagnostics = (defaults.identifier_bytes + 128) * 4 + 512 + 4096 + 4 * 16 + 4096;
    let exact = AdmissionLimits {
        working_storage: storage,
        frontend_work: work,
        diagnostic_storage: diagnostics,
        ..defaults
    };
    for (limits, ok) in [
        (exact, true),
        (
            AdmissionLimits {
                working_storage: storage - 1,
                ..exact
            },
            false,
        ),
        (
            AdmissionLimits {
                frontend_work: work - 1,
                ..exact
            },
            false,
        ),
        (
            AdmissionLimits {
                diagnostic_storage: diagnostics - 1,
                ..exact
            },
            false,
        ),
    ] {
        let c = crate::bounded::Context::new(limits);
        c.reserve_frontend(1, 0).unwrap();
        assert_eq!(c.reserve_completion_catalog(1, 4).is_ok(), ok);
    }
    metric("catalog",&format!("{{\"storage\":{storage},\"work\":{work},\"diagnostics\":{diagnostics},\"protocol_bytes_per_site\":{per_site}}}"));
}

#[test]
fn m7_preparation_storage_work_diagnostics_tokens_and_identifier_neighbors() {
    let mut results = Vec::new();
    for (dimension, upper) in [
        (0, AdmissionLimits::default().working_storage as u64),
        (1, AdmissionLimits::default().frontend_work),
        (2, AdmissionLimits::default().diagnostic_storage as u64),
        (3, AdmissionLimits::default().tokens as u64),
        (4, AdmissionLimits::default().identifier_bytes as u64),
    ] {
        let threshold = minimum(upper, |value| {
            let mut limits = AdmissionLimits::default();
            match dimension {
                0 => limits.working_storage = value as usize,
                1 => limits.frontend_work = value,
                2 => limits.diagnostic_storage = value as usize,
                3 => limits.tokens = value as usize,
                _ => limits.identifier_bytes = value as usize,
            }
            match prepare_execution_with_profile(SOURCE, limits, &selection(Scenario::Sample))
                .and_then(|e| e.validate())
            {
                Ok(()) => true,
                Err(error) => {
                    assert!(
                        matches!(error.kind, ErrorKind::ConfiguredLimit(_)),
                        "{error:?}"
                    );
                    false
                }
            }
        });
        results.push(threshold);
    }
    assert_eq!(results[4], "FinishReceipt".len() as u64);
    metric("preparation",&format!("{{\"storage\":{},\"work\":{},\"diagnostics\":{},\"tokens\":{},\"identifier_bytes\":{}}}",results[0],results[1],results[2],results[3],results[4]));
}

#[test]
fn m7_runtime_peak_and_fuel_neighbors_for_all_three_outcomes() {
    let mut report = String::from("[");
    for (index, scenario) in [
        Scenario::Sample,
        Scenario::FinishIoFailure,
        Scenario::FinishPartialIoFailure,
    ]
    .into_iter()
    .enumerate()
    {
        let e = artifact(scenario);
        let mut full =
            ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
                .unwrap();
        full.advance().unwrap();
        let peak = full.machine.max_storage.get();
        let steps = full.machine.steps;
        for (storage, fuel, ok) in [
            (peak, steps, true),
            (peak - 1, steps, false),
            (peak, steps - 1, false),
        ] {
            let mut s = ExecutionSession::new(
                &e,
                e.entry("demo").unwrap(),
                ExecutionLimits {
                    working_storage: storage,
                    steps: fuel,
                    ..ExecutionLimits::default()
                },
            )
            .unwrap();
            if ok {
                s.advance().unwrap();
                assert_eq!(s.live_sensors(), 0);
            } else {
                assert!(matches!(
                    s.advance().unwrap_err().kind,
                    ErrorKind::ConfiguredLimit(_)
                ));
                conserved(&s.machine);
                let prefix = s.events().to_vec();
                assert!(s.advance().is_err());
                assert_eq!(s.events(), prefix);
            }
        }
        if index > 0 {
            report.push(',');
        }
        report.push_str(&format!(
            "{{\"scenario\":\"{scenario:?}\",\"storage\":{peak},\"fuel\":{steps}}}"
        ));
    }
    report.push(']');
    metric("runtime", &report);
}

#[test]
fn m7_each_container_fits_but_combined_envelope_rejects_before_handoff() {
    let e = artifact(Scenario::Sample);
    let mut full =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    full.advance().unwrap();
    let (base, total, largest) = completion::ENVELOPE.with(|r| r.get().unwrap());
    assert!(total > largest && largest > 0);
    let allowance = base + largest;
    let mut s = ExecutionSession::new(
        &e,
        e.entry("demo").unwrap(),
        ExecutionLimits {
            working_storage: allowance,
            ..ExecutionLimits::default()
        },
    )
    .unwrap();
    assert_eq!(
        s.advance().unwrap_err().kind,
        ErrorKind::ConfiguredLimit("working storage")
    );
    assert_eq!(s.live_sensors(), 1);
    assert!(s.machine.trusted.as_ref().unwrap().completion.is_none());
    assert_eq!(
        s.events()
            .iter()
            .filter(|e| matches!(e, Event::Attempt(..)))
            .count(),
        1
    );
    let mut owners = 0;
    s.machine
        .temporaries
        .as_ref()
        .unwrap()
        .visit_values(|v| owners += usize::from(matches!(v, Value::Resource { .. })));
    assert_eq!(owners, 1);
    metric("combined-envelope",&format!("{{\"retained\":{base},\"sum_growth\":{total},\"largest_growth\":{largest},\"independent_fit_combined_failure\":{allowance}}}"));
}

#[test]
fn m7_every_actual_finish_receiving_host_growth_cut_preserves_current_owner() {
    let e = artifact(Scenario::Sample);
    let mut full =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    full.advance().unwrap();
    let actual = ACTUAL_GROWTH.with(std::cell::Cell::get);
    let sites = [
        Allocation::Provenance,
        Allocation::Audit,
        Allocation::Operand,
        Allocation::ResultCell,
        Allocation::TemporaryOwner,
        Allocation::TemporaryEvent,
        Allocation::CompletionTransfer,
        Allocation::CompletionCleanup,
        Allocation::CompletionDiagnostic,
    ];
    let mut tested = 0u64;
    for site in sites {
        let bit = 1u64 << (site as u32);
        if actual & bit == 0 {
            continue;
        }
        let mut s = ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
            .unwrap();
        s.machine.trusted.as_mut().unwrap().fault = Some((Fault::Allocation(site), 1));
        assert_eq!(
            s.advance().unwrap_err().kind,
            ErrorKind::Host("injected trusted allocation boundary")
        );
        assert_eq!(s.live_sensors(), 1);
        assert_eq!(
            s.events()
                .iter()
                .filter(|e| matches!(e, Event::Attempt(..)))
                .count(),
            1
        );
        let prefix = s.events().to_vec();
        assert!(s.advance().is_err());
        assert_eq!(s.events(), prefix);
        tested |= bit;
    }
    assert_eq!(tested, actual);
    for site in [
        Allocation::CompletionTransfer,
        Allocation::CompletionCleanup,
        Allocation::CompletionDiagnostic,
    ] {
        assert_ne!(tested & (1u64 << site as u32), 0);
    }
    metric(
        "host-growth",
        &format!("{{\"actual_mask\":{actual},\"tested_mask\":{tested}}}"),
    );
}

#[test]
fn m7_post_response_release_uses_prepaid_events_and_no_growth_host_boundary() {
    let e = artifact(Scenario::FinishPartialIoFailure);
    let mut s =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    s.machine.trusted.as_mut().unwrap().fault =
        Some((Fault::Allocation(Allocation::ReleaseAudit), 2));
    s.advance().unwrap();
    assert_eq!(s.live_sensors(), 0);
    assert!(s.events().contains(&Event::ReleaseComplete(0)));
    assert_eq!(
        ACTUAL_GROWTH.with(std::cell::Cell::get) & (1u64 << Allocation::ReleaseAudit as u32),
        0
    );
}

#[test]
fn m7_audit_work_storage_output_and_result_error_render_neighbors() {
    let e = artifact(Scenario::Sample);
    let normal = e.render_authority(ExecutionLimits::default()).unwrap();
    let output = normal.len();
    let work = minimum(ExecutionLimits::default().steps, |value| {
        match e.render_authority(ExecutionLimits {
            steps: value,
            ..ExecutionLimits::default()
        }) {
            Ok(_) => true,
            Err(error) => {
                assert_eq!(
                    error.kind,
                    ErrorKind::ConfiguredLimit("authority audit work")
                );
                false
            }
        }
    });
    let storage = minimum(
        ExecutionLimits::default().working_storage as u64,
        |value| match e.render_authority(ExecutionLimits {
            working_storage: value as usize,
            ..ExecutionLimits::default()
        }) {
            Ok(_) => true,
            Err(error) => {
                assert_eq!(
                    error.kind,
                    ErrorKind::ConfiguredLimit("authority audit storage")
                );
                false
            }
        },
    );
    assert!(e
        .render_authority(ExecutionLimits {
            rendered_output: output,
            ..ExecutionLimits::default()
        })
        .is_ok());
    assert_eq!(
        e.render_authority(ExecutionLimits {
            rendered_output: output - 1,
            ..ExecutionLimits::default()
        })
        .unwrap_err()
        .kind,
        ErrorKind::ConfiguredLimit("rendered output")
    );
    let mut s =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    let value = s.advance().unwrap();
    let rendered = e.render_result(value, 4096).unwrap();
    assert_eq!(e.render_result(value, rendered.len()).unwrap(), rendered);
    assert_eq!(
        e.render_result(value, rendered.len() - 1).unwrap_err().kind,
        ErrorKind::ConfiguredLimit("rendered output")
    );
    let mut stopped =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    stopped.machine.trusted.as_mut().unwrap().fault = Some((Fault::ReleaseFailure, 1));
    let error = stopped.advance().unwrap_err();
    let diagnostic = e.render_error(error, 4096).unwrap();
    assert_eq!(e.render_error(error, diagnostic.len()).unwrap(), diagnostic);
    let fallback = e.render_error(error, diagnostic.len() - 1).unwrap();
    assert_eq!(
        fallback,
        format!(
            "error: diagnostic truncated at {}:{}",
            error.origin.line, error.origin.column
        )
    );
    assert_eq!(e.render_error(error, fallback.len()).unwrap(), fallback);
    assert_eq!(
        e.render_error(error, fallback.len() - 1).unwrap_err().kind,
        ErrorKind::ConfiguredLimit("rendered output")
    );
    metric("audit-output",&format!("{{\"audit_work\":{work},\"audit_storage\":{storage},\"audit_output\":{output},\"result_output\":{},\"diagnostic_output\":{}}}",rendered.len(),diagnostic.len()));
}

#[test]
fn m7_invocation_counter_overflow_stops_before_consumption_or_attempt() {
    let e = artifact(Scenario::Sample);
    let mut s =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    s.machine.start(e.entry("demo").unwrap()).unwrap();
    while let Some(task) = s.machine.tasks.pop() {
        if matches!(task.action,Action::TrustedInvoke(index,_,_) if e.trusted.as_ref().unwrap().invocations[index].completion.is_some())
        {
            s.machine.trusted.as_mut().unwrap().next_invocation = u64::MAX;
            assert_eq!(
                s.machine.step(task).unwrap_err().kind,
                ErrorKind::ConfiguredLimit("trusted invocation identities")
            );
            assert_eq!(s.live_sensors(), 1);
            return;
        }
        s.machine.step(task).unwrap();
    }
    panic!("completion must be reached");
}

#[test]
fn m9_stopped_completion_report_neighbors_preserve_cursor_outcome_and_prefix() {
    let e = artifact(Scenario::FinishPartialIoFailure);
    let mut stopped =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    stopped.machine.trusted.as_mut().unwrap().fault = Some((Fault::ReleaseFailure, 1));
    assert!(matches!(
        stopped.advance().unwrap_err().kind,
        ErrorKind::Host(_)
    ));
    let prefix = stopped.events().to_vec();
    let retained = stopped.machine.storage().unwrap();
    let threshold = minimum(
        ExecutionLimits::default().working_storage as u64,
        |value| match stopped.render_authority(ExecutionLimits {
            working_storage: value as usize,
            ..ExecutionLimits::default()
        }) {
            Ok(_) => true,
            Err(error) => {
                assert_eq!(
                    error.kind,
                    ErrorKind::ConfiguredLimit("authority audit storage")
                );
                false
            }
        },
    );
    assert!(threshold > retained as u64);
    conserved(&stopped.machine);
    assert!(stopped
        .machine
        .resources
        .as_ref()
        .unwrap()
        .current
        .is_some());
    assert!(stopped
        .machine
        .trusted
        .as_ref()
        .unwrap()
        .completion
        .as_ref()
        .unwrap()
        .protected
        .is_some());
    assert_eq!(stopped.events(), prefix);
    assert!(stopped.advance().is_err());
    assert_eq!(stopped.events(), prefix);
    metric(
        "stopped-report",
        &format!("{{\"retained\":{retained},\"audit_storage\":{threshold}}}"),
    );
}
