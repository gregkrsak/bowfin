use bowfin_compiler::{compile_source, hir, mir, parse_source, FrontendError};
const DECL: &str = "type Percent = range 0..100; type OtherPercent = range 0..100; record Battery { charge: Percent } record Capacitor { charge: Percent } record Meter { level: Percent } record Ticket { value: Percent } fn identity(p: Percent) -> Percent { return p; } fn make(p: Percent) -> Battery { return Battery { charge: p }; } fn extract(b: Battery) -> Percent { return b.charge; } fn consume(t: Ticket) -> Percent { return t.value; } fn duplicate(first: Ticket,second: Ticket) -> Percent { return first.value; } fn with_hold(t: Ticket,view: &bool,last: Ticket) -> Percent { return t.value; } fn discard(b: Battery) {} fn receive(b: Battery,t: Ticket) {} fn inspect(view: &Battery) -> Percent { return inspect(view); } fn inspect_mut(access: &mut Battery) -> Percent { return inspect_mut(access); } fn inspect_percent(p: Percent) {} fn copy_then_move(p: Percent,b: Battery) {} fn move_then_copy(b: Battery,p: Percent) {} fn held_mut(access: &mut Battery,p: Percent) {} fn ready(b: Battery) -> bool { return true; } fn flag_of(flag: bool) -> bool { return flag; } fn work() {}";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL} {body}")).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, word: &str) {
    let source = format!("{DECL} {body}");
    let e = compile_source(&source).unwrap_err();
    let actual = match &e {
        FrontendError::Lex(_) => "Lex",
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        FrontendError::Effects(_) => "Effects",
    };
    assert_eq!(actual, phase, "{e}");
    assert!(e.to_string().contains(word), "{e}");
}
#[test]
fn construction_parameter() {
    good("fn f(p: Percent) -> Battery { return Battery { charge: p }; }");
}
#[test]
fn construction_local() {
    good("fn f(p: Percent) -> Battery { let amount = p; return Battery { charge: amount }; }");
}
#[test]
fn construction_call_result() {
    good("fn f(p: Percent) -> Battery { return Battery { charge: identity(p) }; }");
}
#[test]
fn construction_local_destination() {
    good("fn f(p: Percent) -> Battery { let b = Battery { charge: p }; return b; }");
}
#[test]
fn construction_value_call_argument() {
    good("fn f(p: Percent) -> Percent { return extract(Battery { charge: p }); }");
}
#[test]
fn construction_no_value_call_argument() {
    good("fn f(p: Percent) { discard(Battery { charge: p }); }");
}
#[test]
fn construction_conditional_results() {
    good("fn f(flag: bool,p: Percent) -> Battery { let b = if flag { Battery { charge: p } } else { Battery { charge: identity(p) } }; return b; }");
}
#[test]
fn construction_nested_child() {
    good("fn f(p: Percent) -> Battery { return Battery { charge: extract(Battery { charge: p }) }; }");
}
#[test]
fn construction_arithmetic() {
    good("fn f(p: Percent) -> Battery { return Battery { charge: p - 5 }; }");
}
#[test]
fn construction_field_initializer() {
    good("fn f(b: Battery) -> Battery { return Battery { charge: b.charge }; }");
}
#[test]
fn fresh_record_not_alias() {
    good("fn f(b: Battery) -> Battery { let copy = Battery { charge: b.charge }; discard(b); return copy; }");
}
#[test]
fn field_owned_parameter() {
    good("fn f(b: Battery) -> Percent { return b.charge; }");
}
#[test]
fn field_immutable_local() {
    good("fn f(b: Battery) -> Percent { let owned = b; return owned.charge; }");
}
#[test]
fn field_mutable_local() {
    good("fn f(b: Battery) -> Percent { let mut owned = b; return owned.charge; }");
}
#[test]
fn field_repeated() {
    good("fn f(b: Battery) -> Percent { let first = b.charge; let second = b.charge; return first - second; }");
}
#[test]
fn field_then_move() {
    good("fn f(b: Battery) -> Battery { let seen = b.charge; return b; }");
}
#[test]
fn copied_field_argument_then_move() {
    good("fn f(b: Battery) { copy_then_move(b.charge,b); }");
}
#[test]
fn shared_loan_future_handle() {
    good("fn f(b: Battery) -> Percent { let view = &b; let seen = b.charge; let checked = inspect(view); return seen; }");
}
#[test]
fn exclusive_lawful_expiry() {
    good("fn f(b: Battery) -> Percent { let mut owned = b; let access = &mut owned; let checked = inspect_mut(access); return owned.charge; }");
}
#[test]
fn exclusive_finite_control() {
    good("fn f(b: Battery,p: Percent) -> Percent { let mut owned = b; let access = &mut owned; let seen = p; let checked = inspect_mut(access); return seen; }");
}
#[test]
fn exclusive_call_hold_control() {
    good("fn f(b: Battery,p: Percent) { let mut owned = b; held_mut(&mut owned,p); }");
}
#[test]
fn exclusive_loop_control() {
    good("fn f(flag: bool,b: Battery,p: Percent) -> Percent { let mut owned = b; let access = &mut owned; while flag { let seen = p; } return inspect_mut(access); }");
}
#[test]
fn returning_move_branch_excluded() {
    good("fn f(flag: bool,b: Battery,p: Percent) -> Percent { if flag { discard(b); return p; } return b.charge; }");
}
#[test]
fn no_value_returning_move_branch() {
    good("fn f(flag: bool,b: Battery) { if flag { discard(b); return; } let seen = b.charge; }");
}
#[test]
fn repeated_loop_reads() {
    good("fn f(flag: bool,b: Battery) -> Percent { let mut seen = b.charge; while flag { seen = b.charge; } return seen; }");
}
#[test]
fn field_while_condition() {
    good("fn f(b: Battery,p: Percent) -> Percent { while b.charge <= p { inspect_percent(b.charge); } return b.charge; }");
}
#[test]
fn field_local_while_condition() {
    good("fn f(b: Battery,p: Percent) -> Percent { let owned = b; while owned.charge <= p { inspect_percent(owned.charge); } return owned.charge; }");
}
#[test]
fn field_nested_loop() {
    good("fn f(flag: bool,p: Percent) -> Percent { while flag { let b = Battery { charge: p }; while b.charge <= p { inspect_percent(b.charge); } discard(b); } return p; }");
}
#[test]
fn iteration_local_record() {
    good("fn f(flag: bool,p: Percent) { while flag { let b = Battery { charge: p }; inspect_percent(b.charge); discard(b); } }");
}
#[test]
fn identifier_empty_condition_block() {
    good("fn f(flag: bool) { while flag {} if flag {} }");
}
#[test]
fn identifier_condition_call_block() {
    good("fn f(flag: bool) { while flag { work(); } if flag { work(); } }");
}
#[test]
fn identifier_same_spelling_record_condition() {
    good("fn f(Battery: bool) { while Battery { work(); } }");
}
#[test]
fn branch_locals_inferred_records() {
    good("fn f(flag: bool,p: Percent) { if flag { let owned = Battery { charge: p }; inspect_percent(owned.charge); } else { let owned = Capacitor { charge: p }; inspect_percent(owned.charge); } }");
}
#[test]
fn conditional_new_record_then_read() {
    good("fn f(flag: bool,p: Percent) -> Percent { let b = if flag { Battery { charge: p } } else { Battery { charge: p } }; return b.charge; }");
}
#[test]
fn construction_kd033_terminal_owned_move() {
    good("fn f(flag: bool,t: Ticket) -> Battery { while flag { return Battery { charge: consume(t) }; } return Battery { charge: consume(t) }; }");
}
#[test]
fn shared_record_read_then_alias() {
    good("fn f(b: Battery) -> Percent { let view = &b; let seen = b.charge; let alias = view; let checked = inspect(alias); return seen; }");
}
#[test]
fn field_loan_path_sensitivity() {
    good("fn f(flag: bool,b: Battery,p: Percent) -> Percent { let mut owned = b; let access = &mut owned; if flag { let checked = inspect_mut(access); } else { let seen = owned.charge; } return owned.charge; }");
}
#[test]
fn copied_field_nested_value_call() {
    good("fn f(b: Battery) -> Battery { let seen = identity(b.charge); return b; }");
}
#[test]
fn constructed_records_move_independently() {
    good("fn f(p: Percent) -> Battery { let first = Battery { charge: p }; let second = Battery { charge: p }; discard(first); return second; }");
}
#[test]
fn construction_parenthesized() {
    good("fn f(p: Percent) -> Battery { return (Battery { charge: p }); }");
}
#[test]
fn distinct_nominal_range() {
    bad(
        "fn f(p: OtherPercent) -> Battery { return Battery { charge: p }; }",
        "Type",
        "record field initializer type mismatch",
    );
}
#[test]
fn out_of_range_integer_initializer_rejected() {
    bad(
        "fn f() -> Battery { return Battery { charge: 101 }; }",
        "Type",
        "literal outside named range",
    );
}
#[test]
fn unknown_constructor_valid_signature() {
    bad(
        "fn f(p: Percent) -> Battery { return Missing { charge: p }; }",
        "Resolve",
        "unknown record construction `Missing`",
    );
}
#[test]
fn unknown_field() {
    bad(
        "fn f(p: Percent) -> Battery { return Battery { missing: p }; }",
        "Resolve",
        "unknown field `missing`",
    );
}
#[test]
fn wrong_other_record_field() {
    bad(
        "fn f(p: Percent) -> Battery { return Battery { level: p }; }",
        "Resolve",
        "unknown field `level`",
    );
}
#[test]
fn distinct_record_return() {
    bad(
        "fn f(p: Percent) -> Capacitor { return Battery { charge: p }; }",
        "Type",
        "return type mismatch",
    );
}
#[test]
fn distinct_record_conditional() {
    bad("fn f(flag: bool,p: Percent) -> Battery { let b = if flag { Battery { charge: p } } else { Capacitor { charge: p } }; return b; }","Type","same type");
}
#[test]
fn initializer_late_move_failure() {
    bad(
        "fn f(t: Ticket) -> Battery { return Battery { charge: duplicate(t,t) }; }",
        "Ownership",
        "moved value",
    );
}
#[test]
fn initializer_prefix_loan_failure() {
    bad("fn f(t: Ticket,flag: bool) -> Battery { return Battery { charge: with_hold(t,&flag,t) }; }","Ownership","moved value");
}
#[test]
fn whole_no_value_call_late_failure() {
    bad(
        "fn f(t: Ticket) { receive(Battery { charge: consume(t) },t); }",
        "Ownership",
        "moved value",
    );
}
#[test]
fn moved_root() {
    bad(
        "fn f(b: Battery) -> Percent { discard(b); return b.charge; }",
        "Ownership",
        "moved value",
    );
}
#[test]
fn conditionally_moved_root() {
    bad(
        "fn f(flag: bool,b: Battery) -> Percent { if flag { discard(b); } return b.charge; }",
        "Ownership",
        "previous control-flow path",
    );
}
#[test]
fn whole_move_precedes_field_argument() {
    bad(
        "fn f(b: Battery) { move_then_copy(b,b.charge); }",
        "Ownership",
        "moved value",
    );
}
#[test]
fn exclusive_finite_future() {
    bad("fn f(b: Battery) -> Percent { let mut owned = b; let access = &mut owned; let seen = owned.charge; let checked = inspect_mut(access); return seen; }","Ownership","exclusively borrowed");
}
#[test]
fn exclusive_call_hold() {
    bad(
        "fn f(b: Battery) { let mut owned = b; held_mut(&mut owned,owned.charge); }",
        "Ownership",
        "exclusively borrowed",
    );
}
#[test]
fn exclusive_loop_post_use() {
    bad("fn f(flag: bool,b: Battery) -> Percent { let mut owned = b; let access = &mut owned; while flag { let seen = owned.charge; } return inspect_mut(access); }","Ownership","exclusively borrowed");
}
#[test]
fn exclusive_iteration_local() {
    bad("fn f(flag: bool,p: Percent) { while flag { let mut owned = Battery { charge: p }; let access = &mut owned; let seen = owned.charge; let checked = inspect_mut(access); } }","Ownership","exclusively borrowed");
}
#[test]
fn shared_read_does_not_end_loan() {
    bad("fn f(b: Battery) -> Percent { let view = &b; let seen = b.charge; discard(b); let checked = inspect(view); return seen; }","Ownership","shared-borrowed");
}
#[test]
fn shared_reference_root() {
    good("fn f(view: &Battery) -> Percent { return view.charge; }");
}
#[test]
fn mutable_reference_root() {
    good("fn f(view: &mut Battery) -> Percent { return view.charge; }");
}
#[test]
fn nonrecord_root() {
    bad(
        "fn f(p: Percent) -> Percent { return p.charge; }",
        "Type",
        "owned record",
    );
}
#[test]
fn unknown_root() {
    bad(
        "fn f() -> Percent { return missing.charge; }",
        "Resolve",
        "unknown value",
    );
}
#[test]
fn unknown_owned_field() {
    bad(
        "fn f(b: Battery) -> Percent { return b.missing; }",
        "Type",
        "unknown field",
    );
}
#[test]
fn wrong_field_inferred_local() {
    bad(
        "fn f(p: Percent) -> Percent { let b = Battery { charge: p }; return b.level; }",
        "Type",
        "unknown field",
    );
}
#[test]
fn call_temporary_projection() {
    bad(
        "fn f(p: Percent) -> Percent { return make(p).charge; }",
        "Parse",
        "temporary",
    );
}
#[test]
fn construction_temporary_projection() {
    bad(
        "fn f(p: Percent) -> Percent { return Battery { charge: p }.charge; }",
        "Parse",
        "temporary",
    );
}
#[test]
fn nested_projection() {
    bad(
        "fn f(b: Battery) -> Percent { return b.charge.value; }",
        "Parse",
        "nested projection",
    );
}
#[test]
fn field_borrow_target() {
    bad(
        "fn f(b: Battery) { let view = &b.charge; }",
        "Parse",
        "fields",
    );
}
#[test]
fn parameter_field_assignment_remains_ineligible() {
    bad(
        "fn f(b: Battery,p: Percent) { b.charge = p; }",
        "Type",
        "parameter",
    );
}
#[test]
fn field_replacement_target() {
    bad(
        "fn f(b: Battery,p: Percent) { b.charge -= p; }",
        "Parse",
        "compound assignment",
    );
}
#[test]
fn construction_recurring_bool_call() {
    bad(
        "fn f(p: Percent) { while ready(Battery { charge: p }) { work(); } }",
        "Ownership",
        "construction",
    );
}
#[test]
fn construction_deep_recurring_bool_call() {
    bad("fn f(p: Percent) { while flag_of(ready(Battery { charge: identity(p) })) == true { work(); } }","Ownership","construction");
}
#[test]
fn construction_does_not_sanitize_loop_move() {
    bad("fn f(flag: bool,t: Ticket) { while flag { let b = Battery { charge: consume(t) }; return; } }","Ownership","pre-existing non-Copy");
}
#[test]
fn construction_no_value_call_no_terminal_permission() {
    bad("fn f(flag: bool,t: Ticket) { while flag { discard(Battery { charge: consume(t) }); return; } }","Ownership","pre-existing non-Copy");
}
#[test]
fn outer_iteration_record_not_inner_local_move() {
    bad("fn f(flag: bool,p: Percent) { while flag { let b = Battery { charge: p }; while flag { discard(b); } } }","Ownership","pre-existing non-Copy");
}
#[test]
fn hidden_conditional_under_construction() {
    bad("fn f(flag: bool,p: Percent) -> Battery { return Battery { charge: if flag { p } else { p } }; }","Parse","expression");
}
#[test]
fn empty_construction() {
    bad(
        "fn f() -> Battery { return Battery {}; }",
        "Parse",
        "record construction",
    );
}
#[test]
fn missing_construction_colon() {
    bad(
        "fn f(p: Percent) -> Battery { return Battery { charge p }; }",
        "Parse",
        "requires ':'",
    );
}
#[test]
fn missing_construction_initializer() {
    bad(
        "fn f() -> Battery { return Battery { charge: }; }",
        "Parse",
        "field initializer",
    );
}
#[test]
fn repeated_construction_field() {
    bad(
        "fn f(p: Percent) -> Battery { return Battery { charge: p, charge: p }; }",
        "Resolve",
        "duplicate initializer field",
    );
}
#[test]
fn missing_field_name() {
    bad(
        "fn f(p: Percent) -> Battery { return Battery { : p }; }",
        "Parse",
        "named field",
    );
}
#[test]
fn no_value_initializer_call() {
    bad(
        "fn f(p: Percent) -> Battery { return Battery { charge: inspect_percent(p) }; }",
        "Type",
        "as an expression",
    );
}
#[test]
fn construct_as_discarded_statement() {
    bad(
        "fn f(p: Percent) { Battery { charge: p }; }",
        "Parse",
        "bare expression",
    );
}

#[test]
fn constructor_diagnostic_is_at_identifier_with_otherwise_valid_signature() {
    let source =
        format!("{DECL} fn bad(p: Percent) -> Battery {{ return Missing {{ charge: p }}; }}");
    let a = parse_source(&source).unwrap();
    let error = bowfin_compiler::resolve::resolve(&a).unwrap_err();
    assert!(error[0]
        .message
        .contains("unknown record construction `Missing`"));
    assert_eq!(error[0].span.start, source.find("Missing").unwrap());
    assert_eq!(error[0].span.end - error[0].span.start, "Missing".len());
}
#[test]
fn ordinary_records_have_canonical_ids_and_no_anonymous_locals_or_harness_bindings() {
    let p = good(
        "fn f(p: Percent) { discard(Battery { charge: p }); discard(Battery { charge: p }); }",
    );
    assert!(p.bindings().is_empty());
    assert!(p.locals().is_empty());
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let mut ids = vec![];
    for s in &f.body {
        let hir::ValueStatement::CallNoValue { arguments, .. } = s else {
            panic!()
        };
        let hir::ExprKind::RecordConstruct { record, fields } = &arguments[0].kind else {
            panic!()
        };
        assert_eq!(p.record(*record).fields[0], fields[0].field);
        assert_eq!(p.field(fields[0].field).record, *record);
        ids.push(*record);
    }
    assert_eq!(ids[0], ids[1]);
    let p = good("fn f(b: Battery) -> Percent { let owned = b; return owned.charge; }");
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let hir::ValueStatement::Return { value, .. } = &f.body[1] else {
        panic!()
    };
    let hir::ExprKind::CopyFieldRead {
        owner,
        record,
        field,
    } = value.kind
    else {
        panic!()
    };
    assert_eq!(owner, hir::Place::Local(p.locals()[0].id));
    assert_eq!(
        value.ty,
        hir::ExprType::Range(p.field(field).range_type().unwrap())
    );
    assert_eq!(p.record(record).fields[0], field);
}
#[test]
fn public_ast_cannot_hide_conditional_value_beneath_construction() {
    let mut a=parse_source(&format!("{DECL} fn f(flag: bool,p: Percent) -> Battery {{ let chosen = if flag {{ p }} else {{ p }}; return Battery {{ charge: p }}; }}")).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = a.functions.last_mut().unwrap() else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::Let {
        initializer: conditional,
        ..
    } = &f.body[0]
    else {
        panic!()
    };
    let conditional = conditional.clone();
    let bowfin_compiler::ast::ValueStatement::Return { value, .. } = &mut f.body[1] else {
        panic!()
    };
    let bowfin_compiler::ast::ExprKind::RecordConstruct { fields, .. } = &mut value.kind else {
        panic!()
    };
    fields[0].value = conditional;
    let r = bowfin_compiler::resolve::resolve(&a).unwrap();
    let e = bowfin_compiler::types::check(r).unwrap_err();
    assert!(e[0].message.contains("complete local initializers"));
}
#[test]
fn declaration_boundary_does_not_authorize_record_or_bool_fields() {
    let source="type Percent = range 0..100; record Battery { charge: Percent } record Outer { inner: Battery }";
    assert!(
        matches!(compile_source(source),Err(FrontendError::Resolve(e)) if e[0].message.contains("unknown range type `Battery`"))
    );
    assert!(matches!(
        compile_source("record Bad { value: bool }"),
        Err(FrontendError::Parse(_))
    ));
    assert!(matches!(
        compile_source(
            "type Percent = range 0..100; record Bad { first: Percent second: Percent }"
        ),
        Err(FrontendError::Parse(_))
    ));
}
#[test]
fn bare_integer_initializer_parses_and_forms_in_typing() {
    let a = parse_source(&format!(
        "{DECL} fn f() -> Battery {{ return Battery {{ charge: 80 }}; }}"
    ))
    .unwrap();
    let r = bowfin_compiler::resolve::resolve(&a).unwrap();
    bowfin_compiler::types::check(r).unwrap();
}
#[test]
fn identifier_binary_unary_and_nested_condition_blocks_are_preserved() {
    good("fn f(first: bool,second: bool) { while first && second { work(); } if !first { work(); } let chosen = if second { first } else { second }; }");
}
#[test]
fn verified_battery_path_and_literal_harness_remain_separate() {
    let source = include_str!("../../examples/battery_ok.bow");
    let p = compile_source(source).unwrap();
    let report = bowfin_compiler::verify::verify_report(&p);
    assert_eq!(report.functions_proven, 1);
    assert_eq!(report.calls_checked, 1);
    assert!(report.diagnostics.is_empty());
    assert_eq!(p.bindings().len(), 1);
    assert!(mir::lower(&p).functions().is_empty());
}

fn parse_diagnostic_at(body: &str, message: &str, marker: &str, offset: usize) {
    let source = format!("{DECL}\n{body}");
    let FrontendError::Parse(error) = compile_source(&source).unwrap_err() else {
        panic!("expected a Parse diagnostic");
    };
    assert_eq!(error.message, message);
    let start = source.rfind(marker).unwrap() + offset;
    assert_eq!(error.span.start, start);
    assert_eq!(error.span.end, start + usize::from(start < source.len()));
    assert_eq!(
        error.span.line,
        source[..start].bytes().filter(|b| *b == b'\n').count() + 1
    );
    assert_eq!(
        error.span.column,
        source[..start].rsplit('\n').next().unwrap().len() + 1
    );
}
const TEMPORARY_PROJECTION: &str = "field access through a temporary or nested projection is unsupported; use a directly named ordinary record owner or reference";
const MISSING_INITIALIZER: &str = "record construction requires a field initializer";

#[test]
fn parenthesized_call_result_projection_has_targeted_parse_diagnostic() {
    parse_diagnostic_at(
        "fn f(p: Percent) -> Percent {\n    return (make(p)).charge;\n}",
        TEMPORARY_PROJECTION,
        ".charge",
        0,
    );
}
#[test]
fn parenthesized_construction_result_projection_has_targeted_parse_diagnostic() {
    parse_diagnostic_at(
        "fn f(p: Percent) -> Percent {\n    return (Battery { charge: p }).charge;\n}",
        TEMPORARY_PROJECTION,
        ".charge",
        0,
    );
}
#[test]
fn parenthesized_projection_in_call_argument_has_targeted_parse_diagnostic() {
    for operand in ["(make(p)).charge", "(Battery { charge: p }).charge"] {
        parse_diagnostic_at(
            &format!(
                "fn f(p: Percent) -> Percent {{\n    return identity(identity({operand}));\n}}"
            ),
            TEMPORARY_PROJECTION,
            ".charge",
            0,
        );
    }
}
#[test]
fn legal_parenthesized_record_expressions_remain_accepted() {
    good("fn f(p: Percent) -> Battery { return (make((p))); }");
    good("fn f(p: Percent) -> Battery { return (Battery { charge: (identity((p))) }); }");
    good("fn f(b: Battery) -> Percent { return identity((b.charge)); }");
}
#[test]
fn construction_missing_initializer_at_semicolon_has_targeted_parse_diagnostic() {
    parse_diagnostic_at(
        "fn f(p: Percent) -> Battery {\n    return Battery { charge: ; };\n}",
        MISSING_INITIALIZER,
        ": ;",
        2,
    );
}
#[test]
fn construction_missing_initializer_at_eof_has_targeted_parse_diagnostic() {
    parse_diagnostic_at(
        "fn f(p: Percent) -> Battery {\n    return Battery { charge:",
        MISSING_INITIALIZER,
        "charge:",
        "charge:".len(),
    );
}
#[test]
fn nonempty_malformed_constructor_initializer_keeps_its_expression_diagnostic() {
    parse_diagnostic_at(
        "fn f(p: Percent) -> Battery {\n    return Battery { charge: p - ; };\n}",
        "unsupported or malformed expression: Semicolon",
        "- ;",
        2,
    );
}
