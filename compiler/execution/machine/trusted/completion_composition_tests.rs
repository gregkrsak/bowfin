//! M5 source composition; no synthetic source authority or runtime dispatcher.
use super::*;
use crate::trusted::{Grants, Selection};

fn artifact(source: &str, root: &str, scenario: Scenario) -> CheckedExecutable {
    let selection = Selection::sensor_demo(scenario, Grants::ALL, &[root]).unwrap();
    prepare_execution_with_profile(source, AdmissionLimits::default(), &selection).unwrap()
}

fn composition(source: &str, scenario: Scenario, output: &str, finishes: usize, releases: usize) {
    let e = artifact(source, "demo", scenario);
    let mut s =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    assert_eq!(e.render_result(s.advance().unwrap(), 4096).unwrap(), output);
    assert_eq!(s.live_sensors(), 0);
    let finish = e.trusted.as_ref().unwrap().catalog.operations[3].id;
    assert_eq!(
        s.events()
            .iter()
            .filter(|e| matches!(e,Event::Attempt(_,op) if *op == finish))
            .count(),
        finishes
    );
    assert_eq!(
        s.events()
            .iter()
            .filter(|e| matches!(e, Event::ReleaseComplete(_)))
            .count(),
        releases
    );
}

#[test]
fn defining_sample_runs_through_read_finish_release_and_record_delivery() {
    let source = include_str!("../../../../examples/sensor_finish.bow");
    let e = artifact(source, "sample", Scenario::Sample);
    let mut s =
        ExecutionSession::new(&e, e.entry("sample").unwrap(), ExecutionLimits::default()).unwrap();
    assert_eq!(
        e.render_result(s.advance().unwrap(), 4096).unwrap(),
        "Ok(Sample { reading: Reading(42), finished: FinishReceipt(1) })"
    );
    assert_eq!(s.live_sensors(), 0);
    assert!(s
        .events()
        .contains(&Event::FinishProtected(2, hir::ResultTag::Ok)));
    assert!(s.events().contains(&Event::ReleaseComplete(0)));
    assert!(s
        .events()
        .contains(&Event::Published(2, hir::ResultTag::Ok)));
}

#[test]
fn final_shared_use_can_precede_finish_but_a_still_live_handle_rejects() {
    composition("fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;let alias=&sensor;let reading=sensor_read(alias)or return;return sensor_finish(sensor);}", Scenario::Sample, "Ok(FinishReceipt(1))",1,1);
    let selected = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["sample"]).unwrap();
    assert!(prepare_execution_with_profile(
        include_str!("../../../../examples/sensor_finish_live_loan_fail.bow"),
        AdmissionLimits::default(),
        &selected
    )
    .is_err());
}

#[test]
fn owned_helper_parameter_protected_return_and_temporary_finish_once() {
    let source="fn relay(sensor:Sensor)->Sensor{return sensor;}fn finish_owned(sensor:Sensor)->Result<success:FinishReceipt,failure:SensorError>effects io{return sensor_finish(relay(sensor));}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;return finish_owned(relay(sensor));}";
    for (scenario, output) in [
        (Scenario::Sample, "Ok(FinishReceipt(1))"),
        (Scenario::FinishIoFailure, "Error(SensorError(3))"),
    ] {
        composition(source, scenario, output, 1, 1);
    }
}

#[test]
fn nested_protected_sensor_returns_feed_completion_after_existing_outcome_selection() {
    composition("fn wrap(sensor:Sensor)->Result<success:Result<success:Sensor,failure:SensorError>,failure:SensorError>{return Ok(Ok(sensor));}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;let inner=wrap(sensor)or return;let moved=inner or return;return sensor_finish(moved);}",Scenario::Sample,"Ok(FinishReceipt(1))",1,1);
}

#[test]
fn explicit_match_and_or_return_deliver_both_finish_alternatives() {
    for tail in ["let finished=sensor_finish(sensor)or return;return Ok(finished);", "match sensor_finish(sensor){Ok(finished)=>{return Ok(finished);}Err(problem)=>{return Error(problem);}}"] {
        let source=format!("fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{{let sensor=sensor_open()or return;{tail}}}");
        composition(&source,Scenario::Sample,"Ok(FinishReceipt(1))",1,1);
        composition(&source,Scenario::FinishPartialIoFailure,"Error(SensorError(3))",1,1);
    }
}

#[test]
fn consumed_owner_cannot_be_used_read_finished_replaced_or_reinitialized() {
    let selected = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    for tail in [
        "let moved=sensor;return Ok(finished);",
        "let reading=sensor_read(&sensor)or return;return Ok(finished);",
        "return sensor_finish(sensor);",
        "sensor=sensor_open()or return;return Ok(finished);",
        "replace sensor=sensor_open()or return;return Ok(finished);",
    ] {
        let source=format!("fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{{let sensor=sensor_open()or return;let finished=sensor_finish(sensor)or return;{tail}}}");
        assert!(
            prepare_execution_with_profile(&source, AdmissionLimits::default(), &selected).is_err()
        );
    }
    assert!(prepare_execution_with_profile("fn demo()->Result<success:bool,failure:SensorError>effects io{let sensor=sensor_open()or return;sensor_finish(sensor);return Ok(true);}",AdmissionLimits::default(),&selected).is_err());
}

#[test]
fn branch_finish_and_release_only_exit_remain_distinct() {
    for (condition, finishes) in [("true", 1), ("false", 0)] {
        let source=format!("fn status()->FinishReceipt{{return 1;}}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{{let sensor=sensor_open()or return;if {condition}{{return sensor_finish(sensor);}}else{{return Ok(status());}}}}");
        composition(
            &source,
            Scenario::Sample,
            "Ok(FinishReceipt(1))",
            finishes,
            1,
        );
    }
}

#[test]
fn loop_local_owners_finish_each_iteration_without_duplicating_release() {
    composition("fn count()->Reading{return 3;}fn status()->FinishReceipt{return 1;}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let mutable left=count();while 1<=left{let sensor=sensor_open()or return;let finished=sensor_finish(sensor)or return;left=left-1;if left==1{continue;}if left==0{break;}}return Ok(status());}",Scenario::Sample,"Ok(FinishReceipt(1))",3,3);
}

#[test]
fn loop_carried_consumption_remains_rejected_by_existing_checker() {
    let selected = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    assert!(prepare_execution_with_profile("fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;while true{let finished=sensor_finish(sensor)or return;}return sensor_finish(sensor);}",AdmissionLimits::default(),&selected).is_err());
}

#[test]
fn several_finish_result_arguments_preserve_written_order_and_individual_release() {
    let source="fn both(first:Result<success:FinishReceipt,failure:SensorError>,second:Result<success:FinishReceipt,failure:SensorError>)->Result<success:FinishReceipt,failure:SensorError>{match first{Ok(value)=>{return second;}Error(problem)=>{match second{Ok(other)=>{return Error(problem);}Error(other)=>{return Error(problem);}}}}}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let first=sensor_open()or return;let second=sensor_open()or return;return both(sensor_finish(second),sensor_finish(first));}";
    for (scenario, expected) in [
        (Scenario::Sample, "Ok(FinishReceipt(1))"),
        (Scenario::FinishIoFailure, "Error(SensorError(3))"),
    ] {
        let selection = Selection::sensor_demo(scenario, Grants::ALL, &["demo"]).unwrap();
        // This larger fixture uses an explicit supported admission allowance.
        // The production default and all predecessor budgets remain intact.
        let limits = AdmissionLimits {
            working_storage: AdmissionLimits::MAX.working_storage,
            ..AdmissionLimits::default()
        };
        let e = prepare_execution_with_profile(source, limits, &selection).unwrap();
        let mut s = ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
            .unwrap();
        assert_eq!(
            e.render_result(s.advance().unwrap(), 4096).unwrap(),
            expected
        );
        let finish = e.trusted.as_ref().unwrap().catalog.operations[3].id;
        assert_eq!(
            s.events()
                .iter()
                .filter(|event| matches!(event, Event::Attempt(_,op) if *op == finish))
                .count(),
            2
        );
        let released: Vec<_> = s
            .events()
            .iter()
            .filter_map(|event| {
                if let Event::ReleaseComplete(id) = event {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(released, [1, 0]);
        assert_eq!(s.live_sensors(), 0);
    }
}

#[test]
fn later_outer_range_trap_cannot_undo_an_inner_finish_and_release() {
    let source="fn zero()->Reading{return 0;}fn receive(finished:Result<success:FinishReceipt,failure:SensorError>,reading:Reading)->Result<success:FinishReceipt,failure:SensorError>{let value=finished or return;return Ok(value);}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;return receive(sensor_finish(sensor),zero()-1);}";
    let e = artifact(source, "demo", Scenario::Sample);
    let mut s =
        ExecutionSession::new(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
    assert!(matches!(
        s.advance().unwrap_err().kind,
        ErrorKind::RangeTrap { .. }
    ));
    assert_eq!(s.live_sensors(), 0);
    assert!(s
        .events()
        .contains(&Event::Published(1, hir::ResultTag::Ok)));
    assert!(s.events().contains(&Event::ReleaseComplete(0)));
    let prefix = s.events().to_vec();
    assert!(s.advance().is_err());
    assert_eq!(s.events(), prefix);
}

#[test]
fn source_formed_receipt_and_release_only_path_have_no_completion_event() {
    composition("fn status()->FinishReceipt{return 1;}fn demo()->Result<success:FinishReceipt,failure:SensorError>effects io{let sensor=sensor_open()or return;let value=status();let again=value;return Ok(again);}",Scenario::Sample,"Ok(FinishReceipt(1))",0,1);
}

#[test]
fn finish_grant_never_authorizes_verified_contracts_or_harness_calls() {
    let source = "type P=range 0..100;record Battery{charge:P}verified fn take(battery:&mut Battery,amount:P)requires sensor_finish(battery) ensures true{battery.charge-=amount;}fn demo(){}";
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    let error = prepare_execution_with_profile(source, AdmissionLimits::default(), &selection)
        .err()
        .unwrap();
    assert!(matches!(error.kind, ErrorKind::Frontend { .. }));
    assert!(crate::parse_source(source).is_ok());
    // Ordinary source with the same selection already executes above. The
    // grant does not extend the verified/harness domain or add a proof rule.
}
