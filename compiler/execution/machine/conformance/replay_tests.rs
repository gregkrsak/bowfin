//! Private interpreter-contract evidence; no public controlled-input ABI.
use super::*;
use replay::{Address, Birth, Input, Item, Phase, Response, Site};
fn execution() -> ExecutionLimits {
    ExecutionLimits {
        steps: 256,
        ..ExecutionLimits::default()
    }
}
fn run<'a>(
    c: &'a contract::Contract<'a>,
    responses: Option<&[Item]>,
    failures: &[Address],
    execution: ExecutionLimits,
) -> Session<'a> {
    let input = Input {
        responses,
        failures,
        limits: replay::Limits {
            work: 1_000_000_000,
            ..replay::Limits::default()
        },
    };
    let mut session = Session::new_inner(
        c,
        execution,
        Limits {
            records: 262_144,
            work: 10_000_000,
            storage: 67_108_864,
            lineage: 128,
        },
        Some(input),
    )
    .unwrap();
    let _ = session.advance();
    session
}
fn compare_pair(a: &Session<'_>, b: &Session<'_>) -> Comparison {
    compare(
        a,
        b,
        ComparisonLimits {
            work: 100_000_000,
            ..ComparisonLimits::default()
        },
    )
}
fn selected() -> CheckedExecutable {
    let selection = crate::trusted::Selection::sensor_demo(
        crate::trusted::Scenario::Sample,
        crate::trusted::Grants::ALL,
        &["sample"],
    )
    .unwrap();
    prepare_execution_with_profile(
        include_str!("../../../../examples/sensor_finish.bow"),
        AdmissionLimits::default(),
        &selection,
    )
    .unwrap()
}
fn call(plan: usize, phase: Phase) -> Address {
    Address {
        site: Site::Invocation(plan),
        activation: 0,
        occurrence: 1,
        phase,
    }
}
fn release(c: &contract::Contract<'_>, phase: Phase) -> Address {
    let approved = c.executable.trusted.as_ref().unwrap();
    let span = approved.invocations[2]
        .completion
        .as_ref()
        .unwrap()
        .cleanup
        .span;
    Address {
        site: Site::Release(c.entry(), span),
        activation: 0,
        occurrence: 1,
        phase,
    }
}
fn events<'a>(session: &'a Session<'_>) -> &'a [trusted::Event] {
    let Engine::Selected(session) = &session.engine else {
        unreachable!()
    };
    session.events()
}
fn live(session: &Session<'_>) -> usize {
    let Engine::Selected(session) = &session.engine else {
        unreachable!()
    };
    session.live_sensors()
}
#[test]
fn ordinary_modeled_capacity_host_is_equal_and_storage_precedes_consumption() {
    let e = prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let address = Address {
        site: Site::Capacity(c.entry(), e.program.function(c.entry()).span()),
        activation: 0,
        occurrence: 1,
        phase: Phase::Reservation,
    };
    let failures = [address];
    let a = run(&c, None, &failures, execution());
    let b = run(&c, None, &failures, execution());
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::Host("modeled abstract Host")
    );
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
    assert_eq!(a.machine().steps, 1);
    let low = ExecutionLimits {
        working_storage: c.schedule().machine[0] - 1,
        ..execution()
    };
    let a = run(&c, None, &failures, low);
    let b = run(&c, None, &failures, low);
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::ConfiguredLimit("working storage")
    );
    assert!(a
        .machine()
        .control
        .borrow()
        .as_ref()
        .unwrap()
        .stopped()
        .is_none());
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}
#[test]
fn addressed_completion_phases_preserve_manual_prefix_custody_and_non_delivery() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let approved = e.trusted.as_ref().unwrap();
    let finish = approved.invocations[2].operation;
    for phase in [
        Phase::BeforeHandoff,
        Phase::HandoffAccepted,
        Phase::BeforeAttempt,
        Phase::Attempt,
        Phase::Response,
        Phase::Validation,
        Phase::Protection,
        Phase::Delivery,
    ] {
        let failures = [call(2, phase)];
        let mut a = run(&c, None, &failures, execution());
        let b = run(&c, None, &failures, execution());
        assert_eq!(
            a.primary().unwrap().unwrap_err().kind,
            ErrorKind::Host("modeled abstract Host"),
            "{phase:?}"
        );
        let attempted = matches!(
            phase,
            Phase::Attempt
                | Phase::Response
                | Phase::Validation
                | Phase::Protection
                | Phase::Delivery
        );
        assert_eq!(
            events(&a).contains(&trusted::Event::Attempt(2, finish)),
            attempted,
            "{phase:?}"
        );
        assert!(!events(&a)
            .iter()
            .any(|event| matches!(event, trusted::Event::Published(2, _))));
        assert_eq!(live(&a), usize::from(phase != Phase::Delivery));
        assert_eq!(
            events(&a).contains(&trusted::Event::ReleaseComplete(0)),
            phase == Phase::Delivery
        );
        assert_eq!(compare_pair(&a, &b), Comparison::Equal, "{phase:?}");
        let count = a.record_count();
        let fuel = a.machine().steps;
        let prefix = events(&a).to_vec();
        assert!(a.advance().is_err());
        assert_eq!(a.record_count(), count);
        assert_eq!(a.machine().steps, fuel);
        assert_eq!(events(&a), prefix);
    }
}
#[test]
fn strict_typed_replay_commits_partial_error_then_exact_fixed_release() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let catalog = &e.trusted.as_ref().unwrap().catalog;
    let responses = [
        Item {
            address: call(0, Phase::Response),
            response: Response::Open(catalog.sensor),
        },
        Item {
            address: call(1, Phase::Response),
            response: Response::Value(catalog.reading, 37, 0),
        },
        Item {
            address: call(2, Phase::Response),
            response: Response::Error(catalog.error, 3, 1),
        },
        Item {
            address: release(&c, Phase::ReleaseAttempt),
            response: Response::Release(
                Birth {
                    plan: 0,
                    activation: 0,
                    occurrence: 1,
                },
                true,
            ),
        },
    ];
    let a = run(&c, Some(&responses), &[], execution());
    let b = run(&c, Some(&responses), &[], execution());
    let ExecutionResult::Value(RuntimeValue::Result(layers)) = a.primary().unwrap().unwrap() else {
        panic!("ordinary Error required")
    };
    assert_eq!(layers[0].tag, hir::ResultTag::Error);
    assert_eq!(
        layers[0].payload,
        Some(ResultPayload::Range(RangeValue {
            ty: catalog.error,
            value: 3
        }))
    );
    assert_eq!(live(&a), 0);
    let finish = e.trusted.as_ref().unwrap().invocations[2].operation;
    let expected = [
        trusted::Event::Attempt(2, finish),
        trusted::Event::FinishPrefix(2, 1),
        trusted::Event::Received(2, 1),
        trusted::Event::FinishProtected(2, hir::ResultTag::Error),
        trusted::Event::ReleaseAttempt(0, catalog.resource.release),
        trusted::Event::ReleaseComplete(0),
        trusted::Event::Published(2, hir::ResultTag::Error),
    ];
    let start = events(&a)
        .iter()
        .position(|event| *event == expected[0])
        .unwrap();
    assert_eq!(&events(&a)[start..], &expected);
    assert_eq!(
        a.machine()
            .control
            .borrow()
            .as_ref()
            .unwrap()
            .used_responses(),
        4
    );
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}
#[test]
fn release_attempt_and_completed_release_hosts_have_opposite_live_duties() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    for phase in [
        Phase::BeforeAttempt,
        Phase::ReleaseAttempt,
        Phase::ReleaseComplete,
    ] {
        let failures = [release(&c, phase)];
        let a = run(&c, None, &failures, execution());
        let b = run(&c, None, &failures, execution());
        assert_eq!(
            a.primary().unwrap().unwrap_err().kind,
            ErrorKind::Host("modeled abstract Host"),
            "{phase:?}"
        );
        assert_eq!(live(&a), usize::from(phase != Phase::ReleaseComplete));
        assert_eq!(
            events(&a).contains(&trusted::Event::ReleaseFailure(0)),
            phase == Phase::ReleaseAttempt
        );
        assert_eq!(
            events(&a).contains(&trusted::Event::ReleaseComplete(0)),
            phase == Phase::ReleaseComplete
        );
        assert!(!events(&a)
            .iter()
            .any(|event| matches!(event, trusted::Event::Published(2, _))));
        assert_eq!(compare_pair(&a, &b), Comparison::Equal, "{phase:?}");
    }
}
#[test]
fn earlier_fuel_leaves_future_replay_and_host_items_unconsumed() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let failures = [call(2, Phase::Protection)];
    let low = ExecutionLimits {
        steps: 1,
        ..execution()
    };
    let a = run(&c, Some(&[]), &failures, low);
    let b = run(&c, Some(&[]), &failures, low);
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::ConfiguredLimit("steps")
    );
    assert!(events(&a).is_empty());
    let controller = a.machine().control.borrow();
    let controller = controller.as_ref().unwrap();
    assert_eq!(controller.used_responses(), 0);
    assert!(controller.stopped().is_none());
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}

#[test]
fn malformed_and_unsupported_controlled_inputs_reject_before_entry() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let catalog = &e.trusted.as_ref().unwrap().catalog;
    let observed = Limits::default();
    let limits = replay::Limits {
        work: 1_000_000_000,
        ..replay::Limits::default()
    };
    let duplicate = [call(2, Phase::Protection), call(2, Phase::Protection)];
    assert!(Session::new_inner(
        &c,
        execution(),
        observed,
        Some(Input {
            responses: None,
            failures: &duplicate,
            limits
        })
    )
    .is_err());
    let unsupported = [call(0, Phase::Protection)];
    assert!(Session::new_inner(
        &c,
        execution(),
        observed,
        Some(Input {
            responses: None,
            failures: &unsupported,
            limits
        })
    )
    .is_err());
    for item in [
        Item {
            address: call(0, Phase::Attempt),
            response: Response::Open(catalog.sensor),
        },
        Item {
            address: call(0, Phase::Response),
            response: Response::Open(hir::ResourceTypeId(catalog.sensor.0 + 1)),
        },
        Item {
            address: call(1, Phase::Response),
            response: Response::Error(catalog.error, 1, 0),
        },
        Item {
            address: call(2, Phase::Response),
            response: Response::Value(catalog.finish_receipt, 1, 1),
        },
    ] {
        assert!(
            Session::new_inner(
                &c,
                execution(),
                observed,
                Some(Input {
                    responses: Some(&[item]),
                    failures: &[],
                    limits
                })
            )
            .is_err(),
            "{item:?}"
        );
    }
    let absent = [Address {
        occurrence: 0,
        ..call(0, Phase::Attempt)
    }];
    assert!(Session::new_inner(
        &c,
        execution(),
        observed,
        Some(Input {
            responses: None,
            failures: &absent,
            limits
        })
    )
    .is_err());
    assert!(Session::new_inner(
        &c,
        execution(),
        observed,
        Some(Input {
            responses: None,
            failures: &[],
            limits: replay::Limits {
                storage: 1,
                ..limits
            }
        })
    )
    .is_err());
}
#[test]
fn missing_reached_response_rejects_without_attempt_fallback_or_equal() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let a = run(&c, Some(&[]), &[], execution());
    let b = run(&c, Some(&[]), &[], execution());
    assert!(matches!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::Integrity(_)
    ));
    assert!(events(&a).is_empty());
    assert_eq!(live(&a), 0);
    assert_eq!(
        a.machine()
            .control
            .borrow()
            .as_ref()
            .unwrap()
            .used_responses(),
        0
    );
    assert!(matches!(compare_pair(&a, &b), Comparison::Inconclusive(_)));
}

#[test]
fn logical_ancestry_counts_loop_calls_and_skips_short_circuit_without_raw_frames() {
    let source="fn probe()->bool{return true;}fn demo()->bool{let mut first=true;let mut again=true;while again{let selected=probe();if first{first=false;}else{again=false;}}return true||probe();}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let a = run(&c, None, &[], execution());
    let input = Input {
        responses: None,
        failures: &[],
        limits: replay::Limits {
            work: 1_000_000_000,
            ..replay::Limits::default()
        },
    };
    let mut b = Session::new_inner(
        &c,
        execution(),
        Limits {
            records: 262_144,
            work: 10_000_000,
            storage: 67_108_864,
            lineage: 128,
        },
        Some(input),
    )
    .unwrap();
    b.machine_mut().next_frame = 9000;
    b.advance().unwrap();
    assert_eq!(
        a.primary().unwrap().unwrap(),
        &ExecutionResult::Value(RuntimeValue::Bool(true))
    );
    let control = a.machine().control.borrow();
    let control = control.as_ref().unwrap();
    assert_eq!(control.activation_count(), 3);
    let first = control.activation(1).unwrap();
    let second = control.activation(2).unwrap();
    assert_eq!(first.0, e.program.functions()[0].id());
    assert_eq!(first.1, Some(0));
    assert_eq!(second.1, Some(0));
    assert_eq!(first.2, second.2);
    assert_eq!((first.3, second.3), (1, 2));
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}
#[test]
fn every_reached_positive_capacity_cut_of_the_finite_sample_preserves_a_matched_stop() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let discovery = run(&c, None, &[], execution());
    assert!(discovery.primary().unwrap().is_ok());
    // Enumeration only: expected Host category, no delivery, repeat behavior
    // and valid custody are checked independently for each selected boundary.
    let cuts: Vec<_> = discovery
        .capture()
        .records
        .iter()
        .filter_map(|record| {
            if let Record::Control(address, 0) = record {
                if matches!(address.site, Site::Capacity(..)) {
                    return Some(*address);
                }
            }
            None
        })
        .collect();
    assert!(cuts.len() > 12);
    for address in cuts {
        let failures = [address];
        let mut a = run(&c, None, &failures, execution());
        let b = run(&c, None, &failures, execution());
        assert_eq!(
            a.primary().unwrap().unwrap_err().kind,
            ErrorKind::Host("modeled abstract Host"),
            "{address:?}"
        );
        assert_eq!(compare_pair(&a, &b), Comparison::Equal, "{address:?}");
        let records = a.record_count();
        let fuel = a.machine().steps;
        let prefix = events(&a).to_vec();
        assert!(a.advance().is_err());
        assert_eq!(a.record_count(), records);
        assert_eq!(a.machine().steps, fuel);
        assert_eq!(events(&a), prefix);
    }
}

#[test]
fn analytic_preallocation_does_not_erase_logical_reservation_addresses_or_host() {
    let e = prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let span = e.program.function(c.entry()).span();
    let failure = Address {
        site: Site::Capacity(c.entry(), span),
        activation: 0,
        occurrence: 2,
        phase: Phase::Reservation,
    };
    let mut expected = None;
    for preallocated in [false, true] {
        let mut controller = replay::Controller::admit(
            &c,
            execution(),
            Input {
                responses: None,
                failures: &[failure],
                limits: replay::Limits {
                    work: 1_000_000_000,
                    ..replay::Limits::default()
                },
            },
        )
        .unwrap();
        let mut physical = Vec::<u8>::new();
        if preallocated {
            physical.try_reserve_exact(8).unwrap();
        }
        let mut trace = Vec::new();
        for logical_size in 1..=2 {
            let address = controller.capacity(span).unwrap();
            let host = controller.fail(address).unwrap();
            trace.push((address, host));
            if host {
                break;
            }
            if physical.capacity() < logical_size {
                physical
                    .try_reserve_exact(logical_size - physical.len())
                    .unwrap();
            }
            physical.push(0);
        }
        assert_eq!(trace.len(), 2);
        assert_eq!(trace[0].0.occurrence, 1);
        assert!(!trace[0].1);
        assert!(trace[1].1);
        assert_eq!(controller.stopped(), Some(failure));
        if let Some(expected) = &expected {
            assert_eq!(&trace, expected);
        } else {
            expected = Some(trace);
        }
    }
    // Private address/protocol fixture only: no optimized or native execution.
}
#[test]
fn controlled_release_failure_retains_partial_prefix_sensor_and_protected_error() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let catalog = &e.trusted.as_ref().unwrap().catalog;
    let responses = [
        Item {
            address: call(0, Phase::Response),
            response: Response::Open(catalog.sensor),
        },
        Item {
            address: call(1, Phase::Response),
            response: Response::Value(catalog.reading, 37, 0),
        },
        Item {
            address: call(2, Phase::Response),
            response: Response::Error(catalog.error, 3, 1),
        },
        Item {
            address: release(&c, Phase::ReleaseAttempt),
            response: Response::Release(
                Birth {
                    plan: 0,
                    activation: 0,
                    occurrence: 1,
                },
                false,
            ),
        },
    ];
    let a = run(&c, Some(&responses), &[], execution());
    let b = run(&c, Some(&responses), &[], execution());
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::Host("modeled abstract Host")
    );
    assert_eq!(live(&a), 1);
    assert!(events(&a).contains(&trusted::Event::FinishPrefix(2, 1)));
    assert!(events(&a).contains(&trusted::Event::FinishProtected(2, hir::ResultTag::Error)));
    assert!(events(&a).contains(&trusted::Event::ReleaseFailure(0)));
    assert!(!events(&a).iter().any(|event| matches!(
        event,
        trusted::Event::ReleaseComplete(_) | trusted::Event::Published(2, _)
    )));
    assert!(matches!(
        a.machine().resources.as_ref().unwrap().current,
        Some(Value::Resource { .. })
    ));
    assert_eq!(
        a.machine()
            .control
            .borrow()
            .as_ref()
            .unwrap()
            .used_responses(),
        4
    );
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}

#[test]
fn valid_wrong_received_value_is_different_even_when_the_final_error_is_identical() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let catalog = &e.trusted.as_ref().unwrap().catalog;
    let responses = [
        Item {
            address: call(0, Phase::Response),
            response: Response::Open(catalog.sensor),
        },
        Item {
            address: call(1, Phase::Response),
            response: Response::Value(catalog.reading, 37, 0),
        },
        Item {
            address: call(2, Phase::Response),
            response: Response::Error(catalog.error, 3, 1),
        },
        Item {
            address: release(&c, Phase::ReleaseAttempt),
            response: Response::Release(
                Birth {
                    plan: 0,
                    activation: 0,
                    occurrence: 1,
                },
                true,
            ),
        },
    ];
    let a = run(&c, Some(&responses), &[], execution());
    let b = trusted::with_conformance_response(1, catalog.reading, 42, || {
        run(&c, Some(&responses), &[], execution())
    });
    assert_eq!(a.primary(), b.primary());
    assert!(!b.machine().control.borrow().as_ref().unwrap().rejected());
    let compared = compare_pair(&a, &b);
    assert!(
        matches!(compared, Comparison::Different { .. }),
        "{compared:?}"
    );
}

#[test]
fn surviving_physical_receipt_failure_is_unmatched_not_a_common_modeled_host() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let make = || {
        let input = Input {
            responses: None,
            failures: &[],
            limits: replay::Limits {
                work: 1_000_000_000,
                ..replay::Limits::default()
            },
        };
        let mut session = Session::new_inner(
            &c,
            execution(),
            Limits {
                records: 262_144,
                work: 10_000_000,
                storage: 67_108_864,
                lineage: 128,
            },
            Some(input),
        )
        .unwrap();
        trusted::conformance_physical_receipt_failure(session.machine_mut(), 2);
        let _ = session.advance();
        session
    };
    let mut a = make();
    let b = make();
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::Host("injected trusted receipt boundary")
    );
    assert_eq!(live(&a), 1);
    assert!(!events(&a)
        .iter()
        .any(|event| matches!(event, trusted::Event::ReleaseAttempt(..))));
    assert_eq!(
        compare_pair(&a, &b),
        Comparison::Inconclusive(Incomplete::UnmatchedPhysicalFailure)
    );
    let count = a.record_count();
    assert!(a.advance().is_err());
    assert_eq!(a.record_count(), count);
}
#[test]
fn finite_sample_all_fuel_neighbors_follow_an_independent_external_prefix() {
    // Exact source of KED-034's retained 72-fuel measurement.
    let source="fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;return sensor_finish(sensor);}";
    let selection = crate::trusted::Selection::sensor_demo(
        crate::trusted::Scenario::Sample,
        crate::trusted::Grants::ALL,
        &["demo"],
    )
    .unwrap();
    let e = prepare_execution_with_profile(source, AdmissionLimits::default(), &selection).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let approved = e.trusted.as_ref().unwrap();
    let expected = [
        trusted::Event::Attempt(0, approved.invocations[0].operation),
        trusted::Event::Received(0, 0),
        trusted::Event::Published(0, hir::ResultTag::Ok),
        trusted::Event::Attempt(1, approved.invocations[1].operation),
        trusted::Event::Received(1, 0),
        trusted::Event::FinishProtected(1, hir::ResultTag::Ok),
        trusted::Event::ReleaseAttempt(0, approved.catalog.resource.release),
        trusted::Event::ReleaseComplete(0),
        trusted::Event::Published(1, hir::ResultTag::Ok),
    ];
    // 72 is the independently retained exact KED-034 sufficient fuel neighbor.
    for fuel in 1..=72 {
        let limit = ExecutionLimits {
            steps: fuel,
            ..execution()
        };
        let a = run(&c, None, &[], limit);
        let b = run(&c, None, &[], limit);
        assert_eq!(a.machine().steps, fuel);
        assert_eq!(events(&a), &expected[..events(&a).len()]);
        if fuel < 72 {
            assert_eq!(
                a.primary().unwrap().unwrap_err().kind,
                ErrorKind::ConfiguredLimit("steps")
            );
        } else {
            assert!(a.primary().unwrap().is_ok());
            assert_eq!(events(&a), &expected);
        }
        assert_eq!(compare_pair(&a, &b), Comparison::Equal, "fuel {fuel}");
    }
}

#[test]
fn independent_trap_and_mutation_prefix_survive_terminal_stop_without_publication() {
    let source="type N=range 0..2;fn two()->N{return 2;}fn demo()->N{let mut value=two();value=value-1;let failed=value-2;return value;}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let a = run(&c, None, &[], execution());
    let b = run(&c, None, &[], execution());
    let ty = e.program.ranges()[0].id;
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::RangeTrap {
            ty,
            min: 0,
            max: 2,
            computed: -1
        }
    );
    let value = e
        .program
        .locals()
        .iter()
        .find(|local| local.name == "value")
        .unwrap()
        .id;
    let failed = e
        .program
        .locals()
        .iter()
        .find(|local| local.name == "failed")
        .unwrap()
        .id;
    let frame = a.machine().frames.last().unwrap();
    assert!(frame.slots.iter().any(|(place,slot)|*place==Place::Local(value)&&matches!(slot,Slot::Available(Value::Public(RuntimeValue::Range(number))) if number.ty==ty && number.value==1)));
    assert!(!frame
        .slots
        .iter()
        .any(|(place, slot)| *place == Place::Local(failed) && matches!(slot, Slot::Available(_))));
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}
#[test]
fn recursive_call_closure_has_exact_logical_depth_without_physical_frame_identity() {
    let e = prepare_execution(
        "fn again(flag:bool)->bool{return again(flag);}fn demo()->bool{return again(true);}",
        AdmissionLimits::default(),
    )
    .unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    assert_eq!(c.functions().len(), 2);
    let limits = ExecutionLimits {
        call_depth: 3,
        ..execution()
    };
    let a = run(&c, None, &[], limits);
    let b = run(&c, None, &[], limits);
    assert_eq!(
        a.primary().unwrap().unwrap_err().kind,
        ErrorKind::ConfiguredLimit("call depth")
    );
    assert_eq!(a.machine().frames.len(), 3);
    assert_eq!(
        a.machine()
            .control
            .borrow()
            .as_ref()
            .unwrap()
            .activation_count(),
        3
    );
    assert_eq!(compare_pair(&a, &b), Comparison::Equal);
}
#[test]
fn coordinated_accounting_origin_kind_and_replay_marker_corruption_is_inconclusive() {
    let e = prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    for mutation in 0..3 {
        let a = run(&c, None, &[], execution());
        let b = run(&c, None, &[], execution());
        for session in [&a, &b] {
            let mut capture = session.machine().observation.borrow_mut();
            let records = &mut capture.as_mut().unwrap().records;
            match mutation {
                0 => {
                    let record = records
                        .iter_mut()
                        .find(|record| matches!(record, Record::Accounting(_, Some(_))))
                        .unwrap();
                    let Record::Accounting(_, Some(bytes)) = record else {
                        unreachable!()
                    };
                    *bytes += 1;
                }
                1 => {
                    let record = records
                        .iter_mut()
                        .find(|record| matches!(record, Record::OriginKind(_)))
                        .unwrap();
                    *record = Record::OriginKind(OriginKind::Replacement);
                }
                _ => {
                    let record = records
                        .iter_mut()
                        .find(|record| matches!(record, Record::Control(_, 0)))
                        .unwrap();
                    let Record::Control(_, status) = record else {
                        unreachable!()
                    };
                    *status = 2;
                }
            }
        }
        assert!(
            matches!(compare_pair(&a, &b), Comparison::Inconclusive(_)),
            "mutation {mutation}"
        );
    }
}

fn storage_gates(session: &Session<'_>) -> Vec<usize> {
    let mut growth = None;
    let mut gates = Vec::new();
    for record in &session.capture().records {
        match record {
            Record::Cut(Cut::Storage(bytes), ..) => growth = Some(*bytes),
            Record::Cut(..) => growth = None,
            Record::Accounting(_, Some(retained)) => {
                if let Some(bytes) = growth.take() {
                    gates.push(retained.checked_add(bytes).unwrap());
                }
            }
            _ => (),
        }
    }
    gates
}
#[test]
fn finite_sample_storage_neighbors_stop_at_the_first_insufficient_gate() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let discovery = run(&c, None, &[], execution());
    assert!(discovery.primary().unwrap().is_ok());
    let mut wanted = storage_gates(&discovery);
    wanted.sort_unstable();
    wanted.dedup();
    assert!(wanted.len() > 10);
    let maximum = *wanted.last().unwrap();
    for budget in wanted.into_iter().map(|wanted| wanted - 1).chain([maximum]) {
        let limits = ExecutionLimits {
            working_storage: budget,
            ..execution()
        };
        let make = || {
            Session::new_inner(
                &c,
                limits,
                Limits {
                    records: 262_144,
                    work: 10_000_000,
                    storage: 67_108_864,
                    lineage: 128,
                },
                Some(Input {
                    responses: None,
                    failures: &[],
                    limits: replay::Limits {
                        work: 1_000_000_000,
                        ..replay::Limits::default()
                    },
                }),
            )
        };
        let mut a = match make() {
            Ok(session) => session,
            Err(error) => {
                assert!(matches!(error.kind, ErrorKind::ConfiguredLimit(_)));
                continue;
            }
        };
        let mut b = make().unwrap();
        let _ = a.advance();
        let _ = b.advance();
        if budget == maximum {
            assert!(a.primary().unwrap().is_ok());
        } else {
            assert_eq!(
                a.primary().unwrap().unwrap_err().kind,
                ErrorKind::ConfiguredLimit("working storage")
            );
            let gates = storage_gates(&a);
            assert!(*gates.last().unwrap() > budget);
            assert!(gates[..gates.len() - 1]
                .iter()
                .all(|wanted| *wanted <= budget));
        }
        assert_eq!(compare_pair(&a, &b), Comparison::Equal, "storage {budget}");
    }
}

#[test]
fn independent_continuation_scope_completion_and_provenance_corruption_rejects() {
    let e = selected();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    for mutation in 0..4 {
        let a = run(&c, None, &[], execution());
        let b = run(&c, None, &[], execution());
        let mut observer = b.machine().observation.borrow_mut();
        let records = &mut observer.as_mut().unwrap().records;
        match mutation {
            0 => {
                let record = records
                    .iter_mut()
                    .find(|record| matches!(record, Record::Expression(_)))
                    .unwrap();
                *record = Record::Expression(usize::MAX);
            }
            1 => {
                let data = records
                    .iter_mut()
                    .find_map(|record| {
                        if let Record::Meta(Part::Scope, data) = record {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                data[1] = u64::MAX;
            }
            2 => {
                let data = records
                    .iter_mut()
                    .find_map(|record| {
                        if let Record::Meta(Part::Completion, data) = record {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                data[6] = 255;
            }
            _ => {
                let data = records
                    .iter_mut()
                    .find_map(|record| {
                        if let Record::Meta(Part::Production, data) = record {
                            Some(data)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                data[4] = 9_000_000;
            }
        }
        drop(observer);
        assert!(
            matches!(compare_pair(&a, &b), Comparison::Inconclusive(_)),
            "mutation {mutation}"
        );
    }
    let e = prepare_execution(
        "fn demo()->bool{if true{return true;}else{return false;}}",
        AdmissionLimits::default(),
    )
    .unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let a = run(&c, None, &[], execution());
    let b = run(&c, None, &[], execution());
    let mut observer = b.machine().observation.borrow_mut();
    let data = observer
        .as_mut()
        .unwrap()
        .records
        .iter_mut()
        .find_map(|record| {
            if let Record::Meta(Part::Task, data) = record {
                if data[0] == 13 {
                    return Some(data);
                }
                None
            } else {
                None
            }
        })
        .unwrap();
    data[1] = 9_000_000;
    drop(observer);
    assert!(matches!(compare_pair(&a, &b), Comparison::Inconclusive(_)));
}
