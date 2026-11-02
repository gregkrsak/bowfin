//! Ordinary-source frontend conformance. Execution/custody is tested separately.
use bowfin_compiler::{ast, compile_source, hir, mir, parse_source};

const RESULT: &str = "Result<success: bool, failure: bool>";
fn producer() -> String {
    format!("fn produce() -> {RESULT} {{ return Ok(true); }}")
}
fn accepts(source: &str) -> hir::Program {
    compile_source(source).unwrap_or_else(|e| panic!("{source}\n{e}"))
}
fn rejects(source: &str, fragment: &str) {
    let error = compile_source(source).expect_err(source).to_string();
    assert!(
        error.contains(fragment),
        "{source}\nexpected {fragment:?}, got {error}"
    );
}

#[test]
fn failure_aliases_normalize_before_typing_and_coverage() {
    for construction in ["Error", "Err"] {
        for pattern in ["Error", "Err"] {
            for reverse in [false, true] {
                let ok = "Ok(value) => { return value; }";
                let failed = format!("{pattern}(problem) => {{ return problem; }}");
                let arms = if reverse {
                    format!("{failed} {ok}")
                } else {
                    format!("{ok} {failed}")
                };
                let source = format!("fn produce() -> {RESULT} {{ return {construction}(false); }} fn demo() -> bool {{ match produce() {{ {arms} }} }}");
                let p = accepts(&source);
                let hir::ValueStatement::Return { value, .. } =
                    &p.functions()[0].as_ordinary().unwrap().body[0]
                else {
                    panic!()
                };
                assert!(matches!(
                    value.kind,
                    hir::ExprKind::ResultConstruct {
                        tag: hir::ResultTag::Error,
                        ..
                    }
                ));
                assert_eq!(p.result_types().len(), 1);
                let m = mir::lower(&p);
                assert!(m.functions().iter().all(|f| f.validate().is_ok()));
            }
        }
    }
}

#[test]
fn exact_labeled_types_and_nested_contexts_are_canonical() {
    accepts("fn inner() -> Result<success: bool, failure: bool> { return Err(false); } fn outer() -> Result<success: Result<success: bool, failure: bool>, failure: bool> { return Ok(inner()); }");
    accepts("fn nested() -> Result<success: Result<success: bool, failure: bool>, failure: bool> { return Ok(Err(true)); }");
    for ty in [
        "Result<bool, bool>",
        "Result<failure: bool, success: bool>",
        "Result<success: bool, success: bool>",
        "Result<success: bool>",
        "Result<success: bool, failure: bool, extra: bool>",
        "Result<success: &bool, failure: bool>",
        "Result<success: NoValue, failure: bool>",
    ] {
        assert!(
            compile_source(&format!("fn f() -> {ty} {{ return Ok(true); }}")).is_err(),
            "{ty}"
        );
    }
}

#[test]
fn every_constructor_requires_independent_complete_context() {
    for variant in ["Ok", "Error", "Err"] {
        rejects(
            &format!("fn demo() {{ let pending = {variant}(true); }}"),
            "cannot infer complete Result type",
        );
        rejects(
            &format!("fn demo() -> bool {{ return {variant}(true); }}"),
            "cannot infer complete Result type",
        );
        rejects(&format!("fn demo() {{ match {variant}(true) {{ Ok(value) => {{ }} Error(problem) => {{ }} }} }}"), "cannot infer complete Result type");
    }
    accepts(&format!("fn consume(pending: {RESULT}) {{ match pending {{ Ok(value) => {{ }} Err(problem) => {{ }} }} }} fn demo() {{ consume(Ok(true)); }}"));
}

#[test]
fn constructor_payloads_do_not_form_named_range_literals() {
    for variant in ["Ok", "Error", "Err"] {
        let prelude = "type Small = range 0..2; fn zero() -> Small { return 0; }";
        let ty = "Result<success: Small, failure: Small>";
        rejects(
            &format!("{prelude} fn demo() -> {ty} {{ return {variant}(0); }}"),
            "payload type mismatch",
        );
        accepts(&format!(
            "{prelude} fn demo() -> {ty} {{ return {variant}(zero()); }}"
        ));
        rejects(&format!("{prelude} fn demo() -> Result<success: {ty}, failure: {ty}> {{ return Ok({variant}(0)); }}"), "payload type mismatch");
    }
}

#[test]
fn conditional_result_initializers_remain_excluded() {
    rejects(&format!("{} fn demo(flag: bool) -> {RESULT} {{ let selected = if flag {{ produce() }} else {{ produce() }}; return selected; }}", producer()), "conditional initializer cannot produce a Result");
}

#[test]
fn reserved_families_cover_every_identifier_context() {
    for name in [
        "Result", "result", "RESULT", "Ok", "ok", "OK", "Error", "error", "ERROR", "Err", "err",
        "ERR", "match", "Match", "MATCH",
    ] {
        for source in [
            format!("type {name} = range 0..1;"),
            format!("fn {name}() {{ }}"),
            format!("fn demo({name}: bool) {{ }}"),
            format!("fn demo() {{ let {name} = true; }}"),
            format!("type P = range 0..1; record R {{ {name}: P }}"),
            format!("{} fn demo() {{ match produce() {{ Ok({name}) => {{ }} Err(problem) => {{ }} }} }}", producer()),
        ] { assert!(compile_source(&source).is_err(), "reserved {name}: {source}"); }
    }
    accepts("type ErrorCode = range 0..1; fn read_error_code() -> ErrorCode { return 0; } fn ErrCode() -> bool { return true; } fn MatchCode() -> bool { return ErrCode(); }");
}

#[test]
fn noncanonical_constructor_capitalization_never_normalizes() {
    for name in ["ok", "OK", "error", "ERROR", "err", "ERR"] {
        rejects(
            &format!("fn demo() -> {RESULT} {{ return {name}(true); }}"),
            "noncanonical reserved spelling",
        );
    }
}

#[test]
fn match_coverage_is_two_canonical_variants() {
    for arms in [
        "Ok(value) => { }",
        "Error(problem) => { }",
        "Error(a) => { } Err(b) => { }",
        "Ok(value) => { } Error(a) => { } Err(b) => { }",
        "Ok(a) => { } Ok(b) => { }",
    ] {
        assert!(compile_source(&format!(
            "{} fn demo() {{ match produce() {{ {arms} }} }}",
            producer()
        ))
        .is_err());
    }
    rejects(
        &format!(
            "{} fn demo() {{ match produce() {{ Error(a) => {{ }} Err(b) => {{ }} }} }}",
            producer()
        ),
        "duplicate canonical Error arm",
    );
}

#[test]
fn unsupported_pattern_and_match_expression_forms_reject() {
    for arms in [
        "_ => { }",
        "Ok() => { } Error(problem) => { }",
        "Ok(Ok(value)) => { } Error(problem) => { }",
        "Ok(value) if true => { } Error(problem) => { }",
        "Ok(&value) => { } Error(problem) => { }",
        "Ok(a) | Error(a) => { }",
    ] {
        assert!(compile_source(&format!(
            "{} fn demo() {{ match produce() {{ {arms} }} }}",
            producer()
        ))
        .is_err());
    }
    rejects(
        "fn demo() { match true { Ok(value) => { } Error(problem) => { } } }",
        "match requires a Result",
    );
    assert!(parse_source(&format!("{} fn demo() {{ let value = match produce() {{ Ok(a) => {{ true }} Err(b) => {{ false }} }}; }}", producer())).is_err());
}

#[test]
fn unresolved_outcomes_reject_every_structured_exit() {
    for body in [
        "let pending = produce();",
        "let pending = produce(); return;",
        "if true { let pending = produce(); }",
        "while true { let pending = produce(); break; }",
        "while true { let pending = produce(); continue; }",
        "while false { let pending = produce(); }",
    ] {
        rejects(
            &format!("{} fn demo() {{ {body} }}", producer()),
            "unresolved Result outcome",
        );
    }
    rejects(
        &format!("fn demo(pending: {RESULT}) {{ }}"),
        "unresolved Result outcome",
    );
    rejects(
        &format!(
            "{} fn demo() -> bool {{ let pending = produce(); return true; }}",
            producer()
        ),
        "unresolved Result outcome",
    );
}

#[test]
fn conditional_consumption_cannot_launder_an_outcome() {
    let handling = "match pending { Ok(value) => { } Error(problem) => { } }";
    rejects(
        &format!("fn demo(flag: bool, pending: {RESULT}) {{ if flag {{ {handling} }} }}"),
        "unresolved Result outcome",
    );
    accepts(&format!("fn demo(flag: bool, pending: {RESULT}) {{ if flag {{ {handling} }} else {{ {handling} }} }}"));
    let consume = format!("fn consume(pending: {RESULT}) -> bool {{ {handling} return true; }}");
    rejects(&format!("{consume} fn demo(flag: bool, pending: {RESULT}) {{ let observed = flag && consume(pending); }}"), "unresolved Result outcome");
}

#[test]
fn nested_results_require_independent_arm_obligations() {
    let nested = format!("Result<success: {RESULT}, failure: bool>");
    rejects(&format!("fn demo(pending: {nested}) {{ match pending {{ Ok(inner) => {{ }} Error(problem) => {{ }} }} }}"), "unresolved Result outcome");
    accepts(&format!("fn demo(pending: {nested}) {{ match pending {{ Ok(inner) => {{ match inner {{ Ok(value) => {{ }} Err(problem) => {{ }} }} }} Error(problem) => {{ }} }} }}"));
}

#[test]
fn transfer_parameters_locals_and_protected_returns_remain_exact() {
    accepts(&format!(
        "fn forward(pending: {RESULT}) -> {RESULT} {{ let next = pending; return next; }}"
    ));
    accepts(&format!("fn forward(pending: {RESULT}) -> {RESULT} {{ match pending {{ Ok(value) => {{ return Ok(value); }} Err(problem) => {{ return Error(problem); }} }} }}"));
    rejects(&format!("fn demo(pending: {RESULT}) {{ match pending {{ Ok(value) => {{ }} Error(problem) => {{ }} }} match pending {{ Ok(value) => {{ }} Err(problem) => {{ }} }} }}"), "use of moved value");
    rejects(
        &format!("{} fn demo() {{ produce(); }}", producer()),
        "cannot be called as a statement",
    );
    rejects(
        &format!(
            "{} fn demo() {{ let mut pending = produce(); pending = produce(); }}",
            producer()
        ),
        "Result replacement is not admitted",
    );
}

#[test]
fn iteration_local_results_must_resolve_before_backedges() {
    accepts(&format!("{} fn demo(flag: bool) {{ while flag {{ let pending = produce(); match pending {{ Ok(value) => {{ }} Error(problem) => {{ }} }} continue; }} }}", producer()));
    rejects(&format!("fn demo(flag: bool, pending: {RESULT}) {{ while flag {{ match pending {{ Ok(value) => {{ }} Error(problem) => {{ }} }} }} }}"), "cannot move pre-existing non-Copy");
}

#[test]
fn match_bindings_are_lexical_and_cannot_shadow() {
    rejects(&format!("{} fn demo(value: bool) {{ match produce() {{ Ok(value) => {{ }} Error(problem) => {{ }} }} }}", producer()), "invalid Result arm binding");
    rejects(&format!("{} fn demo() -> bool {{ match produce() {{ Ok(value) => {{ }} Error(problem) => {{ }} }} return value; }}", producer()), "unknown value");
}

#[test]
fn effects_check_both_arms_and_complete_callee_contracts() {
    rejects(&format!("fn effectful() -> {RESULT} effects io {{ return Err(false); }} fn demo() {{ match effectful() {{ Ok(value) => {{ }} Error(problem) => {{ }} }} }}"), "effect");
    rejects(&format!("{} fn observe() effects clock {{ }} fn demo() {{ match produce() {{ Ok(value) => {{ }} Error(problem) => {{ observe(); }} }} }}", producer()), "effect");
}

#[test]
fn verified_and_public_record_result_categories_remain_closed() {
    assert!(compile_source(&format!(
        "type P=range 0..1; record R {{ field: {RESULT} }}"
    ))
    .is_err());
    assert!(compile_source(&format!(
        "verified fn f() -> {RESULT} {{ return Ok(true); }}"
    ))
    .is_err());
    assert!(compile_source(&format!("fn f(pending: &{RESULT}) {{ }}")).is_err());
}

#[test]
fn source_alias_metadata_does_not_create_an_err_semantic_variant() {
    for variant in ["Error", "Err"] {
        let source = format!("fn produce() -> {RESULT} {{ return {variant}(false); }}");
        let syntax = parse_source(&source).unwrap();
        let ast::FunctionDecl::Ordinary(f) = &syntax.functions[0] else {
            panic!()
        };
        let ast::ValueStatement::Return { value, .. } = &f.body[0] else {
            panic!()
        };
        let ast::ExprKind::ResultConstruct {
            tag, variant_span, ..
        } = value.kind
        else {
            panic!()
        };
        assert_eq!(tag, ast::ResultTag::Error);
        assert_eq!(&source[variant_span.start..variant_span.end], variant);
    }
}

fn run(source: &str) -> String {
    use bowfin_compiler::execution::{
        execute, prepare_execution, AdmissionLimits, ExecutionLimits,
    };
    let executable = prepare_execution(source, AdmissionLimits::default())
        .unwrap_or_else(|e| panic!("{source}\n{e}"));
    let output = execute(
        &executable,
        executable.entry("demo").unwrap(),
        ExecutionLimits::default(),
    )
    .unwrap_or_else(|e| panic!("{source}\n{e}"));
    executable.render_result(&output, 65536).unwrap()
}

#[test]
fn public_execution_selects_each_failure_alias_combination_and_arm_order() {
    for constructor in ["Error", "Err"] {
        for pattern in ["Error", "Err"] {
            for reverse in [false, true] {
                let ok = "Ok(value) => { return true; }";
                let failure = format!("{pattern}(problem) => {{ return problem; }}");
                let arms = if reverse {
                    format!("{failure} {ok}")
                } else {
                    format!("{ok} {failure}")
                };
                assert_eq!(run(&format!("fn produce() -> {RESULT} {{ return {constructor}(false); }} fn demo() -> bool {{ match produce() {{ {arms} }} }}")), "Bool(false)");
            }
        }
    }
}

#[test]
fn public_match_moves_records_and_copies_range_payloads() {
    assert_eq!(run("type P=range 0..10; fn seven()->P{return 7;} fn produce()->Result<success:P,failure:P>{return Ok(seven());} fn demo()->P{match produce(){Ok(value)=>{return value;}Err(problem)=>{return problem;}}}"), "P(7)");
    assert_eq!(run("type P=range 0..10; record R{value:P} fn produce()->Result<success:R,failure:R>{return Err(R{value:7});} fn demo()->R{match produce(){Ok(value)=>{return value;}Error(problem)=>{let owned=problem;return owned;}}}"), "R { value: P(7) }");
}

#[test]
fn public_nested_matching_preserves_independent_inner_wrappers() {
    assert_eq!(run("fn produce()->Result<success:Result<success:bool,failure:bool>,failure:bool>{return Ok(Err(false));} fn demo()->bool{match produce(){Ok(inner)=>{match inner{Ok(value)=>{return true;}Error(problem)=>{return problem;}}}Error(problem)=>{return true;}}}"), "Bool(false)");
}

#[test]
fn public_manual_propagation_preserves_whole_result_identity_and_payload() {
    let source = format!("fn produce()->{RESULT}{{return Err(false);}} fn forward()->{RESULT}{{match produce(){{Ok(value)=>{{return Ok(value);}}Error(problem)=>{{return Error(problem);}}}}}} fn demo()->bool{{let pending=forward();match pending{{Err(problem)=>{{return problem;}}Ok(value)=>{{return true;}}}}}}");
    assert_eq!(run(&source), "Bool(false)");
}

#[test]
fn public_scrutinee_runs_once_and_unselected_arm_does_no_work() {
    let source = "type P=range 0..10; fn seed()->P{return 2;} fn produce(count:&mut P)->Result<success:bool,failure:bool>{*count=*count-1;return Ok(true);} fn trap()->bool{let zero=seed()-2;let invalid=zero-1;return true;} fn demo()->P{let mut count=seed();match produce(&mut count){Ok(value)=>{}Error(problem)=>{let doomed=trap();}}return count;}";
    assert_eq!(run(source), "P(1)");
}

#[test]
fn public_complete_results_render_a_bounded_canonical_active_chain() {
    for spelling in ["Error", "Err"] {
        assert_eq!(
            run(&format!("fn demo()->{RESULT}{{return {spelling}(false);}}")),
            "Error(Bool(false))"
        );
    }
    assert_eq!(run("fn demo()->Result<success:Result<success:bool,failure:bool>,failure:bool>{return Ok(Err(true));}"), "Ok(Error(Bool(true)))");
    assert_eq!(run("type P=range 0..9; record R{value:P} fn demo()->Result<success:R,failure:bool>{return Ok(R{value:7});}"), "Ok(R { value: P(7) })");
}

#[test]
fn public_ast_cannot_bypass_reserved_declaration_names() {
    use bowfin_compiler::resolve;
    for name in [
        "Result", "result", "Ok", "oK", "Error", "eRrOr", "Err", "eRr", "match", "mAtCh",
    ] {
        let mut a = parse_source("fn ordinary(value:bool){let local=value;}").unwrap();
        let ast::FunctionDecl::Ordinary(f) = &mut a.functions[0] else {
            panic!()
        };
        f.name = name.into();
        assert!(resolve::resolve(&a).is_err());
        let ast::FunctionDecl::Ordinary(f) = &mut a.functions[0] else {
            panic!()
        };
        f.name = "ordinary".into();
        f.parameters[0].name = name.into();
        assert!(resolve::resolve(&a).is_err());
    }
}

#[test]
fn recurring_match_bindings_reenter_their_lexical_scope() {
    let source="type Count=range 0..3;fn three()->Count{return 3;}fn produce()->Result<success:bool,failure:bool>{return Ok(true);}fn demo()->Count{let mut count=three();while !(count==0){match produce(){Ok(value)=>{count=count-1;}Error(problem)=>{count=count-1;}}}return count;}";
    assert_eq!(run(source), "Count(0)");
}
