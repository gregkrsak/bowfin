//! Expression and value-flow type legality. All semantic references are resolved IDs.
use crate::{
    diagnostics::Diagnostic,
    hir::{
        self, BinaryOp, ExprKind, ExprType, ParameterType, Program, Subtract, TypedExpr, ValueType,
        VerifiedFunction,
    },
    resolve::{
        ResolvedExpr, ResolvedExprKind, ResolvedFunction, ResolvedParameterType, ResolvedProgram,
        ResolvedValueStatement, ResolvedValueType,
    },
};
type TypeResult<T> = Result<T, Box<Diagnostic>>;

/// Type-check declarations, expressions and structured acyclic value flow; publish canonical HIR.
pub fn check(resolved: ResolvedProgram) -> Result<Program, Vec<Diagnostic>> {
    let mut program = resolved.declarations;
    for parameter in resolved.parameters {
        let ty = match parameter.ty {
            ResolvedParameterType::MutableRecord(id) => ParameterType::MutableRecord(id),
            ResolvedParameterType::Range(id) => ParameterType::Range(id),
            ResolvedParameterType::Value(ty) => {
                ParameterType::Value(value_type(ty, parameter.span).map_err(|e| vec![*e])?)
            }
        };
        program.parameters.push(hir::Parameter {
            id: parameter.id,
            function: parameter.function,
            name: parameter.name,
            ty,
            span: parameter.span,
        });
    }
    // Signature legality is checked before any body or ownership analysis, including
    // forward calls. No ordinary call can produce an escaping reference result.
    for function in &resolved.functions {
        if let ResolvedFunction::Ordinary(f) = function {
            let ty = f
                .return_type
                .map(|ty| value_type(ty, f.span))
                .transpose()
                .map_err(|e| vec![*e])?;
            if matches!(ty, Some(ValueType::SharedRef(_) | ValueType::MutableRef(_))) {
                return Err(vec![Diagnostic::semantic(
                    f.span,
                    "reference return types are not supported",
                    "escaping references and lifetime relationships remain unresolved",
                )]);
            }
        }
    }
    let mut diagnostics = Vec::new();
    for function in &resolved.functions {
        let result = match function {
            ResolvedFunction::Verified(f) => {
                verified_function(&program, &resolved.functions, f).map(hir::Function::Verified)
            }
            ResolvedFunction::Ordinary(f) => {
                value_function(&mut program, &resolved.functions, f).map(hir::Function::Ordinary)
            }
        };
        match result {
            Ok(function) => program.functions.push(function),
            Err(error) => {
                diagnostics.push(*error);
                // Failed local typing leaves no publishable local table; do not inspect it.
                if matches!(function, ResolvedFunction::Ordinary(_)) {
                    return Err(diagnostics);
                }
            }
        }
    }
    if diagnostics.is_empty() {
        program
            .validate_record_expressions()
            .map_err(|e| vec![*e])?;
        Ok(program)
    } else {
        Err(diagnostics)
    }
}
fn verified_function(
    program: &Program,
    functions: &[ResolvedFunction],
    f: &crate::resolve::ResolvedVerifiedFunction,
) -> TypeResult<VerifiedFunction> {
    let requires = expression(program, functions, &f.requires)?;
    contract(&requires, "requires")?;
    let ensures = expression(program, functions, &f.ensures)?;
    contract(&ensures, "ensures")?;
    let mut body = Vec::new();
    for statement in &f.body {
        let operand = expression(program, functions, &statement.operand)?;
        let target = ExprType::Range(statement.target.ty);
        if operand.ty != target && operand.ty != ExprType::IntegerLiteral {
            return Err(incompatible(
                program,
                statement.span,
                "subtract-assignment",
                target,
                operand.ty,
            ));
        }
        body.push(Subtract {
            target: statement.target.clone(),
            operand,
            span: statement.span,
        });
    }
    Ok(VerifiedFunction {
        id: f.id,
        name: f.name.clone(),
        span: f.span,
        state_param: f.state_param,
        state_type: f.state_type,
        amount_param: f.amount_param,
        requires,
        ensures,
        body,
    })
}
// Targeted transfers terminate a lexical block but while always retains its
// false exit. Resolver has already rejected block suffixes after terminal flow.
fn ordinary_callee(
    functions: &[ResolvedFunction],
    function: hir::FunctionId,
    span: crate::lexer::Span,
) -> TypeResult<&crate::resolve::ResolvedValueFunction> {
    let ResolvedFunction::Ordinary(callee) = &functions[function.0] else {
        return Err(Diagnostic::semantic(
            span,
            "ordinary call requires an ordinary function",
            "mixed-assurance calls are not supported",
        )
        .into());
    };
    Ok(callee)
}
fn call_arguments(
    program: &Program,
    functions: &[ResolvedFunction],
    callee: &crate::resolve::ResolvedValueFunction,
    arguments: &[ResolvedExpr],
    span: crate::lexer::Span,
) -> TypeResult<Vec<TypedExpr>> {
    value_arguments(
        program,
        functions,
        arguments,
        callee.parameters.len(),
        &callee.name,
        span,
        |index| {
            let ParameterType::Value(expected) = program.parameter(callee.parameters[index]).ty
            else {
                return Err(Diagnostic::semantic(
                    span,
                    "invalid ordinary parameter type",
                    "ordinary signatures require bool, named ranges, or records",
                )
                .into());
            };
            Ok(expected)
        },
    )
}
fn value_arguments(
    program: &Program,
    functions: &[ResolvedFunction],
    arguments: &[ResolvedExpr],
    arity: usize,
    name: &str,
    span: crate::lexer::Span,
    mut expected: impl FnMut(usize) -> TypeResult<ValueType>,
) -> TypeResult<Vec<TypedExpr>> {
    if arguments.len() != arity {
        return Err(Diagnostic::semantic(
            span,
            format!("wrong argument count for `{name}`"),
            format!("expected {} arguments, found {}", arity, arguments.len()),
        )
        .into());
    }
    let mut typed = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        let expected = expected(index)?;
        let argument = check_value(program, functions, argument, expected, "call argument")?;
        typed.push(argument);
    }
    Ok(typed)
}
fn falls_through(body: &[ResolvedValueStatement]) -> bool {
    match body.last() {
        Some(ResolvedValueStatement::Match { arms, .. }) => {
            arms.iter().any(|arm| falls_through(&arm.body))
        }
        Some(
            ResolvedValueStatement::Return { .. }
            | ResolvedValueStatement::ReturnNoValue { .. }
            | ResolvedValueStatement::Break { .. }
            | ResolvedValueStatement::Continue { .. },
        ) => false,
        Some(ResolvedValueStatement::If {
            then_body,
            else_body,
            ..
        }) => falls_through(then_body) || falls_through(else_body),
        _ => true,
    }
}
fn value_function(
    program: &mut Program,
    functions: &[ResolvedFunction],
    f: &crate::resolve::ResolvedValueFunction,
) -> TypeResult<hir::ValueFunction> {
    let return_type = f.return_type.map(|ty| value_type(ty, f.span)).transpose()?;
    if return_type.is_some() && falls_through(&f.body) {
        return Err(Diagnostic::semantic(
            f.span,
            "function may reach its closing brace without returning; value-returning Bowfin functions require `return expression;`",
            "every structural function-completing path requires an explicit correctly typed return; while retains its false exit even for constant conditions; implicit block-tail returns are not supported",
        )
        .into());
    }
    let body = value_statements(program, functions, &f.body, return_type)?;
    Ok(hir::ValueFunction {
        id: f.id,
        name: f.name.clone(),
        parameters: f.parameters.clone(),
        return_type,
        effects: f.effects.clone(),
        body,
        span: f.span,
    })
}
fn value_statements(
    program: &mut Program,
    functions: &[ResolvedFunction],
    statements: &[ResolvedValueStatement],
    return_type: Option<ValueType>,
) -> TypeResult<Vec<hir::ValueStatement>> {
    let mut body = Vec::new();
    for statement in statements {
        let statement = match statement {
            ResolvedValueStatement::Match {
                scrutinee,
                arms,
                span,
            } => {
                let scrutinee = expression(program, functions, scrutinee)?;
                let ExprType::Result(result) = scrutinee.ty else {
                    return Err(Diagnostic::semantic(
                        scrutinee.span,
                        "match requires a Result value",
                        "only exhaustive two-variant statement Result matching is admitted",
                    )
                    .into());
                };
                let mut typed_arms = Vec::new();
                for arm in arms {
                    let ty = program.result_payload(result, arm.tag).ok_or_else(|| {
                        Box::new(Diagnostic::semantic(
                            arm.pattern_span,
                            "invalid Result payload identity",
                            "canonical Result type required",
                        ))
                    })?;
                    if arm.binding.0 != program.locals.len() {
                        return Err(Diagnostic::semantic(
                            arm.pattern_span,
                            "invalid Result arm binding identity",
                            "fresh lexical LocalIds must follow source order",
                        )
                        .into());
                    }
                    program.locals.push(hir::Local {
                        id: arm.binding,
                        function: arm.function,
                        name: arm.name.clone(),
                        mutable: false,
                        ty,
                        span: arm.pattern_span,
                    });
                    let body = value_statements(program, functions, &arm.body, return_type)?;
                    typed_arms.push(hir::MatchArm {
                        tag: arm.tag,
                        binding: arm.binding,
                        pattern_span: arm.pattern_span,
                        body,
                        span: arm.span,
                    });
                }
                hir::ValueStatement::Match {
                    scrutinee,
                    arms: typed_arms,
                    span: *span,
                }
            }
            ResolvedValueStatement::Break { span } => hir::ValueStatement::Break { span: *span },
            ResolvedValueStatement::Continue { span } => {
                hir::ValueStatement::Continue { span: *span }
            }
            ResolvedValueStatement::While {
                condition,
                body,
                span,
            } => {
                let condition = expression(program, functions, condition)?;
                if condition.ty != ExprType::Bool {
                    return Err(Diagnostic::semantic(
                        condition.span,
                        "while condition must have type Bool",
                        format!(
                            "found {}; there is no truthiness conversion",
                            display_type(program, condition.ty)
                        ),
                    )
                    .into());
                }
                let body = value_statements(program, functions, body, return_type)?;
                hir::ValueStatement::While {
                    condition,
                    body,
                    span: *span,
                }
            }
            ResolvedValueStatement::If {
                condition,
                then_body,
                else_body,
                span,
            } => {
                let condition = expression(program, functions, condition)?;
                if condition.ty != ExprType::Bool {
                    return Err(Diagnostic::semantic(
                        condition.span,
                        "if condition must have type Bool",
                        format!(
                            "found {}; there is no truthiness conversion",
                            display_type(program, condition.ty)
                        ),
                    )
                    .into());
                }
                let then_body = value_statements(program, functions, then_body, return_type)?;
                let else_body = value_statements(program, functions, else_body, return_type)?;
                hir::ValueStatement::If {
                    condition,
                    then_body,
                    else_body,
                    span: *span,
                }
            }
            ResolvedValueStatement::Let {
                id,
                function,
                name,
                mutable,
                initializer,
                propagation,
                span,
            } => {
                let initializer = expression(program, functions, initializer)?;
                let propagation = if let Some(syntax) = propagation {
                    let ExprType::Result(inner) = initializer.ty else {
                        return Err(Diagnostic::semantic(
                            initializer.span,
                            "or return requires an independently typed Result initializer",
                            "propagation supplies no constructor context",
                        )
                        .into());
                    };
                    let Some(ValueType::Result(outer)) = return_type else {
                        return Err(Diagnostic::semantic(
                            syntax.phrase,
                            "or return requires an ordinary Result-returning Value function",
                            "NoValue and other return types cannot propagate",
                        )
                        .into());
                    };
                    let failure = program
                        .result_payload(inner, hir::ResultTag::Error)
                        .expect("resolved Result");
                    if Some(failure) != program.result_payload(outer, hir::ResultTag::Error) {
                        return Err(Diagnostic::semantic(syntax.phrase, "or return failure payload type mismatch", format!("inner Result #{} and enclosing Result #{} require exact failure identity: {} versus {}",inner.0,outer.0,display_type(program,ExprType::from(failure)),program.result_payload(outer,hir::ResultTag::Error).map(|ty|display_type(program,ExprType::from(ty))).unwrap_or_else(||"invalid canonical failure identity".into()))).into());
                    }
                    Some(hir::Propagation {
                        span: *span,
                        inner,
                        outer,
                        failure,
                        syntax: *syntax,
                    })
                } else {
                    None
                };
                let ty = if let Some(propagation) = propagation {
                    program
                        .result_payload(propagation.inner, hir::ResultTag::Ok)
                        .expect("resolved Result")
                } else {
                    match initializer.ty {
                        ExprType::Result(id) => ValueType::Result(id),
                        ExprType::Bool => ValueType::Bool,
                        ExprType::Range(id) => ValueType::Range(id),
                        ExprType::Record(id) => ValueType::Record(id),
                        ExprType::Resource(id) => ValueType::Resource(id),
                        ExprType::SharedRef(ty) => ValueType::SharedRef(ty),
                        ExprType::MutableRef(ty) => ValueType::MutableRef(ty),
                        ExprType::IntegerLiteral => {
                            return Err(Diagnostic::semantic(
                            initializer.span,
                            "integer literal cannot materialize as a local value",
                            "general integer/literal materialization is unresolved; use a concrete bool, named range, or record expression",
                        ).into());
                        }
                    }
                };
                if *mutable && matches!(ty, ValueType::SharedRef(_) | ValueType::MutableRef(_)) {
                    return Err(Diagnostic::semantic(*span, "mutable local bindings require an owned non-reference value", "let mut marks owned locals as eligible for exclusive borrowing; reference bindings remain immutable").into());
                }
                program.locals.push(hir::Local {
                    mutable: *mutable,
                    id: *id,
                    function: *function,
                    name: name.clone(),
                    ty,
                    span: *span,
                });
                hir::ValueStatement::Let {
                    local: *id,
                    initializer,
                    propagation,
                    span: *span,
                }
            }
            ResolvedValueStatement::FieldAssign {
                owner,
                field_name,
                owner_span,
                field_span,
                value,
                span,
            } => {
                let ty = match owner {
                    hir::Place::Parameter(id) => match program.parameter(*id).ty {
                        ParameterType::Value(ty) => ty,
                        _ => unreachable!("resolved ordinary field target"),
                    },
                    hir::Place::Local(id) => program.local(*id).ty,
                };
                let (record, owned) = match ty {
                    ValueType::MutableRef(hir::ReferentType::Record(id)) => (id, None),
                    ValueType::SharedRef(hir::ReferentType::Record(_)) => return Err(Diagnostic::semantic(*owner_span,
                        "field assignment through a reference requires exact &mut Record authority",
                        "shared record references cannot write; no implicit reborrow or capability conversion is supported").into()),
                    ValueType::SharedRef(_) | ValueType::MutableRef(_) => return Err(Diagnostic::semantic(*owner_span,
                        "reference field assignment requires a record referent",
                        format!("found {}; only exact &mut Record references may assign a declared Copy field", display_type(program, ty.into()))).into()),
                    _ => {
                        let local = match owner {
                            hir::Place::Local(id) => *id,
                            hir::Place::Parameter(id) => return Err(Diagnostic::semantic(*owner_span, format!("cannot assign field of parameter `{}`", program.parameter(*id).name), "field assignment requires a directly named mutable owned record local; mutable owned parameters are unsupported").into()),
                        };
                        let ValueType::Record(record) = ty else {
                            return Err(Diagnostic::semantic(*owner_span, "field assignment requires an owned record local", format!("found {}", display_type(program, ty.into()))).into());
                        };
                        let target = program.local(local);
                        if !target.mutable {
                            return Err(Diagnostic::semantic(*owner_span,
                                format!("cannot assign field of immutable local `{}`", target.name),
                                "field assignment requires a let mut / let mutable owned record local").into());
                        }
                        (record, Some(local))
                    }
                };
                let field = program
                    .record(record)
                    .fields
                    .iter()
                    .copied()
                    .find(|id| program.field(*id).name == *field_name)
                    .ok_or_else(|| {
                        Box::new(Diagnostic::semantic(
                            *field_span,
                            format!(
                                "unknown field `{field_name}` for record `{}`",
                                program.record(record).name
                            ),
                            "assign a declared Copy field of the actual root record",
                        ))
                    })?;
                let value = check_value(
                    program,
                    functions,
                    value,
                    program.field(field).ty,
                    "field assignment RHS",
                )?;
                if let Some(local) = owned {
                    hir::ValueStatement::CopyFieldAssign {
                        owner: local,
                        record,
                        field,
                        owner_span: *owner_span,
                        field_span: *field_span,
                        value,
                        span: *span,
                    }
                } else {
                    hir::ValueStatement::CopyReferenceFieldAssign {
                        reference: *owner,
                        record,
                        field,
                        reference_span: *owner_span,
                        field_span: *field_span,
                        value,
                        span: *span,
                    }
                }
            }
            ResolvedValueStatement::Assign {
                target,
                value,
                span,
            } => {
                let local = match target {
                    hir::Place::Local(local) => *local,
                    hir::Place::Parameter(parameter) => {
                        return Err(Diagnostic::semantic(*span, format!("cannot assign parameter `{}`", program.parameter(*parameter).name), "direct reassignment requires an ordinary local; mutable owned parameters are unsupported").into());
                    }
                };
                let target = program.local(local);
                if matches!(target.ty, ValueType::Result(_)) {
                    return Err(Diagnostic::semantic(*span, "Result replacement is not admitted", "match, return, or transfer the complete Result; no outcome may be overwritten").into());
                }
                if matches!(
                    target.ty,
                    ValueType::SharedRef(_) | ValueType::MutableRef(_)
                ) {
                    return Err(Diagnostic::semantic(*span, format!("cannot reassign reference local `{}`", target.name), "reference provenance replacement is unresolved; direct reassignment requires a mutable owned Copy local").into());
                }
                if !target.mutable {
                    return Err(Diagnostic::semantic(
                        *span,
                        format!("cannot assign immutable local `{}`", target.name),
                        "direct reassignment requires a let mut / let mutable local",
                    )
                    .into());
                }
                let value = check_value(program, functions, value, target.ty, "assignment RHS")?;
                hir::ValueStatement::Assign {
                    local,
                    value,
                    span: *span,
                }
            }
            ResolvedValueStatement::DerefAssign {
                reference,
                value,
                span,
            } => {
                let (kind, referent) = reference_type(program, *reference, *span)?;
                if kind != hir::BorrowKind::Mutable {
                    return Err(Diagnostic::semantic(*span, "write-through requires an exclusive mutable reference", "shared references cannot write; no implicit capability conversion is supported").into());
                }
                let value = check_value(
                    program,
                    functions,
                    value,
                    referent.value_type(),
                    "write-through RHS",
                )?;
                hir::ValueStatement::DerefAssign {
                    reference: *reference,
                    value,
                    span: *span,
                }
            }
            ResolvedValueStatement::ReturnNoValue { span } => {
                if return_type.is_some() {
                    return Err(Diagnostic::semantic(*span, "bare return is permitted only in a no-value function", "value-returning functions require return expression; with the exact declared type").into());
                }
                hir::ValueStatement::ReturnNoValue { span: *span }
            }
            ResolvedValueStatement::CallNoValue {
                function,
                arguments,
                span,
            } => {
                let callee = ordinary_callee(functions, *function, *span)?;
                if callee.return_type.is_some() {
                    return Err(Diagnostic::semantic(*span, format!("value-returning function `{}` cannot be called as a statement", callee.name), "values cannot be silently discarded; no-value ordinary calls alone are statements").into());
                }
                let arguments = call_arguments(program, functions, callee, arguments, *span)?;
                hir::ValueStatement::CallNoValue {
                    function: *function,
                    arguments,
                    span: *span,
                }
            }
            ResolvedValueStatement::Return { value, span } => {
                let Some(return_type) = return_type else {
                    return Err(Diagnostic::semantic(*span, "value return is not permitted in a no-value function", "an omitted result declaration means NoValue, not inference; use return; or normal function completion").into());
                };
                let value = check_value(program, functions, value, return_type, "return")?;
                hir::ValueStatement::Return { value, span: *span }
            }
        };
        body.push(statement);
    }
    Ok(body)
}
fn value_type(ty: ResolvedValueType, span: crate::lexer::Span) -> TypeResult<ValueType> {
    Ok(match ty {
        ResolvedValueType::Result(id) => ValueType::Result(id),
        ResolvedValueType::Bool => ValueType::Bool,
        ResolvedValueType::Range(id) => ValueType::Range(id),
        ResolvedValueType::Record(id) => ValueType::Record(id),
        ResolvedValueType::Resource(id) => ValueType::Resource(id),
        ResolvedValueType::SharedRef(ty) => ValueType::SharedRef(ty),
        ResolvedValueType::MutableRef(ty) => ValueType::MutableRef(ty),
        ResolvedValueType::NestedReference => return Err(Diagnostic::semantic(span, "nested reference types are not supported", "reference referents must be bool, a named range, or a record; reborrowing remains unsupported").into()),
    })
}
// A bounded value check, deliberately not recursive expected-type synthesis.
fn check_value(
    program: &Program,
    functions: &[ResolvedFunction],
    expression: &ResolvedExpr,
    expected: ValueType,
    boundary: &str,
) -> TypeResult<TypedExpr> {
    if let ResolvedExprKind::ResultConstruct {
        tag,
        payload,
        variant_span,
    } = &expression.kind
    {
        let ValueType::Result(id) = expected else {
            return Err(Diagnostic::semantic(expression.span, "cannot infer complete Result type", "the constructor requires an independently established complete Result type, including both payload types; intrinsic recognition never falls back to ordinary lookup").into());
        };
        let payload_type = program.result_payload(id, *tag).ok_or_else(|| {
            Box::new(Diagnostic::semantic(
                expression.span,
                "invalid Result type",
                "canonical Result identity required",
            ))
        })?;
        let boundary = match tag {
            hir::ResultTag::Ok => "Result success payload",
            hir::ResultTag::Error => "Result error payload",
        };
        let payload = if matches!(payload.kind, ResolvedExprKind::ResultConstruct { .. }) {
            check_value(program, functions, payload, payload_type, boundary)?
        } else {
            // BD-037 does not form a named-range literal at a Result boundary.
            let value = self::expression(program, functions, payload)?;
            exact_value(program, &value, payload_type, boundary)?;
            value
        };
        return Ok(TypedExpr {
            kind: ExprKind::ResultConstruct {
                result: id,
                tag: *tag,
                payload: Box::new(payload),
                variant_span: *variant_span,
            },
            ty: ExprType::Result(id),
            span: expression.span,
        });
    }
    if let (ValueType::Range(id), ResolvedExprKind::IntegerLiteral(value)) =
        (expected, &expression.kind)
    {
        let range = program.range(id);
        if *value < range.min || *value > range.max {
            return Err(Diagnostic::semantic(
                expression.span,
                format!("{boundary} literal outside named range `{}`", range.name),
                format!(
                    "value {value} is outside inclusive bounds {}..{} of range `{}`",
                    range.min, range.max, range.name
                ),
            )
            .into());
        }
        return Ok(TypedExpr {
            kind: ExprKind::FormedRangeLiteral(*value),
            ty: ExprType::Range(id),
            span: expression.span,
        });
    }
    let expression = self::expression(program, functions, expression)?;
    exact_value(program, &expression, expected, boundary)?;
    Ok(expression)
}
fn exact_value(
    program: &Program,
    expression: &TypedExpr,
    expected: ValueType,
    boundary: &str,
) -> TypeResult<()> {
    let expected = ExprType::from(expected);
    if expression.ty == expected {
        return Ok(());
    }
    let nominal =
        matches!((expected, expression.ty), (ExprType::Range(a), ExprType::Range(b)) if a != b);
    let message = if nominal {
        format!("{boundary} type mismatch between distinct named ranges")
    } else if matches!((expected, expression.ty), (ExprType::Record(a), ExprType::Record(b)) if a != b)
    {
        format!("{boundary} type mismatch between distinct named records")
    } else {
        format!("{boundary} type mismatch")
    };
    Err(Diagnostic::semantic(
        expression.span,
        message,
        format!(
            "expected {}, found {}; value boundaries require exact concrete types; \
             only direct literals at independently established named-range boundaries may form range values",
            display_type(program, expected),
            display_type(program, expression.ty),
        ),
    )
    .into())
}
fn contract(expression: &TypedExpr, name: &str) -> TypeResult<()> {
    if expression.ty != ExprType::Bool {
        return Err(Diagnostic::semantic(
            expression.span,
            format!("{name} expression must have type Bool"),
            "contracts must be Boolean expressions",
        )
        .into());
    }
    Ok(())
}
fn display_type(program: &Program, ty: ExprType) -> String {
    match ty {
        ExprType::Result(id) => format!("Result type #{}", id.0),
        ExprType::Bool => "Bool".into(),
        ExprType::IntegerLiteral => "IntegerLiteral".into(),
        ExprType::Range(id) => format!("range `{}`", program.range(id).name),
        ExprType::Record(id) => format!("record `{}`", program.record(id).name),
        ExprType::Resource(_) => "resource `Sensor`".into(),
        ExprType::SharedRef(ty) => format!("&{}", display_referent(program, ty)),
        ExprType::MutableRef(ty) => format!("&mut {}", display_referent(program, ty)),
    }
}
fn display_referent(program: &Program, ty: hir::ReferentType) -> String {
    match ty {
        hir::ReferentType::Bool => "bool".into(),
        hir::ReferentType::Range(id) => program.range(id).name.clone(),
        hir::ReferentType::Record(id) => program.record(id).name.clone(),
        hir::ReferentType::Resource(_) => "Sensor".into(),
    }
}
fn incompatible(
    program: &Program,
    span: crate::lexer::Span,
    operator: &str,
    left: ExprType,
    right: ExprType,
) -> Box<Diagnostic> {
    let message = match (left, right) {
        (ExprType::Range(a), ExprType::Range(b)) if a != b => format!(
            "incompatible named range types for {operator}: `{}` and `{}`",
            program.range(a).name,
            program.range(b).name
        ),
        _ => format!("incompatible operand types for `{operator}`"),
    };
    let rule = match operator {
        "&&" | "||" => "both operands must be Bool",
        "==" => "use two Bool operands, the same named range type, or compatible range/literal operands",
        "<=" => "use the same named range type or range/literal operands",
        "subtraction" => "the left operand must be a named range; the right must be that same range or an integer literal",
        "subtract-assignment" => "the right operand must be the target's named range or an integer literal",
        _ => "operands must satisfy the documented operator type rule",
    };
    let nominal = matches!((left, right), (ExprType::Range(a), ExprType::Range(b)) if a != b);
    let restriction = if nominal {
        " This prototype does not permit implicit arithmetic or comparison between distinct named range types."
    } else {
        ""
    };
    Diagnostic::semantic(
        span,
        message,
        format!(
            "found {} and {}; {rule}.{restriction}",
            display_type(program, left),
            display_type(program, right)
        ),
    )
    .into()
}
fn binary_type(
    program: &Program,
    span: crate::lexer::Span,
    op: BinaryOp,
    left: ExprType,
    right: ExprType,
) -> TypeResult<ExprType> {
    use ExprType::*;
    match (op, left, right) {
        (BinaryOp::And | BinaryOp::Or, Bool, Bool) | (BinaryOp::Equal, Bool, Bool) => Ok(Bool),
        (BinaryOp::Equal | BinaryOp::LessEqual, Range(a), Range(b)) if a == b => Ok(Bool),
        (BinaryOp::Equal | BinaryOp::LessEqual, Range(_), IntegerLiteral)
        | (BinaryOp::Equal | BinaryOp::LessEqual, IntegerLiteral, Range(_))
        | (BinaryOp::Equal | BinaryOp::LessEqual, IntegerLiteral, IntegerLiteral) => Ok(Bool),
        (BinaryOp::Subtract, Range(a), Range(b)) if a == b => Ok(Range(a)),
        (BinaryOp::Subtract, Range(a), IntegerLiteral) => Ok(Range(a)),
        _ => Err(incompatible(
            program,
            span,
            if op == BinaryOp::Subtract {
                "subtraction"
            } else {
                op.spelling()
            },
            left,
            right,
        )),
    }
}
fn reference_type(
    program: &Program,
    reference: hir::Place,
    span: crate::lexer::Span,
) -> TypeResult<(hir::BorrowKind, hir::ReferentType)> {
    let ty = match reference {
        hir::Place::Parameter(id) => match program.parameter(id).ty {
            ParameterType::Value(ty) => ty,
            _ => unreachable!("resolver excludes verified dereference"),
        },
        hir::Place::Local(id) => program.local(id).ty,
    };
    match ty {
        ValueType::SharedRef(ty) => Ok((hir::BorrowKind::Shared, ty)),
        ValueType::MutableRef(ty) => Ok((hir::BorrowKind::Mutable, ty)),
        _ => Err(Diagnostic::semantic(
            span,
            "dereference requires a reference value",
            format!(
                "found {}; explicit dereference requires &T or &mut T",
                display_type(program, ty.into())
            ),
        )
        .into()),
    }
}
fn expression(
    program: &Program,
    functions: &[ResolvedFunction],
    input: &ResolvedExpr,
) -> TypeResult<TypedExpr> {
    let span = input.span;
    let (kind, ty) = match &input.kind {
        ResolvedExprKind::ResultConstruct { .. } => return Err(Diagnostic::semantic(span, "cannot infer complete Result type", "including both success and failure types: use an exact ordinary Result return or parameter boundary").into()),
        ResolvedExprKind::IfValue {
            condition,
            then_value,
            else_value,
        } => {
            let condition = expression(program, functions, condition)?;
            if condition.ty != ExprType::Bool {
                return Err(Diagnostic::semantic(
                    condition.span,
                    "if condition must have type Bool",
                    format!("found {}", display_type(program, condition.ty)),
                )
                .into());
            }
            let then_value = expression(program, functions, then_value)?;
            let else_value = expression(program, functions, else_value)?;
            for value in [&then_value, &else_value] {
                match value.ty {
                    ExprType::Resource(_) => return Err(Diagnostic::semantic(value.span, "conditional initializer cannot produce an opaque Resource", "Sensor conditional initialization is excluded").into()),
                    ExprType::Result(_) => return Err(Diagnostic::semantic(value.span, "conditional initializer cannot produce a Result value", "BD-025 permits only bool, named-range, and record conditional results").into()),
                    ExprType::SharedRef(_) | ExprType::MutableRef(_) => {
                        return Err(Diagnostic::semantic(
                            value.span,
                            "conditional initializer cannot produce a reference value",
                            "reference-valued conditional results remain unsupported",
                        )
                        .into())
                    }
                    ExprType::IntegerLiteral => {
                        return Err(Diagnostic::semantic(
                            value.span,
                            "conditional branch must have a concrete owned type",
                            "integer literals do not materialize a range type implicitly",
                        )
                        .into())
                    }
                    ExprType::Bool | ExprType::Range(_) | ExprType::Record(_) => {}
                }
            }
            if then_value.ty != else_value.ty {
                return Err(Diagnostic::semantic(
                    span,
                    "conditional branches must have the same type",
                    format!(
                        "found {} and {}; nominal identities must match exactly",
                        display_type(program, then_value.ty),
                        display_type(program, else_value.ty)
                    ),
                )
                .into());
            }
            let ty = then_value.ty;
            (
                ExprKind::IfValue {
                    condition: Box::new(condition),
                    then_value: Box::new(then_value),
                    else_value: Box::new(else_value),
                },
                ty,
            )
        }
        ResolvedExprKind::RecordConstruct { record, fields } => {
            let mut typed = Vec::new();
            for entry in fields {
                let value = check_value(
                    program,
                    functions,
                    &entry.value,
                    program.field(entry.field).ty,
                    "record field initializer",
                )?;
                typed.push(hir::RecordFieldInit {
                    field: entry.field,
                    value,
                    span: entry.span,
                });
            }
            (
                ExprKind::RecordConstruct {
                    record: *record,
                    fields: typed,
                },
                ExprType::Record(*record),
            )
        }
        ResolvedExprKind::CopyFieldRead {
            owner,
            field_name,
            field_span,
        } => {
            let ty = match owner {
                hir::Place::Parameter(id) => match program.parameter(*id).ty {
                    ParameterType::Value(ty) => ty,
                    _ => unreachable!("ordinary field owner"),
                },
                hir::Place::Local(id) => program.local(*id).ty,
            };
            let record = match ty {
                ValueType::Record(id) => id,
                ValueType::SharedRef(hir::ReferentType::Record(id)) | ValueType::MutableRef(hir::ReferentType::Record(id)) => id,
                ValueType::SharedRef(_) | ValueType::MutableRef(_) => return Err(Diagnostic::semantic(*field_span, "reference field read requires a record referent", format!("the reference has type {}; its referent is not a record with field `{field_name}`", display_type(program, ty.into()))).into()),
                _ => return Err(Diagnostic::semantic(span, "field read requires an owned record parameter or local", format!("found {}", display_type(program, ty.into()))).into()),
            };
            let field = program
                .record(record)
                .fields
                .iter()
                .copied()
                .find(|id| program.field(*id).name == *field_name)
                .ok_or_else(|| {
                    Box::new(Diagnostic::semantic(
                        *field_span,
                        format!(
                            "unknown field `{field_name}` for record `{}`",
                            program.record(record).name
                        ),
                        "read a declared field of the named record",
                    ))
                })?;
            let declaration = program.field(field);
            (
                if matches!(ty, ValueType::Record(_)) {
                    ExprKind::CopyFieldRead {
                        owner: *owner,
                        record,
                        field,
                    }
                } else {
                    ExprKind::CopyReferenceFieldRead {
                        reference: *owner,
                        record,
                        field,
                    }
                },
                ExprType::from(declaration.ty),
            )
        }
        ResolvedExprKind::BoolLiteral(value) => (ExprKind::BoolLiteral(*value), ExprType::Bool),
        ResolvedExprKind::IntegerLiteral(value) => {
            (ExprKind::IntegerLiteral(*value), ExprType::IntegerLiteral)
        }
        ResolvedExprKind::Parameter(id) => {
            let parameter = program.parameter(*id);
            let ty = match parameter.ty {
                ParameterType::Range(id) => ExprType::Range(id),
                ParameterType::Value(ty) => ty.into(),
                ParameterType::MutableRecord(_) => {
                    return Err(Diagnostic::semantic(
                        span,
                        format!(
                            "record parameter `{}` is not a numeric expression",
                            parameter.name
                        ),
                        "use its declared constrained field",
                    )
                    .into())
                }
            };
            (ExprKind::Parameter(*id), ty)
        }
        ResolvedExprKind::Local(id) => (ExprKind::Local(*id), program.local(*id).ty.into()),
        ResolvedExprKind::Deref { reference } => {
            let (_, referent) = reference_type(program, *reference, span)?;
            if matches!(referent, hir::ReferentType::Resource(_)) {
                return Err(Diagnostic::semantic(span, "opaque resources cannot be read through dereference", "shared observation cannot copy or move the Sensor owner").into());
            }
            (
                ExprKind::Deref {
                    reference: *reference,
                },
                referent.value_type().into(),
            )
        }
        ResolvedExprKind::Borrow { kind, place } => {
            let ty = match place {
                hir::Place::Parameter(id) => match program.parameter(*id).ty {
                    ParameterType::Value(ty) => ty,
                    _ => {
                        return Err(Diagnostic::semantic(
                            span,
                            "ordinary borrowing requires an ordinary owned place",
                            "verified state references use the existing restricted prototype",
                        )
                        .into())
                    }
                },
                hir::Place::Local(id) => program.local(*id).ty,
            };
            let referent = match ty {
                    ValueType::Result(_) => return Err(Diagnostic::semantic(span, "borrowing a Result is excluded in this phase", "match, return, or transfer the owned Result; Result is not a reference referent").into()),
                    ValueType::Bool => hir::ReferentType::Bool,
                ValueType::Range(id) => hir::ReferentType::Range(id),
                ValueType::Record(id) => hir::ReferentType::Record(id),
                ValueType::Resource(id) => {
                    if *kind == hir::BorrowKind::Mutable {
                        return Err(Diagnostic::semantic(span, "mutable Sensor references are excluded", "opaque resources permit shared observation only").into());
                    }
                    hir::ReferentType::Resource(id)
                }
                ValueType::SharedRef(_) | ValueType::MutableRef(_) => return Err(Diagnostic::semantic(span, "cannot borrow an existing reference value", "nested references and reborrowing are not supported; borrow an owned non-reference parameter or local").into()),
            };
            let ty = match kind {
                hir::BorrowKind::Shared => ExprType::SharedRef(referent),
                hir::BorrowKind::Mutable => ExprType::MutableRef(referent),
            };
            (
                ExprKind::Borrow {
                    kind: *kind,
                    place: *place,
                },
                ty,
            )
        }
        ResolvedExprKind::Call {
            function,
            arguments,
        } => {
            let callee = ordinary_callee(functions, *function, span)?;
            let Some(return_type) = callee.return_type else {
                return Err(Diagnostic::semantic(span, format!("no-value function `{}` cannot be used as an expression", callee.name), "NoValue calls produce no expression value; invoke them only as standalone call statements").into());
            };
            let typed = call_arguments(program, functions, callee, arguments, span)?;
            let ty = value_type(return_type, callee.span)?.into();
            (
                ExprKind::Call {
                    function: *function,
                    arguments: typed,
                },
                ty,
            )
        }
        ResolvedExprKind::TrustedCall { operation, arguments } => {
            let catalog = program.trusted.as_ref().expect("resolved selected catalog");
            let op = catalog.operation(*operation).expect("resolved trusted operation");
            let name = op.source_name.expect("resolved source-callable operation");
            let typed = value_arguments(program, functions, arguments, usize::from(op.parameter.is_some()), name, span, |_| Ok(op.parameter.expect("checked trusted arity").value_type()))?;
            (ExprKind::TrustedCall { operation: *operation, arguments: typed }, ExprType::Result(op.result.expect("source-callable trusted Result")))
        }
        ResolvedExprKind::FieldAccess(access) => {
            let ty = ExprType::Range(access.ty);
            (ExprKind::FieldAccess(access.clone()), ty)
        }
        ResolvedExprKind::OldField(access) => {
            let ty = ExprType::Range(access.ty);
            (ExprKind::OldField(access.clone()), ty)
        }
        ResolvedExprKind::Unary { op, operand } => {
            let operand = expression(program, functions, operand)?;
            if operand.ty != ExprType::Bool {
                return Err(Diagnostic::semantic(
                    span,
                    "Boolean negation requires Bool",
                    format!("found {}", display_type(program, operand.ty)),
                )
                .into());
            }
            (
                ExprKind::Unary {
                    op: *op,
                    operand: Box::new(operand),
                },
                ExprType::Bool,
            )
        }
        ResolvedExprKind::Binary { op, left, right } => {
            let left = expression(program, functions, left)?;
            let right = expression(program, functions, right)?;
            let ty = binary_type(program, span, *op, left.ty, right.ty)?;
            (
                ExprKind::Binary {
                    op: *op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
                ty,
            )
        }
    };
    Ok(TypedExpr { kind, ty, span })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{parse_source, resolve, verify::verify_report};

    #[test]
    fn conditional_nominal_types_remain_distinct_when_display_names_collide() {
        for declarations in [
            "type First = range 0..100; type Second = range 0..100;",
            "type Percent = range 0..100; record First { value: Percent } record Second { value: Percent }",
        ] {
            let source = format!("{declarations} fn f(flag: bool, a: First, b: Second) -> First {{ let chosen = if flag {{ a }} else {{ b }}; return chosen; }}");
            let ast = crate::parse_source(&source).unwrap();
            let mut resolved = crate::resolve::resolve(&ast).unwrap();
            for range in &mut resolved.declarations.ranges { range.name = "display".into(); }
            for record in &mut resolved.declarations.records { record.name = "display".into(); }
            let errors = check(resolved).unwrap_err();
            assert_eq!(errors[0].message, "conditional branches must have the same type");
            assert!(errors[0].required.contains("nominal identities must match exactly"));
        }
    }

    #[test]
    fn expression_typing_uses_ids_when_diagnostic_names_change() {
        let ast = parse_source(include_str!("../../examples/battery_ok.bow")).unwrap();
        let mut resolved = resolve::resolve(&ast).unwrap();
        let range = resolved.declarations.ranges[0].id;
        for declaration in &mut resolved.declarations.ranges {
            declaration.name = "display".into();
        }
        for parameter in &mut resolved.parameters {
            parameter.name = "display".into();
        }
        for field in &mut resolved.declarations.fields {
            field.name = "display".into();
        }
        let program = check(resolved).unwrap();
        assert_eq!(
            program.functions()[0].as_verified().unwrap().body[0]
                .operand
                .ty,
            ExprType::Range(range)
        );
        assert!(verify_report(&program).diagnostics.is_empty());

        for expression in [
            "amount <= battery.charge",
            "battery.charge == amount",
            "(battery.charge - amount) == 0",
        ] {
            let text = format!(
                "type Other = range 0..100;\n{}",
                include_str!("../../examples/battery.bow")
            )
            .replace("amount: Percent", "amount: Other")
            .replace(
                "requires amount <= battery.charge",
                &format!("requires {expression}"),
            );
            let mut resolved = resolve::resolve(&parse_source(&text).unwrap()).unwrap();
            assert_ne!(
                resolved.declarations.ranges[0].id,
                resolved.declarations.ranges[1].id
            );
            for range in &mut resolved.declarations.ranges {
                range.name = "display".into();
            }
            let errors = check(resolved).unwrap_err();
            assert!(errors[0].message.contains("incompatible named range types"));
            assert!(errors[0].message.contains("`display` and `display`"));
        }
    }
    #[test]
    fn value_flow_typing_uses_function_parameter_and_local_ids_after_metadata_renaming() {
        let source = "type Percent = range 0..100; type Other = range 0..100; fn identity(value: Percent) -> Percent { let predecessor_result = value; return predecessor_result; } fn wrapped(value: Percent) -> Percent { let first = identity(value); let second = first - 0; return second; }";
        fn rename(resolved: &mut ResolvedProgram) {
            for range in &mut resolved.declarations.ranges {
                range.name = "display".into();
            }
            for parameter in &mut resolved.parameters {
                parameter.name = "display".into();
            }
            for f in &mut resolved.functions {
                if let ResolvedFunction::Ordinary(f) = f {
                    f.name = "display".into();
                    for statement in &mut f.body {
                        if let ResolvedValueStatement::Let { name, .. } = statement {
                            *name = "display".into();
                        }
                    }
                }
            }
        }
        let mut resolved = resolve::resolve(&parse_source(source).unwrap()).unwrap();
        rename(&mut resolved);
        let program = check(resolved).unwrap();
        let identity = program.functions()[0].as_ordinary().unwrap();
        let wrapped = program.functions()[1].as_ordinary().unwrap();
        let hir::ValueStatement::Let { initializer, .. } = &wrapped.body[0] else {
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
        assert_eq!(
            arguments[0].kind,
            ExprKind::Parameter(wrapped.parameters[0])
        );
        assert_eq!(initializer.ty, ExprType::Range(program.ranges()[0].id));
        let hir::ValueStatement::Return { value, .. } = wrapped.body.last().unwrap() else {
            panic!("return")
        };
        assert_eq!(value.kind, ExprKind::Local(program.locals()[2].id));
        assert_eq!(program.local(program.locals()[2].id).function, wrapped.id);
        assert_eq!(program.functions()[0].name(), program.functions()[1].name());
        let bad = source.replace("fn wrapped(value: Percent)", "fn wrapped(value: Other)");
        let mut resolved = resolve::resolve(&parse_source(&bad).unwrap()).unwrap();
        rename(&mut resolved);
        let errors = check(resolved).unwrap_err();
        assert!(errors[0].message.contains("distinct named ranges"));
        assert!(errors[0].required.contains("display"));
    }
    #[test]
    fn reference_referent_identity_survives_colliding_diagnostic_names() {
        for (first, second, declarations) in [
            ("First", "Second", "type First = range 0..100; type Second = range 0..100;"),
            ("First", "Second", "type Percent = range 0..100; record First { value: Percent } record Second { value: Percent }"),
        ] {
            for prefix in ["&", "&mut "] {
                let source = format!("{declarations} fn sink(value: {prefix}{first}) -> bool {{ return true; }} fn bad(value: {prefix}{second}) -> bool {{ return sink(value); }}");
                let mut resolved = resolve::resolve(&parse_source(&source).unwrap()).unwrap();
                for range in &mut resolved.declarations.ranges { range.name = "display".into(); }
                for record in &mut resolved.declarations.records { record.name = "display".into(); }
                for parameter in &mut resolved.parameters { parameter.name = "display".into(); }
                let errors = check(resolved).unwrap_err();
                assert_eq!(errors.len(), 1);
                assert!(errors[0].message.contains("call argument type mismatch"));
                assert!(errors[0].required.contains("display"));
            }
        }
    }
    #[test]
    fn dereference_and_write_typing_use_canonical_referents_after_renaming() {
        fn rename(resolved: &mut ResolvedProgram) {
            for range in &mut resolved.declarations.ranges {
                range.name = "display".into();
            }
            for record in &mut resolved.declarations.records {
                record.name = "display".into();
            }
            for parameter in &mut resolved.parameters {
                parameter.name = "display".into();
            }
        }
        let prefix = "type First = range 0..100; type Second = range 0..100;";
        let source = format!("{prefix} fn set(value: &mut First, replacement: First) -> First {{ *value = replacement; return *value; }}");
        let mut resolved = resolve::resolve(&parse_source(&source).unwrap()).unwrap();
        rename(&mut resolved);
        let program = check(resolved).unwrap();
        let f = program.functions()[0].as_ordinary().unwrap();
        let hir::ValueStatement::DerefAssign {
            reference, value, ..
        } = &f.body[0]
        else {
            panic!("write")
        };
        assert_eq!(*reference, hir::Place::Parameter(f.parameters[0]));
        assert_eq!(value.ty, ExprType::Range(program.ranges()[0].id));
        let hir::ValueStatement::Return { value, .. } = &f.body[1] else {
            panic!("return")
        };
        assert_eq!(
            value.kind,
            ExprKind::Deref {
                reference: *reference
            }
        );
        assert_eq!(value.ty, ExprType::Range(program.ranges()[0].id));
        let source = source.replace("replacement: First", "replacement: Second");
        let mut resolved = resolve::resolve(&parse_source(&source).unwrap()).unwrap();
        rename(&mut resolved);
        let errors = check(resolved).unwrap_err();
        assert!(errors[0].message.contains("distinct named ranges"));
        assert!(errors[0].required.contains("display"));
    }
    #[test]
    fn literal_formation_selects_canonical_ids_despite_colliding_display_names() {
        let source="type P = range 0..100; type Q = range 0..100; fn p() -> P {return 80;} fn q() -> Q {return 80;}";
        let mut resolved = resolve::resolve(&parse_source(source).unwrap()).unwrap();
        for range in &mut resolved.declarations.ranges {
            range.name = "display".into();
        }
        let p = check(resolved).unwrap();
        for (index, function) in p.functions().iter().enumerate() {
            let hir::ValueStatement::Return { value, .. } =
                &function.as_ordinary().unwrap().body[0]
            else {
                panic!()
            };
            assert_eq!(value.kind, ExprKind::FormedRangeLiteral(80));
            assert_eq!(value.ty, ExprType::Range(p.ranges()[index].id));
        }
        assert_ne!(p.ranges()[0].id, p.ranges()[1].id);
    }
}
