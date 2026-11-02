use bowfin_compiler::{
    compile_source,
    effects::{Effect, EffectSet},
    execution::{self, AdmissionLimits, ExecutionLimits, ExecutionResult},
    lexer::{lex, TokenKind},
    mir, parse_source, FrontendError,
};

#[test]
fn ordinary_result_kinds_and_keyword_spellings_preserve_contracts() {
    for keyword in ["fn", "function"] {
        for (result, body) in [("", ""), (" -> bool", "return true;")] {
            let no_value = result.is_empty();
            let source = format!("{keyword} demo(){result} effects clock, io {{ {body} }}");
            let p = compile_source(&source).unwrap();
            let f = p.functions()[0].as_ordinary().unwrap();
            assert_eq!(f.effects.set().to_string(), "io, clock");
            assert!(f.effects.origin(Effect::Clock).is_some());
            assert!(f.effects.origin(Effect::Io).is_some());
            assert!(f.effects.origin(Effect::Unsafe).is_none());
            let clause = f.effects.clause().unwrap();
            assert_eq!(&source[clause.start..clause.end], "effects clock, io");
            let m = mir::lower(&p);
            assert_eq!(m.functions()[0].effects(), &f.effects);
            m.functions()[0].validate().unwrap();
            let e = execution::prepare_execution(&source, AdmissionLimits::default()).unwrap();
            let result =
                execution::execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default())
                    .unwrap();
            if no_value {
                assert_eq!(result, ExecutionResult::NoValue);
            } else {
                assert_eq!(
                    result,
                    ExecutionResult::Value(execution::RuntimeValue::Bool(true))
                );
            }
        }
    }
}

#[test]
fn all_new_spellings_remain_identifiers_outside_the_clause() {
    for name in [
        "effects",
        "io",
        "heap",
        "blocking",
        "network",
        "filesystem",
        "clock",
        "random",
        "unsafe",
    ] {
        assert!(matches!(&lex(name).unwrap()[0].kind, TokenKind::Ident(n) if n == name));
        let source = format!("fn {name}({name}: bool) -> bool {{ return {name}; }} fn demo() -> bool {{ return {name}(true); }}");
        let p = compile_source(&source).unwrap();
        assert!(p
            .functions()
            .iter()
            .all(|f| f.as_ordinary().unwrap().effects.set() == EffectSet::EMPTY));
    }
    compile_source("type N=range 0..1; record effects{x:N} fn f()->effects effects clock {return effects{x:0};}").unwrap();
}

#[test]
fn absent_clause_is_empty_and_declarations_collect_before_bodies() {
    let p = compile_source(
        "fn first() effects io {second();} function second() effects io {first();} fn plain() {}",
    )
    .unwrap();
    assert_eq!(
        p.functions()[0]
            .as_ordinary()
            .unwrap()
            .effects
            .set()
            .to_string(),
        "io"
    );
    assert_eq!(
        p.functions()[1]
            .as_ordinary()
            .unwrap()
            .effects
            .set()
            .to_string(),
        "io"
    );
    let plain = p.functions()[2].as_ordinary().unwrap();
    assert_eq!(plain.effects.set(), EffectSet::EMPTY);
    assert_eq!(plain.effects.clause(), None);
    compile_source("fn self_call() effects random {self_call();}").unwrap();
}

#[test]
fn unknown_duplicate_and_malformed_clauses_reject() {
    for (names, expected) in [
        ("io, io", "duplicate"),
        ("mystery", "unknown"),
        ("IO", "unknown"),
    ] {
        let error = compile_source(&format!("fn f() effects {names} {{}}")).unwrap_err();
        assert!(matches!(error, FrontendError::Resolve(_)));
        assert!(error.to_string().contains(expected));
    }
    for source in [
        "fn f() effects {}",
        "fn f() effects io, {}",
        "fn f() effects io effects heap {}",
        "fn f() effects io -> bool {return true;}",
        "fn f() effects io,heap,blocking,network,filesystem,clock,random,unsafe,io {}",
        "fn f(){ effects io; }",
    ] {
        assert!(
            matches!(parse_source(source), Err(FrontendError::Parse(_))),
            "{source}"
        );
    }
}

#[test]
fn verified_and_harness_effect_clauses_remain_excluded() {
    let source = "type N=range 0..100; record Cell{x:N} verified fn consume(c:&mut Cell,n:N) requires n<=c.x ensures c.x==old(c.x)-n {c.x-=n;} let mut cell=Cell{x:80}; consume(&mut cell,50);";
    let p = compile_source(source).unwrap();
    let control = bowfin_compiler::verify::verify_report(&p);
    assert!(control.diagnostics.is_empty());
    assert_eq!(control.functions_proven, 1);
    for malformed in [
        source.replace(") requires", ") effects io requires"),
        source.replace(" requires", " effects unsafe requires"),
        source.replace("let mut cell=", "let mut cell effects io="),
        source.replace(
            "consume(&mut cell,50);",
            "consume(&mut cell,50) effects io;",
        ),
    ] {
        assert!(matches!(
            parse_source(&malformed),
            Err(FrontendError::Parse(_))
        ));
    }
}

#[test]
fn complete_declared_contracts_propagate_through_both_call_forms() {
    for effect in Effect::ALL {
        for keyword in ["fn", "function"] {
            for value in [false, true] {
                let source = if value {
                    format!("{keyword} leaf()->bool effects {0}{{return true;}} {keyword} middle()->bool effects {0}{{return leaf();}} {keyword} top()->bool{{return middle();}}", effect.name())
                } else {
                    format!("{keyword} leaf() effects {0}{{}} {keyword} middle() effects {0}{{leaf();}} {keyword} top(){{middle();}}", effect.name())
                };
                let error = compile_source(&source).unwrap_err();
                assert!(matches!(error, FrontendError::Effects(_)), "{error}");
                let message = error.to_string();
                assert!(message.contains(effect.name()));
                assert!(message.contains("middle"));
                assert!(message.contains("declaration at"));
                let source = if value {
                    source.replace(
                        "top()->bool{",
                        &format!("top()->bool effects {}{{", effect.name()),
                    )
                } else {
                    source.replace("top(){", &format!("top() effects {}{{", effect.name()))
                };
                compile_source(&source).unwrap();
            }
        }
    }
}

#[test]
fn structural_unions_keep_terminal_short_circuit_and_loop_calls() {
    let prelude = "fn emit() effects io {} fn clock()->bool effects clock{return true;}";
    for body in [
        "if true {emit();return;}",
        "if false {emit();} else {return;}",
        "let observed = false && clock();",
        "let observed = true || clock();",
        "while false {emit();break;}",
        "while true {emit();continue;}",
        "while false {if true {emit();return;}else{continue;}}",
        "let chosen=if true {clock()} else {false};",
    ] {
        let error = compile_source(&format!("{prelude} fn demo(){{{body}}}")).unwrap_err();
        assert!(
            matches!(error, FrontendError::Effects(_)),
            "{body}: {error}"
        );
        compile_source(&format!("{prelude} fn demo() effects io,clock {{{body}}}")).unwrap();
    }
}

#[test]
fn recursive_contract_diagnostics_terminate_at_an_honest_declaration_edge() {
    let source = "fn a() effects clock {b();} fn b() effects clock {a();} fn demo(){a();}";
    let error = compile_source(source).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("clock"));
    assert!(message.contains("call to `a`"));
    assert!(message.len() < 512);
    let e = execution::prepare_execution(source, AdmissionLimits::default())
        .err()
        .unwrap();
    assert!(matches!(
        e.kind,
        execution::ErrorKind::Frontend {
            phase: "effects",
            ..
        }
    ));
    let e = execution::prepare_execution_from_reader(source.as_bytes(), AdmissionLimits::default())
        .err()
        .unwrap();
    assert!(matches!(
        e.kind,
        execution::ErrorKind::Frontend {
            phase: "effects",
            ..
        }
    ));
}

#[test]
fn unsafe_membership_cannot_authorize_reusing_a_moved_owner() {
    let error = compile_source(
        "type N=range 0..1; record R{x:N} fn demo(x:R) effects unsafe {let first=x;let second=x;}",
    )
    .unwrap_err();
    assert!(matches!(error, FrontendError::Ownership(_)));
}

#[test]
fn effect_diagnostic_admission_and_rendering_have_exact_limits() {
    let source = "fn leaf() effects unsafe,io {} fn caller(){leaf();}";
    let prepare = |bytes| {
        execution::prepare_execution(
            source,
            AdmissionLimits {
                diagnostic_storage: bytes,
                ..AdmissionLimits::default()
            },
        )
    };
    let (mut low, mut high) = (1, AdmissionLimits::default().diagnostic_storage);
    assert!(matches!(
        prepare(high).err().unwrap().kind,
        execution::ErrorKind::Frontend {
            phase: "effects",
            ..
        }
    ));
    while low < high {
        let middle = low + (high - low) / 2;
        match prepare(middle).err().unwrap().kind {
            execution::ErrorKind::ConfiguredLimit(_) => low = middle + 1,
            execution::ErrorKind::Frontend {
                phase: "effects", ..
            } => high = middle,
            other => panic!("{other:?}"),
        }
    }
    let error = prepare(low).err().unwrap();
    assert!(matches!(
        prepare(low - 1).err().unwrap().kind,
        execution::ErrorKind::ConfiguredLimit(_)
    ));
    let rendered = execution::render_admission_error(&error, 65536).unwrap();
    assert!(rendered.contains("io"));
    assert_eq!(
        execution::render_admission_error(&error, rendered.len()).unwrap(),
        rendered
    );
    let fallback = execution::render_admission_error(&error, rendered.len() - 1).unwrap();
    let expected = format!(
        "error: diagnostic truncated at {}:{}",
        error.origin.line, error.origin.column
    );
    assert_eq!(fallback, expected);
    assert_eq!(
        execution::render_admission_error(&error, fallback.len()).unwrap(),
        fallback
    );
    assert_eq!(
        execution::render_admission_error(&error, fallback.len() - 1)
            .unwrap_err()
            .kind,
        execution::ErrorKind::ConfiguredLimit("rendered output")
    );
}
