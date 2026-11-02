use bowfin_compiler::{ast, compile_source, hir, mir, parse_source, verify, FrontendError};
const DECL: &str = "type Percent = range 0..100; type Temperature = range 0..100; record Battery { charge: Percent, health: Percent, temperature: Temperature } record Capacitor { charge: Percent } record Snapshot { first: Percent, middle: Percent, last: Percent } fn id(p: Percent) -> Percent { return p; } fn three(a: Percent,b: Percent,c: Percent) -> Percent { return a; } fn finish(v: &Battery) {} fn finish_mut(v: &mut Battery) {} fn inspect(v: &Battery) -> Percent { return v.charge; } fn inspect_mut(v: &mut Battery) -> Percent { return v.health; } fn consume(b: Battery) -> Percent { return b.charge; } fn held(v: &Battery,p: Percent,b: Battery) {} fn held_mut(v: &mut Battery,p: Percent,b: Battery) {} fn copy_move(p: Percent,b: Battery) {} fn receive(s: Snapshot,b: Battery) {}";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL}\n{body}")).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, message: &str) {
    let e = compile_source(&format!("{DECL}\n{body}")).unwrap_err();
    let actual = match &e {
        FrontendError::Lex(_) => "Lex",
        FrontendError::Parse(_) => "Parse",
        FrontendError::Resolve(_) => "Resolve",
        FrontendError::Type(_) => "Type",
        FrontendError::Ownership(_) => "Ownership",
        FrontendError::Effects(_) => "Effects",
    };
    assert_eq!(actual, phase, "{e}");
    assert!(e.to_string().contains(message), "{e}");
}
macro_rules! accepted {
    ($name:ident, $body:literal) => {
        #[test]
        fn $name() {
            good($body);
        }
    };
}
macro_rules! rejected {
    ($name:ident, $phase:literal, $message:literal, $body:literal) => {
        #[test]
        fn $name() {
            bad($body, $phase, $message);
        }
    };
}
accepted!(shared_parameter_every_field,"fn f(v: &Battery) -> Temperature { let first = v.charge; let middle = v.health; return v.temperature; }");
accepted!(exclusive_parameter_every_field,"fn f(v: &mut Battery) -> Temperature { let first = v.charge; let middle = v.health; return v.temperature; }");
accepted!(
    inferred_shared_local,
    "fn f(b: Battery) -> Percent { let view = &b; return view.charge; }"
);
accepted!(inferred_exclusive_local,"fn f(b: Battery) -> Percent { let mut owned = b; let access = &mut owned; return access.health; }");
accepted!(
    shared_alias_preserves_origin,
    "fn f(v: &Battery) -> Percent { let alias = v; let seen = v.charge; return alias.health; }"
);
accepted!(
    exclusive_transfer_preserves_origin,
    "fn f(v: &mut Battery) -> Percent { let second = v; let third = second; return third.health; }"
);
accepted!(local_exclusive_transfer_preserves_origin,"fn f(b: Battery) -> Percent { let mut owned = b; let first = &mut owned; let second = first; return second.charge; }");
accepted!(
    exclusive_read_then_transfer,
    "fn f(v: &mut Battery) -> Percent { let seen = v.charge; finish_mut(v); return seen; }"
);
accepted!(initialization_assignment_call_return,"fn f(v: &Battery) -> Percent { let mut seen = v.charge; seen = id(v.health); return id(seen); }");
accepted!(
    write_through_range,
    "fn f(v: &Battery,a: &mut Percent) { *a = v.charge; }"
);
accepted!(owned_field_assignment_rhs,"fn f(v: &Battery,b: Battery) -> Battery { let mut owned = b; owned.charge = v.health; return owned; }");
accepted!(construction_written_order,"fn f(v: &Battery) -> Snapshot { return Snapshot { last: v.health, first: v.charge, middle: id(v.health) }; }");
accepted!(conditional_initializer,"fn f(flag: bool,v: &Battery) -> Percent { let chosen = if flag { v.charge } else { v.health }; return chosen; }");
accepted!(conditional_sibling_expiry,"fn f(flag: bool,b: Battery) -> Battery { let mut owned = b; let view = &owned; if flag { let seen = view.charge; } else { owned.health = 80; } return owned; }");
accepted!(conditional_both_reads_converge,"fn f(flag: bool,v: &Battery) -> Percent { if flag { let seen = v.charge; } else { let seen = v.health; } return v.charge; }");
accepted!(shared_final_use_expires,"fn f(b: Battery) -> Battery { let mut owned = b; let view = &owned; let seen = view.charge; owned.health = seen; return owned; }");
accepted!(exclusive_final_use_expires,"fn f(b: Battery) -> Battery { let mut owned = b; let access = &mut owned; let seen = access.charge; owned.health = seen; return owned; }");
accepted!(
    final_read_argument_creates_no_hold,
    "fn f(b: Battery) { let view = &b; copy_move(view.charge,b); }"
);
accepted!(
    final_read_nested_argument_creates_no_hold,
    "fn f(b: Battery) { let view = &b; copy_move(id(view.charge),b); }"
);
accepted!(stable_shared_loop,"fn f(flag: bool,v: &Battery) -> Percent { let mut seen = v.charge; while flag { seen = v.health; } return seen; }");
accepted!(stable_exclusive_loop_continue,"fn f(flag: bool,v: &mut Battery) -> Percent { let mut seen = v.charge; while flag { seen = v.health; continue; } return seen; }");
accepted!(nested_loops_and_continue,"fn f(flag: bool,v: &Battery) -> Percent { let mut seen = v.charge; while flag { while flag { seen = v.health; continue; } seen = v.charge; continue; } return seen; }");
accepted!(iteration_local_reference,"fn f(flag: bool,b: Battery) -> Battery { while flag { let view = &b; let seen = view.charge; } return b; }");
accepted!(loop_shared_alias,"fn f(flag: bool,v: &Battery) -> Percent { while flag { let alias = v; let seen = alias.charge; continue; } return v.health; }");
accepted!(loop_return_operand,"fn f(flag: bool,v: &Battery) -> Percent { while flag { return id(v.charge); } return v.health; }");
accepted!(
    owned_root_read_still_valid,
    "fn f(b: Battery) -> Temperature { let seen = b.charge; return b.temperature; }"
);
rejected!(
    unknown_field,
    "Type",
    "unknown field `voltage`",
    "fn f(v: &Battery) -> Percent { return v.voltage; }"
);
rejected!(
    non_record_shared_referent,
    "Type",
    "record referent",
    "fn f(v: &Percent) -> Percent { return v.charge; }"
);
rejected!(
    non_record_mutable_referent,
    "Type",
    "record referent",
    "fn f(v: &mut bool) -> Percent { return v.charge; }"
);
rejected!(
    nominal_result_mismatch,
    "Type",
    "return type mismatch",
    "fn f(v: &Battery) -> Temperature { return v.charge; }"
);
rejected!(
    moved_exclusive_handle,
    "Ownership",
    "use of moved value",
    "fn f(v: &mut Battery) -> Percent { finish_mut(v); return v.charge; }"
);
rejected!(
    conditionally_moved_handle,
    "Ownership",
    "previous control-flow path",
    "fn f(flag: bool,v: &mut Battery) -> Percent { if flag { finish_mut(v); } return v.charge; }"
);
rejected!(later_handle_protects_owner,"Ownership","while it is borrowed","fn f(b: Battery) -> Battery { let mut owned = b; let view = &owned; let seen = view.charge; owned.health = seen; finish(view); return owned; }");
rejected!(later_alias_protects_owner,"Ownership","while it is borrowed","fn f(b: Battery) -> Battery { let mut owned = b; let view = &owned; let alias = view; let seen = view.charge; owned.health = seen; finish(alias); return owned; }");
rejected!(exclusive_future_use_protects_owner,"Ownership","exclusively borrowed","fn f(b: Battery) -> Percent { let mut owned = b; let access = &mut owned; let seen = access.charge; let direct = owned.health; return access.health; }");
rejected!(
    legitimate_shared_receiving_hold,
    "Ownership",
    "shared-borrowed",
    "fn f(b: Battery) { let view = &b; held(view,id(view.charge),b); }"
);
rejected!(
    legitimate_exclusive_receiving_hold,
    "Ownership",
    "exclusively borrowed",
    "fn f(b: Battery) { let mut owned = b; let access = &mut owned; held_mut(access,80,owned); }"
);
rejected!(
    condition_handle_protection,
    "Ownership",
    "reference activity is unsupported in while conditions",
    "fn f(v: &Battery,p: Percent) { while v.charge <= p {} }"
);
rejected!(
    nested_condition_handle_protection,
    "Ownership",
    "reference activity is unsupported in while conditions",
    "fn f(v: &Battery,p: Percent) { while id(v.health) <= p {} }"
);
rejected!(recurrent_shared_protection,"Ownership","while it is borrowed","fn f(flag: bool,b: Battery) -> Battery { let mut owned = b; let view = &owned; while flag { let seen = view.charge; owned.health = seen; } return owned; }");
rejected!(
    recurrent_exclusive_transfer_remains_forbidden,
    "Ownership",
    "loop-carried mutable reference",
    "fn f(flag: bool,v: &mut Battery) { while flag { let seen = v.charge; finish_mut(v); } }"
);
accepted!(
    reference_assignment_parameter,
    "fn f(v: &mut Battery,p: Percent) { v.charge = p; }"
);
accepted!(reference_assignment_local,"fn f(b: Battery,p: Percent) { let mut owned = b; let access = &mut owned; access.charge = p; }");
rejected!(
    shared_field_borrow,
    "Parse",
    "direct owned",
    "fn f(v: &Battery) { let field = &v.charge; }"
);
rejected!(
    exclusive_field_borrow,
    "Parse",
    "direct owned",
    "fn f(v: &mut Battery) { let field = &mut v.charge; }"
);
rejected!(
    temporary_borrow_projection,
    "Parse",
    "temporary or nested projection",
    "fn f(b: Battery) -> Percent { return (&b).charge; }"
);
rejected!(
    temporary_call_projection,
    "Parse",
    "temporary or nested projection",
    "fn f(v: &Battery) -> Percent { return inspect(v).charge; }"
);
rejected!(
    dereference_projection,
    "Parse",
    "temporary or nested projection",
    "fn f(v: &Battery) -> Percent { return (*v).charge; }"
);
rejected!(
    nested_projection,
    "Parse",
    "temporary or nested projection",
    "fn f(v: &Battery) -> Percent { return v.charge.other; }"
);
rejected!(
    whole_record_deref_unchanged,
    "Ownership",
    "non-Copy value",
    "fn f(v: &Battery) -> Battery { return *v; }"
);
#[test]
fn same_spelling_selects_distinct_canonical_records_and_fields() {
    let p=good("fn f(v: &Battery) -> Percent { return v.charge; } fn g(v: &Capacitor) -> Percent { return v.charge; }");
    let reads = p
        .functions()
        .iter()
        .rev()
        .take(2)
        .map(|f| {
            let hir::ValueStatement::Return { value, .. } = &f.as_ordinary().unwrap().body[0]
            else {
                panic!()
            };
            let hir::ExprKind::CopyReferenceFieldRead { record, field, .. } = value.kind else {
                panic!()
            };
            assert_eq!(
                value.ty,
                hir::ExprType::Range(p.field(field).range_type().unwrap())
            );
            (record, field)
        })
        .collect::<Vec<_>>();
    assert_ne!(reads[0].0, reads[1].0);
    assert_ne!(reads[0].1, reads[1].1);
}
#[test]
fn field_token_span_survives_parentheses_and_unknown_field_diagnostic() {
    let s = format!("{DECL}\nfn f(v: &Battery) -> Percent {{ return (v.voltage); }}");
    let a = parse_source(&s).unwrap();
    let ast::FunctionDecl::Ordinary(f) = a.functions.last().unwrap() else {
        panic!()
    };
    let ast::ValueStatement::Return { value, .. } = &f.body[0] else {
        panic!()
    };
    let ast::ExprKind::FieldAccess(access) = &value.kind else {
        panic!()
    };
    assert_eq!(
        &s[access.field_span.start..access.field_span.end],
        "voltage"
    );
    assert!(value.span.start < access.field_span.start);
    let FrontendError::Type(d) = compile_source(&s).unwrap_err() else {
        panic!()
    };
    assert_eq!(d[0].span, access.field_span);
}
#[test]
fn first_middle_final_call_and_construction_children_are_real_handle_uses() {
    for position in 0..3 {
        let mut values = ["80", "80", "80"];
        values[position] = "v.charge";
        good(&format!(
            "fn f(v: &Battery) -> Percent {{ return three({}); }}",
            values.join(",")
        ));
        good(&format!("fn f(v: &Battery) -> Snapshot {{ return Snapshot {{ first: {}, middle: {}, last: {} }}; }}",values[0],values[1],values[2]));
        bad(&format!("fn f(b: Battery) {{ let mut owned = b; let v = &mut owned; let seen = three({}); let direct = owned.charge; finish_mut(v); }}",values.join(",")),"Ownership","exclusively borrowed");
    }
}
#[test]
fn ordinary_reference_reads_coexist_with_unchanged_verified_battery() {
    let source = format!(
        "{}\n{}",
        include_str!("../../examples/battery_ok.bow"),
        "fn read_battery(v: &Battery) -> Percent { return v.charge; }"
    );
    let p = compile_source(&source).unwrap();
    let result = verify::verify_report(&p);
    assert!(result.diagnostics.is_empty());
    assert_eq!((result.functions_proven, result.calls_checked), (1, 1));
}

accepted!(transferred_exclusive_handle_across_nested_loops,"fn f(flag: bool,v: &mut Battery) -> Percent { let alias = v; let mut seen = alias.charge; while flag { while flag { seen = alias.health; continue; } seen = alias.charge; } return seen; }");
accepted!(branch_local_alias_does_not_escape,"fn f(flag: bool,b: Battery) -> Battery { let mut owned = b; let v = &owned; if flag {let alias = v; let seen = alias.charge;} else {owned.health=80;} return owned; }");
rejected!(
    malformed_dotted_read,
    "Parse",
    "expected identifier",
    "fn f(v: &Battery) -> Percent {return v.;}"
);
rejected!(unsupported_verified_shared_state,"Parse","Mut","verified fn bad(v: &Battery,p: Percent) requires p <= v.charge ensures v.charge == old(v.charge) - p {v.charge -= p;}");
#[test]
fn harness_reference_roots_remain_outside_supported_literal_state() {
    let source = format!(
        "{}\nlet view = &battery;",
        include_str!("../../examples/battery_ok.bow")
    );
    assert!(matches!(
        compile_source(&source),
        Err(FrontendError::Parse(_))
    ));
}
