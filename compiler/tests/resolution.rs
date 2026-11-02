use bowfin_compiler::{
    compile_source, hir::*, parse_source, resolve, verify::verify_report, FrontendError,
};

const SOURCE: &str = "\
let mut first = Battery { charge: 80 };
consume(&mut first, 50);
let mut second = Other { charge: 70 };
drain(&mut second, 20);
record Battery { charge: Percent }
record Other { charge: Percent }
type Unused = range 0..100;
type Percent = range 0..100;
verified fn consume(battery: &mut Battery, amount: Percent)
 requires amount <= battery.charge
 ensures battery.charge == old(battery.charge) - amount
 { battery.charge -= amount; }
verified function drain(battery: &mutable Other, amount: Percent)
 requires amount <= battery.charge
 ensures battery.charge == old(battery.charge) - amount
 { battery.charge -= amount; }
";

#[test]
fn declarations_and_all_function_references_have_canonical_identities() {
    let program = compile_source(SOURCE).unwrap();
    let percent = program.ranges()[1].id;
    assert_ne!(program.ranges()[0].id, percent);
    for (record, function) in program
        .records()
        .iter()
        .zip(program.functions().iter().map(|f| f.as_verified().unwrap()))
    {
        let field = program.field(record.fields[0]);
        assert_eq!(field.record, record.id);
        assert_eq!(field.range_type(), Some(percent));
        assert_eq!(function.state_type, record.id);
        let state = program.parameter(function.state_param);
        assert_eq!(state.function, function.id);
        assert_eq!(state.ty, ParameterType::MutableRecord(record.id));
        let amount = program.parameter(function.amount_param.parameter);
        assert_eq!(amount.function, function.id);
        assert_eq!(amount.ty, ParameterType::Range(percent));
        assert_eq!(function.amount_param.ty, percent);
        let ExprKind::Binary {
            left: requires_amount,
            right: requires_state,
            ..
        } = &function.requires.kind
        else {
            panic!("expected comparison")
        };
        let ExprKind::Binary {
            left: target,
            right: subtraction,
            ..
        } = &function.ensures.kind
        else {
            panic!("expected equality")
        };
        let ExprKind::Binary {
            left: old,
            right: ensures_amount,
            ..
        } = &subtraction.kind
        else {
            panic!("expected subtraction")
        };
        for parameter in [
            requires_amount.as_ref(),
            ensures_amount.as_ref(),
            &function.body[0].operand,
        ] {
            assert_eq!(parameter.kind, ExprKind::Parameter(amount.id));
            assert_eq!(parameter.ty, ExprType::Range(percent));
        }
        for expression in [requires_state.as_ref(), target.as_ref(), old.as_ref()] {
            let access = match &expression.kind {
                ExprKind::FieldAccess(access) | ExprKind::OldField(access) => access,
                _ => panic!("expected field"),
            };
            assert_eq!(access.parameter, state.id);
            assert_eq!(access.field, field.id);
            assert_eq!(expression.ty, ExprType::Range(percent));
            assert_eq!(access.ty, percent);
            assert!(expression.span.end > expression.span.start);
        }
        assert_eq!(function.body[0].target.parameter, state.id);
        assert_eq!(function.body[0].target.field, field.id);
        assert_eq!(
            ExprType::Range(function.body[0].target.ty),
            function.body[0].operand.ty
        );
        assert_eq!(function.requires.ty, ExprType::Bool);
        assert_eq!(function.ensures.ty, ExprType::Bool);
    }
    assert_ne!(
        program.records()[0].fields[0],
        program.records()[1].fields[0]
    );
    assert_ne!(
        program.functions()[0].as_verified().unwrap().state_param,
        program.functions()[1].as_verified().unwrap().state_param
    );
    assert_ne!(
        program.functions()[0]
            .as_verified()
            .unwrap()
            .amount_param
            .parameter,
        program.functions()[1]
            .as_verified()
            .unwrap()
            .amount_param
            .parameter
    );
}

#[test]
fn ordered_construction_and_calls_resolve_to_the_correct_entities() {
    let program = compile_source(SOURCE).unwrap();
    for index in 0..2 {
        let binding = &program.bindings()[index];
        assert_eq!(binding.record_type, program.records()[index].id);
        assert_eq!(binding.field, program.records()[index].fields[0]);
        assert_eq!(
            program.statements()[index * 2],
            Statement::Binding(binding.id)
        );
        let Statement::Call(call) = &program.statements()[index * 2 + 1] else {
            panic!("expected call")
        };
        assert_eq!(
            call.function,
            program.functions()[index].as_verified().unwrap().id
        );
        assert_eq!(call.binding, binding.id);
    }
    let result = verify_report(&program);
    assert!(result.diagnostics.is_empty());
    assert_eq!(result.functions_proven, 2);
    assert_eq!(result.calls_checked, 2);
    assert_eq!(result.final_values["first"], 30);
    assert_eq!(result.final_values["second"], 50);
}

#[test]
fn equal_bounds_do_not_create_equal_range_identities() {
    let program = compile_source("type First = range 0..100; type Second = range 0..100;").unwrap();
    assert_ne!(program.ranges()[0].id, program.ranges()[1].id);
    assert_eq!(program.ranges()[0].min, program.ranges()[1].min);
    assert_eq!(program.ranges()[0].max, program.ranges()[1].max);
    let source = SOURCE.replace("amount: Percent", "amount: Unused");
    let Err(FrontendError::Type(errors)) = compile_source(&source) else {
        panic!("expected nominal error")
    };
    assert!(errors
        .iter()
        .all(|error| error.message
            == "incompatible named range types for <=: `Unused` and `Percent`"));
}

#[test]
fn parsing_preserves_unresolved_precondition_names_and_spans() {
    let source = SOURCE.replace(
        "requires amount <= battery.charge",
        "requires missing <= other.charge",
    );
    let ast = parse_source(&source).unwrap();
    let requires = &ast.functions[0].as_verified().unwrap().requires;
    let bowfin_compiler::ast::ExprKind::Binary { left, right, .. } = &requires.kind else {
        panic!("expected comparison")
    };
    assert_eq!(
        left.kind,
        bowfin_compiler::ast::ExprKind::Name("missing".into())
    );
    let bowfin_compiler::ast::ExprKind::FieldAccess(access) = &right.kind else {
        panic!("expected field")
    };
    assert_eq!(access.binding, "other");
    assert_eq!(access.field, "charge");
    assert!(access.span.end > access.span.start);
    let errors = resolve::resolve(&ast).unwrap_err();
    assert!(errors[0].message.contains("unknown parameter `missing`"));
    assert_eq!(errors[0].span, left.span);
    let state_only = SOURCE.replace(
        "requires amount <= battery.charge",
        "requires amount <= other.charge",
    );
    let Err(FrontendError::Resolve(errors)) = compile_source(&state_only) else {
        panic!("expected resolution error")
    };
    assert!(errors[0]
        .message
        .contains("invalid field reference `other.charge`"));
}

#[test]
fn later_bindings_and_shadowing_are_not_resolved() {
    for suffix in [
        "consume(&mut later, 0); let mut later = Battery { charge: 0 };",
        "let later = Battery { charge: 0 }; let later = Battery { charge: 0 };",
    ] {
        let Err(FrontendError::Resolve(errors)) = compile_source(&format!("{SOURCE}{suffix}"))
        else {
            panic!("expected resolution error")
        };
        assert!(
            errors[0].message.contains("unknown binding")
                || errors[0].message.contains("duplicate declaration")
        );
    }
}

#[test]
fn existing_separate_function_type_and_binding_scopes_are_preserved() {
    let source = SOURCE
        .replace("consume", "Percent")
        .replace("first", "Percent");
    let program = compile_source(&source).unwrap();
    assert_eq!(program.ranges()[1].name, "Percent");
    assert_eq!(
        program.functions()[0].as_verified().unwrap().name,
        "Percent"
    );
    assert_eq!(program.bindings()[0].name, "Percent");
    assert!(verify_report(&program).diagnostics.is_empty());
}

#[test]
fn public_collection_views_and_id_lookups_expose_the_same_entities() {
    let program = compile_source(SOURCE).unwrap();
    let ranges: &[RangeType] = program.ranges();
    let records: &[Record] = program.records();
    let fields: &[Field] = program.fields();
    let functions: &[Function] = program.functions();
    let parameters: &[Parameter] = program.parameters();
    let bindings: &[RecordBinding] = program.bindings();
    let statements: &[Statement] = program.statements();
    assert_eq!(ranges.len(), 2);
    assert_eq!(records.len(), 2);
    assert_eq!(fields.len(), 2);
    assert_eq!(functions.len(), 2);
    assert_eq!(parameters.len(), 4);
    assert_eq!(bindings.len(), 2);
    assert_eq!(statements.len(), 4);
    for range in ranges {
        assert_eq!(program.range(range.id), range);
    }
    for record in records {
        assert_eq!(program.record(record.id), record);
    }
    for field in fields {
        assert_eq!(program.field(field.id), field);
    }
    for function in functions {
        assert_eq!(program.function(function.id()), function);
    }
    for parameter in parameters {
        assert_eq!(program.parameter(parameter.id), parameter);
    }
    for binding in bindings {
        assert_eq!(program.binding(binding.id), binding);
    }
}
