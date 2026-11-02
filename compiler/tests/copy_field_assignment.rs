use bowfin_compiler::{ast, compile_source, hir, mir, parse_source, resolve, types, FrontendError};
const DECL: &str = "type Percent = range 0..100; type OtherPercent = range 0..100; type Voltage = range -5..5000; record Battery { charge: Percent, voltage: Voltage, health: Percent } record Ticket { value: Percent } record Capacitor { charge: Percent } record Meter { level: Percent } fn id(p: Percent) -> Percent { return p; } fn consume(t: Ticket) -> Percent { return t.value; } fn measure(b: Battery) -> Percent { return b.charge; } fn shared(v: &Battery) -> Percent { return 80; } fn exclusive(v: &mut Battery) -> Percent { return 80; } fn finish_shared(v: &Battery) {} fn finish_exclusive(v: &mut Battery) {} fn held(v: &Battery,p: Percent) -> Percent { return p; } fn held_mut(v: &mut Battery,p: Percent) -> Percent { return p; } fn pair(a: Percent,b: Percent,c: Percent) -> Percent { return a; }";
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
fn bad(body: &str, phase: &str, word: &str) {
    let e = compile_source(&source(body)).unwrap_err();
    let actual = match e {
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
macro_rules! accept {
    ($n:ident,$s:expr) => {
        #[test]
        fn $n() {
            good($s);
        }
    };
}
macro_rules! reject {
    ($n:ident,$s:expr,$p:expr,$w:expr) => {
        #[test]
        fn $n() {
            bad($s, $p, $w);
        }
    };
}
accept!(all_destination_positions_reversed_written_order,"fn f(b: Battery) -> Battery { let mutable working = b; working.health = 100; working.voltage = -5; working.charge = 0; return working; }");
accept!(exact_parameter_local_call_dereference_and_cross_record_values,"fn f(b: Battery,p: Percent,c: Capacitor) -> Battery { let mut working = b; let local = p; let view = &local; working.charge = *view; working.health = id(p); working.charge = c.charge; working.health = local; return working; }");
accept!(rhs_reads_old_destination_and_other_field,"fn f(b: Battery) -> Battery { let mut working = b; working.charge = id(working.charge); working.health = working.charge; return working; }");
accept!(parenthesized_literal_endpoints,"fn f(b: Battery) -> Battery { let mut working = b; working.charge = (0); working.health = (100); working.voltage = 5000; return working; }");
accept!(temporary_shared_borrow_finishes_before_write,"fn f(b: Battery) -> Battery { let mut working = b; working.charge = shared(&working); return working; }");
accept!(temporary_exclusive_borrow_finishes_before_write,"fn f(b: Battery) -> Battery { let mut working = b; working.charge = exclusive(&mut working); return working; }");
accept!(final_shared_rhs_handle_use_expires,"fn f(b: Battery) -> Battery { let mut working = b; let view = &working; working.charge = shared(view); return working; }");
accept!(final_mutable_rhs_handle_use_expires,"fn f(b: Battery) -> Battery { let mut working = b; let access = &mut working; working.charge = exclusive(access); return working; }");
accept!(unused_exclusive_handle_expires,"fn f(b: Battery) -> Battery { let mut working = b; let access = &mut working; working.charge = 80; return working; }");
accept!(unused_shared_handle_expires,"fn f(b: Battery) -> Battery { let mut working = b; let view = &working; working.charge = 80; return working; }");
accept!(completed_handle_expires,"fn f(b: Battery) -> Battery { let mut working = b; let view = &working; finish_shared(view); working.charge = 80; return working; }");
accept!(unrelated_rhs_move_commits_without_consuming_target,"fn f(b: Battery,t: Ticket) -> Battery { let mut working = b; working.charge = consume(t); return working; }");
reject!(unrelated_rhs_move_is_not_restored_on_success,"fn f(b: Battery,t: Ticket) -> Ticket { let mut working = b; working.charge = consume(t); return t; }","Ownership","moved value");
reject!(rhs_consumes_target_before_final_availability,"fn f(b: Battery) -> Battery { let mut working = b; working.charge = measure(working); return working; }","Ownership","moved value");
reject!(
    prior_whole_record_move,
    "fn f(b: Battery) { let mut working = b; let moved = working; working.charge = 80; }",
    "Ownership",
    "moved value"
);
reject!(conditional_root_unavailability,"fn f(flag: bool,b: Battery) { let mut working = b; if flag { let moved = working; } working.charge = 80; }","Ownership","may have been moved");
reject!(finite_shared_loan_blocks_write,"fn f(b: Battery) { let mut working = b; let view = &working; working.charge = 80; finish_shared(view); }","Ownership","while it is borrowed");
reject!(finite_exclusive_loan_blocks_write,"fn f(b: Battery) { let mut working = b; let access = &mut working; working.charge = 80; finish_exclusive(access); }","Ownership","while it is borrowed");
reject!(nested_rhs_call_hold_blocks_target_move,"fn f(b: Battery) { let mut working = b; working.charge = held(&working,id(measure(working))); }","Ownership","borrowed");
reject!(nested_rhs_exclusive_hold_blocks_read,"fn f(b: Battery) { let mut working = b; working.charge = held_mut(&mut working,id(working.health)); }","Ownership","exclusively borrowed");
reject!(
    immutable_root,
    "fn f(b: Battery) { let working = b; working.charge = 80; }",
    "Type",
    "immutable local"
);
reject!(
    parameter_root,
    "fn f(b: Battery) { b.charge = 80; }",
    "Type",
    "parameter"
);
reject!(
    shared_reference_root_category_before_mutability,
    "fn f(b: Battery) { let view = &b; view.charge = 80; }",
    "Type",
    "through a reference"
);
accept!(
    mutable_reference_root_category_before_mutability,
    "fn f(b: Battery) { let mut working = b; let access = &mut working; access.charge = 80; }"
);
reject!(
    non_record_root_category_before_mutability,
    "fn f(p: Percent) { let x = p; x.charge = 80; }",
    "Type",
    "owned record local"
);
reject!(
    nominal_equal_bounds_remain_distinct,
    "fn f(b: Battery,p: OtherPercent) { let mut working = b; working.charge = p; }",
    "Type",
    "field assignment RHS"
);
reject!(
    field_not_selected_from_another_record,
    "fn f(b: Battery) { let mut working = b; working.level = 80; }",
    "Type",
    "unknown field `level`"
);
reject!(
    nonliteral_no_contextual_propagation,
    "fn f(b: Battery) { let mut working = b; working.charge = 81 - 1; }",
    "Type",
    "incompatible operand types"
);
reject!(
    unresolved_rhs_precedes_parameter_type_error,
    "fn f(b: Battery) { b.charge = missing; }",
    "Resolve",
    "unknown"
);
reject!(
    unresolved_root,
    "fn f(p: Percent) { missing.charge = p; }",
    "Resolve",
    "unknown"
);
reject!(no_terminal_permission_in_assignment_rhs,"fn f(flag: bool,b: Battery) -> Battery { let mut working = b; while flag { working.charge = measure(working); return working; } return working; }","Ownership","pre-existing non-Copy");
reject!(recurrent_shared_rhs_use_blocks_write,"fn f(flag: bool,b: Battery) { let mut working = b; let view = &working; while flag { working.charge = shared(view); } }","Ownership","while it is borrowed");
reject!(field_rhs_preserves_carried_mutable_handle_transfer_restriction,"fn f(flag: bool,b: Battery) { let mut working = b; let access = &mut working; while flag { working.charge = exclusive(access); } }","Ownership","loop-carried mutable reference");
accept!(loop_local_final_reference_use_is_finite,"fn f(flag: bool,b: Battery) -> Battery { let mut working = b; while flag { let view = &working; working.charge = shared(view); } return working; }");
accept!(branches_loops_continue_break_and_returns,"fn f(flag: bool,b: Battery) -> Battery { let mut working = b; while flag { if flag { working.charge = 80; continue; } else { working.health = 100; break; } } if flag { working.voltage = 0; return working; } else { working.charge = 90; } return working; }");
accept!(conditional_initializer_rhs_value_used_normally,"fn f(flag: bool,b: Battery,p: Percent) -> Battery { let mut working = b; let replacement = if flag { p } else { id(p) }; working.charge = replacement; return working; }");
accept!(sibling_path_last_use_expiry,"fn f(flag: bool,b: Battery) -> Battery { let mut working = b; let view = &working; if flag { let observed = shared(view); working.charge = 80; } else { working.health = 100; } return working; }");
#[test]
fn malformed_field_statement_diagnostics_and_locations() {
    for (statement, word, location) in [
        ("working. = 80;", "field name", "="),
        (
            "working.charge.health = 80;",
            "nested field projection",
            ".health",
        ),
        ("working.charge -= 80;", "requires `=`", "-="),
        ("working.charge;", "requires `=`", ";"),
        ("working.charge = ;", "RHS expression", ";"),
        ("working.charge = 80 }", "expected", "}"),
    ] {
        let s = source(&format!(
            "fn f(b: Battery) {{ let mut working = b; {statement} }}"
        ));
        let FrontendError::Parse(e) = compile_source(&s).unwrap_err() else {
            panic!("{s}")
        };
        assert!(e.message.contains(word), "{}", e.message);
        let start = s.rfind(statement).unwrap() + statement.find(location).unwrap();
        assert_eq!(e.span.start, start, "{statement}");
    }
}
#[test]
fn contextual_bounds_fail_at_literal_before_ownership() {
    for n in ["-1", "101"] {
        let s = source(&format!(
            "fn f(b: Battery) {{ let mut working = b; working.charge = {n}; }}"
        ));
        let FrontendError::Type(e) = compile_source(&s).unwrap_err() else {
            panic!()
        };
        assert!(e[0].message.contains("outside"), "{}", e[0].message);
        assert_eq!(&s[e[0].span.start..e[0].span.end], n);
    }
}
#[test]
fn projection_and_expression_assignment_exclusions_stay_parse_errors() {
    for statement in [
        "(working).charge = 80;",
        "measure(working).charge = 80;",
        "(Battery { charge: 80, voltage: 0, health: 100 }).charge = 80;",
        "working.charge.health = 80;",
        "working.charge = working.health = 80;",
        "let x = (working.charge = 80);",
        "working.charge = if true { 80 } else { 90 };",
        "working.charge -= 80;",
    ] {
        bad(
            &format!("fn f(b: Battery) {{ let mut working = b; {statement} }}"),
            "Parse",
            "",
        );
    }
}
#[test]
fn ast_hir_mir_identity_and_each_source_span_survive() {
    let s=source("fn f(b: Battery) -> Battery { let mut working = b; working.health = working.charge; return working; }");
    let a = parse_source(&s).unwrap();
    let ast::FunctionDecl::Ordinary(f) = a.functions.last().unwrap() else {
        panic!()
    };
    let ast::ValueStatement::FieldAssign {
        owner,
        field,
        owner_span,
        field_span,
        value,
        span,
    } = &f.body[1]
    else {
        panic!()
    };
    assert_eq!(owner, "working");
    assert_eq!(field, "health");
    assert_eq!(&s[owner_span.start..owner_span.end], "working");
    assert_eq!(&s[field_span.start..field_span.end], "health");
    assert_eq!(&s[value.span.start..value.span.end], "working.charge");
    assert_eq!(&s[span.start..span.end], "working.health = working.charge;");
    let p = compile_source(&s).unwrap();
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let hir::ValueStatement::CopyFieldAssign {
        owner,
        record,
        field,
        value,
        ..
    } = &f.body[1]
    else {
        panic!()
    };
    assert_eq!(p.local(*owner).ty, hir::ValueType::Record(*record));
    assert_eq!(p.field(*field).name, "health");
    assert_eq!(p.field(*field).record, *record);
    assert_eq!(
        value.ty,
        hir::ExprType::Range(p.field(*field).range_type().unwrap())
    );
    let m = mir::lower(&p);
    let f = m.functions().last().unwrap();
    f.validate().unwrap();
    let mir::Statement::CopyFieldAssign {
        owner: mo,
        record: mr,
        field: mf,
        value: mv,
        ..
    } = &f.block(f.entry()).statements()[1]
    else {
        panic!()
    };
    assert_eq!((mo, mr, mf, mv), (owner, record, field, value));
    assert!(p.bindings().is_empty());
    assert!(p.statements().is_empty());
}
#[test]
fn public_ast_receives_identical_root_and_rhs_gates() {
    for (body, word) in [
        ("fn f(b: Battery) { b.charge = 80; }", "parameter"),
        (
            "fn f(b: Battery) { let view = &b; view.charge = 80; }",
            "reference",
        ),
        (
            "fn f(b: Battery) { let working = b; working.charge = 80; }",
            "immutable",
        ),
    ] {
        let ast = parse_source(&source(body)).unwrap();
        let errors = types::check(resolve::resolve(&ast).unwrap()).unwrap_err();
        assert!(errors[0].message.contains(word));
    }
}
#[test]
fn nested_rhs_first_middle_final_call_and_construction_children() {
    for position in 0..3 {
        let mut entries = ["id(80)", "id(80)", "id(80)"];
        entries[position] = "id(shared(view))";
        let rhs = format!("id(pair({}, {}, {}))", entries[0], entries[1], entries[2]);
        good(&format!("fn f(b: Battery) -> Battery {{ let mut working = b; let view = &working; working.charge = {rhs}; return working; }}"));
        bad(&format!("fn f(flag: bool,b: Battery) {{ let mut working = b; let view = &working; while flag {{ working.charge = {rhs}; }} }}"),"Ownership","borrowed");
        let names = ["charge", "health", "voltage"];
        let mut order = names.to_vec();
        order.swap(position, 0);
        let values = order
            .iter()
            .map(|name| {
                format!(
                    "{name}: {}",
                    if *name == "voltage" {
                        "0"
                    } else if *name == "charge" {
                        "id(shared(view))"
                    } else {
                        "80"
                    }
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        good(&format!("fn f(b: Battery) -> Battery {{ let mut working = b; let view = &working; working.charge = id(measure(Battery {{ {values} }})); return working; }}"));
    }
}
#[test]
fn verified_and_harness_path_isolation_with_ordinary_field_write() {
    let battery = include_str!("../../examples/battery_ok.bow");
    let ordinary="fn update(b: Battery) -> Battery { let mut working = b; working.charge = 80; return working; }";
    let before = bowfin_compiler::verify::verify_report(&compile_source(battery).unwrap());
    let after = bowfin_compiler::verify::verify_report(
        &compile_source(&format!("{battery}\n{ordinary}")).unwrap(),
    );
    assert_eq!(before.diagnostics, after.diagnostics);
    assert_eq!(before.functions_proven, after.functions_proven);
    assert_eq!(before.calls_checked, after.calls_checked);
    assert_eq!(before.final_values, after.final_values);
}

fn final_conflict_diagnostic(borrow: &str, finish: &str, field: &str, kind: &str) {
    let body = format!("fn f(b: Battery) {{\n    let mut working = b;\n    let handle = {borrow};\n    working.{field} = id(80);\n    {finish}(handle);\n}}");
    let s = source(&body);
    // Resolution and typing succeed before the ownership-only final conflict.
    let p = types::check(resolve::resolve(&parse_source(&s).unwrap()).unwrap()).unwrap();
    let f = p.functions().last().unwrap().as_ordinary().unwrap();
    let hir::ValueStatement::CopyFieldAssign {
        owner,
        record,
        field: selected,
        value,
        span,
        ..
    } = &f.body[2]
    else {
        panic!()
    };
    assert_eq!(p.local(*owner).name, "working");
    assert_eq!(p.record(*record).name, "Battery");
    assert_eq!(p.field(*selected).name, field);
    assert_eq!(
        value.ty,
        hir::ExprType::Range(p.field(*selected).range_type().unwrap())
    );
    assert_eq!(
        p.range(p.field(*selected).range_type().unwrap()).name,
        "Percent"
    );
    let FrontendError::Ownership(errors) = compile_source(&s).unwrap_err() else {
        panic!()
    };
    let e = &errors[0];
    assert_eq!(
        e.message,
        format!("cannot assign `working.{field}` while it is borrowed")
    );
    assert!(e.required.contains(&format!("field `Battery.{field}`")));
    assert!(e.required.contains("declared named-range type `Percent`"));
    assert!(e
        .required
        .contains(&format!("active {kind} whole-record loan")));
    assert!(!e.required.contains(if kind == "shared" {
        "exclusive"
    } else {
        "shared"
    }));
    assert_eq!(e.span, *span);
    assert_eq!(
        &s[e.span.start..e.span.end],
        format!("working.{field} = id(80);")
    );
    // Borrow expressions retain their exact origin, and the existing location
    // machinery reports it independently of the destination statement span.
    let hir::ValueStatement::Let { initializer, .. } = &f.body[1] else {
        panic!()
    };
    let origin = initializer.span;
    assert_eq!(&s[origin.start..origin.end], borrow);
    assert_eq!(
        e.known,
        vec![format!(
            "the conflicting borrow began at line {}, column {}",
            origin.line, origin.column
        )]
    );
    let rendered = e.render("final_conflict.bow", &s);
    assert!(rendered.contains(&format!(
        "--> final_conflict.bow:{}:{}",
        span.line, span.column
    )));
}
#[test]
fn final_shared_conflict_reports_canonical_destination_type_and_loan_origin() {
    final_conflict_diagnostic("&working", "finish_shared", "health", "shared");
}
#[test]
fn final_exclusive_conflict_reports_canonical_destination_type_and_loan_origin() {
    final_conflict_diagnostic("&mut working", "finish_exclusive", "charge", "exclusive");
}
