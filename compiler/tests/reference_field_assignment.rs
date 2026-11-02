use bowfin_compiler::{ast, compile_source, hir, mir, parse_source, verify, FrontendError};
const DECL: &str = "type Percent = range 0..100; type Other = range 0..100; record Battery { charge: Percent, health: Percent } record Capacitor { charge: Percent } record Triple { first: Percent, middle: Percent, last: Percent } fn id(p: Percent) -> Percent {return p;} fn three(a: Percent,b: Percent,c: Percent) -> Percent {return a;} fn triple(t: Triple) -> Percent {return t.first;} fn consume(b: Battery) -> Percent {return b.charge;} fn inspect(v: &Battery) -> Percent {return v.charge;} fn inspect_mut(v: &mut Battery) -> Percent {return v.charge;} fn held(v: &Battery,p: Percent) -> Percent {return p;} fn finish(v: &mut Battery) {}";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL}\n{body}")).unwrap();
    let m = mir::lower(&p);
    for f in m.functions() {
        f.validate().unwrap();
    }
    assert_eq!(m, mir::lower(&p));
    p
}
fn bad(body: &str, phase: &str, word: &str) {
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
    assert!(e.to_string().contains(word), "{e}");
}
macro_rules! accepted {
    ($n:ident,$s:literal) => {
        #[test]
        fn $n() {
            good($s);
        }
    };
}
macro_rules! rejected {
    ($n:ident,$p:literal,$w:literal,$s:literal) => {
        #[test]
        fn $n() {
            bad($s, $p, $w);
        }
    };
}
accepted!(
    external_parameter,
    "fn f(v: &mut Battery,p: Percent) {v.charge=p;}"
);
accepted!(contextual_literal, "fn f(v: &mut Battery) {v.charge=80;}");
accepted!(immutable_handle_binding_and_final_expiry,"fn f(b: Battery,p: Percent) -> Battery {let mut owned=b; let v=&mut owned; v.charge=p; return owned;}");
accepted!(
    same_handle_other_field,
    "fn f(v: &mut Battery) {v.charge=v.health;}"
);
accepted!(
    same_handle_same_field,
    "fn f(v: &mut Battery) {v.charge=v.charge;}"
);
accepted!(later_handle_use,"fn f(b: Battery) -> Battery {let mut owned=b; let v=&mut owned; v.charge=80; finish(v); return owned;}");
accepted!(
    transferred_external_recipient,
    "fn f(v: &mut Battery) {let next=v; next.charge=80;}"
);
accepted!(transferred_local_recipient,"fn f(b: Battery) -> Battery {let mut owned=b; let first=&mut owned; let next=first; next.charge=80; return owned;}");
accepted!(
    other_record_exact_value,
    "fn f(v: &mut Battery,c: Capacitor) {v.health=c.charge;}"
);
accepted!(
    unrelated_owner_consumption,
    "fn f(v: &mut Battery,b: Battery) {v.charge=consume(b);}"
);
accepted!(
    unrelated_temporary_shared_borrow,
    "fn f(v: &mut Battery,b: Battery) {v.charge=inspect(&b);}"
);
accepted!(
    unrelated_temporary_exclusive_borrow,
    "fn f(v: &mut Battery,b: Battery) {let mut other=b; v.charge=inspect_mut(&mut other);}"
);
accepted!(
    nested_calls_preserve_target_hold,
    "fn f(v: &mut Battery,b: Battery) {v.charge=held(&b,id(three(v.health,v.charge,80)));}"
);
accepted!(
    stable_loop,
    "fn f(flag: bool,v: &mut Battery,p: Percent) {while flag {v.charge=p;}}"
);
accepted!(
    loop_same_handle_reads,
    "fn f(flag: bool,v: &mut Battery) {while flag {v.charge=v.health;}}"
);
accepted!(nested_loop_continue,"fn f(flag: bool,v: &mut Battery) {while flag {while flag {v.charge=v.health; continue;} v.health=80; continue;}}");
accepted!(loop_break_discharge,"fn f(flag: bool,b: Battery) -> Battery {let mut owned=b; let v=&mut owned; while flag {v.charge=80; break;} return owned;}");
accepted!(loop_iteration_local,"fn f(flag: bool,b: Battery) -> Battery {let mut owned=b; while flag {let v=&mut owned; v.charge=80;} return owned;}");
accepted!(branches_converge,"fn f(flag: bool,v: &mut Battery) {if flag {v.charge=80;} else {v.health=90;} v.charge=v.health;}");
accepted!(
    owned_field_write_unchanged,
    "fn f(b: Battery) -> Battery {let mut owned=b; owned.charge=owned.health; return owned;}"
);
rejected!(
    shared_parameter,
    "Type",
    "exact &mut Record",
    "fn f(v: &Battery) {v.charge=80;}"
);
rejected!(
    shared_local,
    "Type",
    "exact &mut Record",
    "fn f(b: Battery) {let v=&b; v.charge=80;}"
);
rejected!(
    exclusive_nonrecord,
    "Type",
    "record referent",
    "fn f(v: &mut Percent) {v.charge=80;}"
);
rejected!(
    shared_nonrecord,
    "Type",
    "record referent",
    "fn f(v: &bool) {v.charge=80;}"
);
rejected!(
    owned_parameter,
    "Type",
    "parameter",
    "fn f(b: Battery) {b.charge=80;}"
);
rejected!(
    immutable_owned,
    "Type",
    "immutable",
    "fn f(b: Battery) {let owned=b; owned.charge=80;}"
);
rejected!(
    unknown_actual_field,
    "Type",
    "unknown field",
    "fn f(v: &mut Capacitor) {v.health=80;}"
);
rejected!(
    nominal_mismatch,
    "Type",
    "field assignment RHS",
    "fn f(v: &mut Battery,p: Other) {v.charge=p;}"
);
rejected!(
    literal_outside,
    "Type",
    "outside",
    "fn f(v: &mut Battery) {v.charge=101;}"
);
rejected!(
    no_nested_literal_context,
    "Type",
    "incompatible operand",
    "fn f(v: &mut Battery) {v.charge=81-1;}"
);
rejected!(
    original_owner_read,
    "Ownership",
    "exclusively borrowed",
    "fn f(b: Battery) {let mut owned=b; let v=&mut owned; v.charge=owned.health;}"
);
rejected!(
    original_owner_move,
    "Ownership",
    "borrowed",
    "fn f(b: Battery) {let mut owned=b; let v=&mut owned; v.charge=consume(owned);}"
);
rejected!(
    original_owner_shared_borrow,
    "Ownership",
    "borrow",
    "fn f(b: Battery) {let mut owned=b; let v=&mut owned; v.charge=inspect(&owned);}"
);
rejected!(
    original_owner_exclusive_borrow,
    "Ownership",
    "borrow",
    "fn f(b: Battery) {let mut owned=b; let v=&mut owned; v.charge=inspect_mut(&mut owned);}"
);
rejected!(
    rhs_transfers_target,
    "Ownership",
    "moved value",
    "fn f(v: &mut Battery) {v.charge=inspect_mut(v);}"
);
rejected!(
    moved_transfer_source,
    "Ownership",
    "moved value",
    "fn f(v: &mut Battery) {let next=v; v.charge=80; finish(next);}"
);
rejected!(
    conditional_transfer,
    "Ownership",
    "moved",
    "fn f(flag: bool,v: &mut Battery) {if flag {finish(v);} v.charge=80;}"
);
rejected!(later_use_protects_owner,"Ownership","borrowed","fn f(b: Battery) {let mut owned=b; let v=&mut owned; v.charge=80; owned.health=90; finish(v);}");
rejected!(
    loop_transfer_during_rhs,
    "Ownership",
    "loop-carried mutable reference",
    "fn f(flag: bool,v: &mut Battery) {while flag {v.charge=inspect_mut(v);}}"
);
rejected!(
    condition_reference_still_forbidden,
    "Ownership",
    "while conditions",
    "fn f(v: &mut Battery,p: Percent) {while v.charge<=p {v.health=p;}}"
);
rejected!(
    whole_record_extract,
    "Ownership",
    "non-Copy",
    "fn f(v: &mut Battery) {v.charge=consume(*v);}"
);
rejected!(
    whole_record_replace,
    "Ownership",
    "non-Copy",
    "fn f(v: &mut Battery,b: Battery) {*v=b;}"
);
#[test]
fn projection_and_assignment_expression_exclusions() {
    for s in [
        "(v).charge=80;",
        "(*v).charge=80;",
        "v.charge.health=80;",
        "inspect_mut(v).charge=80;",
        "v.charge-=80;",
        "let x=(v.charge=80);",
        "v.charge=v.health=80;",
        "v.charge=if true {80} else {90};",
    ] {
        bad(&format!("fn f(v: &mut Battery) {{{s}}}"), "Parse", "");
    }
}
#[test]
fn all_rhs_positions_calls_and_constructions() {
    for i in 0..3 {
        let mut a = ["80", "80", "80"];
        a[i] = "id(v.health)";
        good(&format!(
            "fn f(v: &mut Battery) {{v.charge=three({});}}",
            a.join(",")
        ));
        good(&format!(
            "fn f(v: &mut Battery) {{v.charge=triple(Triple {{last:{},first:{},middle:{}}});}}",
            a[0], a[1], a[2]
        ));
    }
}
#[test]
fn distinct_operation_canonical_ids_and_all_spans() {
    let s = format!("{DECL}\nfn f(v: &mut Battery) {{v.health = v.charge;}}");
    let a = parse_source(&s).unwrap();
    let ast::FunctionDecl::Ordinary(f) = a.functions.last().unwrap() else {
        panic!()
    };
    let ast::ValueStatement::FieldAssign {
        owner_span,
        field_span,
        value,
        span,
        ..
    } = &f.body[0]
    else {
        panic!()
    };
    assert_eq!(&s[owner_span.start..owner_span.end], "v");
    assert_eq!(&s[field_span.start..field_span.end], "health");
    assert_eq!(&s[value.span.start..value.span.end], "v.charge");
    assert_eq!(&s[span.start..span.end], "v.health = v.charge;");
    let p = compile_source(&s).unwrap();
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let hir::ValueStatement::CopyReferenceFieldAssign {
        reference,
        record,
        field,
        value,
        reference_span,
        field_span: fs,
        span: ss,
    } = &f.body[0]
    else {
        panic!()
    };
    assert_eq!(*reference, hir::Place::Parameter(f.parameters[0]));
    assert_eq!(p.field(*field).name, "health");
    assert_eq!(p.field(*field).record, *record);
    assert_eq!(
        value.ty,
        hir::ExprType::Range(p.field(*field).range_type().unwrap())
    );
    assert_eq!((reference_span, fs, ss), (owner_span, field_span, span));
    let m = mir::lower(&p);
    let mf = m.functions().last().unwrap();
    let mir::Statement::CopyReferenceFieldAssign {
        reference: mr,
        record: rr,
        field: ff,
        value: vv,
        reference_span: rs,
        field_span: ms,
        span: ts,
    } = &mf.block(mf.entry()).statements()[0]
    else {
        panic!()
    };
    assert_eq!(
        (mr, rr, ff, vv, rs, ms, ts),
        (reference, record, field, value, reference_span, fs, ss)
    );
}
#[test]
fn type_and_parse_diagnostic_token_spans() {
    for (s, word, token, phase) in [
        ("v.missing=80;", "unknown field", "missing", "Type"),
        ("v.charge=101;", "outside", "101", "Type"),
        ("v. = 80;", "field name", "=", "Parse"),
    ] {
        let source = format!("{DECL}\nfn f(v: &mut Battery) {{{s}}}");
        let e = compile_source(&source).unwrap_err();
        let (message, span) = match &e {
            FrontendError::Type(d) => {
                assert_eq!(phase, "Type");
                (&d[0].message, d[0].span)
            }
            FrontendError::Parse(d) => {
                assert_eq!(phase, "Parse");
                (&d.message, d.span)
            }
            _ => panic!("{e}"),
        };
        assert!(message.contains(word));
        assert_eq!(&source[span.start..span.end], token);
    }
}
#[test]
fn verified_and_harness_positive_controls_and_new_form_exclusions() {
    let s = include_str!("../../examples/battery_ok.bow");
    let before = verify::verify_report(&compile_source(s).unwrap());
    assert_eq!((before.functions_proven, before.calls_checked), (1, 1));
    let after = verify::verify_report(
        &compile_source(&format!("{s}\nfn set(v: &mut Battery) {{v.charge=80;}}")).unwrap(),
    );
    assert_eq!(before.diagnostics, after.diagnostics);
    assert_eq!(before.final_values, after.final_values);
    assert_eq!(
        (before.functions_proven, before.calls_checked),
        (after.functions_proven, after.calls_checked)
    );
    let verified = s.replace("battery.charge -= amount;", "battery.charge = amount;");
    assert!(matches!(
        compile_source(&verified),
        Err(FrontendError::Parse(_))
    ));
    let harness = format!("{s}\n battery.charge = 80;");
    assert!(matches!(
        compile_source(&harness),
        Err(FrontendError::Parse(_))
    ));
}

#[test]
fn deep_boolean_and_binary_rhs_visit_all_reference_occurrences() {
    good("fn choose(flag: bool,p: Percent) -> Percent {return p;} fn f(v: &mut Battery,p: Percent) {v.charge=choose(!(v.charge<=p) || (v.health==p && v.charge<=p),id(v.health)-p);}");
    bad("fn choose(flag: bool,p: Percent) -> Percent {return p;} fn f(b: Battery,p: Percent) {let mut owned=b; let v=&mut owned; v.charge=choose(!(v.health<=p),owned.charge-p);}","Ownership","exclusively borrowed");
}
