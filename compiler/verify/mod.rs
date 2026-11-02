use std::collections::BTreeMap;

use crate::diagnostics::Diagnostic;
use crate::hir::{
    BinaryOp, BindingId, ExprKind, FieldId, Program, RangeType, RecordBinding, Statement,
    TypedExpr, VerifiedFunction,
};
use crate::lexer::Span;

#[derive(Debug, Default)]
pub struct VerificationReport {
    pub diagnostics: Vec<Diagnostic>,
    pub functions_proven: usize,
    pub calls_checked: usize,
    pub final_values: BTreeMap<String, i64>,
}

type VerificationResult<T = ()> = Result<T, Box<Diagnostic>>;

/// Convenience API for callers that only need diagnostics.
pub fn verify(program: &Program) -> Vec<Diagnostic> {
    verify_report(program).diagnostics
}

/// Establish only the documented straight-line integer prototype obligations.
/// Input must be canonical typed HIR produced by the resolver.
pub fn verify_report(program: &Program) -> VerificationReport {
    let mut report = VerificationReport::default();
    // Prove each verified body, even when uncalled. Ordinary functions are frontend-only.
    for function in program.functions().iter().filter_map(|f| f.as_verified()) {
        match prove_function(program, function) {
            Ok(()) => report.functions_proven += 1,
            Err(error) => report.diagnostics.push(*error),
        }
    }
    if !report.diagnostics.is_empty() {
        return report;
    }

    let mut values = BTreeMap::<BindingId, i64>::new();
    for statement in &program.statements {
        let result = match statement {
            Statement::Binding(id) => {
                let binding = program.binding(*id);
                validate_binding(program, binding).map(|()| {
                    values.insert(*id, binding.value);
                    report
                        .final_values
                        .insert(binding.name.clone(), binding.value);
                })
            }
            Statement::Call(call) => check_call(program, call, &mut values).map(|()| {
                report.calls_checked += 1;
                report.final_values.insert(
                    program.binding(call.binding).name.clone(),
                    values[&call.binding],
                );
            }),
        };
        if let Err(error) = result {
            report.diagnostics.push(*error);
            // Do not reason about later calls with stale state after a failure.
            break;
        }
    }
    report
}

fn semantic_error(
    span: Span,
    message: impl Into<String>,
    detail: impl Into<String>,
) -> Box<Diagnostic> {
    Diagnostic::semantic(span, message, detail).into()
}

/// The input domain is a rectangle clipped by amount <= state. Its vertices
/// are rectangle corners and intersections of amount == state with its edges.
/// All supported expressions are affine. An affine bound/equality holds over
/// the entire polygon iff it holds at every vertex. These integral vertices
/// establish the obligations for ALL admissible integer inputs, without
/// enumerating a range or sampling a few calls.
fn input_vertices(state: &RangeType, amount: &RangeType) -> Vec<(i128, i128)> {
    let mut points = Vec::new();
    for x in [state.min, state.max] {
        for y in [amount.min, amount.max] {
            if y <= x {
                points.push((x as i128, y as i128));
            }
        }
    }
    for value in [state.min, state.max, amount.min, amount.max] {
        if value >= state.min && value <= state.max && value >= amount.min && value <= amount.max {
            points.push((value as i128, value as i128));
        }
    }
    points.sort_unstable();
    points.dedup();
    points
}

enum Operand {
    Parameter,
    Literal(i64),
}
struct ProofSubtract {
    operand: Operand,
    span: Span,
}

fn unsupported(expression: &TypedExpr, role: &str) -> Box<Diagnostic> {
    semantic_error(
        expression.span,
        "well-typed expression is not yet supported by the prototype verifier",
        format!(
            "unsupported {role}; this verifier requires amount <= state.field, \
             state.field == old(state.field) - amount, and body operands that are \
             the amount parameter or an integer literal"
        ),
    )
}
fn is_amount(expression: &TypedExpr, function: &VerifiedFunction) -> bool {
    matches!(expression.kind, ExprKind::Parameter(id) if id == function.amount_param.parameter)
}
fn is_state(
    expression: &TypedExpr,
    function: &VerifiedFunction,
    field: FieldId,
    old: bool,
) -> bool {
    let access = match (&expression.kind, old) {
        (ExprKind::FieldAccess(access), false) | (ExprKind::OldField(access), true) => access,
        _ => return false,
    };
    access.parameter == function.state_param && access.field == field
}
/// Recognize the original semantic proof shape; do not simplify richer expressions.
fn proof_body(
    program: &Program,
    function: &VerifiedFunction,
) -> VerificationResult<Vec<ProofSubtract>> {
    let fields = &program.record(function.state_type).fields;
    if fields.len() != 1 {
        return Err(semantic_error(
            function.span,
            "unsupported multi-field state in prototype verifier",
            "the specialized proof requires exactly one named-range state field",
        ));
    }
    let field = fields[0];
    let supported_requires = match &function.requires.kind {
        ExprKind::Binary {
            op: BinaryOp::LessEqual,
            left,
            right,
        } => is_amount(left, function) && is_state(right, function, field, false),
        _ => false,
    };
    if !supported_requires {
        return Err(unsupported(&function.requires, "requires expression"));
    }
    let supported_ensures = match &function.ensures.kind {
        ExprKind::Binary {
            op: BinaryOp::Equal,
            left,
            right,
        } if is_state(left, function, field, false) => match &right.kind {
            ExprKind::Binary {
                op: BinaryOp::Subtract,
                left,
                right,
            } => is_state(left, function, field, true) && is_amount(right, function),
            _ => false,
        },
        _ => false,
    };
    if !supported_ensures {
        return Err(unsupported(&function.ensures, "ensures expression"));
    }
    let mut body = Vec::new();
    for statement in &function.body {
        if statement.target.parameter != function.state_param || statement.target.field != field {
            return Err(semantic_error(
                statement.target.span,
                "well-typed expression is not yet supported by the prototype verifier",
                "the subtract-assignment target must be this function's resolved mutable state field",
            ));
        }
        let operand = match statement.operand.kind {
            ExprKind::Parameter(id) if id == function.amount_param.parameter => Operand::Parameter,
            ExprKind::IntegerLiteral(value) => Operand::Literal(value),
            _ => {
                return Err(unsupported(
                    &statement.operand,
                    "subtract-assignment operand",
                ))
            }
        };
        body.push(ProofSubtract {
            operand,
            span: statement.span,
        });
    }
    Ok(body)
}
fn prove_function(program: &Program, function: &VerifiedFunction) -> VerificationResult {
    let body = proof_body(program, function)?;
    let state = program.range(
        program
            .field(program.record(function.state_type).fields[0])
            .range_type()
            .expect("verified field is range"),
    );
    let amount = program.range(function.amount_param.ty);
    prove_affine_function(program, function, state, amount, &body)
}

/// Private numerical kernel. Source-language nominal compatibility is checked
/// before this is reached; unit tests can vary the numeric domains independently.
fn prove_affine_function(
    program: &Program,
    function: &VerifiedFunction,
    state: &RangeType,
    amount: &RangeType,
    body: &[ProofSubtract],
) -> VerificationResult {
    let vertices = input_vertices(state, amount);
    if vertices.is_empty() {
        return Err(semantic_error(
            function.requires.span,
            "precondition has no admissible inputs",
            "this prototype requires a nonempty input domain rather than reporting a vacuous proof",
        ));
    }
    let mut current: Vec<i128> = vertices.iter().map(|(x, _)| *x).collect();
    for statement in body {
        for (index, &(initial, input_amount)) in vertices.iter().enumerate() {
            let operand = match &statement.operand {
                Operand::Parameter => input_amount,
                Operand::Literal(value) => *value as i128,
            };
            // Both operands fit i64; i128 subtraction cannot overflow.
            let next = current[index] - operand;
            if next < i64::MIN as i128 || next > i64::MAX as i128 {
                return Err(proof_failure(
                    program,
                    function,
                    statement.span,
                    "subtraction overflow cannot be excluded",
                    initial,
                    input_amount,
                    next,
                ));
            }
            if next < state.min as i128 || next > state.max as i128 {
                return Err(proof_failure(
                    program,
                    function,
                    statement.span,
                    "field range preservation cannot be established",
                    initial,
                    input_amount,
                    next,
                ));
            }
            current[index] = next;
        }
    }
    for (index, &(initial, input_amount)) in vertices.iter().enumerate() {
        if current[index] != initial - input_amount {
            return Err(proof_failure(
                program,
                function,
                function.ensures.span,
                "postcondition cannot be established",
                initial,
                input_amount,
                current[index],
            ));
        }
    }
    Ok(())
}

fn proof_failure(
    program: &Program,
    function: &VerifiedFunction,
    span: Span,
    message: &str,
    state: i128,
    amount: i128,
    actual: i128,
) -> Box<Diagnostic> {
    Diagnostic {
        message: message.into(), span,
        required: format!("`{}` must preserve its field range and establish its postcondition for every input satisfying requires", function.name),
        known: vec![
            format!("old({}.{}) == {state}", program.parameter(function.state_param).name, program.field(program.record(function.state_type).fields[0]).name),
            format!("{} == {amount}", program.parameter(function.amount_param.parameter).name),
            format!("{amount} <= {state} is true"),
            format!("computed field value == {actual}"),
        ],
        conclusion: "admissible counterexample; this function has not been verified".into(),
    }.into()
}

fn validate_binding(program: &Program, binding: &RecordBinding) -> VerificationResult {
    let fields = &program.record(binding.record_type).fields;
    if fields.len() != 1 || fields[0] != binding.field {
        return Err(semantic_error(
            binding.span,
            "unsupported multi-field state in prototype harness",
            "harness bindings require exactly one canonical named-range state field",
        ));
    }
    let field = &program.field(binding.field).name;
    let range = program.range(
        program
            .field(binding.field)
            .range_type()
            .expect("verified field is range"),
    );
    if binding.value < range.min || binding.value > range.max {
        return Err(Diagnostic {
            message: "range constraint cannot be established".into(),
            span: binding.span,
            required: format!(
                "{} <= {}.{} <= {}",
                range.min, binding.name, field, range.max
            ),
            known: vec![format!("{}.{} == {}", binding.name, field, binding.value)],
            conclusion: format!("{} is outside {}..{}", binding.value, range.min, range.max),
        }
        .into());
    }
    Ok(())
}

fn check_call(
    program: &Program,
    call: &crate::hir::Call,
    values: &mut BTreeMap<BindingId, i64>,
) -> VerificationResult {
    let Some(function) = program.function(call.function).as_verified() else {
        return Err(semantic_error(
            call.span,
            "top-level prototype calls require a verified function",
            "ordinary functions are frontend-only",
        ));
    };
    let binding = program.binding(call.binding);
    if binding.record_type != function.state_type {
        return Err(semantic_error(
            call.span,
            "record argument type mismatch",
            format!(
                "expected `{}`, found `{}`; record types are distinct",
                program.record(function.state_type).name,
                program.record(binding.record_type).name
            ),
        ));
    }
    if !binding.mutable {
        return Err(semantic_error(
            call.span,
            "cannot mutably borrow immutable binding",
            format!("declare `{}` with `let mut` or `let mutable`", binding.name),
        ));
    }
    let amount_range = program.range(function.amount_param.ty);
    if call.amount < amount_range.min || call.amount > amount_range.max {
        return Err(Diagnostic {
            message: "argument violates range constraint".into(),
            span: call.span,
            required: format!(
                "{} <= {} <= {}",
                amount_range.min,
                program.parameter(function.amount_param.parameter).name,
                amount_range.max
            ),
            known: vec![format!(
                "{} == {}",
                program.parameter(function.amount_param.parameter).name,
                call.amount
            )],
            conclusion: format!(
                "{} is outside {}..{}",
                call.amount, amount_range.min, amount_range.max
            ),
        }
        .into());
    }
    let initial = values[&call.binding];
    if call.amount > initial {
        return Err(Diagnostic {
            message: "precondition cannot be established".into(),
            span: call.span,
            required: format!(
                "{} <= {}.{}",
                program.parameter(function.amount_param.parameter).name,
                binding.name,
                program
                    .field(program.record(function.state_type).fields[0])
                    .name
            ),
            known: vec![
                format!(
                    "{} == {}",
                    program.parameter(function.amount_param.parameter).name,
                    call.amount
                ),
                format!(
                    "{}.{} == {initial}",
                    binding.name,
                    program
                        .field(program.record(function.state_type).fields[0])
                        .name
                ),
            ],
            conclusion: format!("{} <= {initial} is false", call.amount),
        }
        .into());
    }
    // Modular reasoning uses the postcondition only AFTER the body proof.
    let next = initial.checked_sub(call.amount).ok_or_else(|| {
        semantic_error(
            call.span,
            "subtraction overflow",
            "the checked subtraction must fit signed i64",
        )
    })?;
    values.insert(call.binding, next);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Independent finite model checks the universal proof over many small domains.
    /// It enumerates every admissible input and every intermediate assignment,
    /// rather than reconstructing the verifier's polygon/vertex implementation.
    #[test]
    fn affine_proofs_match_exhaustive_execution_on_small_domains() {
        let prototype =
            crate::compile_source(include_str!("../../examples/battery_ok.bow")).unwrap();
        let template = prototype.functions[0].as_verified().unwrap();
        let mut combinations = 0;
        let bodies: &[&[Option<i64>]] = &[
            &[],
            &[None],
            &[Some(0)],
            &[Some(1)],
            &[Some(-1)],
            &[None, None],
            &[Some(1), Some(-1), None],
            &[None, Some(1), Some(-1)],
        ];
        for low in -2..=2 {
            for high in low..=2 {
                for amount_low in -2..=2 {
                    for amount_high in amount_low..=2 {
                        for body in bodies {
                            let mut has_inputs = false;
                            let mut valid = true;
                            for initial in low..=high {
                                for amount in amount_low..=amount_high {
                                    if amount > initial {
                                        continue;
                                    }
                                    has_inputs = true;
                                    let mut current = initial;
                                    for operand in *body {
                                        current -= operand.unwrap_or(amount);
                                        valid &= current >= low && current <= high;
                                    }
                                    valid &= current == initial - amount;
                                }
                            }
                            let state = RangeType {
                                id: prototype
                                    .field(prototype.record(template.state_type).fields[0])
                                    .range_type()
                                    .unwrap(),
                                name: "State".into(),
                                min: low,
                                max: high,
                                origin: crate::hir::DeclarationOrigin::Source(template.span),
                            };
                            let amount = RangeType {
                                id: template.amount_param.ty,
                                name: "Amount".into(),
                                min: amount_low,
                                max: amount_high,
                                origin: crate::hir::DeclarationOrigin::Source(template.span),
                            };
                            let proof_body: Vec<ProofSubtract> = body
                                .iter()
                                .map(|operand| ProofSubtract {
                                    operand: match operand {
                                        Some(value) => Operand::Literal(*value),
                                        None => Operand::Parameter,
                                    },
                                    span: template.body[0].span,
                                })
                                .collect();
                            let case = format!("state={low}..{high}, amount={amount_low}..{amount_high}, body={body:?}");
                            assert_eq!(
                                prove_affine_function(
                                    &prototype,
                                    template,
                                    &state,
                                    &amount,
                                    &proof_body
                                )
                                .is_ok(),
                                has_inputs && valid,
                                "{case}"
                            );
                            combinations += 1;
                        }
                    }
                }
            }
        }
        assert_eq!(combinations, 1_800);
    }

    #[test]
    fn numerical_kernel_preserves_extreme_and_empty_domain_coverage() {
        let program = crate::compile_source(include_str!("../../examples/battery_ok.bow")).unwrap();
        let function = program.functions[0].as_verified().unwrap();
        let body = proof_body(&program, function).unwrap();
        let full = RangeType {
            id: function.amount_param.ty,
            name: "Full".into(),
            min: i64::MIN,
            max: i64::MAX,
            origin: crate::hir::DeclarationOrigin::Source(function.span),
        };
        let zero = RangeType {
            id: function.amount_param.ty,
            name: "Zero".into(),
            min: 0,
            max: 0,
            origin: crate::hir::DeclarationOrigin::Source(function.span),
        };
        assert!(prove_affine_function(&program, function, &full, &zero, &body).is_ok());
        let small = RangeType {
            id: function.amount_param.ty,
            name: "Small".into(),
            min: 0,
            max: 100,
            origin: crate::hir::DeclarationOrigin::Source(function.span),
        };
        let too_much = RangeType {
            id: function.amount_param.ty,
            name: "TooMuch".into(),
            min: 101,
            max: 200,
            origin: crate::hir::DeclarationOrigin::Source(function.span),
        };
        let error =
            prove_affine_function(&program, function, &small, &too_much, &body).unwrap_err();
        assert_eq!(error.message, "precondition has no admissible inputs");
    }
    #[test]
    fn unsupported_multi_field_verified_and_harness_state_is_gated_before_scalar_selection() {
        let p = crate::compile_source(include_str!("../../examples/battery_ok.bow")).unwrap();
        let original = verify_report(&p);
        assert_eq!((original.functions_proven, original.calls_checked), (1, 1));
        let mut multiple = p.clone();
        let mut second = multiple.fields[0].clone();
        second.id = crate::hir::FieldId(multiple.fields.len());
        second.name = "second".into();
        multiple.records[0].fields.push(second.id);
        multiple.fields.push(second);
        let report = verify_report(&multiple);
        assert_eq!((report.functions_proven, report.calls_checked), (0, 0));
        assert!(report.diagnostics[0]
            .message
            .contains("unsupported multi-field state"));
        assert!(validate_binding(&multiple, &multiple.bindings[0])
            .unwrap_err()
            .message
            .contains("unsupported multi-field state"));
        assert_eq!(
            (
                verify_report(&p).functions_proven,
                verify_report(&p).calls_checked
            ),
            (1, 1)
        );
    }
}
