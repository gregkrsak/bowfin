use bowfin_compiler::{ast, compile_source, hir, mir, parse_source, verify, FrontendError};
const DECL: &str = "type Percent = range 0..100; type OtherPercent = range 0..100; type Voltage = range 0..5000; record Ticket { value: Percent } record Pair { first: Percent, second: Voltage, } record Same { first: Percent, second: Voltage } record Triple { first: Percent, middle: Percent, last: Percent } fn measure(t: Ticket) -> Percent { return t.value; } fn read(view: &Percent) -> Percent { return *view; } fn identity(p: Percent) -> Percent { return p; } fn volts(v: Voltage) -> Voltage { return v; } fn accept(p: Pair) {} fn accept_triple(p: Triple) {} fn receive(p: Triple,t: Ticket) {} fn inspect(p: &Pair) -> Percent { return inspect(p); } fn inspect_mut(p: &mut Pair) -> Percent { return inspect_mut(p); } fn held(p: &mut Pair,other: Pair) {} fn test_record(p: Pair) -> bool { return true; } fn test_triple(p: Triple) -> bool { return true; }";
fn source(body: &str) -> String {
    format!("{DECL}\n{body}")
}
fn good(body: &str) -> hir::Program {
    let p = compile_source(&source(body)).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, message: &str) -> (String, bowfin_compiler::lexer::Span) {
    let s = source(body);
    let e = compile_source(&s).unwrap_err();
    let (actual, text, span) = match e {
        FrontendError::Parse(e) => ("Parse", e.message, e.span),
        FrontendError::Resolve(e) => ("Resolve", e[0].message.clone(), e[0].span),
        FrontendError::Type(e) => ("Type", e[0].message.clone(), e[0].span),
        FrontendError::Ownership(e) => ("Ownership", e[0].message.clone(), e[0].span),
        FrontendError::Effects(e) => ("Effects", e[0].message.clone(), e[0].span),
        FrontendError::Lex(e) => ("Lex", e.message, e.span),
    };
    assert_eq!(actual, phase, "{text}");
    assert!(text.contains(message), "{text}");
    (s, span)
}
macro_rules! accepted {
    ($($name:ident: $body:literal;)*) => { $(#[test] fn $name() { good($body); })* }
}
accepted! {
    two_fields_declaration_order: "fn f(p: Percent,v: Voltage) -> Pair { return Pair { first: p, second: v }; }";
    reversed_construction: "fn f(p: Percent,v: Voltage) -> Pair { return Pair { second: v, first: p, }; }";
    three_repeated_range_fields: "fn f(p: Percent) -> Triple { return Triple { first: p, middle: p, last: p }; }";
    parameters_locals_calls_and_trailing_commas: "fn f(p: Percent,v: Voltage) -> Pair { let local = p; let b = Pair { second: volts(v), first: identity(local), }; return b; }";
    constructor_call_argument: "fn f(p: Percent,v: Voltage) { accept(Pair { second: v, first: p }); }";
    constructor_conditional_branch_results: "fn f(flag: bool,p: Percent,v: Voltage) -> Pair { let b = if flag { Pair { second: v, first: p } } else { Pair { first: p, second: v } }; return b; }";
    cross_record_exact_copy_initializers: "fn f(s: Same) -> Pair { return Pair { second: s.second, first: s.first }; }";
    alternating_field_reads_then_whole_move: "fn f(b: Pair) -> Pair { let a = b.first; let c = b.second; let again = b.first; return b; }";
    shared_loans_cover_every_field: "fn f(b: Pair) -> Pair { let view = &b; let a = b.first; let c = b.second; let seen = inspect(view); return b; }";
    exclusive_loan_legitimate_expiry: "fn f(b: Pair) -> Pair { let mut owned = b; let access = &mut owned; let seen = inspect_mut(access); let a = owned.first; let c = owned.second; return owned; }";
    repeated_reads_across_backedge: "fn f(flag: bool,b: Pair) -> Pair { while flag { let a = b.first; let c = b.second; let again = b.first; } return b; }";
    copy_fields_in_recurring_condition: "fn f(b: Pair,p: Percent,v: Voltage) -> Pair { while b.first == p && b.second == v {} return b; }";
    construction_in_loop_body: "fn f(flag: bool,p: Percent,v: Voltage) { while flag { let b = Pair { second: v, first: p }; let x = b.second; accept(b); } }";
    nested_loop_body_construction: "fn f(flag: bool,p: Percent,v: Voltage) { while flag { while flag { accept(Pair { first: p, second: v }); } } }";
    kd033_terminal_move_only_in_return_operand: "fn f(flag: bool,t: Ticket,p: Percent) -> Triple { while flag { return Triple { middle: p, last: p, first: measure(t) }; } return Triple { first: measure(t), middle: p, last: p }; }";
    legitimate_enclosing_call_hold: "fn f(b: Pair,v: Voltage) { let view = &b; accept(Pair { first: inspect(view), second: v }); }";
    single_field_with_trailing_comma: "record Single { value: Percent, } fn f(p: Percent) -> Single { return Single { value: p, }; }";
    identifier_conditions_and_blocks: "fn f(flag: bool,p: Percent,v: Voltage) { if flag {} while flag { if (flag) { accept(Pair { second: v, first: p }); } else {} break; } }";
}
#[test]
fn canonical_orders_spans_and_direct_read_representation_survive_to_mir() {
    let body = "fn f(p: Percent,v: Voltage) -> Pair { let b = Pair { second: v, first: p }; let seen = b.second; return b; }";
    let a = parse_source(&source(body)).unwrap();
    assert_eq!(
        a.records[1]
            .fields
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    let ast::FunctionDecl::Ordinary(f) = a.functions.last().unwrap() else {
        panic!()
    };
    let ast::ValueStatement::Let { initializer, .. } = &f.body[0] else {
        panic!()
    };
    let ast::ExprKind::RecordConstruct { fields, .. } = &initializer.kind else {
        panic!()
    };
    assert_eq!(
        fields.iter().map(|f| f.name.as_str()).collect::<Vec<_>>(),
        ["second", "first"]
    );
    assert!(fields[0].span.start < fields[1].span.start);
    let p = good(body);
    let pair = &p.records()[1];
    assert_eq!(
        pair.fields
            .iter()
            .map(|id| p.field(*id).name.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert!(pair.fields[0] < pair.fields[1]);
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let hir::ValueStatement::Let { initializer, .. } = &f.body[0] else {
        panic!()
    };
    let hir::ExprKind::RecordConstruct { fields, .. } = &initializer.kind else {
        panic!()
    };
    assert_eq!(
        fields.iter().map(|e| e.field).collect::<Vec<_>>(),
        [pair.fields[1], pair.fields[0]]
    );
    let hir::ValueStatement::Let {
        initializer: read, ..
    } = &f.body[1]
    else {
        panic!()
    };
    assert!(
        matches!(read.kind,hir::ExprKind::CopyFieldRead { field, .. } if field==pair.fields[1])
    );
    assert_eq!(
        read.ty,
        hir::ExprType::Range(p.field(pair.fields[1]).range_type().unwrap())
    );
    let m = mir::lower(&p);
    assert!(m
        .functions()
        .last()
        .unwrap()
        .blocks()
        .flat_map(|(_, b)| b.statements().iter())
        .any(|s| matches!(s,mir::Statement::Let {initializer:e,..} if e==initializer)));
    assert!(p.bindings().is_empty());
    assert_eq!(f.body.len(), 3); // no synthesized factory/accessor statements
    assert_eq!(p.locals().len(), 2); // only written locals
}
#[test]
fn static_diagnostics_have_frozen_order_and_occurrence_spans() {
    let (s,span)=bad("fn f(p: Percent) -> Triple { return Triple { last: p, unknown: missing, first: p, first: p }; }", "Resolve", "unknown field `unknown`");
    assert_eq!(span.start, s.rfind("unknown:").unwrap());
    let (s, span) = bad(
        "fn f(p: Percent) -> Triple { return Triple { last: p, last: missing, unknown: p }; }",
        "Resolve",
        "duplicate initializer field `last`",
    );
    assert_eq!(span.start, s.rfind("last:").unwrap());
    bad(
        "fn f(p: Percent) -> Triple { return Triple { last: missing }; }",
        "Resolve",
        "missing initializer fields: first, middle",
    );
    let (s, span) = bad(
        "record Bad { a: Percent, b: Percent, a: Voltage }",
        "Resolve",
        "duplicate declaration `a`",
    );
    assert_eq!(span.start, s.rfind("a: Voltage").unwrap());
}
#[test]
fn exact_type_errors_follow_written_order_in_every_position() {
    for position in 0..3 {
        let names = ["last", "first", "middle"];
        let entries = names
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{n}: {}", if i == position { "other" } else { "p" }))
            .collect::<Vec<_>>()
            .join(", ");
        let (s,span)=bad(&format!("fn f(p: Percent,other: OtherPercent) -> Triple {{ return Triple {{ {entries} }}; }}"),"Type","record field initializer");
        assert_eq!(&s[span.start..span.end], "other");
    }
    let (s, span) = bad(
        "fn f(p: Percent,v: Voltage) -> Pair { return Pair { second: p, first: v }; }",
        "Type",
        "record field initializer",
    );
    assert_eq!(&s[span.start..span.end], "p");
}
#[test]
fn bare_literal_is_parsed_then_formed_in_typing() {
    let s = source("fn f(v: Voltage) -> Pair { return Pair { first: 80, second: v }; }");
    let a = parse_source(&s).unwrap();
    let r = bowfin_compiler::resolve::resolve(&a).unwrap();
    bowfin_compiler::types::check(r).unwrap();
}
#[test]
fn declarations_reject_unsupported_types_and_empty_shapes() {
    for fields in [
        "first: bool, second: Percent",
        "first: &Percent, second: Percent",
    ] {
        bad(&format!("record Bad {{ {fields} }}"), "Parse", "expected");
    }
    bad(
        "record Bad { first: Ticket, second: Percent }",
        "Resolve",
        "unknown range type `Ticket`",
    );
    bad("record Bad {}", "Parse", "at least one");
}
#[test]
fn malformed_construction_diagnostics_are_exact_and_located() {
    for (entries,msg,marker) in [
        ("", "record construction requires at least one named field: initializer", "}"),
        (": p", "record construction requires at least one named field: initializer", ": p"),
        ("first p", "record construction requires ':' after the field name", "p }"),
        ("first: , second: v", "record construction requires a field initializer", ", second"),
        ("first: p second: v", "record construction requires ',' between entries and a closing brace", "second: v"),
        ("first: p; second: v", "record construction requires ',' between entries; semicolon separators are unsupported", "; second"),
        ("first: p, ;", "record construction requires a field name after ','", "; }"),
    ] {
        let (s,span)=bad(&format!("fn f(p: Percent,v: Voltage) -> Pair {{\n return Pair {{ {entries} }};\n}}"),"Parse",msg);
        // For the empty case select the constructor's brace, rather than the function's brace.
        let expected=if entries.is_empty() { s.rfind("};").unwrap() }else { s.rfind(marker).unwrap() };
        assert_eq!(span.start,expected,"{entries}");
        assert_eq!(span.line,3);
        assert_eq!(span.column,s[..span.start].rsplit('\n').next().unwrap().len()+1);
    }
    bad(
        "fn f(p: Percent,v: Voltage) -> Pair { return Pair { first: p, second: v",
        "Parse",
        "requires a closing brace",
    );
}
#[test]
fn declaration_separators_are_required_and_located() {
    for sep in [" ", "; "] {
        let (source, span) = bad(
            &format!("record Bad {{ first: Percent{sep}second: Percent }}"),
            "Parse",
            "requires ',' between fields",
        );
        let marker = if sep.starts_with(';') {
            "; second:"
        } else {
            "second:"
        };
        assert_eq!(span.start, source.rfind(marker).unwrap());
    }
}
#[test]
fn records_with_identical_fields_remain_nominally_distinct() {
    bad("fn f(s: Same) -> Pair { return s; }", "Type", "return");
}
#[test]
fn every_field_read_rejects_after_whole_move_and_exclusive_borrow() {
    for field in ["first", "second"] {
        bad(
            &format!("fn f(b: Pair) {{ accept(b); let x = b.{field}; }}"),
            "Ownership",
            "moved value",
        );
        bad(&format!("fn f(b: Pair) {{ let mut owned = b; let access = &mut owned; let x = owned.{field}; let y = inspect_mut(access); }}"),"Ownership","exclusively borrowed");
    }
}
#[test]
fn source_field_boundaries_remain_restricted() {
    good("fn f(b: Pair) -> Percent { let view = &b; return view.first; }");
    bad(
        "fn f(b: Pair,p: Percent) { b.first = p; }",
        "Type",
        "parameter",
    );
    bad(
        "fn f(b: Pair) { let view = &b.first; }",
        "Parse",
        "direct owned",
    );
    bad(
        "fn f(b: Pair) -> Percent { return b.first.value; }",
        "Parse",
        "temporary or nested",
    );
    bad(
        "fn f(p: Percent,v: Voltage) -> Percent { return (Pair { first: p, second: v }).first; }",
        "Parse",
        "temporary or nested",
    );
}
#[test]
fn later_initializer_failure_and_written_order_are_observable() {
    let (s,span)=bad("fn f(t: Ticket,p: Percent) -> Triple { return Triple { last: measure(t), first: p, middle: measure(t) }; }","Ownership","moved value");
    assert_eq!(span.start, s.rfind("t)").unwrap());
    bad("fn f(t: Ticket,p: Percent) { receive(Triple { last: measure(t), first: p, middle: p },t); }","Ownership","moved value");
}
#[test]
fn recurring_conditions_reject_outer_construction_with_varied_initializer_contents() {
    for position in 0..3 {
        let names = ["first", "middle", "last"];
        let entries = names
            .iter()
            .enumerate()
            .map(|(i, n)| {
                format!(
                    "{n}: {}",
                    if i == position {
                        "identity(identity(p))"
                    } else {
                        "p"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        bad(
            &format!("fn f(p: Percent) {{ while test_triple(Triple {{ {entries} }}) {{}} }}"),
            "Ownership",
            "construction is unsupported in recurring",
        );
    }
    bad(
        "fn f(p: Percent,v: Voltage) { while !test_record(Pair { second: v, first: p }) {} }",
        "Ownership",
        "construction is unsupported in recurring",
    );
    bad("fn f(p: Percent,v: Voltage) { while true == test_record(Pair { second: v, first: p }) {} }","Ownership","construction is unsupported in recurring");
}
#[test]
fn multi_field_verified_and_harness_candidates_are_explicitly_ineligible() {
    bad("verified fn consume(b: &mut Triple,p: Percent) requires p <= b.first ensures b.first == old(b.first) - p { b.first -= p; }","Resolve","unsupported multi-field state");
    bad(
        "let mut b = Triple { first: 80 };",
        "Resolve",
        "unsupported multi-field state",
    );
}
#[test]
fn ordinary_multi_field_records_coexist_with_supported_verified_battery() {
    let s=format!("{}\nrecord Multiple {{ first: Percent, second: Percent }} fn ordinary(p: Percent) -> Multiple {{ return Multiple {{ second: p, first: p }}; }}",include_str!("../../examples/battery_ok.bow"));
    let p = compile_source(&s).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    let r = verify::verify_report(&p);
    assert!(r.diagnostics.is_empty());
    assert_eq!((r.functions_proven, r.calls_checked), (1, 1));
}

#[test]
fn inherited_exclusive_call_holds_protect_all_initializer_positions() {
    for position in 0..3 {
        let entries = ["first", "middle", "last"]
            .iter()
            .enumerate()
            .map(|(i, n)| format!("{n}: {}", if i == position { "owned.middle" } else { "p" }))
            .collect::<Vec<_>>()
            .join(",");
        bad(&format!("fn protected(access: &mut Triple,b: Triple) {{}} fn f(b: Triple,p: Percent) {{ let mut owned = b; protected(&mut owned,Triple {{ {entries} }}); }}"),"Ownership","exclusively borrowed");
    }
}
#[test]
fn recurring_condition_visits_nested_constructions_in_every_argument_position() {
    for position in 0..3 {
        let args = (0..3)
            .map(|i| {
                if i == position {
                    "selected(Triple { last: p, first: p, middle: p })"
                } else {
                    "p"
                }
            })
            .collect::<Vec<_>>()
            .join(",");
        bad(&format!("fn selected(b: Triple) -> Percent {{ return b.middle; }} fn predicate(a: Percent,b: Percent,c: Percent) -> bool {{ return a == b; }} fn f(p: Percent) {{ while predicate({args}) {{}} }}"),"Ownership","construction is unsupported in recurring");
    }
}
#[test]
fn public_ast_hidden_conditionals_are_rejected_in_every_initializer_position() {
    for position in 0..3 {
        let mut a=parse_source(&source("fn f(flag: bool,p: Percent) -> Triple { let chosen = if flag { p } else { p }; return Triple { last: p, first: p, middle: p }; }")).unwrap();
        let ast::FunctionDecl::Ordinary(f) = a.functions.last_mut().unwrap() else {
            panic!()
        };
        let ast::ValueStatement::Let {
            initializer: hidden,
            ..
        } = &f.body[0]
        else {
            panic!()
        };
        let hidden = hidden.clone();
        let ast::ValueStatement::Return { value, .. } = &mut f.body[1] else {
            panic!()
        };
        let ast::ExprKind::RecordConstruct { fields, .. } = &mut value.kind else {
            panic!()
        };
        fields[position].value = hidden;
        let r = bowfin_compiler::resolve::resolve(&a).unwrap();
        assert!(bowfin_compiler::types::check(r).unwrap_err()[0]
            .message
            .contains("complete local initializers"));
    }
}
