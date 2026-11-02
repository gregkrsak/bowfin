//! Private canonical-resource BED-032 protocol evidence, not production APIs.
use super::cleanup::{conservation_tests::conserved, Event};
use super::*;
use crate::effects::{Effect, EffectSet};
use crate::execution::{prepare_ordinary_synthetic, AdmissionLimits};
fn io() -> EffectSet {
    let mut s = EffectSet::EMPTY;
    s.insert(Effect::Io);
    s
}
fn artifact(source: &str) -> CheckedExecutable {
    prepare_ordinary_synthetic(
        hir::resources::fixture_effects(&format!("fn zero()->R{{return 0;}}{source}"), io(), io()),
        AdmissionLimits::MAX,
    )
    .unwrap()
}
fn run(e: &CheckedExecutable) -> Machine<'_> {
    let mut m = Machine::new(e, ExecutionLimits::default());
    m.start(e.entry("demo").unwrap()).unwrap();
    while let Some(task) = m.tasks.pop() {
        m.step(task).unwrap();
        if m.resources.is_some() {
            conserved(&m);
        }
    }
    m
}
fn completed(m: &Machine<'_>) -> Vec<u64> {
    m.resources.as_ref().map_or(Vec::new(), |s| {
        s.events
            .iter()
            .filter_map(|e| {
                if let Event::Complete(id) = e {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    })
}
fn stopped(m: &mut Machine<'_>, kind: fn(&ErrorKind) -> bool) {
    let e = m.resume().unwrap_err();
    assert!(kind(&e.kind), "{e:?}");
    if m.resources.is_some() {
        conserved(m);
    }
    assert!(m.resume().is_err());
}
const PRODUCE: &str =
    "fn produce_b()->Result<success:Leaf,failure:bool> effects io{return Ok(acquire());}";
const DIRECT:&str="fn sample()->Result<success:bool,failure:bool> effects io{let mutable earlier=acquire();let later=produce_b() or return;earlier=acquire();return Ok(true);}";
const MATCH:&str="fn sample()->Result<success:bool,failure:bool> effects io{let mutable earlier=acquire();match produce_b(){Ok(later)=>{earlier=acquire();return Ok(true);}Error(problem)=>{return Error(problem);}}}";
const DEMO: &str = "fn demo() effects io{match sample(){Ok(v)=>{}Error(e)=>{}}}";
#[test]
fn propagation_actual_scope_and_unchanged_match_have_distinct_acb_abc_traces() {
    for (body, expected) in [
        (DIRECT, vec![0, 2, 1]), (MATCH, vec![0, 1, 2]),
        ("fn sample()->Result<success:bool,failure:bool> effects io{let guard=acquire();while true{let later=produce_b() or return;break;}return Ok(true);}",vec![1,0]),
        ("fn sample()->Result<success:bool,failure:bool> effects io{let guard=acquire();let mutable again=true;while again{let later=produce_b() or return;again=false;continue;}return Ok(true);}",vec![1,0]),
        ("fn sample()->Result<success:bool,failure:bool> effects io{let guard=acquire();let mutable again=true;while again{let later=produce_b() or return;again=false;}return Ok(true);}",vec![1,0]),
        ("fn sample()->Result<success:bool,failure:bool> effects io{let guard=acquire();while true{let outer=acquire();while true{let later=produce_b() or return;return Ok(true);}break;}return Ok(true);}",vec![2,1,0]),
        ("fn sample()->Result<success:bool,failure:bool> effects io{let guard=acquire();match produce_b(){Ok(outer)=>{let later=produce_b() or return;return Ok(true);}Error(problem)=>{return Error(problem);}}}",vec![2,1,0]),
        ("fn sample()->Result<success:bool,failure:bool> effects io{let guard=acquire();let mutable later=produce_b() or return;if true{later=acquire();}return Ok(true);}",vec![1,2,0]),
    ] {
        let e = artifact(&format!("{PRODUCE}{body}{DEMO}"));
        let m = run(&e);
        assert_eq!(completed(&m), expected);
    }
}
#[test]
fn propagation_c_release_failure_retains_b_unattempted_and_protected_return() {
    let e = artifact(&format!("{PRODUCE}{DIRECT}{DEMO}"));
    let mut m = Machine::new(&e, ExecutionLimits::default());
    m.enable_resources().unwrap();
    m.resources.as_mut().unwrap().failure_at = Some(2);
    m.start(e.entry("demo").unwrap()).unwrap();
    stopped(&mut m, |k| matches!(k, ErrorKind::Host(_)));
    let state = m.resources.as_ref().unwrap();
    let events: Vec<_> = state
        .events
        .iter()
        .filter(|e| !matches!(e, Event::Acquire(_) | Event::Retire(_)))
        .collect();
    assert_eq!(
        events,
        vec![
            &Event::Attempt(0),
            &Event::Complete(0),
            &Event::Attempt(2),
            &Event::Failure(2)
        ]
    );
    assert!(state.protected.is_some());
    assert_eq!(completed(&m), vec![0]);
    assert!(!state
        .events
        .iter()
        .any(|e| matches!(e, Event::Attempt(1) | Event::Complete(1))));
    assert!(m
        .frames
        .iter()
        .any(|f| e.program.function(f.function).name() == "sample"));
}
fn scenarios() -> Vec<String> {
    let mut sources = Vec::new();
    for (s, payload, handle) in [
        ("bool", "true", ""),
        ("Packet", "Packet{data:0}", ""),
        ("Leaf", "acquire()", ""),
        (
            "Result<success:Leaf,failure:bool>",
            "Ok(acquire())",
            "match value{Ok(v)=>{}Error(e)=>{}}",
        ),
    ] {
        sources.push(format!("record Packet{{data:R}}fn make()->Result<success:{s},failure:bool> effects io{{return Ok({payload});}}fn sample()->Result<success:bool,failure:bool> effects io{{let value=make() or return;{handle}return Ok(true);}}{DEMO}"));
    }
    for (f, payload) in [
        ("bool", "false"),
        ("Packet", "Packet{data:0}"),
        ("Leaf", "acquire()"),
        ("Result<success:Leaf,failure:bool>", "Ok(acquire())"),
    ] {
        let handle = if f.starts_with("Result") {
            "match problem{Ok(v)=>{}Error(e)=>{}}"
        } else {
            ""
        };
        sources.push(format!("record Packet{{data:R}}fn make()->Result<success:bool,failure:{f}> effects io{{return Error({payload});}}fn sample()->Result<success:R,failure:{f}> effects io{{let guard=acquire();let value=make() or return;return Ok(zero());}}fn demo() effects io{{match sample(){{Ok(v)=>{{}}Error(problem)=>{{{handle}}}}}}}"));
    }
    sources
}
#[test]
fn propagation_all_four_payload_classes_on_both_alternatives_conserve() {
    for source in scenarios() {
        let e = artifact(&source);
        let m = run(&e);
        conserved(&m);
        assert!(m.frames.is_empty());
    }
}
#[test]
fn propagation_every_fuel_cut_and_exact_neighbor_preserve_custody() {
    for (scenario, source) in scenarios().into_iter().enumerate() {
        let e = artifact(&source);
        let complete = run(&e);
        let steps = complete.steps;
        assert_eq!(
            steps,
            [53, 59, 65, 88, 58, 64, 72, 95][scenario],
            "published KED-032 protocol fuel"
        );
        assert_eq!(
            complete.max_storage.get().max(complete.storage().unwrap()),
            [3904, 4344, 4392, 5232, 4080, 4544, 4624, 5504][scenario],
            "published KED-032 protocol working storage"
        );
        for budget in 1..=steps {
            let mut m = Machine::new(
                &e,
                ExecutionLimits {
                    steps: budget,
                    ..ExecutionLimits::default()
                },
            );
            let outcome = m.start(e.entry("demo").unwrap()).and_then(|_| m.resume());
            if m.resources.is_some() {
                conserved(&m);
            }
            if budget == steps {
                assert!(outcome.is_ok(), "{budget}: {outcome:?}");
            } else {
                assert!(matches!(
                    outcome.unwrap_err().kind,
                    ErrorKind::ConfiguredLimit("steps")
                ));
                assert!(m.resume().is_err());
            }
        }
    }
}
#[test]
fn propagation_every_allocation_growth_cut_is_host_without_payload_loss() {
    for source in scenarios() {
        let e = artifact(&source);
        let mut reached = false;
        for cut in 0..1000 {
            let mut m = Machine::new(&e, ExecutionLimits::default());
            temporaries::FAIL_ALLOCATION.with(|p| p.set(Some(cut)));
            let result = m.start(e.entry("demo").unwrap()).and_then(|_| m.resume());
            temporaries::FAIL_ALLOCATION.with(|p| p.set(None));
            if m.resources.is_some() {
                conserved(&m);
            }
            match result {
                Ok(_) => {
                    reached = true;
                    break;
                }
                Err(e) => {
                    assert!(matches!(e.kind, ErrorKind::Host(_)), "{cut}: {e:?}");
                    assert!(m.resume().is_err());
                }
            }
        }
        assert!(reached);
    }
}
#[test]
fn propagation_combined_storage_has_exact_sufficient_and_insufficient_neighbors() {
    for source in [
        scenarios().remove(2),
        scenarios().remove(3),
        scenarios().remove(6),
        scenarios().remove(7),
    ] {
        let e = artifact(&source);
        let complete = run(&e);
        let peak = complete.max_storage.get().max(complete.storage().unwrap());
        for budget in 1..=peak {
            let mut m = Machine::new(
                &e,
                ExecutionLimits {
                    working_storage: budget,
                    ..ExecutionLimits::default()
                },
            );
            let result = m.start(e.entry("demo").unwrap()).and_then(|_| m.resume());
            if m.resources.is_some() {
                conserved(&m);
            }
            if budget == peak {
                assert!(result.is_ok(), "{peak}: {result:?}");
            } else {
                assert!(matches!(
                    result.unwrap_err().kind,
                    ErrorKind::ConfiguredLimit("working storage")
                ));
                assert!(m.resume().is_err());
            }
        }
    }
}
#[test]
fn propagation_failure_return_cleans_guard_before_delivering_failure_payload() {
    let source="fn make()->Result<success:bool,failure:Leaf> effects io{return Err(acquire());}fn sample()->Result<success:R,failure:Leaf> effects io{let guard=acquire();let value=make() or return;return Ok(zero());}fn demo() effects io{match sample(){Ok(v)=>{}Error(problem)=>{}}}";
    let e = artifact(source);
    assert_eq!(completed(&run(&e)), vec![0, 1]);
    let mut m = Machine::new(&e, ExecutionLimits::default());
    m.enable_resources().unwrap();
    m.resources.as_mut().unwrap().failure_at = Some(0);
    m.start(e.entry("demo").unwrap()).unwrap();
    stopped(&mut m, |k| matches!(k, ErrorKind::Host(_)));
    assert!(matches!(
        m.resources.as_ref().unwrap().protected,
        Some(Value::Result(_))
    ));
    assert!(completed(&m).is_empty());
    assert!(!m
        .resources
        .as_ref()
        .unwrap()
        .events
        .iter()
        .any(|e| matches!(e, Event::Attempt(1))));
}

#[test]
fn propagation_independent_hir_mir_scope_transfer_and_origin_mutations_reject() {
    let source = format!(
        "{PRODUCE}{}{DEMO}",
        DIRECT.replace(";earlier=acquire();", ";if true{earlier=acquire();}")
    );
    for mutation in 0..18 {
        let mut e = artifact(&source);
        crate::mir::mutate_propagation_mir(&mut e.mir, mutation);
        assert!(
            matches!(e.validate().unwrap_err().kind, ErrorKind::Integrity(_)),
            "MIR mutation {mutation}"
        );
    }
    for mutation in 0..7 {
        let mut e = artifact(&source);
        let f = e
            .program
            .functions
            .iter_mut()
            .find(|f| f.name() == "sample")
            .unwrap();
        let hir::Function::Ordinary(f) = f else {
            unreachable!()
        };
        let hir::ValueStatement::Let {
            propagation: Some(mode),
            local,
            initializer,
            ..
        } = &mut f.body[1]
        else {
            unreachable!()
        };
        match mutation {
            0 => mode.inner = hir::ResultTypeId(999),
            1 => mode.outer = hir::ResultTypeId(999),
            2 => mode.failure = ValueType::Range(hir::RangeTypeId(999)),
            3 => *local = hir::LocalId(999),
            4 => mode.syntax.phrase.end -= 1,
            5 => mode.syntax.binding.start += 1,
            6 => initializer.ty = ExprType::Bool,
            _ => unreachable!(),
        }
        assert!(
            matches!(e.validate().unwrap_err().kind, ErrorKind::Integrity(_)),
            "HIR mutation {mutation}"
        );
    }
}
#[test]
fn propagation_cleanup_effects_cover_error_exit_but_not_returned_failure_payload() {
    let mut filesystem = EffectSet::EMPTY;
    filesystem.insert(Effect::Filesystem);
    let producer = "fn make()->Result<success:bool,failure:bool> effects io{return Error(false);}";
    let sample="fn sample()->Result<success:Leaf,failure:bool> effects io{let guard=acquire();let x=make() or return;return Ok(guard);}fn demo(){}";
    let p = hir::resources::fixture_effects(&format!("{producer}{sample}"), io(), filesystem);
    assert!(prepare_ordinary_synthetic(p, AdmissionLimits::MAX).is_err());
    let p = hir::resources::fixture_effects(
        &format!(
            "{producer}{}",
            sample.replace("effects io{let guard", "effects io,filesystem{let guard")
        ),
        io(),
        filesystem,
    );
    let e = prepare_ordinary_synthetic(p, AdmissionLimits::MAX).unwrap();
    let sample = e
        .program
        .functions()
        .iter()
        .find(|f| f.name() == "sample")
        .unwrap()
        .id();
    assert!(e
        .approval
        .cleanup_sites
        .iter()
        .any(|site| site.function == sample
            && site.effects == filesystem
            && site.kind == crate::effects::custody::Kind::Return));
    let p=hir::resources::fixture_effects("fn make()->Result<success:bool,failure:Leaf> effects io{return Error(acquire());}fn forward()->Result<success:bool,failure:Leaf> effects io{let x=make() or return;return Ok(x);}fn demo(){}",io(),filesystem);
    let e = prepare_ordinary_synthetic(p, AdmissionLimits::MAX).unwrap();
    let forward = e
        .program
        .functions()
        .iter()
        .find(|f| f.name() == "forward")
        .unwrap()
        .id();
    assert!(!e
        .approval
        .cleanup_sites
        .iter()
        .any(|site| site.function == forward));
}
#[test]
fn propagation_preparation_lowering_and_revalidation_allocation_failures_stay_typed() {
    let source="fn make()->Result<success:bool,failure:bool>{return Err(false);}fn demo()->Result<success:bool,failure:bool>{let x=make() or return;return Ok(x);}";
    let mut reached = false;
    for cut in 0..1000 {
        crate::mir::temporaries::FAIL_ALLOCATION.with(|p| p.set(Some(cut)));
        let result = crate::execution::prepare_execution(source, AdmissionLimits::MAX);
        crate::mir::temporaries::FAIL_ALLOCATION.with(|p| p.set(None));
        match result {
            Ok(_) => {
                reached = true;
                break;
            }
            Err(e) => assert!(matches!(e.kind, ErrorKind::Host(_)), "{cut}: {e:?}"),
        }
    }
    assert!(reached);
    let e = crate::execution::prepare_execution(source, AdmissionLimits::MAX).unwrap();
    crate::mir::temporaries::FAIL_ALLOCATION.with(|p| p.set(Some(0)));
    let result = e.validate();
    crate::mir::temporaries::FAIL_ALLOCATION.with(|p| p.set(None));
    assert!(matches!(result.unwrap_err().kind, ErrorKind::Host(_)));
}
#[test]
fn propagation_resource_free_and_terminal_traps_do_not_become_source_errors() {
    let source="fn make()->Result<success:bool,failure:bool>{return Err(false);}fn demo()->Result<success:bool,failure:bool>{let x=make() or return;return Ok(x);}";
    let e = crate::execution::prepare_execution(source, AdmissionLimits::MAX).unwrap();
    let m = run(&e);
    assert!(m.resources.is_none());
    let trap="type N=range 0..1;fn n()->N{return 0;}fn make()->Result<success:bool,failure:bool>{let x=n()-1;return Err(false);}fn demo()->Result<success:bool,failure:bool>{let x=make() or return;return Ok(x);}";
    let e = crate::execution::prepare_execution(trap, AdmissionLimits::MAX).unwrap();
    let mut m = Machine::new(&e, ExecutionLimits::default());
    m.start(e.entry("demo").unwrap()).unwrap();
    stopped(&mut m, |k| matches!(k, ErrorKind::RangeTrap { .. }));
}

#[test]
fn propagation_forwarded_cell_tag_origin_parent_and_foreign_approval_reject() {
    let source="fn make()->Result<success:bool,failure:Result<success:Leaf,failure:bool>> effects io{return Error(Ok(acquire()));}fn sample()->Result<success:R,failure:Result<success:Leaf,failure:bool>> effects io{let guard=acquire();let x=make() or return;return Ok(zero());}fn demo() effects io{match sample(){Ok(v)=>{}Error(e)=>{match e{Ok(resource)=>{}Error(flag)=>{}}}}}";
    for mutation in 0..3 {
        let e = artifact(source);
        let function = e
            .program
            .functions()
            .iter()
            .find(|f| f.name() == "sample")
            .unwrap()
            .as_ordinary()
            .unwrap();
        let phrase = function
            .body
            .iter()
            .find_map(|s| match s {
                hir::ValueStatement::Let {
                    propagation: Some(p),
                    ..
                } => Some(p.syntax.phrase),
                _ => None,
            })
            .unwrap();
        let mut m = Machine::new(&e, ExecutionLimits::default());
        m.start(e.entry("demo").unwrap()).unwrap();
        let id = loop {
            let task = m.tasks.pop().unwrap();
            m.step(task).unwrap();
            if let Some(id) = m.results.as_ref().and_then(|s| {
                s.cells
                    .iter()
                    .position(|c| c.origin == phrase && c.phase == results::Phase::Active)
            }) {
                break id;
            }
        };
        let state = m.results.as_mut().unwrap();
        match mutation {
            0 => state.cells[id].tag = 0,
            1 => state.cells[id].origin.start += 1,
            2 => {
                let Some(Value::Result(child)) = state.cells[id].payload else {
                    panic!()
                };
                state.cells[child.0].parent = None;
            }
            _ => unreachable!(),
        }
        assert!(
            matches!(m.resume().unwrap_err().kind, ErrorKind::Integrity(_)),
            "mutation {mutation}"
        );
        assert!(m.resume().is_err());
    }
    let mut e = artifact(source);
    let foreign = artifact(source);
    e.mir_approval = foreign.approval.clone();
    assert!(matches!(
        e.validate().unwrap_err().kind,
        ErrorKind::Integrity(_)
    ));
}
