use bowfin_compiler::{
    ast, compile_source,
    diagnostics::Diagnostic,
    hir::{
        self, BorrowKind, ExprKind, ExprType, ParameterType, Place, ReferentType, ValueStatement,
        ValueType,
    },
    parse_source,
    verify::verify_report,
    FrontendError,
};

const DECLS: &str = "type Percent = range 0..100; type OtherPercent = range 0..100;
record Ticket { value: Percent } record Voucher { value: Percent }
fn inspect(value: &Ticket) -> bool { return true; }
fn exclusive(value: &mut Ticket) -> bool { return true; }
fn consume(value: Ticket) -> Ticket { return value; }";
fn source(body: &str) -> String {
    format!("{DECLS}\n{body}")
}
fn valid(body: &str) -> hir::Program {
    compile_source(&source(body)).unwrap()
}
fn ownership_error(body: &str) -> Diagnostic {
    let Err(FrontendError::Ownership(mut errors)) = compile_source(&source(body)) else {
        panic!("expected ownership error: {body}")
    };
    assert_eq!(errors.len(), 1);
    errors.remove(0)
}
fn type_error(body: &str) -> Diagnostic {
    let Err(FrontendError::Type(mut errors)) = compile_source(&source(body)) else {
        panic!("expected type error: {body}")
    };
    assert_eq!(errors.len(), 1);
    errors.remove(0)
}
fn example(body: &str) -> String {
    format!("fn example(ticket: Ticket) -> Ticket {{ {body} }}")
}

#[test]
fn shared_direct_and_stored_borrows_do_not_move_the_owner() {
    valid("fn example(ticket: Ticket) -> bool { return inspect(&ticket); }");
    valid(&example(
        "let view = &ticket; let predecessor_ok = inspect(view); return consume(ticket);",
    ));
}
#[test]
fn overlapping_shared_loans_and_copied_handles_are_valid() {
    valid(&example("let first = &ticket; let second = &ticket; let copy = first; let a = inspect(first); let b = inspect(second); let c = inspect(copy); return ticket;"));
}
#[test]
fn shared_handle_repeated_calls_are_copy() {
    valid(&example(
        "let view = &ticket; let first = inspect(view); let second = inspect(view); return ticket;",
    ));
}
#[test]
fn mutable_owned_local_allows_sequential_direct_exclusive_calls() {
    valid(&example("let mut owned = ticket; let first = exclusive(&mut owned); let second = exclusive(&mut owned); return owned;"));
}
#[test]
fn stored_exclusive_reference_moves_into_call_then_owner_recovers() {
    valid(&example("let mut owned = ticket; let access = &mut owned; let predecessor_ok = exclusive(access); return owned;"));
}
#[test]
fn mutable_reference_transfer_preserves_the_exclusive_loan() {
    valid(&example("let mut owned = ticket; let first = &mut owned; let second = first; let predecessor_ok = exclusive(second); return owned;"));
    let error = ownership_error(&example("let mut owned = ticket; let first = &mut owned; let second = first; let moved = owned; let predecessor_ok = exclusive(second); return moved;"));
    assert!(error.message.contains("exclusively borrowed"));
}
#[test]
fn unused_shared_and_mutable_handles_end_at_the_binding() {
    valid(&example("let view = &ticket; return ticket;"));
    valid(&example(
        "let mut owned = ticket; let access = &mut owned; return owned;",
    ));
    valid(&example(
        "let mut owned = ticket; let access = &mut owned; let next = access; return owned;",
    ));
}
#[test]
fn last_shared_use_permits_later_exclusive_borrow() {
    valid(&example("let mut owned = ticket; let view = &owned; let predecessor_ok = inspect(view); let access = &mut owned; let predecessor_result = exclusive(access); return owned;"));
}
#[test]
fn future_shared_use_blocks_owner_move_at_the_move_site() {
    let body =
        example("let view = &ticket;\nlet moved = ticket;\nlet predecessor_ok = inspect(view); return moved;");
    let error = ownership_error(&body);
    assert_eq!(
        error.message,
        "cannot move `ticket` while it is shared-borrowed"
    );
    assert_eq!(&source(&body)[error.span.start..error.span.end], "ticket");
    assert!(error.known[0].contains("borrow began at line 6"));
    assert!(error.conclusion.contains("ownership checking stopped"));
}
#[test]
fn loan_survives_the_last_use_of_only_one_shared_alias() {
    for body in [
        "let view = &ticket; let first = view; let second = view; let predecessor_ok = inspect(first); let moved = ticket; let later = inspect(second); return moved;",
        "let view = &ticket; let first = view; let second = first; let predecessor_ok = inspect(view); let moved = ticket; let later = inspect(second); return moved;",
    ] { assert!(ownership_error(&example(body)).message.contains("shared-borrowed")); }
    valid(&example("let view = &ticket; let first = view; let second = first; let a = inspect(view); let b = inspect(second); return ticket;"));
}
#[test]
fn mutable_borrow_requires_a_mutable_owned_local() {
    for body in [
        "return exclusive(&mut ticket);",
        "let owned = ticket; return exclusive(&mut owned);",
    ] {
        let error = ownership_error(&format!("fn bad(ticket: Ticket) -> bool {{ {body} }}"));
        assert!(error
            .message
            .contains("cannot mutably borrow immutable value"));
        assert!(error.required.contains("mutable owned local"));
    }
}
#[test]
fn shared_borrow_accepts_immutable_and_mutable_owned_locals() {
    for mutability in ["", "mut "] {
        valid(&example(&format!(
            "let {mutability}owned = ticket; let predecessor_ok = inspect(&owned); return owned;"
        )));
    }
}
#[test]
fn overlapping_shared_and_exclusive_loans_conflict_in_both_orders() {
    let error = ownership_error(&example("let mut owned = ticket; let view = &owned; let access = &mut owned; let predecessor_ok = inspect(view); return owned;"));
    assert!(error
        .message
        .contains("exclusive borrow of `owned` while shared borrow is active"));
    let error = ownership_error(&example("let mut owned = ticket; let access = &mut owned; let view = &owned; let predecessor_ok = exclusive(access); return owned;"));
    assert!(error
        .message
        .contains("shared borrow of `owned` while exclusive borrow is active"));
    let error = ownership_error(&example("let mut owned = ticket; let first = &mut owned; let second = &mut owned; let predecessor_ok = exclusive(first); return owned;"));
    assert!(error.message.contains("second exclusive borrow"));
}
#[test]
fn moved_reference_remains_moved_even_after_its_loan_ends() {
    let error = ownership_error(&example("let mut owned = ticket; let access = &mut owned; let predecessor_ok = exclusive(access); let recovered = owned; let bad = exclusive(access); return recovered;"));
    assert_eq!(error.message, "use of moved value `access`");
    assert!(error.known[0].contains("into by-value parameter `value`"));
    let error = ownership_error(&example("let mut owned = ticket; let first = &mut owned; let second = first; let bad = exclusive(first); return owned;"));
    assert_eq!(error.message, "use of moved value `first`");
}
#[test]
fn both_kinds_of_borrow_reject_a_moved_owner() {
    for borrow in ["&ticket", "&mut owned"] {
        let body = if borrow == "&ticket" {
            "let moved = ticket; let view = &ticket; return moved;"
        } else {
            "let mut owned = ticket; let moved = owned; let access = &mut owned; return moved;"
        };
        assert!(ownership_error(&example(body))
            .message
            .contains("use of moved value"));
    }
}
#[test]
fn copy_owners_allow_shared_reads_but_not_exclusive_reads() {
    for ty in ["bool", "Percent"] {
        let decl = format!("fn read(value: &{ty}) -> bool {{ return true; }} fn write(value: &mut {ty}) -> bool {{ return true; }}");
        valid(&format!("{decl} fn good(value: {ty}) -> {ty} {{ let view = &value; let copy = value; let predecessor_ok = read(view); return copy; }}"));
        let error = ownership_error(&format!("{decl} fn bad(value: {ty}) -> {ty} {{ let mut owned = value; let access = &mut owned; let copy = owned; let predecessor_ok = write(access); return copy; }}"));
        assert_eq!(
            error.message,
            "cannot use `owned` while it is exclusively borrowed"
        );
        valid(&format!("{decl} fn good(value: {ty}) -> {ty} {{ let mut owned = value; let predecessor_ok = write(&mut owned); return owned; }}"));
    }
}
#[test]
fn shared_parameters_are_copy_and_mutable_parameters_move() {
    valid("fn relay(value: &Ticket) -> bool { let first = inspect(value); return inspect(value); } fn relay_mut(value: &mut Ticket) -> bool { let next = value; return exclusive(next); }");
    let error = ownership_error("fn bad(value: &mut Ticket) -> bool { let first = exclusive(value); return exclusive(value); }");
    assert_eq!(error.message, "use of moved value `value`");
}
#[test]
fn two_shared_direct_arguments_may_overlap() {
    valid("fn compare(first: &Ticket, second: &Ticket) -> bool { return true; } fn example(ticket: Ticket) -> bool { return compare(&ticket, &ticket); }");
    valid("fn compare(first: &Ticket, second: &Ticket) -> bool { return true; } fn example(ticket: Ticket) -> bool { let view = &ticket; return compare(view, view); }");
}
#[test]
fn direct_call_arguments_hold_all_conflicting_loans_until_completion() {
    for (types, arguments, fragment) in [
        (
            "&mut Ticket, second: &mut Ticket",
            "&mut owned, &mut owned",
            "second exclusive",
        ),
        (
            "&Ticket, second: &mut Ticket",
            "&owned, &mut owned",
            "while shared borrow is active",
        ),
        (
            "&mut Ticket, second: &Ticket",
            "&mut owned, &owned",
            "while exclusive borrow is active",
        ),
        (
            "&Ticket, second: Ticket",
            "&owned, owned",
            "shared-borrowed",
        ),
        (
            "&mut Ticket, second: Ticket",
            "&mut owned, owned",
            "exclusively borrowed",
        ),
    ] {
        let error = ownership_error(&format!("fn pair(first: {types}) -> bool {{ return true; }} fn bad(ticket: Ticket) -> bool {{ let mut owned = ticket; return pair({arguments}); }}"));
        assert!(error.message.contains(fragment), "{}", error.message);
    }
}
#[test]
fn final_handle_argument_use_stays_call_held() {
    for (ty, borrow, argument, fragment) in [
        ("&Ticket", "&owned", "owned", "shared-borrowed"),
        ("&mut Ticket", "&mut owned", "owned", "exclusively borrowed"),
    ] {
        let error = ownership_error(&format!("fn pair(first: {ty}, second: Ticket) -> bool {{ return true; }} fn bad(ticket: Ticket) -> bool {{ let mut owned = ticket; let access = {borrow}; return pair(access, {argument}); }}"));
        assert!(error.message.contains(fragment));
    }
    let error = ownership_error("fn pair(first: &mut Ticket, second: &mut Ticket) -> bool { return true; } fn bad(ticket: Ticket) -> bool { let mut owned = ticket; let access = &mut owned; return pair(access, &mut owned); }");
    assert!(error.message.contains("second exclusive"));
}
#[test]
fn outer_argument_loans_survive_nested_call_completion() {
    for first in ["&owned", "view"] {
        let error = ownership_error(&format!("fn pair(first: &Ticket, second: Ticket) -> bool {{ return true; }} fn bad(ticket: Ticket) -> bool {{ let mut owned = ticket; let view = &owned; return pair({first}, consume(owned)); }}"));
        assert!(error.message.contains("shared-borrowed"));
    }
    let error = ownership_error("fn pair(first: &mut Ticket, second: bool) -> bool { return true; } fn bad(ticket: Ticket) -> bool { let mut owned = ticket; return pair(&mut owned, exclusive(&mut owned)); }");
    assert!(error.message.contains("second exclusive"));
    let error = ownership_error("fn pair(first: &mut Ticket, second: bool) -> bool { return true; } fn bad(ticket: Ticket) -> bool { let mut owned = ticket; let access = &mut owned; return pair(access, exclusive(&mut owned)); }");
    assert!(error.message.contains("second exclusive"));
    // Copies of one provenance can be held by both outer and inner calls.
    let error = ownership_error("fn three(first: &Ticket, second: bool, third: Ticket) -> bool { return true; } fn bad(ticket: Ticket) -> bool { let view = &ticket; return three(view, inspect(view), ticket); }");
    assert!(error.message.contains("shared-borrowed"));
    // Completing an unrelated inner call must not release the outer loan either.
    let error = ownership_error("fn three(first: &Ticket, second: bool, third: Ticket) -> bool { return true; } fn bad(ticket: Ticket) -> bool { return three(&ticket, inspect(&ticket), ticket); }");
    assert!(error.message.contains("shared-borrowed"));
}
#[test]
fn completed_inner_call_releases_only_its_own_ephemeral_loans() {
    valid("fn pair(first: bool, second: Ticket) -> Ticket { return second; } fn example(ticket: Ticket) -> Ticket { let mut owned = ticket; return pair(exclusive(&mut owned), owned); }");
    valid("fn pair(first: bool, second: bool) -> bool { return second; } fn example(ticket: Ticket) -> Ticket { let mut owned = ticket; let predecessor_ok = pair(exclusive(&mut owned), exclusive(&mut owned)); return owned; }");
    valid("fn pair(first: bool, second: Ticket) -> Ticket { return second; } fn example(ticket: Ticket) -> Ticket { let view = &ticket; return pair(inspect(view), ticket); }");
}
#[test]
fn exact_reference_capability_and_owned_boundaries_fail_in_type_phase() {
    for body in [
        "fn bad(ticket: Ticket) -> bool { let mut owned = ticket; return inspect(&mut owned); }",
        "fn bad(ticket: Ticket) -> bool { return exclusive(&ticket); }",
        "fn bad(ticket: Ticket) -> Ticket { return consume(&ticket); }",
        "fn bad(ticket: Ticket) -> bool { return inspect(ticket); }",
        "fn bad(ticket: Ticket) -> Ticket { let view = &ticket; return view; }",
    ] {
        assert!(type_error(body).message.contains("type mismatch"));
    }
    let error = type_error(
        "fn bad(ticket: Ticket) -> bool { let mut owned = ticket; return inspect(&mut owned); }",
    );
    assert!(error.required.contains("&Ticket") && error.required.contains("&mut Ticket"));
}
#[test]
fn distinct_nominal_referents_remain_distinct_inside_both_reference_kinds() {
    for (a, b) in [("Ticket", "Voucher"), ("Percent", "OtherPercent")] {
        for prefix in ["&", "&mut "] {
            let error = type_error(&format!("fn sink(value: {prefix}{a}) -> bool {{ return true; }} fn bad(value: {prefix}{b}) -> bool {{ return sink(value); }}"));
            assert!(error.required.contains(a) && error.required.contains(b));
        }
    }
}
#[test]
fn nested_reference_signatures_and_reference_returns_fail_before_ownership() {
    for ty in [
        "&&Ticket",
        "&mut &Ticket",
        "&&mut Ticket",
        "&mut &mut Ticket",
    ] {
        assert!(
            type_error(&format!("fn bad(value: {ty}) -> bool {{ return true; }}"))
                .message
                .contains("nested reference")
        );
    }
    for ty in ["&Ticket", "&mut Ticket"] {
        assert!(
            type_error(&format!("fn bad(value: {ty}) -> {ty} {{ return value; }}"))
                .message
                .contains("reference return")
        );
        // Even forward calls cannot obtain reference-valued call results.
        assert!(type_error(&format!("fn caller(value: {ty}) -> bool {{ let predecessor_result = bad(value); return true; }} fn bad(value: {ty}) -> {ty} {{ return value; }}")).message.contains("reference return"));
    }
}
#[test]
fn borrowing_reference_values_and_mutable_reference_bindings_are_type_errors() {
    for body in [
        "fn bad(value: &Ticket) -> bool { let nested = &value; return true; }",
        "fn bad(value: &mut Ticket) -> bool { let nested = &mut value; return true; }",
        "fn bad(ticket: Ticket) -> bool { let view = &ticket; let nested = &view; return true; }",
        "fn bad(ticket: Ticket) -> bool { let mut owned = ticket; let access = &mut owned; let nested = &mut access; return true; }",
    ] { assert!(type_error(body).message.contains("cannot borrow an existing reference")); }
    for borrow in ["&owned", "&mut owned"] {
        assert!(type_error(&format!("fn bad(ticket: Ticket) -> bool {{ let mut owned = ticket; let mut access = {borrow}; return true; }}")).message.contains("owned non-reference"));
    }
}
#[test]
fn references_do_not_acquire_arithmetic_comparison_or_boolean_operators() {
    for (ty, op) in [
        ("Ticket", "=="),
        ("Percent", "<="),
        ("Percent", "-"),
        ("bool", "&&"),
        ("bool", "||"),
    ] {
        assert!(type_error(&format!(
            "fn bad(first: &{ty}, second: &{ty}) -> bool {{ return first {op} second; }}"
        ))
        .message
        .contains("incompatible operand types"));
    }
    assert!(
        type_error("fn bad(value: &bool) -> bool { return !value; }")
            .message
            .contains("negation requires Bool")
    );
}
#[test]
fn borrow_targets_resolve_in_source_order_without_shadowing() {
    for body in [
        "fn bad(ticket: Ticket) -> bool { let view = &missing; return true; }",
        "fn bad(ticket: Ticket) -> bool { let view = &later; let later = ticket; return true; }",
        "fn bad(ticket: Ticket) -> bool { let view = &view; return true; }",
        "fn bad(ticket: Ticket) -> bool { let view = &ticket; let view = &ticket; return true; }",
    ] {
        assert!(matches!(
            compile_source(&source(body)),
            Err(FrontendError::Resolve(_))
        ));
    }
}
#[test]
fn unsupported_borrow_targets_and_mutation_syntax_remain_rejected() {
    // Preserve the former blanket rejection witness as the newly accepted case.
    valid("fn bad(value: &mut Ticket) -> bool { value.value = 0; return true; }");
    assert!(
        type_error("fn bad(ticket: Ticket) -> bool { let view = *ticket; return true; }")
            .message
            .contains("dereference requires a reference")
    );
    for expression in [
        "&(ticket)",
        "&consume(ticket)",
        "&ticket.value",
        "&true",
        "&5",
    ] {
        assert!(
            parse_source(&source(&format!(
                "fn bad(ticket: Ticket) -> bool {{ let view = {expression}; return true; }}"
            )))
            .is_err(),
            "{expression}"
        );
    }
    for body in [
        "fn bad(mut ticket: Ticket) -> bool { return true; }",
        "fn bad(ticket: Ticket) -> Ticket { let mut owned = ticket; owned = ticket; return owned; }",
        "fn bad(value: &Ticket) -> bool { value.value = 0; return true; }",
        "fn bad(ticket: Ticket) -> bool { inspect(&ticket); return true; }",
    ] { assert!(compile_source(&source(body)).is_err(), "{body}"); }
}
#[test]
fn hir_borrows_reference_signatures_and_local_mutability_are_canonical() {
    let program = valid(&example("let mut owned = ticket; let view = &owned; let predecessor_ok = inspect(view); let access = &mut owned; let predecessor_result = exclusive(access); return owned;"));
    let record = program.records()[0].id;
    let inspect = program.functions()[0].as_ordinary().unwrap();
    assert_eq!(
        program.parameter(inspect.parameters[0]).ty,
        ParameterType::Value(ValueType::SharedRef(ReferentType::Record(record)))
    );
    let exclusive = program.functions()[1].as_ordinary().unwrap();
    assert_eq!(
        program.parameter(exclusive.parameters[0]).ty,
        ParameterType::Value(ValueType::MutableRef(ReferentType::Record(record)))
    );
    let f = program.functions()[3].as_ordinary().unwrap();
    let owned = &program.locals()[0];
    assert!(owned.mutable);
    assert!(!program.locals()[1].mutable);
    for (index, kind, ty) in [
        (
            1,
            BorrowKind::Shared,
            ExprType::SharedRef(ReferentType::Record(record)),
        ),
        (
            3,
            BorrowKind::Mutable,
            ExprType::MutableRef(ReferentType::Record(record)),
        ),
    ] {
        let ValueStatement::Let {
            initializer, local, ..
        } = &f.body[index]
        else {
            panic!("let")
        };
        assert_eq!(
            initializer.kind,
            ExprKind::Borrow {
                kind,
                place: Place::Local(owned.id)
            }
        );
        assert_eq!(initializer.ty, ty);
        assert_eq!(ExprType::from(program.local(*local).ty), ty);
    }
    let program = valid("fn example(ticket: Ticket) -> bool { return inspect(&ticket); }");
    let f = program.functions()[3].as_ordinary().unwrap();
    let ValueStatement::Return { value, .. } = &f.body[0] else {
        panic!("return")
    };
    let ExprKind::Call {
        function,
        arguments,
    } = &value.kind
    else {
        panic!("call")
    };
    assert_eq!(*function, program.functions()[0].id());
    assert_eq!(
        arguments[0].kind,
        ExprKind::Borrow {
            kind: BorrowKind::Shared,
            place: Place::Parameter(f.parameters[0])
        }
    );
}
#[test]
fn mut_and_mutable_aliases_preserve_source_syntax_and_normalize_semantics() {
    let compact = source("fn example(ticket: Ticket) -> Ticket { let mut owned = ticket; let predecessor_ok = exclusive(&mut owned); return owned; }");
    let explicit = compact
        .replace("fn ", "function ")
        .replace("mut ", "mutable ");
    let a = compile_source(&compact).unwrap();
    let b = compile_source(&explicit).unwrap();
    for (a, b) in a.parameters().iter().zip(b.parameters()) {
        assert_eq!(a.ty, b.ty);
    }
    for (a, b) in a.locals().iter().zip(b.locals()) {
        assert_eq!((a.id, a.ty, a.mutable), (b.id, b.ty, b.mutable));
    }
    let ast = parse_source(&explicit).unwrap();
    let ast::FunctionDecl::Ordinary(f) = &ast.functions[1] else {
        panic!("ordinary")
    };
    assert_eq!(f.parameters[0].ty.references, vec![BorrowKind::Mutable]);
    let ast::FunctionDecl::Ordinary(f) = &ast.functions[3] else {
        panic!("ordinary")
    };
    assert!(matches!(
        f.body[0],
        ast::ValueStatement::Let { mutable: true, .. }
    ));
}
#[test]
fn borrowing_does_not_change_verified_proofs_or_execute_ordinary_functions() {
    let ordinary = "fn inspect(value: &Battery) -> bool { return true; } fn example(battery: Battery) -> Battery { let predecessor_ok = inspect(&battery); return battery; }";
    let program = compile_source(&format!(
        "{} {ordinary}",
        include_str!("../../examples/battery_ok.bow")
    ))
    .unwrap();
    let report = verify_report(&program);
    assert!(report.diagnostics.is_empty());
    assert_eq!((report.functions_proven, report.calls_checked), (1, 1));
    let report = verify_report(&valid(&example(
        "let predecessor_ok = inspect(&ticket); return ticket;",
    )));
    assert!(report.diagnostics.is_empty());
    assert_eq!((report.functions_proven, report.calls_checked), (0, 0));
}
