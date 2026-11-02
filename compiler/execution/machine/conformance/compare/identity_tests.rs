//! Actual interpreter controls and separately labeled observation-protocol
//! falsification. None selects or executes a native engine.
use super::*;

fn observation() -> Limits {
    Limits {
        records: 262_144,
        work: 10_000_000,
        storage: 67_108_864,
        lineage: 128,
    }
}
fn comparison() -> ComparisonLimits {
    ComparisonLimits {
        work: 100_000_000,
        ..ComparisonLimits::default()
    }
}
fn ordinary<'a>(c: &'a contract::Contract<'a>, first_frame: u64) -> Session<'a> {
    let mut s = Session::new(c, ExecutionLimits::default(), observation()).unwrap();
    // Raw frame allocation is private, not a new source/runtime input.
    s.machine_mut().next_frame = first_frame;
    s.advance().unwrap();
    s
}

#[test]
fn lawful_raw_frame_changes_preserve_shared_alias_equality_and_ancestry() {
    let source="fn read(view:&bool)->bool{return *view;}fn demo()->bool{let value=true;let first=&value;let second=first;return read(second);}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let a = ordinary(&c, 0);
    let b = ordinary(&c, 900);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
}

#[test]
fn conserving_wrong_parameter_association_is_different_despite_the_same_value() {
    use crate::trusted::{Grants, Scenario, Selection};
    let source="fn relay(sensor:Sensor)->Sensor{return sensor;}fn consume(first:Sensor,second:Sensor)->bool effects io{return true;}fn demo()->Result<success:bool,failure:SensorError>effects io{let first=sensor_open()or return;let second=sensor_open()or return;return Ok(consume(relay(second),relay(first)));}";
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["demo"]).unwrap();
    let e = prepare_execution_with_profile(source, AdmissionLimits::default(), &selection).unwrap();
    let c = contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
        .unwrap();
    let mut a = Session::new(&c, ExecutionLimits::default(), observation()).unwrap();
    a.advance().unwrap();
    let function = e.program.functions()[1].id();
    let params = &e
        .program
        .function(function)
        .as_ordinary()
        .unwrap()
        .parameters;
    SWAP_PARAMETERS.with(|slot| {
        slot.set(Some((
            function,
            Place::Parameter(params[0]),
            Place::Parameter(params[1]),
        )))
    });
    let mut b = Session::new(&c, ExecutionLimits::default(), observation()).unwrap();
    let result = b.advance();
    SWAP_PARAMETERS.with(|slot| slot.set(None));
    result.unwrap();
    assert_eq!(a.primary(), b.primary());
    let compared = compare(&a, &b, comparison());
    assert!(
        matches!(compared, Comparison::Different { .. }),
        "{compared:?}"
    );
}

#[test]
fn foreign_realm_stale_generation_and_bad_payload_reject_before_projection() {
    use crate::trusted::{Grants, Scenario, Selection};
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["sample"]).unwrap();
    let e = prepare_execution_with_profile(
        include_str!("../../../../../examples/sensor_finish.bow"),
        AdmissionLimits::default(),
        &selection,
    )
    .unwrap();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let mut a = Session::new(&c, ExecutionLimits::default(), observation()).unwrap();
    a.advance().unwrap();
    for mutation in 0..4 {
        let mut b = Session::new(&c, ExecutionLimits::default(), observation()).unwrap();
        b.advance().unwrap();
        let mut observer = b.machine().observation.borrow_mut();
        let records = &mut observer.as_mut().unwrap().records;
        match mutation {
            0 => {
                let d = records
                    .iter_mut()
                    .find_map(|r| {
                        if let Record::Meta(Part::Token, d) = r {
                            Some(d)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                d[0] += 1;
            }
            1 => {
                let d = records
                    .iter_mut()
                    .find_map(|r| {
                        if let Record::Meta(Part::GenerationSlot, d) = r {
                            Some(d)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                d[2] = 9_000_000;
            }
            2 => {
                let d = records
                    .iter_mut()
                    .find_map(|r| {
                        if let Record::Meta(Part::ResultCell, d) = r {
                            Some(d)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                d[5] = 255;
            }
            _ => {
                let d = records
                    .iter_mut()
                    .find_map(|r| {
                        if let Record::Meta(Part::TemporaryOwner, d) = r {
                            Some(d)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                d[3] = 3;
            }
        }
        drop(observer);
        assert!(
            matches!(compare(&a, &b, comparison()), Comparison::Inconclusive(_)),
            "mutation {mutation}"
        );
    }
}

#[test]
fn nested_results_branch_match_propagation_and_loop_controls_have_complete_evidence() {
    for body in [
        "if true{let x=produce() or return;return Ok(x);}else{return Error(false);}",
        "match produce(){Ok(flag)=>{let x=produce() or return;return Ok(x);}Error(e)=>{return Error(e);}}",
        "let mut flag=true;while flag{let x=produce() or return;flag=false;continue;}return Ok(true);",
        "while true{let x=produce() or return;break;}return Ok(true);",
        "let mut flag=true;while flag{while true{let x=produce() or return;break;}flag=false;continue;}return Ok(true);",
    ] {
        let source=format!("fn produce()->Result<success:bool,failure:bool>{{return Ok(true);}}fn demo()->Result<success:bool,failure:bool>{{{body}}}");
        let e=prepare_execution(&source,AdmissionLimits::default()).unwrap();
        let c=contract::Contract::extract(&e,e.entry("demo").unwrap(),contract::Limits::default()).unwrap();
        let a=ordinary(&c,0);let b=ordinary(&c,700);
        let compared=compare(&a,&b,comparison());
        assert_eq!(compared,Comparison::Equal,"{body}");
    }
    for source in [
        "fn p()->Result<success:Result<success:bool,failure:bool>,failure:bool>{return Ok(Error(false));}fn demo()->Result<success:bool,failure:bool>{let inner=p() or return;let x=inner or return;return Ok(x);}",
        "fn p()->Result<success:bool,failure:Result<success:bool,failure:bool>>{return Error(Ok(true));}fn demo()->Result<success:bool,failure:Result<success:bool,failure:bool>>{let x=p() or return;return Ok(x);}",
    ] {
        let e=prepare_execution(source,AdmissionLimits::default()).unwrap();
        let c=contract::Contract::extract(&e,e.entry("demo").unwrap(),contract::Limits::default()).unwrap();
        let a=ordinary(&c,0);let b=ordinary(&c,700);
        assert_eq!(compare(&a,&b,comparison()),Comparison::Equal,"{source}");
    }
}

#[test]
fn private_resource_protocol_cleanup_and_replacement_are_separate_positive_evidence() {
    use crate::effects::{Effect, EffectSet};
    let mut io = EffectSet::EMPTY;
    io.insert(Effect::Io);
    for body in [
        "let first=acquire();let second=acquire();",
        "let mutable first=acquire();first=acquire();",
        "let mutable again=true;while again{let next=acquire();again=false;continue;}",
    ] {
        let source = format!("fn demo() effects io{{{body}}}");
        let e = crate::execution::prepare_ordinary_synthetic(
            hir::resources::fixture_effects(&source, io, io),
            AdmissionLimits::MAX,
        )
        .unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        let a = ordinary(&c, 0);
        let b = ordinary(&c, 700);
        assert_eq!(
            compare(&a, &b, comparison()),
            Comparison::Equal,
            "private protocol: {body}"
        );
    }
}

#[test]
fn no_value_calls_and_short_circuit_paths_keep_exact_static_correspondence() {
    for source in [
        "fn helper(){let value=true;}fn demo(){helper();}",
        "fn set(view:&mut bool){*view=false;}fn demo()->bool{let mut value=true;set(&mut value);return value;}",
        "fn never()->bool{while true{}return true;}fn demo()->bool{return true||never();}",
        "type N=range 0..5;record Pair{left:N,right:N}fn demo()->Pair{return Pair{right:2,left:1};}",
    ] {
        let e=prepare_execution(source,AdmissionLimits::default()).unwrap();
        let c=contract::Contract::extract(&e,e.entry("demo").unwrap(),contract::Limits::default()).unwrap();
        let a=ordinary(&c,0);let b=ordinary(&c,700);
        assert_eq!(compare(&a,&b,comparison()),Comparison::Equal,"{source}");
    }
}

// Analytic representation fixture, not independent execution or native evidence.
fn relabel(record: Record) -> Record {
    fn id(raw: u64) -> u64 {
        if raw == ABSENT {
            raw
        } else {
            raw + 10_000
        }
    }
    match record {
        Record::Meta(part, mut d) => {
            let original = d;
            for (index, value) in d.iter_mut().enumerate() {
                if meta_kind(part, original, index).is_some()
                    || matches!(
                        (part, index),
                        (Part::TemporaryHandle | Part::TemporaryOwner, 1) | (Part::Completion, 4)
                    )
                {
                    *value = id(*value);
                }
            }
            Record::Meta(part, d)
        }
        Record::Frame(frame, function, block, statement, parent, span) => {
            Record::Frame(id(frame), function, block, statement, parent.map(id), span)
        }
        Record::Reference(kind, ty, frame, place) => Record::Reference(kind, ty, id(frame), place),
        Record::TemporaryBegin(act, frame, region, parent) => {
            Record::TemporaryBegin(id(act), id(frame), region, parent.map(id))
        }
        Record::TemporaryRegister(act, owner, site) => {
            Record::TemporaryRegister(id(act), id(owner as u64) as usize, site)
        }
        Record::TemporaryTransfer(act, owner, destination) => {
            Record::TemporaryTransfer(id(act), id(owner as u64) as usize, destination)
        }
        Record::TemporaryCleanup(act, owner) => {
            Record::TemporaryCleanup(id(act), id(owner as u64) as usize)
        }
        Record::TemporaryRetire(act) => Record::TemporaryRetire(id(act)),
        Record::ResourceEvent(kind, raw) => Record::ResourceEvent(kind, id(raw)),
        Record::Continuation(frame) => Record::Continuation(id(frame)),
        Record::Destination(frame, place, field) => Record::Destination(id(frame), place, field),
        Record::Publication(result, done) => Record::Publication(id(result as u64) as usize, done),
        Record::Transit(result, done) => Record::Transit(id(result as u64) as usize, done),
        Record::CleanupLeaf(leaf, done) => Record::CleanupLeaf(id(leaf), done),
        Record::TrustedEvent(event) => {
            use trusted::Event::*;
            Record::TrustedEvent(match event {
                Attempt(inv, op) => Attempt(id(inv), op),
                Received(inv, tag) => Received(id(inv), tag),
                Published(inv, tag) => Published(id(inv), tag),
                FinishProtected(inv, tag) => FinishProtected(id(inv), tag),
                FinishPrefix(inv, prefix) => FinishPrefix(id(inv), prefix),
                ReleaseAttempt(sensor, op) => ReleaseAttempt(id(sensor), op),
                ReleaseComplete(sensor) => ReleaseComplete(id(sensor)),
                ReleaseFailure(sensor) => ReleaseFailure(id(sensor)),
            })
        }
        other => other,
    }
}

#[test]
fn schema_relabeling_preserves_one_global_map_for_cells_owners_generations_and_tickets() {
    use crate::trusted::{Grants, Scenario, Selection};
    let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["sample"]).unwrap();
    let e = prepare_execution_with_profile(
        include_str!("../../../../../examples/sensor_finish.bow"),
        AdmissionLimits::default(),
        &selection,
    )
    .unwrap();
    let c =
        contract::Contract::extract(&e, e.entry("sample").unwrap(), contract::Limits::default())
            .unwrap();
    let mut a = Session::new(&c, ExecutionLimits::default(), observation()).unwrap();
    a.advance().unwrap();
    let mut b = Session::new(&c, ExecutionLimits::default(), observation()).unwrap();
    b.advance().unwrap();
    let mut observer = b.machine().observation.borrow_mut();
    for record in &mut observer.as_mut().unwrap().records {
        *record = relabel(*record);
    }
    drop(observer);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
}

#[test]
fn coordinated_unbound_origins_and_stale_alias_targets_cannot_self_certify() {
    for source in [
        "fn demo()->bool{return true;}",
        "fn read(view:&bool)->bool{return *view;}fn demo()->bool{let value=true;return read(&value);}",
    ] {
        let e=prepare_execution(source,AdmissionLimits::default()).unwrap();
        let c=contract::Contract::extract(&e,e.entry("demo").unwrap(),contract::Limits::default()).unwrap();
        let a=ordinary(&c,0);let b=ordinary(&c,0);
        for session in [&a,&b] {
            let mut observer=session.machine().observation.borrow_mut();
            let records=&mut observer.as_mut().unwrap().records;
            if source.contains("fn read") {
                let record=records.iter_mut().find(|r|matches!(r,Record::Reference(..))).unwrap();
                let Record::Reference(_,_,frame,_) = record else {unreachable!()};
                *frame=7_000_000;
            } else {
                let record=records.iter_mut().find(|r|matches!(r,Record::Cut(..))).unwrap();
                let Record::Cut(_,_,_,origin,_,_) = record else {unreachable!()};
                origin.start=7_000_000;origin.end=7_000_001;
            }
        }
        assert!(matches!(compare(&a,&b,comparison()),Comparison::Inconclusive(_)));
    }
}
