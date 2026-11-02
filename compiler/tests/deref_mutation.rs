use bowfin_compiler::{
    ast, compile_source,
    diagnostics::Diagnostic,
    hir::{ExprKind, ExprType, ParameterType, Place, ValueStatement, ValueType},
    lexer::{lex, TokenKind},
    parse_source,
    verify::verify_report,
    FrontendError,
};
const DECLS: &str = "type Percent = range 0..100; type OtherPercent = range 0..100;
record Ticket { value: Percent } record Voucher { value: Percent }
fn read(value: &Percent) -> Percent { return *value; }
fn read_mut(value: &mut Percent) -> Percent { return *value; }
fn identity(value: Percent) -> Percent { return value; }
fn consume(value: Ticket) -> Ticket { return value; }";
fn source(body: &str) -> String {
    format!("{DECLS}\n{body}")
}
fn valid(body: &str) -> bowfin_compiler::hir::Program {
    compile_source(&source(body)).unwrap()
}
fn type_error(body: &str) -> Diagnostic {
    let Err(FrontendError::Type(mut errors)) = compile_source(&source(body)) else {
        panic!("expected Type error: {body}")
    };
    assert_eq!(errors.len(), 1);
    errors.remove(0)
}
fn ownership_error(body: &str) -> Diagnostic {
    let Err(FrontendError::Ownership(mut errors)) = compile_source(&source(body)) else {
        panic!("expected Ownership error: {body}")
    };
    assert_eq!(errors.len(), 1);
    errors.remove(0)
}
#[test]
fn star_is_explicit_dereference_and_not_multiplication() {
    let tokens = lex("*value = *value;").unwrap();
    assert_eq!(tokens[0].kind, TokenKind::Star);
    assert_eq!(tokens[3].kind, TokenKind::Star);
    assert!(matches!(
        parse_source("fn bad(value: bool) -> bool { return value * value; }"),
        Err(FrontendError::Parse(_))
    ));
}
#[test]
fn parser_preserves_named_dereferences_write_statements_and_spans() {
    let text = "fn set(value: &mut bool) -> bool { *value = !*value; return *value; }";
    let program = parse_source(text).unwrap();
    let ast::FunctionDecl::Ordinary(f) = &program.functions[0] else {
        panic!("ordinary")
    };
    let ast::ValueStatement::DerefAssign {
        reference,
        value,
        span,
    } = &f.body[0]
    else {
        panic!("write")
    };
    assert_eq!(reference, "value");
    assert_eq!(&text[span.start..span.end], "*value = !*value;");
    let ast::ExprKind::Unary { operand, .. } = &value.kind else {
        panic!("not")
    };
    assert_eq!(
        operand.kind,
        ast::ExprKind::Deref {
            reference: "value".into()
        }
    );
    assert_eq!(&text[operand.span.start..operand.span.end], "*value");
    let ast::ValueStatement::Return { value, .. } = &f.body[1] else {
        panic!("return")
    };
    assert_eq!(value.kind, operand.kind);
}
#[test]
fn shared_and_mutable_copy_reads_work_repeatedly() {
    for ty in ["bool", "Percent"] {
        for prefix in ["&", "&mut "] {
            valid(&format!("fn copy(value: {prefix}{ty}) -> {ty} {{ let first = *value; let second = *value; return *value; }}"));
        }
    }
}
#[test]
fn dereference_has_exact_canonical_referent_type_and_place() {
    let program = valid("fn example(value: Percent) -> Percent { let view = &value; let predecessor_result = *view; return predecessor_result; } fn set(value: &mut Percent, replacement: Percent) -> Percent { *value = replacement; return *value; }");
    let range = program.ranges()[0].id;
    let example = program.functions()[4].as_ordinary().unwrap();
    let ValueStatement::Let {
        local, initializer, ..
    } = &example.body[1]
    else {
        panic!("let")
    };
    assert_eq!(initializer.ty, ExprType::Range(range));
    assert_eq!(
        initializer.kind,
        ExprKind::Deref {
            reference: Place::Local(program.locals()[0].id)
        }
    );
    assert_eq!(program.local(*local).ty, ValueType::Range(range));
    let set = program.functions()[5].as_ordinary().unwrap();
    let ValueStatement::DerefAssign {
        reference, value, ..
    } = &set.body[0]
    else {
        panic!("write")
    };
    assert_eq!(*reference, Place::Parameter(set.parameters[0]));
    assert_eq!(value.ty, ExprType::Range(range));
    assert_eq!(value.kind, ExprKind::Parameter(set.parameters[1]));
    assert_eq!(
        program.parameter(set.parameters[1]).ty,
        ParameterType::Value(ValueType::Range(range))
    );
}
#[test]
fn bool_and_range_writes_are_copy_safe_and_non_consuming() {
    valid("fn set(value: &mut bool) -> bool { *value = true; *value = false; return *value; }");
    valid("fn set(value: &mut Percent, replacement: Percent) -> Percent { let before = *value; *value = replacement; let after = *value; *value = after; return *value; }");
}
#[test]
fn self_read_write_through_one_exclusive_capability_is_valid() {
    for ty in ["bool", "Percent"] {
        valid(&format!(
            "fn same(value: &mut {ty}) -> {ty} {{ *value = *value; return *value; }}"
        ));
    }
    valid("fn same(value: &mut Percent) -> Percent { *value = identity(*value); return *value; }");
}
#[test]
fn shared_alias_reads_preserve_copy_reference_handles() {
    valid("fn example(value: Percent) -> Percent { let view = &value; let first = view; let second = view; let a = *view; let b = *first; let c = *second; return c; }");
}
#[test]
fn mutable_handle_can_be_read_written_then_transferred() {
    valid("fn relay(value: &mut Percent, replacement: Percent) -> Percent { let before = *value; *value = replacement; let after = *value; return read_mut(value); }");
    valid("fn example(value: Percent) -> Percent { let mut owned = value; let first = &mut owned; let before = *first; let second = first; let after = *second; return owned; }");
}
#[test]
fn non_copy_read_materialization_is_rejected_in_every_value_context() {
    for prefix in ["&", "&mut "] {
        for body in [
            "return *value;",
            "let extracted = *value; return extracted;",
            "return consume(*value);",
        ] {
            let error = ownership_error(&format!(
                "fn bad(value: {prefix}Ticket) -> Ticket {{ {body} }}"
            ));
            assert_eq!(
                error.message,
                "cannot move non-Copy value `Ticket` out through borrowed reference"
            );
            assert!(error.required.contains("only Copy referents"));
        }
    }
}
#[test]
fn non_copy_replacement_is_an_ownership_error_without_destruction() {
    let body = "fn bad(value: &mut Ticket, replacement: Ticket) -> bool { *value = replacement; return true; }";
    let error = ownership_error(body);
    assert!(error
        .message
        .contains("cannot replace non-Copy value `Ticket`"));
    assert!(error
        .message
        .contains("before destruction semantics are defined"));
    assert_eq!(
        &source(body)[error.span.start..error.span.end],
        "*value = replacement;"
    );
}
#[test]
fn transferred_mutable_parameters_cannot_be_read_or_written() {
    for operation in ["let after = *value;", "*value = replacement;"] {
        let body = format!("fn bad(value: &mut Percent, replacement: Percent) -> Percent {{ let before = *value; let moved = read_mut(value); {operation} return moved; }}");
        let error = ownership_error(&body);
        assert_eq!(error.message, "use of moved value `value`");
        assert!(error.known[0].contains("by-value parameter"));
    }
}
#[test]
fn moved_from_local_reference_cannot_be_read_or_written() {
    for operation in ["let predecessor_result = *first;", "*first = replacement;"] {
        let error = ownership_error(&format!("fn bad(value: Percent, replacement: Percent) -> Percent {{ let mut owned = value; let first = &mut owned; let second = first; {operation} return owned; }}"));
        assert_eq!(error.message, "use of moved value `first`");
        assert!(error.known[0].contains("into local `second`"));
    }
}
#[test]
fn writing_through_shared_and_non_reference_values_is_a_type_error() {
    assert!(
        type_error("fn bad(value: &bool) -> bool { *value = true; return true; }")
            .message
            .contains("requires an exclusive mutable reference")
    );
    assert!(type_error("fn bad(value: &Ticket, replacement: Ticket) -> bool { *value = replacement; return true; }").message.contains("requires an exclusive mutable reference"));
    for body in [
        "fn bad(value: Percent) -> Percent { return *value; }",
        "fn bad(value: bool) -> bool { *value = true; return value; }",
    ] {
        assert!(type_error(body)
            .message
            .contains("dereference requires a reference value"));
    }
}
#[test]
fn write_rhs_requires_exact_concrete_referent_identity() {
    valid("fn formed(value: &mut Percent) -> bool { *value = 0; return true; }");
    assert!(
        type_error("fn bad(value: &mut Percent) -> bool { *value = 101; return true; }")
            .message
            .contains("literal outside named range")
    );
    for body in [
        "fn bad(value: &mut bool, replacement: Percent) -> bool { *value = replacement; return true; }",
        "fn bad(value: &mut Percent, replacement: OtherPercent) -> bool { *value = replacement; return true; }",
        "fn bad(value: &mut Percent, replacement: &Percent) -> bool { *value = replacement; return true; }",
        "fn bad(value: &mut Ticket, replacement: Voucher) -> bool { *value = replacement; return true; }",
    ] { assert!(type_error(body).message.contains("write-through RHS type mismatch")); }
    let error = type_error("fn bad(value: &mut Percent, replacement: OtherPercent) -> bool { *value = replacement; return true; }");
    assert!(error.required.contains("Percent") && error.required.contains("OtherPercent"));
}
#[test]
fn dereference_does_not_add_nominal_operator_or_return_conversions() {
    assert!(
        type_error("fn bad(value: &OtherPercent) -> Percent { return *value; }")
            .message
            .contains("distinct named ranges")
    );
    assert!(type_error(
        "fn bad(first: &Percent, second: &OtherPercent) -> Percent { return *first - *second; }"
    )
    .message
    .contains("incompatible named range types"));
    valid("fn subtract(first: &Percent, second: &Percent) -> Percent { return *first - *second; }");
    valid("fn compare(first: &Percent, second: &Percent) -> bool { return *first <= *second; }");
    valid("fn invert(value: &bool) -> bool { return !*value; }");
}
#[test]
fn named_access_resolution_preserves_order_and_function_scope() {
    for body in [
        "fn bad(value: Percent) -> Percent { return *missing; }",
        "fn bad(value: Percent) -> bool { *missing = value; return true; }",
        "fn bad(value: Percent) -> Percent { let copy = *later; let later = &value; return copy; }",
        "fn bad(value: Percent) -> Percent { let view = *view; return view; }",
        "fn first(value: Percent) -> Percent { let view = &value; return *view; } fn second(value: Percent) -> Percent { return *view; }",
    ] { assert!(matches!(compile_source(&source(body)), Err(FrontendError::Resolve(_))), "{body}"); }
}
#[test]
fn arbitrary_dereference_operands_and_field_projection_remain_unsupported() {
    for operand in [
        "*(&value)",
        "*(&mut value)",
        "*(view)",
        "*read(view)",
        "**view",
        "*view.value",
        "(*view).value",
    ] {
        let body =
            format!("fn bad(value: Percent) -> Percent {{ let view = &value; return {operand}; }}");
        assert!(
            matches!(parse_source(&source(&body)), Err(FrontendError::Parse(_))),
            "{operand}"
        );
    }
}
#[test]
fn assignment_expressions_compound_assignment_and_reference_projected_writes_stay_absent() {
    for body in [
        "let predecessor_result = (*value = true); return predecessor_result;",
        "return (*value = true);",
        "*value = *value = true; return true;",
        "*value -= true; return true;",
        "*value += true; return true;",
        "*value *= true; return true;",
        "(*value)++; return true;",
        "value = other; return true;",
        "(*value).field = true; return true;",
        "value.field = true; return true;",
        "*value; return true;",
    ] {
        assert!(
            compile_source(&source(&format!(
                "fn bad(value: &mut bool, other: &mut bool) -> bool {{ {body} }}"
            )))
            .is_err(),
            "{body}"
        );
    }
}
#[test]
fn final_shared_read_allows_a_later_exclusive_loan() {
    valid("fn example(value: Percent, replacement: Percent) -> Percent { let mut owned = value; let view = &owned; let before = *view; let access = &mut owned; *access = replacement; return owned; }");
}
#[test]
fn future_shared_read_and_alias_read_keep_shared_loans_live() {
    for later in ["view", "alias"] {
        let error = ownership_error(&format!("fn bad(value: Percent) -> Percent {{ let mut owned = value; let view = &owned; let alias = view; let before = *view; let access = &mut owned; let after = *{later}; return owned; }}"));
        assert!(error.message.contains("while shared borrow is active"));
    }
    valid("fn example(value: Percent) -> Percent { let mut owned = value; let view = &owned; let alias = view; let before = *view; let after = *alias; let access = &mut owned; let last = *access; return owned; }");
}
#[test]
fn final_mutable_read_or_write_allows_owner_recovery() {
    for last in [
        "let predecessor_result = *access;",
        "*access = replacement;",
    ] {
        valid(&format!("fn example(value: Percent, replacement: Percent) -> Percent {{ let mut owned = value; let access = &mut owned; {last} return owned; }}"));
    }
    valid("fn example(value: Percent, replacement: Percent) -> Percent { let mut owned = value; let access = &mut owned; *access = replacement; let observed = *access; return owned; }");
}
#[test]
fn future_read_or_write_preserves_exclusive_owner_conflicts() {
    for future in ["let later = *access;", "*access = replacement;"] {
        let error = ownership_error(&format!("fn bad(value: Percent, replacement: Percent) -> Percent {{ let mut owned = value; let access = &mut owned; let copy = owned; {future} return copy; }}"));
        assert_eq!(
            error.message,
            "cannot use `owned` while it is exclusively borrowed"
        );
    }
}
#[test]
fn moved_to_handle_controls_future_access_liveness() {
    let error = ownership_error("fn bad(value: Percent) -> Percent { let mut owned = value; let first = &mut owned; let second = first; let copy = owned; let observed = *second; return copy; }");
    assert!(error.message.contains("exclusively borrowed"));
    valid("fn example(value: Percent) -> Percent { let mut owned = value; let first = &mut owned; let second = first; let observed = *second; return owned; }");
    valid("fn example(value: Percent) -> Percent { let mut owned = value; let first = &mut owned; let second = first; return owned; }");
}
#[test]
fn copy_owner_reads_still_coexist_with_shared_read_through() {
    valid("fn example(value: Percent) -> Percent { let view = &value; let direct = value; let indirect = *view; return direct; }");
}
#[test]
fn direct_owner_rhs_conflicts_even_on_the_last_write() {
    for rhs in ["owned", "identity(owned)"] {
        let error = ownership_error(&format!("fn bad(value: Percent) -> Percent {{ let mut owned = value; let access = &mut owned; *access = {rhs}; return owned; }}"));
        assert!(error.message.contains("exclusively borrowed"));
    }
    // A final RHS dereference and inner call must not expire the write-held loan.
    for rhs in ["*access == owned", "identity_bool(*access) == owned"] {
        let error = ownership_error(&format!("fn identity_bool(value: bool) -> bool {{ return value; }} fn bad(value: bool) -> bool {{ let mut owned = value; let access = &mut owned; *access = {rhs}; return owned; }}"));
        assert!(error.message.contains("exclusively borrowed"));
    }
}
#[test]
fn rhs_cannot_transfer_the_handle_and_then_write_through_it() {
    let error = ownership_error("fn bad(value: &mut Percent) -> Percent { *value = read_mut(value); return true_percent(*value); } fn true_percent(value: Percent) -> Percent { return value; }");
    assert_eq!(error.message, "use of moved value `value`");
    assert!(error.known[0].contains("by-value parameter"));
}
#[test]
fn writes_hold_exclusivity_through_nested_rhs_borrowing() {
    let error = ownership_error("fn bad(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; *access = read_mut(&mut owned); return owned; }");
    assert!(error.message.contains("second exclusive borrow"));
    let error = ownership_error("fn bad(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; *access = read(&owned); return owned; }");
    assert!(error.message.contains("while exclusive borrow is active"));
}
#[test]
fn copy_dereference_arguments_do_not_hold_reference_loans_for_the_call() {
    valid("fn pair(first: Percent, second: Percent) -> Percent { return second; } fn example(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; return pair(*access, owned); }");
    valid("fn pair(first: Percent, second: Percent) -> Percent { return second; } fn example(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; return pair(identity(*access), owned); }");
    valid("fn pair(first: Percent, second: Percent) -> Percent { return second; } fn example(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; return pair(*access, read_mut(&mut owned)); }");
    let error = ownership_error("fn pair(first: &mut Percent, second: Percent) -> Percent { return second; } fn bad(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; return pair(access, owned); }");
    assert!(error.message.contains("exclusively borrowed"));
}
#[test]
fn reference_call_holds_still_survive_final_nested_dereference() {
    let error = ownership_error("fn pair(first: &mut Percent, second: Percent) -> Percent { return second; } fn bad(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; return pair(&mut owned, *access); }");
    assert!(error.message.contains("second exclusive"));
    let error = ownership_error("fn three(first: &Percent, second: Percent, third: &mut Percent) -> Percent { return second; } fn bad(value: Percent) -> Percent { let mut owned = value; let view = &owned; return three(view, identity(*view), &mut owned); }");
    assert!(error.message.contains("while shared borrow is active"));
}
#[test]
fn type_errors_precede_non_copy_or_moved_handle_ownership_errors() {
    assert!(type_error("fn bad(value: &mut Ticket, replacement: Voucher) -> bool { *value = replacement; return true; }").message.contains("distinct named records"));
    assert!(type_error("fn bad(value: &mut Percent) -> bool { let before = read_mut(value); *value = true; return true; }").message.contains("write-through RHS type mismatch"));
}
#[test]
fn reference_returns_reborrowing_and_implicit_conversions_remain_rejected() {
    for body in [
        "fn bad(value: &Percent) -> &Percent { return value; }",
        "fn bad(value: &&Percent) -> bool { return true; }",
        "fn bad(value: &Percent) -> Percent { return identity(value); }",
        "fn bad(value: &mut Percent) -> Percent { return read(value); }",
        "fn bad(value: &mut Percent) -> bool { let nested = &value; return true; }",
        "fn bad(value: &mut Percent) -> bool { let nested = &*value; return true; }",
    ] {
        assert!(compile_source(&source(body)).is_err(), "{body}");
    }
}
#[test]
fn ordinary_copy_mutation_is_not_executed_or_counted_as_verified() {
    let program = valid("fn set(value: &mut Percent, replacement: Percent) -> Percent { *value = replacement; return *value; }");
    let report = verify_report(&program);
    assert!(report.diagnostics.is_empty());
    assert_eq!((report.functions_proven, report.calls_checked), (0, 0));
    let combined = format!("{} fn set(value: &mut Percent, replacement: Percent) -> Percent {{ *value = replacement; return *value; }}", include_str!("../../examples/battery_ok.bow"));
    let report = verify_report(&compile_source(&combined).unwrap());
    assert!(report.diagnostics.is_empty());
    assert_eq!((report.functions_proven, report.calls_checked), (1, 1));
}
