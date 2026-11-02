use bowfin_compiler::{
    ast, compile_source, diagnostics::Diagnostic, hir::*, parse_source, verify::verify_report,
    FrontendError,
};

const RANGE: &str = "type Percent = range 0..100; type Other = range 0..100;";
const IDENTITY: &str = "fn identity(value: Percent) -> Percent { return value; }";

fn compile(text: &str) -> Program {
    compile_source(&format!("{RANGE}\n{text}")).unwrap()
}
fn ordinary(program: &Program, index: usize) -> &ValueFunction {
    program.functions()[index].as_ordinary().unwrap()
}
fn returned(function: &ValueFunction) -> &TypedExpr {
    let ValueStatement::Return { value, .. } = function.body.last().unwrap() else {
        panic!("explicit return expected")
    };
    value
}
fn type_error(text: &str) -> Diagnostic {
    let Err(FrontendError::Type(errors)) = compile_source(&format!("{RANGE}\n{text}")) else {
        panic!("expected type error: {text}")
    };
    errors.into_iter().next().unwrap()
}
fn resolve_error(text: &str) -> Diagnostic {
    let Err(FrontendError::Resolve(errors)) = compile_source(&format!("{RANGE}\n{text}")) else {
        panic!("expected resolution error: {text}")
    };
    errors.into_iter().next().unwrap()
}

#[test]
fn parser_preserves_ordinary_signatures_lets_returns_calls_and_spans() {
    let text = "function wrapped(current: Percent, used: Percent, enabled: bool) -> bool {\n let predecessor_result = remaining(current, identity(used));\n return enabled && predecessor_result == current;\n}";
    let source = format!("{RANGE}\n{text}");
    let parsed = parse_source(&source).unwrap();
    let ast::FunctionDecl::Ordinary(f) = &parsed.functions[0] else {
        panic!("ordinary declaration expected")
    };
    assert_eq!(f.name, "wrapped");
    assert_eq!(f.parameters.len(), 3);
    assert_eq!(f.parameters[0].ty.name, "Percent");
    assert_eq!(f.parameters[2].ty.name, "bool");
    assert_eq!(f.return_type.as_ref().unwrap().name, "bool");
    let ast::ValueStatement::Let {
        name,
        initializer,
        span,
        ..
    } = &f.body[0]
    else {
        panic!("local expected")
    };
    assert_eq!(name, "predecessor_result");
    assert_eq!(
        &source[span.start..span.end],
        "let predecessor_result = remaining(current, identity(used));"
    );
    let ast::ExprKind::Call { callee, arguments } = &initializer.kind else {
        panic!("call expected")
    };
    assert_eq!(callee, "remaining");
    assert!(
        matches!(&arguments[1].kind, ast::ExprKind::Call { callee, .. } if callee == "identity")
    );
    let ast::ValueStatement::Return { value, span } = &f.body[1] else {
        panic!("return expected")
    };
    assert_eq!(
        &source[span.start..span.end],
        "return enabled && predecessor_result == current;"
    );
    assert!(matches!(
        value.kind,
        ast::ExprKind::Binary {
            op: ast::BinaryOp::And,
            ..
        }
    ));
    assert_eq!(&source[f.span.start..f.span.end], text);
}

#[test]
fn keyword_aliases_have_identical_semantics() {
    let compact = compile(IDENTITY);
    let explicit = compile(&IDENTITY.replace("fn identity", "function identity"));
    assert_eq!(
        ordinary(&compact, 0).return_type,
        ordinary(&explicit, 0).return_type
    );
    assert_eq!(
        returned(ordinary(&compact, 0)).kind,
        returned(ordinary(&explicit, 0)).kind
    );
}

#[test]
fn builtin_bool_zero_parameters_and_multiple_parameters_are_typed() {
    let program = compile("fn yes() -> bool { return true; } fn valid(current: Percent, used: Percent, enabled: bool) -> bool { return enabled && used <= current; } fn wrapper() -> bool { return yes() && yes(); }");
    assert!(ordinary(&program, 0).parameters.is_empty());
    assert_eq!(ordinary(&program, 1).parameters.len(), 3);
    for f in program.functions() {
        assert_eq!(f.as_ordinary().unwrap().return_type, Some(ValueType::Bool));
        assert_eq!(returned(f.as_ordinary().unwrap()).ty, ExprType::Bool);
    }
    assert_eq!(
        program.parameter(ordinary(&program, 1).parameters[2]).ty,
        ParameterType::Value(ValueType::Bool)
    );
    for declaration in [
        "type bool = range 0..100;",
        "record bool { field: Percent }",
    ] {
        assert!(matches!(
            parse_source(declaration),
            Err(FrontendError::Parse(_))
        ));
    }
}

#[test]
fn ordinary_value_interfaces_accept_records_and_references_with_exact_results() {
    compile("record R { value: Percent } fn ignore(value: R) -> bool { return true; } fn identity(value: R) -> R { return value; }");
    assert!(
        type_error("record R { value: Percent } fn bad() -> R { return true; }")
            .message
            .contains("return type mismatch")
    );
    assert!(
        resolve_error("fn bad(value: Missing) -> bool { return true; }")
            .message
            .contains("unknown value type")
    );
    compile("fn inspect(value: &Percent) -> bool { return true; } fn exclusive(value: &mut Percent) -> bool { return true; }");
    assert!(parse_source("fn bad() -> () { return true; }").is_err());
}

#[test]
fn explicit_returns_are_required_and_cannot_be_implicit_tails() {
    for body in ["", "let predecessor_result = value;"] {
        assert!(
            type_error(&format!("fn bad(value: Percent) -> Percent {{ {body} }}"))
                .message
                .contains("require `return expression;`")
        );
    }
    for body in [
        "value",
        "value;",
        "value - 1",
        "value - 1;",
        "return value",
        "return value; let later = value;",
        "return value; return value;",
    ] {
        let Err(FrontendError::Parse(error)) =
            parse_source(&format!("fn bad(value: Percent) -> Percent {{ {body} }}"))
        else {
            panic!("must reject {body}")
        };
        assert!(error.message.contains("return"), "{}", error.message);
    }
}

#[test]
fn mutable_copy_assignment_does_not_enable_annotations_or_expression_statements() {
    for spelling in ["mut", "mutable"] {
        compile(&format!(
            "fn predecessor_ok(value: Percent) -> Percent {{ let {spelling} x = value; x = value; return x; }}"
        ));
    }
    for body in [
        "let x: Percent = value; return x;",
        "true; return value;",
        "return if true { value } else { value };",
    ] {
        assert!(
            parse_source(&format!("fn bad(value: Percent) -> Percent {{ {body} }}")).is_err(),
            "{body}"
        );
    }
}

#[test]
fn local_initializers_propagate_range_and_boolean_types_by_ids() {
    let program = compile("fn remaining(current: Percent, used: Percent) -> Percent { let first = current - used; let second = first - 0; return second; } fn can_use(current: Percent, used: Percent) -> bool { let enough = used <= current; return enough; }");
    let range = program.ranges()[0].id;
    let locals: &[Local] = program.locals();
    assert_eq!(locals.len(), 3);
    assert_eq!(locals[0].ty, ValueType::Range(range));
    assert_eq!(locals[1].ty, ValueType::Range(range));
    assert_eq!(locals[2].ty, ValueType::Bool);
    for local in locals {
        assert_eq!(program.local(local.id), local);
        assert!(local.span.end > local.span.start);
    }
    let first = ordinary(&program, 0);
    assert_eq!(locals[0].function, first.id);
    assert_eq!(locals[1].function, first.id);
    let ValueStatement::Let {
        local, initializer, ..
    } = &first.body[0]
    else {
        panic!("let")
    };
    assert_eq!(*local, locals[0].id);
    let ExprKind::Binary { left, right, .. } = &initializer.kind else {
        panic!("subtract")
    };
    assert_eq!(left.kind, ExprKind::Parameter(first.parameters[0]));
    assert_eq!(right.kind, ExprKind::Parameter(first.parameters[1]));
    let ValueStatement::Let { initializer, .. } = &first.body[1] else {
        panic!("let")
    };
    let ExprKind::Binary { left, .. } = &initializer.kind else {
        panic!("subtract")
    };
    assert_eq!(left.kind, ExprKind::Local(locals[0].id));
    assert_eq!(returned(first).kind, ExprKind::Local(locals[1].id));
    assert_eq!(locals[2].function, ordinary(&program, 1).id);
    assert_eq!(
        returned(ordinary(&program, 1)).kind,
        ExprKind::Local(locals[2].id)
    );
}

#[test]
fn function_scope_is_ordered_has_no_self_reference_and_does_not_leak() {
    for text in ["fn bad(value: Percent) -> Percent { return missing; }", "fn bad(value: Percent) -> Percent { let predecessor_result = later; let later = value; return predecessor_result; }", "fn bad(value: Percent) -> Percent { let predecessor_result = predecessor_result - 1; return predecessor_result; }", "fn first(value: Percent) -> Percent { let predecessor_result = value; return predecessor_result; } fn second(other: Percent) -> Percent { return predecessor_result; }", "fn first(value: Percent) -> Percent { return value; } fn second(other: Percent) -> Percent { return value; }"] {
        assert!(resolve_error(text).message.contains("unknown value"), "{text}");
    }
}

#[test]
fn duplicate_parameters_locals_and_shadowing_are_resolution_errors() {
    for text in [
        "fn bad(value: Percent, value: bool) -> bool { return true; }",
        "fn bad(value: Percent) -> Percent { let value = value; return value; }",
        "fn bad(value: Percent) -> Percent { let x = value; let x = value; return x; }",
    ] {
        assert!(resolve_error(text)
            .message
            .contains("duplicate declaration"));
    }
    let program = compile("fn first(value: Percent) -> Percent { let predecessor_result = value; return predecessor_result; } fn second(value: Percent) -> Percent { let predecessor_result = value; return predecessor_result; }");
    assert_ne!(program.locals()[0].id, program.locals()[1].id);
    assert_ne!(
        ordinary(&program, 0).parameters[0],
        ordinary(&program, 1).parameters[0]
    );
}

#[test]
fn forward_calls_and_direct_and_indirect_recursion_resolve() {
    let program = compile("fn first(value: Percent) -> Percent { return second(value); } fn second(value: Percent) -> Percent { return first(value); } fn direct(value: Percent) -> Percent { return direct(value); }");
    for (caller, callee) in [(0, 1), (1, 0), (2, 2)] {
        let ExprKind::Call {
            function,
            arguments,
        } = &returned(ordinary(&program, caller)).kind
        else {
            panic!("call")
        };
        assert_eq!(*function, program.functions()[callee].id());
        assert_eq!(program.function(*function), &program.functions()[callee]);
        assert_eq!(
            arguments[0].kind,
            ExprKind::Parameter(ordinary(&program, caller).parameters[0])
        );
    }
    assert!(verify_report(&program).diagnostics.is_empty());
    assert!(
        resolve_error("fn bad(value: Percent) -> Percent { return missing(value); }")
            .message
            .contains("unknown function `missing`")
    );
}

#[test]
fn nested_calls_binary_calls_and_call_initializers_carry_declared_result_types() {
    let program = compile(&format!("{IDENTITY} fn wrapped(current: Percent, used: Percent) -> Percent {{ let predecessor_result = identity(identity(current - used)); return predecessor_result; }} fn same(value: Percent) -> bool {{ return identity(value) == value; }} fn enabled(value: bool) -> bool {{ return value; }} fn yes() -> bool {{ return enabled(true); }}"));
    let range = program.ranges()[0].id;
    let ValueStatement::Let { initializer, .. } = &ordinary(&program, 1).body[0] else {
        panic!("let")
    };
    assert_eq!(initializer.ty, ExprType::Range(range));
    let ExprKind::Call {
        function,
        arguments,
    } = &initializer.kind
    else {
        panic!("call")
    };
    assert_eq!(*function, ordinary(&program, 0).id);
    assert!(matches!(arguments[0].kind, ExprKind::Call { .. }));
    let ExprKind::Binary { left, .. } = &returned(ordinary(&program, 2)).kind else {
        panic!("equality")
    };
    assert_eq!(left.ty, ExprType::Range(range));
    assert_eq!(returned(ordinary(&program, 4)).ty, ExprType::Bool);
}

#[test]
fn calls_require_exact_arity_and_boolean_or_nominal_argument_types() {
    for args in ["", "value, value"] {
        assert!(type_error(&format!(
            "{IDENTITY} fn bad(value: Percent) -> Percent {{ return identity({args}); }}"
        ))
        .message
        .contains("wrong argument count"));
    }
    for (signature, argument) in [("bool", "value"), ("Percent", "true")] {
        let error = type_error(&format!("fn take(arg: {signature}) -> {signature} {{ return arg; }} fn bad(value: Percent) -> {signature} {{ return take({argument}); }}"));
        assert!(error.message.contains("call argument type mismatch"));
        assert!(error.span.end > error.span.start);
    }
    compile(&format!(
        "{IDENTITY} fn predecessor_ok(value: Percent) -> Percent {{ return identity(value - 5); }}"
    ));
    compile(&format!(
        "{IDENTITY} fn formed() -> Percent {{ return identity(5); }}"
    ));
    assert!(type_error(&format!(
        "{IDENTITY} fn bad() -> Percent {{ return identity(101); }}"
    ))
    .message
    .contains("literal outside named range"));
}

#[test]
fn nominal_call_and_return_mismatches_identify_both_types() {
    for text in ["fn bad(value: Other) -> Percent { return value; }", "fn take(value: Percent) -> Percent { return value; } fn bad(value: Other) -> Percent { return take(value); }"] {
        let error = type_error(text);
        assert!(error.message.contains("distinct named ranges"));
        let rendered = error.render("nominal.bow", &format!("{RANGE}\n{text}"));
        assert!(rendered.contains("Percent") && rendered.contains("Other"));
        assert!(!rendered.contains("RangeTypeId"));
    }
    let program = compile(IDENTITY);
    assert_ne!(program.ranges()[0].id, program.ranges()[1].id);
}

#[test]
fn literal_locals_remain_untyped_while_range_return_forms() {
    assert!(
        type_error("fn bad(value: Percent) -> Percent { let five = 5; return value; }")
            .message
            .contains("cannot materialize as a local value")
    );
    for text in [
        "fn bad() -> bool { return 1; }",
        "fn bad(value: Percent) -> bool { return value; }",
        "fn bad() -> Percent { return true; }",
    ] {
        assert!(type_error(text).message.contains("return type mismatch"));
    }
    compile("fn predecessor_ok() -> Percent { return 0; }");
    compile("fn predecessor_ok(value: Percent) -> Percent { let predecessor_result = value - 5; return predecessor_result; }");
}

#[test]
fn ordinary_old_and_additional_operators_remain_rejected() {
    assert!(
        resolve_error("fn bad(value: Percent) -> Percent { return old(value.field); }")
            .message
            .contains("only in ensures")
    );
    for expr in [
        "value + 1",
        "value * 2",
        "value / 2",
        "value < 1",
        "value != 1",
    ] {
        assert!(compile_source(&format!(
            "{RANGE} fn bad(value: Percent) -> Percent {{ return {expr}; }}"
        ))
        .is_err());
    }
    assert!(type_error("fn bad() -> Percent { return 5 - 1; }")
        .message
        .contains("incompatible operand types"));
}

#[test]
fn mixed_declarations_share_one_source_ordered_function_table_and_namespace() {
    let battery = include_str!("../../examples/battery_ok.bow");
    let program = compile_source(&format!("fn before() -> bool {{ return true; }}\n{battery}\nfn after() -> bool {{ return before(); }}")).unwrap();
    assert_eq!(program.functions().len(), 3);
    assert_eq!(program.functions()[0].name(), "before");
    assert_eq!(program.functions()[1].name(), "consume");
    assert_eq!(program.functions()[2].name(), "after");
    assert!(program.functions()[0].id() < program.functions()[1].id());
    assert!(program.functions()[1].id() < program.functions()[2].id());
    let Statement::Call(call) = program.statements().last().unwrap() else {
        panic!("harness call")
    };
    assert_eq!(call.function, program.functions()[1].id());
    assert_eq!(verify_report(&program).functions_proven, 1);
    for source in [
        format!("fn consume() -> bool {{ return true; }} {battery}"),
        format!("{battery} fn consume() -> bool {{ return true; }}"),
        "fn duplicate() -> bool { return true; } fn duplicate() -> bool { return false; }".into(),
    ] {
        let Err(FrontendError::Resolve(errors)) = compile_source(&source) else {
            panic!("duplicate expected")
        };
        assert!(errors[0].message.contains("duplicate declaration"));
    }
}

#[test]
fn cross_kind_calls_are_rejected_before_verification() {
    let battery = include_str!("../../examples/battery_ok.bow");
    let Err(FrontendError::Resolve(errors)) = compile_source(&format!(
        "{battery} fn bad(value: Percent) -> Percent {{ return consume(value); }}"
    )) else {
        panic!("cross-kind call")
    };
    assert!(errors[0]
        .message
        .contains("only between ordinary value functions"));
    let ordinary = "fn helper(value: Percent) -> Percent { return value; }";
    for verified in [
        battery.replace(
            "requires amount <= battery.charge",
            "requires helper(amount) <= battery.charge",
        ),
        battery.replace("old(battery.charge) - amount", "helper(amount)"),
        battery.replace(
            "battery.charge -= amount",
            "battery.charge -= helper(amount)",
        ),
    ] {
        let Err(FrontendError::Resolve(errors)) =
            compile_source(&format!("{ordinary}\n{verified}"))
        else {
            panic!("verified expression call")
        };
        assert!(errors[0]
            .message
            .contains("only between ordinary value functions"));
    }
    let source = battery.replace(
        "consume(&mutable battery, 50);",
        "helper(&mutable battery, 50);",
    );
    // Use the harness spelling from the fixture, independent of keyword aliases.
    let source = source.replace("consume(&mut battery, 50);", "helper(&mut battery, 50);");
    let Err(FrontendError::Resolve(errors)) = compile_source(&format!("{ordinary}\n{source}"))
    else {
        panic!("ordinary harness call")
    };
    assert!(errors[0]
        .message
        .contains("top-level prototype calls require a verified function"));
}

#[test]
fn ordinary_range_results_are_typed_but_not_executed_or_proven() {
    let text = "fn subtract(a: Percent, b: Percent) -> Percent { return a - b; }";
    let program = compile(text);
    let report = verify_report(&program);
    assert!(report.diagnostics.is_empty());
    assert_eq!(report.functions_proven, 0);
    assert_eq!(report.calls_checked, 0);
    assert!(report.final_values.is_empty());
    assert_eq!(
        returned(ordinary(&program, 0)).ty,
        ExprType::Range(program.ranges()[0].id)
    );
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
        let report = verify_report(&compile_source(&format!("{fixture}\n{text}")).unwrap());
        assert!(report.diagnostics[0].message.contains(message));
    }
}
