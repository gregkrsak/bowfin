use bowfin_compiler::{
    compile_source, diagnostics::Diagnostic, hir::*, ownership, parse_source, resolve, types,
    verify::verify_report, FrontendError,
};

const DECLARATIONS: &str = "type Percent = range 0..100; record Ticket { value: Percent } record Voucher { value: Percent }";
const IDENTITY: &str = "fn identity(ticket: Ticket) -> Ticket { return ticket; }";
const INSPECT: &str = "fn inspect(ticket: Ticket) -> bool { return true; }";

fn source(body: &str) -> String {
    format!("{DECLARATIONS}\n{body}")
}
fn compile(body: &str) -> Program {
    compile_source(&source(body)).unwrap()
}
fn typed(body: &str) -> Program {
    types::check(resolve::resolve(&parse_source(&source(body)).unwrap()).unwrap()).unwrap()
}
fn moved(body: &str) -> Diagnostic {
    let Err(FrontendError::Ownership(errors)) = compile_source(&source(body)) else {
        panic!("expected ownership error: {body}")
    };
    assert_eq!(errors.len(), 1);
    errors.into_iter().next().unwrap()
}
fn type_error(body: &str) -> Diagnostic {
    let Err(FrontendError::Type(errors)) = compile_source(&source(body)) else {
        panic!("expected type error before ownership: {body}")
    };
    errors.into_iter().next().unwrap()
}

#[test]
fn record_signatures_parameters_locals_calls_and_returns_have_canonical_types() {
    let program = compile(&format!("{IDENTITY} fn forward(ticket: Ticket) -> Ticket {{ let first = ticket; let next = identity(first); return next; }}"));
    let ticket = program.records()[0].id;
    let identity = program.functions()[0].as_ordinary().unwrap();
    let forward = program.functions()[1].as_ordinary().unwrap();
    assert_eq!(identity.return_type, Some(ValueType::Record(ticket)));
    assert_eq!(forward.return_type, Some(ValueType::Record(ticket)));
    assert_eq!(
        program.parameter(forward.parameters[0]).ty,
        ParameterType::Value(ValueType::Record(ticket))
    );
    let locals: &[Local] = program.locals();
    assert_eq!(locals.len(), 2);
    for local in locals {
        assert_eq!(local.ty, ValueType::Record(ticket));
        assert_eq!(local.function, forward.id);
        assert_eq!(program.local(local.id), local);
    }
    let ValueStatement::Let { initializer, .. } = &forward.body[0] else {
        panic!("let")
    };
    assert_eq!(initializer.ty, ExprType::Record(ticket));
    assert_eq!(initializer.kind, ExprKind::Parameter(forward.parameters[0]));
    let ValueStatement::Let { initializer, .. } = &forward.body[1] else {
        panic!("let")
    };
    let ExprKind::Call {
        function,
        arguments,
    } = &initializer.kind
    else {
        panic!("call")
    };
    assert_eq!(*function, identity.id);
    assert_eq!(initializer.ty, ExprType::Record(ticket));
    assert_eq!(arguments[0].kind, ExprKind::Local(locals[0].id));
    let ValueStatement::Return { value, .. } = &forward.body[2] else {
        panic!("return")
    };
    assert_eq!(value.ty, ExprType::Record(ticket));
    assert_eq!(value.kind, ExprKind::Local(locals[1].id));
}

#[test]
fn owned_parameters_and_locals_move_through_bindings_and_returns() {
    compile("fn forward(ticket: Ticket) -> Ticket { let first = ticket; let second = first; return second; } fn direct(ticket: Ticket) -> Ticket { return ticket; }");
}

#[test]
fn copy_parameters_and_locals_remain_reusable_after_bindings_and_calls() {
    for ty in ["bool", "Percent"] {
        compile(&format!("fn identity(value: {ty}) -> {ty} {{ return value; }} fn repeat(value: {ty}) -> {ty} {{ let first = value; let second = value; let third = first; let fourth = first; let predecessor_result = identity(value); let again = identity(value); return value; }} fn local_repeat(value: {ty}) -> {ty} {{ let first = value; let a = identity(first); let b = identity(first); return first; }}"));
    }
}

#[test]
fn parameter_reuse_after_binding_reports_the_illegal_use_and_prior_move() {
    let body = "fn bad(ticket: Ticket) -> Ticket {\n    let next = ticket;\n    return ticket;\n}";
    let text = source(body);
    let diagnostic = moved(body);
    assert_eq!(diagnostic.message, "use of moved value `ticket`");
    assert_eq!(&text[diagnostic.span.start..diagnostic.span.end], "ticket");
    assert_eq!(diagnostic.span.line, 4);
    assert_eq!(diagnostic.known.len(), 1);
    assert!(diagnostic.known[0].contains("into local `next` at line 3, column 16"));
    let rendered = diagnostic.render("move.bow", &text);
    assert!(rendered.contains("move.bow:4:12"));
    assert!(rendered.contains("ownership checking stopped"));
    for internal in ["ParameterId", "LocalId", "Available", "Moved", "borrow"] {
        assert!(!rendered.contains(internal), "{rendered}");
    }
}

#[test]
fn double_binding_consumption_stops_at_the_second_use() {
    let body = "fn bad(ticket: Ticket) -> Ticket { let first = ticket; let second = ticket; let third = ticket; return ticket; }";
    let diagnostic = moved(body);
    assert_eq!(
        diagnostic.span.start,
        source(body).find("let second = ticket").unwrap() + "let second = ".len()
    );
    assert!(diagnostic.known[0].contains("into local `first`"));
}

#[test]
fn local_cannot_be_reused_after_transfer_to_another_local() {
    let diagnostic = moved("fn bad(ticket: Ticket) -> Ticket { let first = ticket; let second = first; return first; }");
    assert_eq!(diagnostic.message, "use of moved value `first`");
    assert!(diagnostic.known[0].contains("into local `second`"));
}

#[test]
fn record_arguments_are_consumed_even_when_the_result_is_copy() {
    let diagnostic = moved(&format!("{INSPECT} fn bad(ticket: Ticket) -> bool {{ let first = inspect(ticket); let second = inspect(ticket); return first; }}"));
    assert_eq!(diagnostic.message, "use of moved value `ticket`");
    assert!(diagnostic.known[0].contains("into by-value parameter `ticket`"));
    let diagnostic = moved(&format!("{INSPECT} fn bad(ticket: Ticket) -> Ticket {{ let first = ticket; let predecessor_result = inspect(first); return first; }}"));
    assert_eq!(diagnostic.message, "use of moved value `first`");
    compile(&format!("{INSPECT} fn once(ticket: Ticket) -> bool {{ let predecessor_result = inspect(ticket); return predecessor_result; }}"));
}

#[test]
fn call_result_can_be_bound_transferred_or_returned_directly() {
    compile(&format!("{IDENTITY} fn bound(ticket: Ticket) -> Ticket {{ let next = identity(ticket); return next; }} fn direct(ticket: Ticket) -> Ticket {{ return identity(ticket); }} fn chained(ticket: Ticket) -> Ticket {{ let first = identity(ticket); let second = first; return identity(second); }}"));
    assert_eq!(moved(&format!("{IDENTITY} fn bad(ticket: Ticket) -> Ticket {{ let next = identity(ticket); return ticket; }}")).message, "use of moved value `ticket`");
    assert_eq!(moved(&format!("{IDENTITY} fn bad(ticket: Ticket) -> Ticket {{ let first = identity(ticket); let second = identity(first); return first; }}")).message, "use of moved value `first`");
}

#[test]
fn nested_calls_transfer_temporary_results_without_extra_places() {
    compile(&format!("{IDENTITY} fn outer(ticket: Ticket) -> Ticket {{ return identity(identity(identity(ticket))); }}"));
    let diagnostic = moved(&format!("{IDENTITY} fn bad(ticket: Ticket) -> Ticket {{ let first = identity(identity(ticket)); return ticket; }}"));
    assert!(diagnostic.known[0].contains("into by-value parameter `ticket`"));
}

#[test]
fn duplicate_consuming_arguments_report_the_later_source_occurrence() {
    let choose = "fn choose(first: Ticket, second: Ticket) -> Ticket { return first; }";
    let body =
        format!("{choose} fn bad(ticket: Ticket) -> Ticket {{ return choose(ticket, ticket); }}");
    let diagnostic = moved(&body);
    assert_eq!(
        diagnostic.span.start,
        source(&body).rfind("ticket);").unwrap()
    );
    assert!(diagnostic.known[0].contains("into by-value parameter `first`"));
    compile(&format!("{choose} fn good(first: Ticket, second: Ticket) -> Ticket {{ return choose(first, second); }}"));
}

#[test]
fn nested_arguments_and_binary_call_children_are_checked_in_source_order() {
    let body = format!("{IDENTITY} fn choose(a: Ticket, b: Ticket) -> Ticket {{ return a; }} fn bad(ticket: Ticket) -> Ticket {{ return choose(identity(ticket), identity(ticket)); }}");
    assert_eq!(
        moved(&body).span.start,
        source(&body).rfind("ticket));").unwrap()
    );
    for expression in [
        "inspect(ticket) && inspect(ticket)",
        "inspect(ticket) || inspect(ticket)",
        "inspect(ticket) == inspect(ticket)",
        "!inspect(ticket) && inspect(ticket)",
    ] {
        let body = format!("{INSPECT} fn bad(ticket: Ticket) -> bool {{ return {expression}; }}");
        assert_eq!(
            moved(&body).span.start,
            source(&body).rfind("ticket)").unwrap()
        );
    }
}

#[test]
fn forward_direct_and_indirect_recursive_record_calls_remain_structurally_valid() {
    compile("fn first(ticket: Ticket) -> Ticket { return second(ticket); } fn second(ticket: Ticket) -> Ticket { return first(ticket); } fn recurse(ticket: Ticket) -> Ticket { return recurse(ticket); }");
    let error = moved("fn bad(ticket: Ticket) -> Ticket { let next = later(ticket); return ticket; } fn later(ticket: Ticket) -> Ticket { return ticket; }");
    assert_eq!(error.message, "use of moved value `ticket`");
}

#[test]
fn unused_owned_values_and_identical_names_in_separate_functions_are_valid() {
    compile("fn first(ticket: Ticket) -> Ticket { let next = ticket; return next; } fn second(ticket: Ticket) -> Ticket { let next = ticket; return next; } fn ignore(ticket: Ticket) -> bool { return true; } fn unused_local(ticket: Ticket) -> bool { let unused = ticket; return true; }");
}

#[test]
fn record_nominal_mismatches_and_record_operators_fail_during_typing() {
    for body in ["fn bad(voucher: Voucher) -> Ticket { return voucher; }", "fn identity(ticket: Ticket) -> Ticket { return ticket; } fn bad(voucher: Voucher) -> Ticket { return identity(voucher); }"] {
        let error = type_error(body);
        assert!(error.message.contains("distinct named records"));
        assert!(error.required.contains("Ticket") && error.required.contains("Voucher"));
    }
    for (expression, result) in [
        ("ticket == ticket", "bool"),
        ("ticket <= ticket", "bool"),
        ("ticket - ticket", "Ticket"),
        ("ticket && true", "bool"),
        ("ticket || true", "bool"),
        ("!ticket", "bool"),
    ] {
        let error = type_error(&format!(
            "fn bad(ticket: Ticket) -> {result} {{ return {expression}; }}"
        ));
        assert!(!error.message.contains("moved"));
    }
    for body in [
        "fn bad(ticket: Ticket) -> Percent { return ticket; }",
        "fn bad(value: Percent) -> Ticket { return value; }",
        "fn bad() -> Ticket { return 0; }",
    ] {
        assert!(type_error(body).message.contains("return type mismatch"));
    }
}

#[test]
fn name_and_type_errors_precede_ownership_errors() {
    let body = "fn bad(ticket: Ticket) -> Ticket { let next = ticket; return missing; }";
    assert!(matches!(
        compile_source(&source(body)),
        Err(FrontendError::Resolve(_))
    ));
    let body = "fn bad(ticket: Ticket, voucher: Voucher) -> Ticket { let first = ticket; let second = ticket; return voucher; }";
    assert!(type_error(body).message.contains("return type mismatch"));
    let body = "fn identity(ticket: Ticket) -> Ticket { return ticket; } fn bad(voucher: Voucher) -> Ticket { let next = voucher; return identity(voucher); }";
    assert!(type_error(body)
        .message
        .contains("call argument type mismatch"));
    let body = "fn bad(ticket: Ticket) -> Ticket { let next = ticket; return ticket; }";
    let typed = typed(body);
    assert_eq!(ownership::check(&typed).unwrap_err().len(), 1);
    assert!(compile_source(&source(body))
        .unwrap_err()
        .to_string()
        .contains("use of moved value `ticket`"));
}

#[test]
fn unsupported_record_features_and_existing_restrictions_are_not_broadened() {
    for body in [
        "return ticket.value;",
        "return Ticket { value: 101 };",
        "let mut next = ticket; next = ticket; return next;",
        "let mutable next = ticket; next = ticket; return next;",
        "let next = ticket; next = ticket; return next;",
        "return &ticket;",
        "return &mut ticket;",
        "ticket",
        "ticket;",
        "let five = 5; return ticket;",
        "return ticket + ticket;",
        "let next = ticket; next.value -= 1; return next;",
    ] {
        assert!(
            compile_source(&source(&format!(
                "fn bad(ticket: Ticket) -> Ticket {{ {body} }}"
            )))
            .is_err(),
            "{body}"
        );
    }
    let battery = include_str!("../../examples/battery_ok.bow");
    let ordinary = "fn bad(ticket: Battery) -> Battery { return consume(ticket); }";
    assert!(matches!(
        compile_source(&format!("{battery}\n{ordinary}")),
        Err(FrontendError::Resolve(_))
    ));
}

#[test]
fn ownership_checked_record_functions_are_not_verified_and_preserve_battery_behavior() {
    let ordinary = "fn identity(ticket: Battery) -> Battery { let next = ticket; return next; }";
    let program = compile_source(&format!(
        "{}\n{ordinary}",
        include_str!("../../examples/battery_ok.bow")
    ))
    .unwrap();
    let report = verify_report(&program);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.functions_proven, 1);
    assert_eq!(report.calls_checked, 1);
    for (fixture, message) in [
        (
            include_str!("../../examples/battery_fail.bow"),
            "precondition cannot be established",
        ),
        (
            include_str!("../../examples/battery_sequence_fail.bow"),
            "precondition cannot be established",
        ),
        (
            include_str!("../../examples/battery_body_fail.bow"),
            "postcondition cannot be established",
        ),
    ] {
        let report = verify_report(&compile_source(&format!("{fixture}\n{ordinary}")).unwrap());
        assert!(report.diagnostics[0].message.contains(message));
    }
    let report = verify_report(&compile(IDENTITY));
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.functions_proven, 0);
    assert_eq!(report.calls_checked, 0);
    assert!(report.final_values.is_empty());
}
