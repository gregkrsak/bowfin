//! BED-032 public syntax and independent typing controls.
use crate::*;
const DECL: &str = "fn produce()->Result<success:bool,failure:bool>{return Ok(true);}";
fn typed(source: &str) -> Result<hir::Program, Vec<diagnostics::Diagnostic>> {
    types::check(resolve::resolve(&parse_source(source).unwrap())?)
}
#[test]
fn propagation_contextual_alias_and_trivia_parsing() {
    for function in ["fn", "function"] {
        for mutable in ["", "mut ", "mutable "] {
            for phrase in ["or return", "or\nreturn", "or // comment\n return"] {
                let source=format!("{DECL}{function} demo()->Result<success:bool,failure:bool>{{let {mutable}value=(produce()) {phrase};return Ok(value);}}");
                let p = typed(&source).unwrap();
                let hir::ValueStatement::Let {
                    propagation: Some(mode),
                    ..
                } = &p.functions()[1].as_ordinary().unwrap().body[0]
                else {
                    panic!()
                };
                assert_eq!(mode.failure, hir::ValueType::Bool);
            }
        }
    }
}
#[test]
fn propagation_or_identifier_discriminator_is_preserved() {
    typed("function extract(or:Result<success:bool,failure:bool>)->Result<success:bool,failure:bool>{let value=or or return;return Ok(value);}").unwrap();
    typed("fn or()->Result<success:bool,failure:bool>{return Err(false);}fn demo()->Result<success:bool,failure:bool>{let value=or() or return;return Ok(value);}").unwrap();
    assert!(compile_source("fn keep()->bool{let or=true;let Or=or;let OR=Or;return OR;}").is_ok());
}
#[test]
fn propagation_differing_success_types_keep_exact_failure_identity() {
    typed(&format!("{DECL}type N=range 0..1;fn n()->N{{return 0;}}fn demo()->Result<success:N,failure:bool>{{let value=produce() or return;return Ok(n());}}")).unwrap();
    let e=typed("type A=range 0..1;type B=range 0..1;fn p()->Result<success:bool,failure:A>{return Ok(true);}fn demo()->Result<success:bool,failure:B>{let value=p() or return;return Ok(value);}").unwrap_err();
    assert!(e[0].message.contains("failure payload type mismatch"));
    assert!(typed("type N=range 0..1;record A{x:N}record B{x:N}fn p()->Result<success:bool,failure:A>{return Ok(true);}fn demo()->Result<success:bool,failure:B>{let value=p() or return;return Ok(value);}").is_err());
}
#[test]
fn propagation_does_not_supply_constructor_context_or_complete_return() {
    for constructor in ["Ok(true)", "Error(false)", "Err(false)"] {
        assert!(typed(&format!("fn demo()->Result<success:bool,failure:bool>{{let x={constructor} or return;return Ok(true);}}")).is_err());
    }
    assert!(typed(&format!(
        "{DECL}fn demo()->Result<success:bool,failure:bool>{{let x=produce() or return;}}"
    ))
    .is_err());
    for result in ["", "->bool"] {
        assert!(typed(&format!(
            "{DECL}fn demo(){result}{{let x=produce() or return;return;}}"
        ))
        .is_err());
    }
}
#[test]
fn propagation_malformed_and_nested_source_positions_reject() {
    for tail in [
        "or",
        "or Error",
        "or failure",
        "or abort",
        "or return true",
        "Or return",
        "OR return",
        "or Return",
        "?",
        "or return or return",
    ] {
        assert!(parse_source(&format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{let x=produce() {tail};return Ok(true);}}")).is_err(),"{tail}");
    }
    for body in [
        "let x=(produce() or return);return Ok(true);",
        "let x=try produce();return Ok(true);",
        "return produce() or return;",
        "produce() or return;",
        "if produce() or return {}return Ok(true);",
        "while produce() or return {}return Ok(true);",
        "let mut x=true;x=produce() or return;return Ok(true);",
        "let x=consume(produce() or return);return Ok(true);",
        "match produce() or return {Ok(x)=>{}Error(y)=>{}}return Ok(true);",
    ] {
        assert!(
            parse_source(&format!(
                "{DECL}fn demo()->Result<success:bool,failure:bool>{{{body}}}"
            ))
            .is_err(),
            "{body}"
        );
    }
}

#[test]
fn propagation_ownership_preserves_earlier_and_nested_obligations() {
    assert!(compile_source(&format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{let x=produce() or return;return Ok(x);}}")).is_ok());
    assert!(compile_source(&format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{let pending=produce();let x=produce() or return;match pending{{Ok(v)=>{{}}Error(e)=>{{}}}}return Ok(x);}}")).is_err());
    let nested="fn nested()->Result<success:Result<success:bool,failure:bool>,failure:bool>{return Ok(Ok(true));}";
    assert!(compile_source(&format!("{nested}fn demo()->Result<success:bool,failure:bool>{{let pending=nested() or return;return Ok(true);}}")).is_err());
    assert!(compile_source(&format!("{nested}fn demo()->Result<success:bool,failure:bool>{{let pending=nested() or return;let x=pending or return;return Ok(x);}}")).is_ok());
}
#[test]
fn propagation_initializer_never_receives_terminal_loop_permission() {
    assert!(compile_source("fn demo(carried:Result<success:bool,failure:bool>,again:bool)->Result<success:bool,failure:bool>{while again{let x=carried or return;return Ok(x);}return carried;}").is_err());
    assert!(compile_source(&format!("{DECL}fn demo(again:bool)->Result<success:bool,failure:bool>{{while again{{let x=produce() or return;return Ok(x);}}return Ok(true);}}")).is_ok());
}

#[test]
fn propagation_public_execution_selects_success_and_each_failure_alias() {
    for constructor in ["Ok(true)", "Error(false)", "Err(false)"] {
        let source=format!("fn produce()->Result<success:bool,failure:bool>{{return {constructor};}}fn demo()->Result<success:bool,failure:bool>{{let value=produce() or return;return Ok(value);}}");
        let e =
            execution::prepare_execution(&source, execution::AdmissionLimits::default()).unwrap();
        let r = execution::execute(
            &e,
            e.entry("demo").unwrap(),
            execution::ExecutionLimits::default(),
        )
        .unwrap();
        let text = e.render_result(&r, 4096).unwrap();
        assert!(
            text.contains(if constructor.starts_with("Ok") {
                "Ok(Bool(true))"
            } else {
                "Error(Bool(false))"
            }),
            "{text}"
        );
    }
}

fn execute_text(source: &str) -> String {
    let e = execution::prepare_execution(source, execution::AdmissionLimits::MAX).unwrap();
    let r = execution::execute(
        &e,
        e.entry("demo").unwrap(),
        execution::ExecutionLimits::default(),
    )
    .unwrap();
    e.render_result(&r, 4096).unwrap()
}
#[test]
fn propagation_evaluates_once_and_executes_only_selected_continuation() {
    let source="type Count=range 0..2;fn two()->Count{return 2;}fn probe(count:&mut Count)->Result<success:bool,failure:bool>{*count=*count-1;return Ok(true);}fn demo()->Result<success:Count,failure:bool>{let mut count=two();let selected=probe(&mut count) or return;return Ok(count);}";
    assert_eq!(execute_text(source), "Ok(Count(1))");
    assert_eq!(execute_text("type N=range 0..1;record Packet{item:N}fn p()->Result<success:Packet,failure:bool>{return Ok(Packet{item:0});}fn demo()->Result<success:N,failure:bool>{let mutable packet=p() or return;let view=&mut packet;view.item=1;return Ok(view.item);}"),"Ok(N(1))");
    for alias in ["Error", "Err"] {
        assert_eq!(execute_text(&format!("fn p()->Result<success:bool,failure:bool>{{return {alias}(false);}}fn demo()->Result<success:bool,failure:bool>{{let x=p() or return;return Error(true);}}")),"Error(Bool(false))");
    }
}
#[test]
fn propagation_real_branches_match_arms_and_loop_exits_compose() {
    for body in [
        "if true{let x=produce() or return;return Ok(x);}else{return Err(false);}",
        "match produce(){Ok(flag)=>{let x=produce() or return;return Ok(x);}Error(e)=>{return Error(e);}}",
        "let mut flag=true;while flag{let x=produce() or return;flag=false;continue;}return Ok(true);",
        "while true{let x=produce() or return;break;}return Ok(true);",
        "let mut flag=true;while flag{while true{let x=produce() or return;break;}flag=false;continue;}return Ok(true);",
    ] {assert_eq!(execute_text(&format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{{body}}}")),"Ok(Bool(true))");}
    assert!(compile_source(&format!("{DECL}fn demo(flag:bool)->Result<success:bool,failure:bool>{{let pending=produce();if flag{{let x=pending or return;}}match pending{{Ok(v)=>{{}}Error(e)=>{{}}}}return Ok(true);}}")).is_err());
}
#[test]
fn propagation_public_nested_success_and_failure_obligations_remain_compositional() {
    assert_eq!(execute_text("fn p()->Result<success:Result<success:bool,failure:bool>,failure:bool>{return Ok(Err(false));}fn demo()->Result<success:bool,failure:bool>{let inner=p() or return;let x=inner or return;return Ok(x);}"),"Error(Bool(false))");
    assert_eq!(execute_text("fn p()->Result<success:bool,failure:Result<success:bool,failure:bool>>{return Error(Ok(true));}fn demo()->Result<success:bool,failure:Result<success:bool,failure:bool>>{let x=p() or return;return Ok(x);}"),"Error(Ok(Bool(true)))");
}
#[test]
fn propagation_excluded_fields_harness_verified_and_conditional_values_reject() {
    for source in [
        "type N=range 0..1;record Box{item:N}fn p()->Result<success:N,failure:bool>{return Err(false);}fn demo()->Result<success:bool,failure:bool>{let x=Box{item:p() or return};return Ok(true);}",
        "type N=range 0..1;record Box{item:N}let x=Box{item:0} or return;",
        "type N=range 0..1;record Box{item:N}verified fn work(x:&mut Box,n:N) requires true ensures true{let y=x or return;}",
    ]{assert!(parse_source(source).is_err(),"{source}");}
    assert!(compile_source(&format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{let x=if true{{produce()}}else{{produce()}} or return;return Ok(true);}}")).is_err());
}
#[test]
fn propagation_admission_host_failures_keep_cli_class_two_and_publish_no_executable() {
    use crate::bounded::result_admission::{with_failure, Allocation};
    let source=format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{let x=produce() or return;return Ok(x);}}");
    for site in [Allocation::Syntax, Allocation::Scratch, Allocation::Table] {
        let e = with_failure(site, 0, || {
            execution::prepare_execution(&source, execution::AdmissionLimits::MAX)
        })
        .err()
        .unwrap();
        assert!(matches!(e.kind, execution::ErrorKind::Host(_)));
        assert_eq!(
            crate::run_reporting::preparation_error(&e, 4096, &mut Vec::new()),
            Err(2)
        );
    }
}

#[test]
fn propagation_repeated_sites_have_linear_cfg_and_exact_admission_neighbors() {
    let mut selected = None;
    for count in [8, 16] {
        let mut lets = String::new();
        for n in 0..count {
            use std::fmt::Write as _;
            write!(lets, "let v{n}=produce() or return;").unwrap();
        }
        let source =
            format!("{DECL}fn demo()->Result<success:bool,failure:bool>{{{lets}return Ok(true);}}");
        let p = compile_source(&source).unwrap();
        let m = mir::lower(&p);
        let f = m
            .functions()
            .iter()
            .find(|f| p.function(f.function()).name() == "demo")
            .unwrap();
        assert_eq!(f.blocks().len(), 2 * count + 1);
        assert_eq!(execute_text(&source), "Ok(Bool(true))");
        selected = Some(source);
    }
    let source = selected.unwrap();
    let mut low = 1;
    let mut high = execution::AdmissionLimits::MAX.working_storage;
    while low < high {
        let middle = low + (high - low) / 2;
        let limits = execution::AdmissionLimits {
            working_storage: middle,
            ..execution::AdmissionLimits::MAX
        };
        match execution::prepare_execution(&source, limits) {
            Ok(_) => high = middle,
            Err(e) => {
                assert!(
                    matches!(e.kind, execution::ErrorKind::ConfiguredLimit(_)),
                    "{e:?}"
                );
                low = middle + 1;
            }
        }
    }
    assert!(execution::prepare_execution(
        &source,
        execution::AdmissionLimits {
            working_storage: low,
            ..execution::AdmissionLimits::MAX
        }
    )
    .is_ok());
    let e = execution::prepare_execution(
        &source,
        execution::AdmissionLimits {
            working_storage: low - 1,
            ..execution::AdmissionLimits::MAX
        },
    )
    .err()
    .unwrap();
    assert!(matches!(e.kind, execution::ErrorKind::ConfiguredLimit(_)));
}
