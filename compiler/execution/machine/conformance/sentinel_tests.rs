//! One sealed addressed control, not a public scenario or input facility.
use super::*;
use replay::{Address, Birth, Input, Item, Phase, Response, Site};

#[test]
fn sentinel_combined_read_and_finish_failure_preserves_both_original_facts() {
    let selection = crate::trusted::Selection::sensor_demo(
        crate::trusted::Scenario::Sample,
        crate::trusted::Grants::ALL,
        &["monitor"],
    )
    .unwrap();
    let e = prepare_execution_with_profile(
        include_str!("../../../../examples/sentinel.bow"),
        AdmissionLimits {
            tokens: 2048,
            frontend_work: 10_000_000_000_000_000,
            working_storage: 1_073_741_824,
            diagnostic_storage: 4_194_304,
            ..AdmissionLimits::default()
        },
        &selection,
    )
    .unwrap();
    let c =
        contract::Contract::extract(&e, e.entry("monitor").unwrap(), contract::Limits::default())
            .unwrap();
    let approved = e.trusted.as_ref().unwrap();
    let catalog = &approved.catalog;
    let open = approved
        .invocations
        .iter()
        .find(|p| p.operation == catalog.operations[0].id)
        .unwrap();
    let read = approved
        .invocations
        .iter()
        .find(|p| p.operation == catalog.operations[1].id)
        .unwrap();
    // The first written finish is in the read-error arm. Select its canonical
    // approved row, not a guessed global function/table position.
    let finish = approved
        .invocations
        .iter()
        .filter(|p| p.completion.is_some())
        .min_by_key(|p| p.source.start)
        .unwrap();
    let address = |ordinal, phase| Address {
        site: Site::Invocation(ordinal),
        activation: 0,
        occurrence: 1,
        phase,
    };
    let responses = [
        Item {
            address: address(open.ordinal, Phase::Response),
            response: Response::Open(catalog.sensor),
        },
        Item {
            address: address(read.ordinal, Phase::Response),
            response: Response::Error(catalog.error, 2, 0),
        },
        Item {
            address: address(finish.ordinal, Phase::Response),
            response: Response::Error(catalog.error, 3, 1),
        },
        Item {
            address: Address {
                site: Site::Release(c.entry(), finish.completion.as_ref().unwrap().cleanup.span),
                activation: 0,
                occurrence: 1,
                phase: Phase::ReleaseAttempt,
            },
            response: Response::Release(
                Birth {
                    plan: open.ordinal,
                    activation: 0,
                    occurrence: 1,
                },
                true,
            ),
        },
    ];
    let mut s = Session::new_inner(
        &c,
        ExecutionLimits {
            steps: 512,
            call_depth: 8,
            ..ExecutionLimits::default()
        },
        Limits {
            records: 262_144,
            work: 100_000_000,
            storage: 67_108_864,
            lineage: 128,
        },
        Some(Input {
            responses: Some(&responses),
            failures: &[],
            limits: replay::Limits {
                work: 1_000_000_000,
                ..replay::Limits::default()
            },
        }),
    )
    .unwrap();
    s.advance().unwrap();
    let ExecutionResult::Value(RuntimeValue::Result(layers)) = s.primary().unwrap().unwrap() else {
        panic!("expected application Result")
    };
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].tag, hir::ResultTag::Error);
    let Some(ResultPayload::Record { fields, .. }) = &layers[0].payload else {
        panic!("expected Failure")
    };
    let field = |name| {
        fields
            .iter()
            .find(|(id, _)| e.program.fields()[id.0].name == name)
            .unwrap()
            .1
            .value
    };
    assert_eq!(
        (
            field("phase"),
            field("problem"),
            field("read_status"),
            field("completion")
        ),
        (3, 3, 2, 2)
    );
    assert_eq!((field("observations"), field("completed")), (0, 0));
    assert_eq!(
        s.machine()
            .control
            .borrow()
            .as_ref()
            .unwrap()
            .used_responses(),
        4
    );
    let Engine::Selected(selected) = &s.engine else {
        unreachable!()
    };
    assert_eq!(selected.live_sensors(), 0);
    let events = selected.events();
    assert!(events.contains(&trusted::Event::FinishPrefix(2, 1)));
    let release = events
        .iter()
        .position(|e| matches!(e, trusted::Event::ReleaseComplete(0)))
        .unwrap();
    let delivery = events
        .iter()
        .position(|e| matches!(e, trusted::Event::Published(2, hir::ResultTag::Error)))
        .unwrap();
    assert!(release < delivery);
}
