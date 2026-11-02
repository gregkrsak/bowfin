//! Owned moves, acyclic loans, and stable structured-loop references in ordinary HIR.
//! Availability and active loans are orthogonal private state; no runtime behavior.
use std::{
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
pub(crate) mod outcomes;
use outcomes::Outcome;

use crate::{
    diagnostics::Diagnostic,
    hir,
    hir::{
        BorrowKind, ExprKind, ExprType, FieldId, FunctionId, LocalId, ParameterId, ParameterType,
        Place, Program, RecordId, ReferentType, TypedExpr, ValueFunction, ValueStatement,
        ValueType,
    },
    lexer::Span,
};

fn statement_span(statement: &ValueStatement) -> Span {
    match statement {
        ValueStatement::Let { span, .. }
        | ValueStatement::Assign { span, .. }
        | ValueStatement::DerefAssign { span, .. }
        | ValueStatement::CopyFieldAssign { span, .. }
        | ValueStatement::CopyReferenceFieldAssign { span, .. }
        | ValueStatement::CallNoValue { span, .. }
        | ValueStatement::If { span, .. }
        | ValueStatement::Match { span, .. }
        | ValueStatement::While { span, .. }
        | ValueStatement::Return { span, .. }
        | ValueStatement::ReturnNoValue { span }
        | ValueStatement::Break { span }
        | ValueStatement::Continue { span } => *span,
    }
}

type OwnershipResult<T = ()> = Result<T, Box<Diagnostic>>;

// A count is consumed only by analysis of its own path. At a conditional the
// suffix is shared, but sibling-only counts are removed before edge expiry.
type FutureUses = BTreeMap<Place, usize>;
fn future_uses(statements: &[ValueStatement]) -> FutureUses {
    let mut uses = FutureUses::new();
    count_statements(statements, &mut uses);
    uses
}
fn count_statements(statements: &[ValueStatement], uses: &mut FutureUses) {
    for statement in statements {
        match statement {
            ValueStatement::Continue { .. }
            | ValueStatement::Break { .. }
            | ValueStatement::ReturnNoValue { .. } => {}
            ValueStatement::While {
                condition, body, ..
            } => {
                // Finite traversal/suffix bookkeeping only. Recurrent loan
                // obligations are discovered and retained separately, not counted.
                count_expression(condition, uses);
                count_statements(body, uses);
            }
            ValueStatement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                count_expression(condition, uses);
                count_statements(then_body, uses);
                count_statements(else_body, uses);
            }
            ValueStatement::Match {
                scrutinee, arms, ..
            } => {
                count_expression(scrutinee, uses);
                for arm in arms {
                    count_statements(&arm.body, uses);
                }
            }
            ValueStatement::CallNoValue { arguments, .. } => {
                for argument in arguments {
                    count_expression(argument, uses);
                }
            }
            ValueStatement::Let { initializer, .. } => count_expression(initializer, uses),
            ValueStatement::CopyFieldAssign { value, .. }
            | ValueStatement::Return { value, .. }
            | ValueStatement::Assign { value, .. } => count_expression(value, uses),
            ValueStatement::CopyReferenceFieldAssign {
                reference, value, ..
            }
            | ValueStatement::DerefAssign {
                reference, value, ..
            } => {
                *uses.entry(*reference).or_default() += 1;
                count_expression(value, uses);
            }
        }
    }
}
fn count_expression(expression: &TypedExpr, uses: &mut FutureUses) {
    match &expression.kind {
        ExprKind::ResultConstruct { payload, .. } => count_expression(payload, uses),
        ExprKind::IfValue {
            condition,
            then_value,
            else_value,
        } => {
            count_expression(condition, uses);
            count_expression(then_value, uses);
            count_expression(else_value, uses);
        }
        ExprKind::RecordConstruct { fields, .. } => {
            for entry in fields {
                count_expression(&entry.value, uses);
            }
        }
        ExprKind::CopyFieldRead { owner, .. } => *uses.entry(*owner).or_default() += 1,
        ExprKind::Deref { reference } | ExprKind::CopyReferenceFieldRead { reference, .. } => {
            *uses.entry(*reference).or_default() += 1
        }
        ExprKind::Parameter(id) => *uses.entry(Place::Parameter(*id)).or_default() += 1,
        ExprKind::Local(id) => *uses.entry(Place::Local(*id)).or_default() += 1,
        ExprKind::Call { arguments, .. } | ExprKind::TrustedCall { arguments, .. } => {
            for argument in arguments {
                count_expression(argument, uses);
            }
        }
        ExprKind::Unary { operand, .. } => count_expression(operand, uses),
        ExprKind::Binary { left, right, .. } => {
            count_expression(left, uses);
            count_expression(right, uses);
        }
        ExprKind::IntegerLiteral(_)
        | ExprKind::FormedRangeLiteral(_)
        | ExprKind::BoolLiteral(_)
        | ExprKind::Borrow { .. }
        | ExprKind::FieldAccess(_)
        | ExprKind::OldField(_) => {}
    }
}
fn subtract_uses(future: &mut FutureUses, uses: &FutureUses) {
    for (place, count) in uses {
        let remaining = future.get_mut(place).expect("counted branch use");
        *remaining = remaining
            .checked_sub(*count)
            .expect("branch uses belong to suffix");
    }
}
fn with_branch_uses(continuation: &FutureUses, branch: &FutureUses) -> FutureUses {
    let mut future = continuation.clone();
    for (place, count) in branch {
        *future.entry(*place).or_default() += count;
    }
    future
}

// Finite, syntax-directed summaries. Break/continue replace the same-iteration
// suffix with the target loop's outside continuation. Recurrence is NOT encoded
// here: the independently scoped LoanId obligations remain authoritative.
fn block_future(
    body: &[ValueStatement],
    after: &FutureUses,
    continuing: &FutureUses,
) -> FutureUses {
    let mut future = after.clone();
    for statement in body.iter().rev() {
        future = statement_future(statement, &future, continuing);
    }
    future
}
fn statement_future(
    statement: &ValueStatement,
    after: &FutureUses,
    continuing: &FutureUses,
) -> FutureUses {
    match statement {
        ValueStatement::Match {
            scrutinee, arms, ..
        } => {
            let mut future = FutureUses::new();
            for arm in arms {
                for (place, count) in block_future(&arm.body, after, continuing) {
                    let entry = future.entry(place).or_default();
                    *entry = (*entry).max(count);
                }
            }
            count_expression(scrutinee, &mut future);
            future
        }
        ValueStatement::ReturnNoValue { .. } => FutureUses::new(),
        ValueStatement::Return { value, .. } => {
            let mut uses = FutureUses::new();
            count_expression(value, &mut uses);
            uses
        }
        ValueStatement::Continue { .. } | ValueStatement::Break { .. } => continuing.clone(),
        ValueStatement::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            let mut future = block_future(then_body, after, continuing);
            for (place, count) in block_future(else_body, after, continuing) {
                let entry = future.entry(place).or_default();
                *entry = (*entry).max(count);
            }
            count_expression(condition, &mut future);
            future
        }
        _ => {
            let mut future = after.clone();
            count_statements(std::slice::from_ref(statement), &mut future);
            future
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Fallthrough,
    Continue,
    Break,
    NoFallthrough,
    Return,
}

// Discover syntactic reference-place uses on all checked paths, including nested
// loops. This set creates scoped recurrent obligations; it is not FutureUses.
fn reference_uses(body: &[ValueStatement]) -> BTreeSet<Place> {
    fn expression(expr: &TypedExpr, uses: &mut BTreeSet<Place>) {
        match &expr.kind {
            ExprKind::ResultConstruct { payload, .. } => expression(payload, uses),
            ExprKind::Parameter(id) => {
                uses.insert(Place::Parameter(*id));
            }
            ExprKind::Local(id) => {
                uses.insert(Place::Local(*id));
            }
            ExprKind::RecordConstruct { fields, .. } => {
                for entry in fields {
                    expression(&entry.value, uses);
                }
            }
            ExprKind::CopyFieldRead { .. } => {} // Named owned root, never a reference handle.
            ExprKind::Deref { reference } | ExprKind::CopyReferenceFieldRead { reference, .. } => {
                uses.insert(*reference);
            }
            ExprKind::Call { arguments, .. } | ExprKind::TrustedCall { arguments, .. } => {
                for argument in arguments {
                    expression(argument, uses);
                }
            }
            ExprKind::IfValue {
                condition,
                then_value,
                else_value,
            } => {
                expression(condition, uses);
                expression(then_value, uses);
                expression(else_value, uses);
            }
            ExprKind::Unary { operand, .. } => expression(operand, uses),
            ExprKind::Binary { left, right, .. } => {
                expression(left, uses);
                expression(right, uses);
            }
            ExprKind::IntegerLiteral(_)
            | ExprKind::FormedRangeLiteral(_)
            | ExprKind::BoolLiteral(_)
            | ExprKind::Borrow { .. }
            | ExprKind::FieldAccess(_)
            | ExprKind::OldField(_) => {}
        }
    }
    fn statements(body: &[ValueStatement], uses: &mut BTreeSet<Place>) {
        for statement in body {
            match statement {
                ValueStatement::Match {
                    scrutinee, arms, ..
                } => {
                    expression(scrutinee, uses);
                    for arm in arms {
                        statements(&arm.body, uses);
                    }
                }
                ValueStatement::Continue { .. }
                | ValueStatement::Break { .. }
                | ValueStatement::ReturnNoValue { .. } => {}
                ValueStatement::While {
                    condition, body, ..
                } => {
                    expression(condition, uses);
                    statements(body, uses);
                }
                ValueStatement::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    expression(condition, uses);
                    statements(then_body, uses);
                    statements(else_body, uses);
                }
                ValueStatement::CallNoValue { arguments, .. } => {
                    for argument in arguments {
                        expression(argument, uses);
                    }
                }
                ValueStatement::Let { initializer, .. } => expression(initializer, uses),
                ValueStatement::CopyFieldAssign { value, .. }
                | ValueStatement::Assign { value, .. }
                | ValueStatement::Return { value, .. } => expression(value, uses),
                ValueStatement::CopyReferenceFieldAssign {
                    reference, value, ..
                }
                | ValueStatement::DerefAssign {
                    reference, value, ..
                } => {
                    uses.insert(*reference);
                    expression(value, uses);
                }
            }
        }
    }
    let mut uses = BTreeSet::new();
    statements(body, &mut uses);
    uses
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnershipKind {
    Copy,
    Move,
}
fn ownership_kind(ty: ValueType) -> OwnershipKind {
    match ty {
        ValueType::Bool | ValueType::Range(_) | ValueType::SharedRef(_) => OwnershipKind::Copy,
        ValueType::Record(_)
        | ValueType::Resource(_)
        | ValueType::MutableRef(_)
        | ValueType::Result(_) => OwnershipKind::Move,
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Transfer {
    Match,
    Propagate(LocalId),
    ResultPayload(hir::ResultTypeId, hir::ResultTag),
    Local(LocalId),
    Argument(ParameterId),
    TrustedArgument(hir::TrustedOperationId, usize),
    Return(FunctionId),
    WriteThrough(Place),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Available,
    Moved { span: Span, transfer: Transfer },
    ConditionalMove { span: Span },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct LoanId(usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LoanOrigin {
    ExternalParameter(ParameterId),
    LocalBinding(LocalId),
    Unbound, // A direct borrow argument has no stored handle lineage.
}
#[derive(Clone)]
struct Loan {
    // External reference parameters have no owner identity in this function.
    owner: Option<Place>,
    // Binding origin distinguishes compatible loans without a second LoanId map.
    origin: LoanOrigin,
    kind: BorrowKind,
    span: Span,
    held: usize,
}

/// Inspect canonical typed HIR; report the first substantive move/loan error.
/// compile_source runs resolution and typing before this phase.
pub fn check(program: &Program) -> Result<(), Vec<Diagnostic>> {
    analyze(program).map(|_| ())
}

pub(crate) fn analyze(
    program: &Program,
) -> Result<Vec<crate::effects::custody::Site>, Vec<Diagnostic>> {
    analyze_with(program, None).map(|a| a.0)
}

pub(crate) fn evidence_bounded(
    program: &Program,
    context: &crate::bounded::Context,
) -> Result<(Vec<crate::effects::custody::Site>, Vec<outcomes::Witness>), Vec<Diagnostic>> {
    analyze_with(program, Some(context))
}

fn effect_analysis_error(
    error: crate::hir::resources::PlanError,
    context: Option<&crate::bounded::Context>,
) -> Vec<Diagnostic> {
    use crate::hir::resources::PlanError;
    if let Some(context) = context {
        let kind = match error {
            PlanError::Limit(_) => Some(crate::bounded::ResourceKind::Limit),
            PlanError::Host(_) => Some(crate::bounded::ResourceKind::Host),
            PlanError::Integrity(_) => None,
        };
        if let Some(kind) = kind {
            context.fail(crate::bounded::ResourceFailure {
                kind,
                resource: error.message(),
                span: crate::bounded::START,
            });
        }
    }
    vec![Diagnostic::semantic(
        crate::bounded::START,
        error.message(),
        "canonical bounded cleanup effect evidence is required",
    )]
}

fn analyze_with(
    program: &Program,
    context: Option<&crate::bounded::Context>,
) -> Result<(Vec<crate::effects::custody::Site>, Vec<outcomes::Witness>), Vec<Diagnostic>> {
    use crate::effects::{custody::Custody, EffectSet};
    use crate::hir::resources::PlanError;
    let active = program
        .resources
        .leaves
        .iter()
        .any(|l| l.cleanup_effects != EffectSet::EMPTY)
        || program
            .resources
            .operations
            .iter()
            .any(|o| o.effects != EffectSet::EMPTY);
    let summaries = if active {
        program
            .resources
            .validate(program)
            .map_err(|e| effect_analysis_error(e, context))?;
        let plans = program
            .resources
            .plans(program)
            .map_err(|e| effect_analysis_error(e, context))?;
        program
            .resources
            .summaries(program, &plans)
            .map_err(|e| effect_analysis_error(e, context))?
    } else {
        Vec::new()
    };
    if !summaries.is_empty() || program.trusted.is_some() {
        if let Some(context) = context {
            let nodes = context.resource_nodes.get()
                + program.functions.len()
                + program.locals.len()
                + program.parameters.len();
            context
                .reserve_cleanup_effects(nodes, crate::effects::custody::holder_bound(program))
                .map_err(|e| {
                    effect_analysis_error(
                        match e.kind {
                            crate::bounded::ResourceKind::Limit => PlanError::Limit(e.resource),
                            crate::bounded::ResourceKind::Host => PlanError::Host(e.resource),
                        },
                        Some(context),
                    )
                })?;
        }
    }
    let trail =
        (!program.result_types().is_empty()).then(|| Rc::new(std::cell::RefCell::new(Vec::new())));
    if trail.is_some() {
        if let Some(c) = context {
            c.reserve_outcomes(
                program.locals.len()
                    + program.parameters.len()
                    + program.result_types().len()
                    + program.functions.len(),
            )
            .map_err(|e| effect_analysis_error(PlanError::Limit(e.resource), Some(c)))?;
        }
    }
    let summaries = Rc::new(summaries);
    let mut sites = Vec::new();
    for function in program.functions().iter().filter_map(|f| f.as_ordinary()) {
        let mut checker = Checker::new(program, function);
        checker.context = context;
        checker.outcome_trail = trail.clone();
        for (place, fact) in checker.outcomes.clone() {
            checker
                .witness(
                    fact.origin,
                    outcomes::Action::Initialize(place, fact.result),
                )
                .map_err(|e| vec![*e])?;
        }
        // Registered stubs are fixed primitive boundaries, never callback bodies.
        if (!summaries.is_empty() || program.trusted.is_some())
            && !program
                .resources
                .operations
                .iter()
                .any(|o| o.function == function.id)
        {
            let mut custody = Custody::new(
                function.id,
                summaries.clone(),
                program
                    .trusted
                    .as_ref()
                    .map(|c| (c.sensor, c.resource.effects)),
            );
            for parameter in &function.parameters {
                if let ParameterType::Value(ty) = program.parameter(*parameter).ty {
                    custody.initialize(Place::Parameter(*parameter), ty);
                }
            }
            checker.cleanup_custody = Some(custody);
        }
        checker.body(function).map_err(|e| vec![*e])?;
        if let Some(custody) = checker.cleanup_custody {
            let new_sites = custody.into_sites();
            crate::effects::custody::reserve_output(&mut sites, new_sites.len())
                .map_err(|e| effect_analysis_error(e, context))?;
            sites.extend(new_sites);
        }
    }
    let witnesses = trail
        .map(|t| {
            Rc::try_unwrap(t)
                .expect("ownership trail retained after analysis")
                .into_inner()
        })
        .unwrap_or_default();
    Ok((sites, witnesses))
}
fn checked_place_type(program: &Program, function: FunctionId, place: Place) -> Option<ValueType> {
    match place {
        Place::Parameter(id) => program
            .parameters()
            .get(id.0)
            .filter(|p| p.id == id && p.function == function)
            .and_then(|p| match p.ty {
                ParameterType::Value(ty) => Some(ty),
                _ => None,
            }),
        Place::Local(id) => program
            .locals()
            .get(id.0)
            .filter(|l| l.id == id && l.function == function)
            .map(|l| l.ty),
    }
}
/// Immutable location metadata only; never caches state-dependent lineage.
/// Borrow canonical expressions and share this once-built index across snapshots.
struct Initializers<'a> {
    function: FunctionId,
    entries: BTreeMap<LocalId, &'a TypedExpr>,
    arm_bindings: BTreeSet<LocalId>,
    invalid: Option<Span>,
}
impl<'a> Initializers<'a> {
    fn new(program: &'a Program, function: &'a ValueFunction) -> Self {
        let mut index = Self {
            function: function.id,
            entries: BTreeMap::new(),
            arm_bindings: BTreeSet::new(),
            invalid: None,
        };
        let result = (|| {
            if !program
                .functions()
                .get(function.id.0)
                .and_then(|f| f.as_ordinary())
                .is_some_and(|canonical| std::ptr::eq(canonical, function))
            {
                return Err(function.span);
            }
            index.statements(program, &function.body)?;
            // Require the complete reciprocal table/statement relationship,
            // including locals not reached by an actual reference use.
            for (position, local) in program.locals().iter().enumerate() {
                if local.function == function.id
                    && (local.id != LocalId(position)
                        || (!index.entries.contains_key(&local.id)
                            && !index.arm_bindings.contains(&local.id)))
                {
                    return Err(local.span);
                }
            }
            Ok(())
        })();
        index.invalid = result.err();
        index
    }
    fn statements(&mut self, program: &Program, body: &'a [ValueStatement]) -> Result<(), Span> {
        for statement in body {
            #[cfg(test)]
            initializer_probe::update(|c| c.construction_visits += 1);
            match statement {
                ValueStatement::Match {
                    scrutinee,
                    arms,
                    span,
                } => {
                    let ExprType::Result(result) = scrutinee.ty else {
                        return Err(*span);
                    };
                    if arms.len() != 2 || arms[0].tag == arms[1].tag {
                        return Err(*span);
                    }
                    for arm in arms {
                        let local = program
                            .locals()
                            .get(arm.binding.0)
                            .ok_or(arm.pattern_span)?;
                        if local.id != arm.binding
                            || local.function != self.function
                            || local.mutable
                            || Some(local.ty) != program.result_payload(result, arm.tag)
                            || self.entries.contains_key(&arm.binding)
                            || !self.arm_bindings.insert(arm.binding)
                        {
                            return Err(arm.pattern_span);
                        }
                        self.statements(program, &arm.body)?;
                    }
                }
                ValueStatement::Let {
                    local,
                    initializer,
                    span,
                    propagation,
                } => {
                    let declaration = program
                        .locals()
                        .get(local.0)
                        .filter(|l| l.id == *local && l.function == self.function)
                        .ok_or(*span)?;
                    // Check structural source identity before traversal can index
                    // a forged root. No availability/loan/transfer fact is cached.
                    if matches!(
                        declaration.ty,
                        ValueType::SharedRef(_) | ValueType::MutableRef(_)
                    ) {
                        let source = match initializer.kind {
                            ExprKind::Parameter(id) => Some(Place::Parameter(id)),
                            ExprKind::Local(id) => Some(Place::Local(id)),
                            ExprKind::Borrow { place, .. } => Some(place),
                            _ => None,
                        };
                        if source.is_some_and(|place| {
                            checked_place_type(program, self.function, place).is_none()
                        }) {
                            return Err(*span);
                        }
                    }
                    if propagation.map_or(
                        initializer.ty != ExprType::from(declaration.ty),
                        |mode| {
                            mode.validate(program, self.function, *local, initializer, *span)
                                .is_err()
                        },
                    ) || self.entries.insert(*local, initializer).is_some()
                    {
                        return Err(*span);
                    }
                }
                ValueStatement::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    self.statements(program, then_body)?;
                    self.statements(program, else_body)?;
                }
                ValueStatement::While { body, .. } => self.statements(program, body)?,
                ValueStatement::ReturnNoValue { .. }
                | ValueStatement::CallNoValue { .. }
                | ValueStatement::Break { .. }
                | ValueStatement::Continue { .. }
                | ValueStatement::CopyFieldAssign { .. }
                | ValueStatement::CopyReferenceFieldAssign { .. }
                | ValueStatement::Assign { .. }
                | ValueStatement::DerefAssign { .. }
                | ValueStatement::Return { .. } => {}
            }
        }
        Ok(())
    }
    fn get(&self, function: FunctionId, local: LocalId) -> Option<&'a TypedExpr> {
        #[cfg(test)]
        initializer_probe::update(|c| c.lookups += 1);
        if self.invalid.is_some() || function != self.function {
            return None;
        }
        self.entries.get(&local).copied()
    }
}
// Test-only actual operation counts, separate from ownership transaction state.
#[cfg(test)]
mod initializer_probe {
    use std::cell::Cell;
    #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
    pub(super) struct Counts {
        pub construction_visits: usize,
        pub lookups: usize,
        pub origin_calls: usize,
        pub max_depth: usize,
    }
    std::thread_local! { static COUNTS: Cell<Counts> = Cell::new(Counts::default()); }
    pub(super) fn reset() {
        COUNTS.with(|c| c.set(Counts::default()));
    }
    pub(super) fn counts() -> Counts {
        COUNTS.with(Cell::get)
    }
    pub(super) fn update(f: impl FnOnce(&mut Counts)) {
        COUNTS.with(|c| {
            let mut value = c.get();
            f(&mut value);
            c.set(value);
        });
    }
}
// Each nested loop owns a distinct header boundary. Branch snapshots inherit it;
// locals introduced in an outer iteration are pre-existing at an inner header.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopScope {
    header_places: BTreeSet<Place>,
    header_handles: BTreeSet<Place>,
    header_loans: BTreeSet<LoanId>,
    condition: bool,
}
impl LoopScope {
    fn new(header: &Checker<'_>, condition: bool) -> Self {
        Self {
            header_places: header.states.keys().copied().collect(),
            header_handles: header.handles.keys().copied().collect(),
            header_loans: header.loans.keys().copied().collect(),
            condition,
        }
    }
}
// Each saved header predates pushing its own target. Rc shares immutable
// snapshots among branch copies; nesting is a finite stack, never a CFG solver.
#[derive(Clone)]
struct LoopTarget<'a> {
    header: Rc<Checker<'a>>,
    exit: Rc<Checker<'a>>,
    outside: FutureUses,
}
#[derive(Clone)]
struct Checker<'a> {
    program: &'a Program,
    context: Option<&'a crate::bounded::Context>,
    function: FunctionId,
    initializers: Rc<Initializers<'a>>,
    states: BTreeMap<Place, State>,
    loans: BTreeMap<LoanId, Loan>,
    handles: BTreeMap<Place, LoanId>,
    remaining: FutureUses,
    next_loan: usize,
    loop_scope: Option<LoopScope>,
    // One frame per structured loop; branches inherit all enclosing obligations.
    recurrent: Vec<BTreeSet<LoanId>>,
    targets: Vec<LoopTarget<'a>>,
    // Scoped ONLY to an actual return operand, not Transfer::Return (which also
    // labels non-terminal conditions). Nested argument evaluation inherits it.
    terminal_return: bool,
    // Operation-local continuity restriction, never a synthetic loan.
    replacement_target: Option<Place>,
    cleanup_custody: Option<crate::effects::custody::Custody>,
    outcomes: BTreeMap<Place, Outcome>,
    outcome_trail: Option<Rc<std::cell::RefCell<Vec<outcomes::Witness>>>>,
}
impl<'a> Checker<'a> {
    fn new(program: &'a Program, function: &'a ValueFunction) -> Self {
        let mut checker = Self {
            program,
            context: None,
            function: function.id,
            initializers: Rc::new(Initializers::new(program, function)),
            states: BTreeMap::new(),
            loans: BTreeMap::new(),
            handles: BTreeMap::new(),
            remaining: future_uses(&function.body),
            next_loan: 0,
            loop_scope: None,
            recurrent: Vec::new(),
            targets: Vec::new(),
            terminal_return: false,
            replacement_target: None,
            cleanup_custody: None,
            outcomes: BTreeMap::new(),
            outcome_trail: None,
        };
        if checker.initializers.invalid.is_some() {
            return checker;
        }
        for id in &function.parameters {
            if let ParameterType::Value(ty) = program.parameter(*id).ty {
                let place = Place::Parameter(*id);
                checker.initialize_outcome(place, ty, program.parameter(*id).span);
                if ownership_kind(ty) == OwnershipKind::Move {
                    checker.states.insert(place, State::Available);
                }
                let kind = match ty {
                    ValueType::SharedRef(_) => Some(BorrowKind::Shared),
                    ValueType::MutableRef(_) => Some(BorrowKind::Mutable),
                    _ => None,
                };
                if let Some(kind) = kind {
                    let loan = checker.add_loan(None, kind, program.parameter(*id).span);
                    checker.loans.get_mut(&loan).unwrap().origin =
                        LoanOrigin::ExternalParameter(*id);
                    checker.handles.insert(place, loan);
                }
            }
        }
        checker
    }
    fn body(&mut self, function: &ValueFunction) -> OwnershipResult {
        if let Some(span) = self.initializers.invalid {
            return Err(self.diagnostic(span, "invalid canonical local initializer index".into(),
                "each current-function LocalId must have exactly one canonical, exact-typed initializer in its statement tree", None));
        }
        let flow = self.statements_with(&function.body, function.id, &FutureUses::new())?;
        if function.return_type.is_some() {
            assert_eq!(flow, Flow::Return);
        } else {
            assert!(matches!(flow, Flow::Return | Flow::Fallthrough));
            if flow == Flow::Fallthrough {
                self.check_outcome_exit(|_| true, function.span)?;
            }
            self.complete_no_value(function.span)?;
            if flow == Flow::Fallthrough {
                if let Some(custody) = &mut self.cleanup_custody {
                    custody.retire(
                        |_| true,
                        function.span,
                        crate::effects::custody::Kind::Completion,
                    );
                }
            }
        }
        Ok(())
    }
    #[cfg(test)]
    fn statements(
        &mut self,
        statements: &[ValueStatement],
        function: FunctionId,
    ) -> OwnershipResult {
        let mut after = self.remaining.clone();
        subtract_uses(&mut after, &future_uses(statements));
        self.statements_with(statements, function, &after)
            .map(|_| ())
    }
    fn statements_with(
        &mut self,
        statements: &[ValueStatement],
        function: FunctionId,
        after: &FutureUses,
    ) -> OwnershipResult<Flow> {
        let call_before = self.clone();
        let continuing = self
            .targets
            .last()
            .map(|t| t.outside.clone())
            .unwrap_or_default();
        let mut suffix = after.clone();
        let mut summaries = Vec::with_capacity(statements.len());
        for statement in statements.iter().rev() {
            let before = statement_future(statement, &suffix, &continuing);
            summaries.push((before.clone(), suffix));
            suffix = before;
        }
        summaries.reverse();
        self.remaining = suffix;
        self.expire();
        for (statement, (before_uses, after_uses)) in statements.iter().zip(summaries) {
            self.remaining = before_uses;
            self.expire();
            match self.statement(statement, function, &after_uses) {
                Ok(Flow::Fallthrough) => {
                    self.remaining = after_uses;
                    self.expire();
                }
                Ok(flow) => return Ok(flow),
                Err(error) => {
                    // Return owns a narrower transaction: its static operand-only
                    // continuation and pre-operation expiry must NOT be undone.
                    // Enclosing compound operations still retain their own atomicity.
                    if !matches!(
                        statement,
                        ValueStatement::Return { .. } | ValueStatement::ReturnNoValue { .. }
                    ) {
                        *self = call_before;
                    }
                    return Err(error);
                }
            }
        }
        self.remaining = after.clone();
        self.expire();
        Ok(Flow::Fallthrough)
    }
    fn statement(
        &mut self,
        statement: &ValueStatement,
        function: FunctionId,
        after: &FutureUses,
    ) -> OwnershipResult<Flow> {
        match statement {
            ValueStatement::Match {
                scrutinee,
                arms,
                span,
            } => return self.match_result(scrutinee, arms, *span, function, after),
            ValueStatement::ReturnNoValue { span } => return self.bare_return(*span),
            ValueStatement::CallNoValue {
                function,
                arguments,
                span,
            } => {
                self.call_statement(*function, arguments, *span)?;
            }
            ValueStatement::Continue { span } => return self.continue_edge(*span),
            ValueStatement::Break { span } => return self.break_edge(*span),
            ValueStatement::While {
                condition,
                body,
                span,
            } => {
                self.while_loop(condition, body, *span, function)?;
            }
            ValueStatement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                return self.conditional(condition, then_body, else_body, function, after);
            }
            ValueStatement::Let {
                local,
                initializer,
                propagation,
                ..
            } => {
                let loan = if let Some(mode) = propagation {
                    self.propagating_local(*local, initializer, *mode)?;
                    None
                } else {
                    self.expression(initializer, Transfer::Local(*local))?
                };
                let place = Place::Local(*local);
                self.initialize_outcome(place, self.program.local(*local).ty, initializer.span);
                if let ValueType::Result(ty) = self.program.local(*local).ty {
                    self.witness(initializer.span, outcomes::Action::Initialize(place, ty))?;
                }
                if let Some(custody) = &mut self.cleanup_custody {
                    custody.initialize(place, self.program.local(*local).ty);
                }
                if ownership_kind(self.program.local(*local).ty) == OwnershipKind::Move {
                    self.states.insert(place, State::Available);
                }
                if let Some(loan) = loan {
                    if matches!(initializer.kind, ExprKind::Borrow { .. }) {
                        self.loans.get_mut(&loan).expect("new borrow loan").origin =
                            LoanOrigin::LocalBinding(*local);
                    }
                    self.handles.insert(place, loan);
                }
            }
            ValueStatement::CopyReferenceFieldAssign {
                reference,
                record,
                field,
                value,
                reference_span,
                span,
                ..
            } => {
                self.copy_reference_field_assign(
                    *reference,
                    *record,
                    *field,
                    value,
                    *reference_span,
                    *span,
                )?;
            }
            ValueStatement::CopyFieldAssign {
                owner,
                record,
                field,
                value,
                owner_span,
                span,
                ..
            } => {
                self.copy_field_assign(*owner, *record, *field, value, *owner_span, *span)?;
            }
            ValueStatement::Assign { local, value, span } => {
                let place = Place::Local(*local);
                if ownership_kind(self.program.local(*local).ty) != OwnershipKind::Copy {
                    self.replace_owned(*local, value, *span)?;
                    return Ok(Flow::Fallthrough);
                }
                // The RHS sees the old value. Unlike write-through, direct
                // assignment does not hold a target loan during RHS evaluation.
                self.expression(value, Transfer::Local(*local))?;
                self.expire();
                self.available(place, *span)?;
                if let Some(loan) = self.conflict(place, Some(BorrowKind::Mutable)) {
                    return Err(self.diagnostic(*span, format!("cannot assign `{}` while it is borrowed", self.metadata(place).0), "direct assignment requires exclusive mutation access; any active shared or exclusive borrow conflicts with the write", Some(loan.span)));
                }
                // Copy replacement leaves the existing owner available and
                // creates neither a new identity nor loan provenance.
            }
            ValueStatement::DerefAssign {
                reference,
                value,
                span,
            } => {
                let loan = self.borrowed_access(*reference, *span, true)?;
                // A final target use must still protect the whole write statement,
                // including RHS dereferences and nested calls that trigger expiry.
                self.loans
                    .get_mut(&loan)
                    .expect("live write provenance")
                    .held += 1;
                self.expression(value, Transfer::WriteThrough(*reference))?;
                // RHS calls may transfer this move-only handle. That must not
                // leave an apparently valid write through a now-moved handle.
                self.available(*reference, *span)?;
                self.loans.get_mut(&loan).expect("write-held loan").held -= 1;
            }
            ValueStatement::Return { value, span } => {
                return self.return_edge(value, function, *span);
            }
        }
        self.expire();
        Ok(Flow::Fallthrough)
    }
    fn replace_owned(&mut self, local: LocalId, value: &TypedExpr, span: Span) -> OwnershipResult {
        let before = self.clone();
        let result = (|| {
            let place = Place::Local(local);
            let declaration = self.program.local(local);
            if !declaration.mutable
                || !matches!(declaration.ty, ValueType::Record(_))
                || value.ty != ExprType::from(declaration.ty)
            {
                return Err(self.provenance_error(span));
            }
            self.available(place, span)?;
            if self
                .loop_scope
                .as_ref()
                .is_some_and(|scope| scope.header_places.contains(&place))
            {
                return Err(self.diagnostic(span, format!("cannot replace pre-existing non-Copy local `{}` in while loop", declaration.name), "only an iteration-local owner may be replaced; a later return, break or continue does not relax the current loop header boundary", None));
            }
            self.replacement_target = Some(place);
            self.expression(value, Transfer::Local(local))?;
            self.expire();
            self.available(place, span)?;
            if let Some(loan) = self.conflict(place, Some(BorrowKind::Mutable)) {
                return Err(self.diagnostic(span, format!("cannot replace `{}` while it is borrowed", declaration.name), "replacement displaces this whole owner and requires all shared and exclusive target loans to have ended", Some(loan.span)));
            }
            self.replacement_target = before.replacement_target;
            if let Some(custody) = &mut self.cleanup_custody {
                custody.retire(
                    |p| p == place,
                    span,
                    crate::effects::custody::Kind::Replacement,
                );
                custody.initialize(place, declaration.ty);
            }
            Ok(())
        })();
        if result.is_err() {
            *self = before;
        }
        result
    }
    // Snapshot the prepared operation entry, not the broader statement-list entry.
    // No target reservation: all lawful RHS effects commit only on success.
    fn copy_field_assign(
        &mut self,
        owner: LocalId,
        record: RecordId,
        field: FieldId,
        value: &TypedExpr,
        owner_span: Span,
        span: Span,
    ) -> OwnershipResult {
        let before = self.clone();
        let result = (|| {
            self.expression(value, Transfer::Local(owner))?;
            self.expire();
            let place = Place::Local(owner);
            self.available(place, owner_span)?;
            if let Some(loan) = self.conflict(place, Some(BorrowKind::Mutable)) {
                // Destination IDs were validated at HIR publication; they are
                // diagnostic metadata only. Conflict checking remains whole-root.
                let root = self.metadata(place).0;
                let record = self.program.record(record);
                let field = self.program.field(field);
                let range = self
                    .program
                    .range(field.range_type().expect("Copy-field is range"));
                let kind = match loan.kind {
                    BorrowKind::Shared => "shared",
                    BorrowKind::Mutable => "exclusive",
                };
                return Err(self.diagnostic(
                    span,
                    format!("cannot assign `{root}.{}` while it is borrowed", field.name),
                    &format!("field `{}.{}` has declared named-range type `{}`; writing `{root}.{}` requires whole-record mutable access, but an active {kind} whole-record loan conflicts", record.name, field.name, range.name, field.name),
                    Some(loan.span),
                ));
            }
            Ok(())
        })();
        if result.is_err() {
            *self = before;
        }
        result
    }
    /// Read-only authorization; never accounts a use, expires, or reconstructs authority.
    fn exclusive_record_handle(
        &self,
        reference: Place,
        record: RecordId,
        span: Span,
    ) -> OwnershipResult<LoanId> {
        self.loop_handle(reference, span)?;
        if self.checked_type(reference) != Some(ValueType::MutableRef(ReferentType::Record(record)))
            || self
                .program
                .records()
                .get(record.0)
                .filter(|r| r.id == record)
                .is_none()
        {
            return Err(self.provenance_error(span));
        }
        self.available(reference, span)?;
        self.live_handle(reference, span)
    }
    /// The captured ID is operation-local continuity evidence, not an identity map.
    fn validate_reference_write_completion(
        &self,
        reference: Place,
        record: RecordId,
        original: LoanId,
        span: Span,
    ) -> OwnershipResult {
        if self.exclusive_record_handle(reference, record, span)? != original
            || !self.loans.get(&original).is_some_and(|loan| loan.held > 0)
        {
            return Err(self.provenance_error(span));
        }
        Ok(())
    }
    /// BED-025: snapshot after caller liveness/expiry, before target accounting.
    /// Retain existing authority continuously; successful RHS effects stay committed.
    fn copy_reference_field_assign(
        &mut self,
        reference: Place,
        record: RecordId,
        field: FieldId,
        value: &TypedExpr,
        reference_span: Span,
        span: Span,
    ) -> OwnershipResult {
        let before = self.clone();
        let result = (|| {
            let id = self.exclusive_record_handle(reference, record, reference_span)?;
            let declaration = self
                .program
                .fields()
                .get(field.0)
                .filter(|f| f.id == field && f.record == record)
                .filter(|_| self.program.record(record).fields.contains(&field))
                .ok_or_else(|| self.provenance_error(span))?;
            let range = declaration
                .range_type()
                .ok_or_else(|| self.provenance_error(span))?;
            if value.ty != ExprType::Range(range)
                || self
                    .program
                    .ranges()
                    .get(range.0)
                    .filter(|r| r.id == range && r.min <= r.max)
                    .is_none()
            {
                return Err(self.provenance_error(span));
            }
            let remaining = self
                .remaining
                .get_mut(&reference)
                .ok_or_else(|| before.provenance_error(reference_span))?;
            *remaining = remaining
                .checked_sub(1)
                .ok_or_else(|| before.provenance_error(reference_span))?;
            // No expiry between the target's final finite use and this retention.
            let loan = self.loans.get_mut(&id).expect("validated live target loan");
            loan.held = loan
                .held
                .checked_add(1)
                .ok_or_else(|| before.provenance_error(reference_span))?;
            self.expression(value, Transfer::WriteThrough(reference))?;
            // Availability is independent of a held loan. No additional finite use.
            self.validate_reference_write_completion(reference, record, id, reference_span)?;
            // Only now is the distinct Copy-field write authorized. No runtime
            // field state, target transfer, allocation or provenance is created.
            self.loans
                .get_mut(&id)
                .expect("revalidated retained target loan")
                .held -= 1;
            self.expire();
            Ok(())
        })();
        if result.is_err() {
            *self = before;
        }
        result
    }
    fn loop_reference_error(&self, span: Span) -> Box<Diagnostic> {
        self.diagnostic(span, "reference activity is unsupported in while conditions".into(), "recurring condition reference activity requires future cyclic loan/lifetime analysis; reference access is permitted only in the body", None)
    }
    fn complete_no_value(&self, span: Span) -> OwnershipResult {
        if self.loans.values().any(|loan| loan.held != 0) {
            return Err(self.diagnostic(span, "no-value function completion requires balanced operation holds".into(), "receiving calls and borrowed writes must complete their holds; completion never clears holds or repairs ownership", None));
        }
        Ok(())
    }
    fn bare_return(&mut self, span: Span) -> OwnershipResult<Flow> {
        self.remaining.clear();
        self.expire();
        let before = self.clone(); // Empty summary/expiry precede this snapshot.
                                   // No operand, no terminal-expression permission, no local normalization.
        let completed = self
            .check_outcome_exit(|_| true, span)
            .and_then(|_| self.complete_no_value(span));
        match completed {
            Ok(()) => {
                if let Some(custody) = &mut self.cleanup_custody {
                    custody.retire(|_| true, span, crate::effects::custody::Kind::BareReturn);
                }
                Ok(Flow::Return)
            }
            Err(error) => {
                *self = before;
                Err(error)
            }
        }
    }
    fn call_statement(
        &mut self,
        function: FunctionId,
        arguments: &[TypedExpr],
        span: Span,
    ) -> OwnershipResult {
        let before = self.clone(); // Caller prepared finite liveness and expiry.
        self.terminal_return = false;
        let result = (|| {
            self.call_arguments(function, arguments)?;
            // Unlike nested expression calls, the entire statement must balance.
            self.complete_no_value(span)
        })();
        match result {
            Ok(()) => {
                self.terminal_return = before.terminal_return;
                Ok(())
            }
            Err(error) => {
                *self = before;
                Err(error)
            }
        }
    }
    fn call_arguments(&mut self, function: FunctionId, arguments: &[TypedExpr]) -> OwnershipResult {
        let program = self.program;
        let callee = program
            .function(function)
            .as_ordinary()
            .expect("typed ordinary calls");
        self.held_arguments(arguments, |index| {
            Transfer::Argument(callee.parameters[index])
        })
    }
    fn held_arguments(
        &mut self,
        arguments: &[TypedExpr],
        destination: impl Fn(usize) -> Transfer,
    ) -> OwnershipResult {
        let mut held = Vec::new();
        for (index, argument) in arguments.iter().enumerate() {
            if let Some(loan) = self.expression(argument, destination(index))? {
                self.loans
                    .get_mut(&loan)
                    .expect("live reference provenance")
                    .held += 1;
                held.push(loan);
            }
            self.expire();
        }
        // Nested calls release only their own holds, never the receiving call's.
        for id in held {
            self.loans.get_mut(&id).expect("call-held loan").held -= 1;
        }
        self.expire();
        Ok(())
    }
    fn return_edge(
        &mut self,
        value: &TypedExpr,
        function: FunctionId,
        span: Span,
    ) -> OwnershipResult<Flow> {
        // Exact typing already happened. This finite context is static input,
        // not an operand side effect: install/expire BEFORE taking the snapshot.
        let mut uses = FutureUses::new();
        count_expression(value, &mut uses);
        self.remaining = uses;
        self.expire();
        let before = self.clone();
        self.terminal_return = true;
        let result = (|| {
            self.expression(value, Transfer::Return(function))?;
            self.check_outcome_exit(|_| true, span)?;
            // Fallible final validation belongs in the same transaction. Never
            // discharge recurrence, reset provenance/allocator or clear holds.
            if self.loans.values().any(|loan| loan.held != 0) {
                return Err(self.diagnostic(span,
                    "return cannot complete with active reference operation holds".into(),
                    "receiving calls and borrowed writes must complete their holds before return; terminal permission does not bypass loan protection", None));
            }
            Ok(Flow::Return)
        })();
        match result {
            Ok(flow) => {
                self.terminal_return = before.terminal_return;
                if let Some(custody) = &mut self.cleanup_custody {
                    custody.retire(|_| true, span, crate::effects::custody::Kind::Return);
                }
                Ok(flow) // No local successor normalization or ownership-state join.
            }
            Err(error) => {
                *self = before;
                Err(error)
            }
        }
    }
    fn loop_handle(&self, _reference: Place, span: Span) -> OwnershipResult {
        if self
            .loop_scope
            .as_ref()
            .is_some_and(|scope| scope.condition)
        {
            return Err(self.loop_reference_error(span));
        }
        Ok(())
    }
    fn loop_type(&self, ty: ExprType, span: Span) -> OwnershipResult {
        if self
            .loop_scope
            .as_ref()
            .is_some_and(|scope| scope.condition)
        {
            match ty {
                ExprType::SharedRef(_) | ExprType::MutableRef(_) => return Err(self.loop_reference_error(span)),
                ExprType::Record(_) | ExprType::Resource(_) | ExprType::Result(_) => return Err(self.diagnostic(span, "non-Copy ownership activity is unsupported in while conditions".into(), "recurring condition ownership requires future generalized cyclic ownership analysis; only the loop body permits iteration-local Move values", None)),
                _ => {}
            }
        }
        Ok(())
    }
    fn same_loop_state(&self, header: &Self) -> bool {
        // Compare availability here, not diagnostic move sites. Semantic origin
        // spans and exclusive-transfer payloads are checked by live_handle before
        // edge cleanup; display names and finite FutureUses are not loop state.
        let availability = |state: &State| std::mem::discriminant(state);
        self.function == header.function
            && self.outcomes == header.outcomes
            && self
                .states
                .iter()
                .map(|(p, s)| (p, availability(s)))
                .eq(header.states.iter().map(|(p, s)| (p, availability(s))))
            && self.handles == header.handles
            && self
                .loans
                .iter()
                .map(|(id, l)| (id, l.owner, l.origin, l.kind, l.held))
                .eq(header
                    .loans
                    .iter()
                    .map(|(id, l)| (id, l.owner, l.origin, l.kind, l.held)))
            && self.next_loan == header.next_loan
            && self.recurrent == header.recurrent
    }
    fn while_loop(
        &mut self,
        condition: &TypedExpr,
        body: &[ValueStatement],
        span: Span,
        function: FunctionId,
    ) -> OwnershipResult {
        self.assert_no_holds();
        let mut outside = self.remaining.clone();
        let mut loop_uses = future_uses(body);
        count_expression(condition, &mut loop_uses);
        subtract_uses(&mut outside, &loop_uses);
        // Establish obligations before any expiry or body operation. No missing
        // provenance may be reconstructed, even on a path skipping its body use.
        let mut iteration = self.clone();
        let mut obligations = BTreeSet::new();
        for place in reference_uses(body) {
            if let Some(id) = iteration.handles.get(&place) {
                if !iteration.loans.contains_key(id) {
                    return Err(self.diagnostic(span, "required loop-carried loan is no longer active".into(), "a carried reference must retain its original continuously active provenance; expired loans cannot be reconstructed", None));
                }
                iteration.available(place, span)?;
                iteration.live_handle(place, span)?;
                obligations.insert(*id);
            }
        }
        iteration.recurrent.push(obligations);
        iteration.expire();
        let header = Rc::new(iteration.clone());
        iteration.loop_scope = Some(LoopScope::new(&header, true));
        iteration.expression(condition, Transfer::Return(function))?;
        iteration.expire();
        iteration
            .loop_scope
            .as_mut()
            .expect("current loop")
            .condition = false;
        // Capture the normalized canonical false exit BEFORE inspecting any
        // candidate. A break can validate against it but can never define it.
        let false_exit = Rc::new(iteration.false_exit(&outside, self.loop_scope.clone()));
        iteration.targets.push(LoopTarget {
            header: header.clone(),
            exit: false_exit.clone(),
            outside: outside.clone(),
        });
        if iteration.statements_with(body, function, &outside)? == Flow::Fallthrough {
            iteration.validate_loop_backedge(&header, span)?;
            if let Some(custody) = &mut iteration.cleanup_custody {
                custody.retire(
                    |p| !header.states.contains_key(&p),
                    span,
                    crate::effects::custody::Kind::Iteration,
                );
            }
        }
        // Every terminal edge was independently checked. No arbitrary exit join:
        // the independently checked false state is the one canonical successor.
        let mut exit = (*false_exit).clone();
        if let (Some(exit_custody), Some(iteration_custody)) =
            (&mut exit.cleanup_custody, &iteration.cleanup_custody)
        {
            exit_custody.merge_sites(iteration_custody);
        }
        *self = exit;
        Ok(())
    }
    fn false_exit(&self, outside: &FutureUses, enclosing_scope: Option<LoopScope>) -> Self {
        let mut exit = self.clone();
        // Install finite outside uses before any post-discharge expiry.
        exit.remaining = outside.clone();
        exit.recurrent.pop().expect("current loop obligation frame");
        exit.loop_scope = enclosing_scope;
        exit.expire();
        exit
    }
    fn break_edge(&mut self, span: Span) -> OwnershipResult<Flow> {
        let before = self.clone();
        let target = self
            .targets
            .last()
            .expect("resolved break inside while")
            .clone();
        let result = (|| {
            self.break_integrity(&target, span)?;
            self.project_break_exit(&target, span)?;
            if !self.same_loop_state(&target.exit) {
                return Err(self.diagnostic(span, "break exit does not match canonical loop-exit ownership and loan state".into(), "break may clean target-iteration locals and end only its own recurrence; carried availability and surviving provenance cannot be repaired or replaced", None));
            }
            if let Some(custody) = &mut self.cleanup_custody {
                custody.retire(
                    |p| !target.header.states.contains_key(&p),
                    span,
                    crate::effects::custody::Kind::Break,
                );
            }
            Ok(Flow::Break)
        })();
        if result.is_err() {
            *self = before;
        }
        result
    }
    fn break_integrity(&self, target: &LoopTarget<'a>, span: Span) -> OwnershipResult {
        let header = &target.header;
        // Frame evidence would disappear at pop; validate its exact position and
        // membership (including an empty target frame) and every enclosing frame.
        if self.function != header.function
            || self.recurrent != header.recurrent
            || self.recurrent.is_empty()
        {
            return Err(self.diagnostic(span, "break exit recurrent-obligation integrity check failed".into(), "the exact target frame and every enclosing obligation must remain unchanged until break normalization", None));
        }
        let enclosing_targets_match = self.targets.len() == header.targets.len() + 1
            && self.targets.iter().zip(&header.targets).all(|(a, b)| {
                Rc::ptr_eq(&a.header, &b.header)
                    && Rc::ptr_eq(&a.exit, &b.exit)
                    && a.outside == b.outside
            });
        if !enclosing_targets_match
            || self.loop_scope != Some(LoopScope::new(header, false))
            || target.outside != target.exit.remaining
        {
            return Err(self.diagnostic(span, "break exit target-scope integrity check failed".into(), "break must retain its saved innermost target and canonical outside continuation without changing enclosing loop context", None));
        }
        if self.loans.values().any(|loan| loan.held != 0) {
            return Err(self.diagnostic(span, "break exit has unbalanced operation holds".into(), "call and write holds must complete before loop exit; normalization never clears holds to manufacture equality", None));
        }
        // Carried provenance must be continuously valid BEFORE recurrence-only
        // loans can expire. This is not whole-state header equality: fresh locals,
        // their availability, handles and loans may still exist before cleanup.
        for (place, id) in &header.handles {
            if self.handles.get(place) != Some(id) {
                return Err(self.diagnostic(span, "break exit changed carried reference provenance".into(), "a carried handle cannot be removed, refreshed or replaced, even if exit expiry would hide the change", None));
            }
        }
        for (id, original) in &header.loans {
            if !self.loans.get(id).is_some_and(|loan| {
                loan.owner == original.owner
                    && loan.origin == original.origin
                    && loan.kind == original.kind
            }) {
                return Err(self.diagnostic(span, "break exit carried-loan integrity check failed".into(), "original carried loan identity, owner, kind and active status must remain continuously valid before target-frame discharge", None));
            }
        }
        if self
            .loans
            .keys()
            .any(|id| id.0 < header.next_loan && !header.loans.contains_key(id))
        {
            return Err(self.diagnostic(
                span,
                "break exit contains resurrected carried provenance".into(),
                "expired loans may not be recreated and then erased by exit normalization",
                None,
            ));
        }
        if self.next_loan < header.next_loan {
            return Err(self.diagnostic(
                span,
                "while loop does not preserve loan allocation state".into(),
                "loan allocation must remain monotonic before break discharge",
                None,
            ));
        }
        self.validate_incoming_lineage(header, span)?;
        Ok(())
    }
    fn project_break_exit(&mut self, target: &LoopTarget<'a>, span: Span) -> OwnershipResult {
        // Restricted, non-repairing projection of THIS candidate, never a copy of
        // the canonical exit. The integrity gate must precede these reductions.
        self.check_outcome_exit(|p| !target.header.states.contains_key(&p), span)?;
        self.remaining = target.outside.clone();
        self.scope_loop_locals(&target.header);
        self.recurrent.pop().expect("validated exact target frame");
        self.targets.pop().expect("validated innermost target");
        self.loop_scope = target.header.loop_scope.clone();
        self.expire();
        self.normalize_loop_allocator(&target.header, span, "break exit")
    }
    fn continue_edge(&mut self, span: Span) -> OwnershipResult<Flow> {
        let before = self.clone();
        let target = self
            .targets
            .last()
            .expect("resolved continue inside while")
            .clone();
        // Cleanup and comparison precede (and never perform) frame discharge.
        match self.validate_loop_backedge(&target.header, span) {
            Ok(()) => {
                if let Some(custody) = &mut self.cleanup_custody {
                    custody.retire(
                        |p| !target.header.states.contains_key(&p),
                        span,
                        crate::effects::custody::Kind::Continue,
                    );
                }
                Ok(Flow::Continue)
            }
            Err(mut error) => {
                error.message = format!("continue backedge rejected: {}", error.message);
                *self = before;
                Err(error)
            }
        }
    }
    fn validate_loop_backedge(&mut self, header: &Self, span: Span) -> OwnershipResult {
        if header.handles.keys().any(|place| {
            self.handles
                .get(place)
                .is_some_and(|id| id.0 >= header.next_loan)
        }) {
            return Err(self.diagnostic(
                span,
                "iteration-created reference state survives while backedge".into(),
                "carried handles cannot be refreshed before allocator normalization",
                None,
            ));
        }
        self.validate_incoming_lineage(header, span)?;
        self.finish_loop_provenance(header, span)?;
        self.assert_no_holds();
        // This check precedes discharge of the current recurrent frame.
        if !self.same_loop_state(header) {
            return Err(self.diagnostic(span, "while loop does not preserve ownership and loan state".into(), "every accepted backedge must preserve carried availability and exact continuously active loan identity/owner/kind; general cyclic ownership/lifetime analysis remains future work", None));
        }
        Ok(())
    }
    fn finish_loop_provenance(&mut self, header: &Self, span: Span) -> OwnershipResult {
        self.check_outcome_exit(|p| !header.states.contains_key(&p), span)?;
        self.scope_loop_locals(header);
        self.expire();
        self.normalize_loop_allocator(header, span, "while backedge")
    }
    fn normalize_loop_allocator(
        &mut self,
        header: &Self,
        span: Span,
        edge: &str,
    ) -> OwnershipResult {
        // Never roll the allocator back until every new loan and every surviving
        // handle referring to newly allocated provenance is proven absent.
        if self.loans.keys().any(|id| !header.loans.contains_key(id))
            || self.handles.values().any(|id| id.0 >= header.next_loan)
        {
            return Err(self.diagnostic(span, format!("iteration-created reference state survives {edge}"), "all newly created provenance must be discharged before allocator normalization; allocator reuse cannot hide surviving handles or loans", None));
        }
        if self.next_loan < header.next_loan {
            return Err(self.diagnostic(span, "while loop does not preserve loan allocation state".into(), "loan allocation must remain monotonic during an iteration; normalization is permitted only after proving newly allocated provenance absent", None));
        }
        self.next_loan = header.next_loan;
        Ok(())
    }
    fn scope_loop_locals(&mut self, header: &Self) {
        // Static body-local IDs may evolve within this iteration, but do not
        // become header state. This is lexical cleanup, not runtime destruction.
        self.states
            .retain(|place, _| header.states.contains_key(place));
        self.handles
            .retain(|place, _| header.handles.contains_key(place));
    }
    fn conditional(
        &mut self,
        condition: &TypedExpr,
        then_body: &[ValueStatement],
        else_body: &[ValueStatement],
        function: FunctionId,
        after: &FutureUses,
    ) -> OwnershipResult<Flow> {
        self.expression(condition, Transfer::Return(function))?;
        self.expire();
        self.assert_no_holds();
        let mut yes = self.clone();
        let mut no = self.clone();
        let yes_flow = yes.statements_with(then_body, function, after)?;
        let no_flow = no.statements_with(else_body, function, after)?;
        self.finish_branches(
            yes,
            no,
            yes_flow,
            no_flow,
            then_body,
            else_body,
            after,
            condition.span,
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn finish_branches(
        &mut self,
        mut yes: Self,
        mut no: Self,
        yes_flow: Flow,
        no_flow: Flow,
        then_body: &[ValueStatement],
        else_body: &[ValueStatement],
        after: &FutureUses,
        span: Span,
    ) -> OwnershipResult<Flow> {
        for (branch, flow, body) in [
            (&mut yes, yes_flow, then_body),
            (&mut no, no_flow, else_body),
        ] {
            if flow == Flow::Fallthrough {
                branch.check_outcome_exit(
                    |p| !self.states.contains_key(&p),
                    body.last().map(statement_span).unwrap_or(span),
                )?;
                if let Some(custody) = &mut branch.cleanup_custody {
                    let origin = body.last().map(statement_span).unwrap_or(span);
                    custody.retire(
                        |p| !self.states.contains_key(&p),
                        origin,
                        crate::effects::custody::Kind::Fallthrough,
                    );
                }
            }
        }
        let mut merged_custody = self.cleanup_custody.clone();
        if let (Some(merged), Some(a), Some(b)) = (
            &mut merged_custody,
            &yes.cleanup_custody,
            &no.cleanup_custody,
        ) {
            merged.branches(
                a,
                b,
                yes_flow == Flow::Fallthrough,
                no_flow == Flow::Fallthrough,
            );
        }
        match (yes_flow, no_flow) {
            (a, b) if a != Flow::Fallthrough && b != Flow::Fallthrough => {
                self.cleanup_custody = merged_custody;
                return Ok(if a == b { a } else { Flow::NoFallthrough });
            }
            (Flow::Fallthrough, Flow::Fallthrough) => self.join(yes, no, after.clone(), span)?,
            _ => {
                let mut survivor = if yes_flow == Flow::Fallthrough {
                    yes
                } else {
                    no
                };
                survivor.assert_no_holds();
                survivor.scope_loop_locals(self);
                survivor.expire();
                *self = survivor;
            }
        }
        self.cleanup_custody = merged_custody;
        Ok(Flow::Fallthrough)
    }
    fn split(&self, then_uses: &FutureUses, else_uses: &FutureUses) -> (Self, Self, FutureUses) {
        let mut continuation = self.remaining.clone();
        subtract_uses(&mut continuation, then_uses);
        subtract_uses(&mut continuation, else_uses);
        self.assert_no_holds();
        let mut then_state = self.clone();
        let mut else_state = self.clone();
        then_state.remaining = with_branch_uses(&continuation, then_uses);
        else_state.remaining = with_branch_uses(&continuation, else_uses);
        then_state.expire();
        else_state.expire();
        (then_state, else_state, continuation)
    }
    fn join(
        &mut self,
        then_state: Self,
        else_state: Self,
        continuation: FutureUses,
        span: Span,
    ) -> OwnershipResult {
        self.outcomes = then_state
            .outcomes
            .iter()
            .chain(&else_state.outcomes)
            .filter(|(p, _)| self.states.contains_key(p))
            .map(|(p, o)| {
                let mut fact = *o;
                if !then_state.outcomes.contains_key(p) || !else_state.outcomes.contains_key(p) {
                    fact.conditional = Some(span);
                }
                (*p, fact)
            })
            .collect();
        then_state.validate_incoming_lineage(self, span)?;
        else_state.validate_incoming_lineage(self, span)?;
        then_state.assert_no_holds();
        else_state.assert_no_holds();
        if let (Some(merged), Some(a), Some(b)) = (
            &mut self.cleanup_custody,
            &then_state.cleanup_custody,
            &else_state.cleanup_custody,
        ) {
            merged.branches(a, b, true, true);
        }
        // Branch-created loans are scoped away, never merged. An incoming
        // loan's canonical origin/authority must not change on either survivor.
        for (id, original) in &self.loans {
            for incoming in [&then_state, &else_state] {
                if incoming.loans.get(id).is_some_and(|loan| {
                    loan.owner != original.owner
                        || loan.origin != original.origin
                        || loan.kind != original.kind
                }) {
                    return Err(self.provenance_error(span));
                }
            }
        }
        for (place, state) in &mut self.states {
            if *state == State::Available {
                let moved = [then_state.states.get(place), else_state.states.get(place)]
                    .into_iter()
                    .flatten()
                    .find_map(|state| match state {
                        State::Moved { span, .. } | State::ConditionalMove { span } => Some(*span),
                        State::Available => None,
                    });
                if let Some(span) = moved {
                    *state = State::ConditionalMove { span };
                }
            }
        }
        // Only provenance present before the split may survive. Sibling-created
        // loan IDs can overlap privately, but are never merged or allowed to
        // escape through branch-local handles. Activity is a union, not an
        // intersection: a loan required on any incoming path protects the join.
        self.loans
            .retain(|id, _| then_state.loans.contains_key(id) || else_state.loans.contains_key(id));
        self.remaining = continuation;
        self.next_loan = then_state.next_loan.max(else_state.next_loan);
        self.assert_no_holds();
        self.expire();
        Ok(())
    }
    fn assert_no_holds(&self) {
        assert!(
            self.loans.values().all(|loan| loan.held == 0),
            "operation holds cannot escape a statement or branch"
        );
    }
    fn expression(
        &mut self,
        expression: &TypedExpr,
        transfer: Transfer,
    ) -> OwnershipResult<Option<LoanId>> {
        let before = self.clone();
        match self.expression_inner(expression, transfer) {
            Ok(loan) => Ok(loan),
            Err(error) => {
                *self = before;
                Err(error)
            }
        }
    }
    fn expression_inner(
        &mut self,
        expression: &TypedExpr,
        transfer: Transfer,
    ) -> OwnershipResult<Option<LoanId>> {
        if self.loop_scope.as_ref().is_some_and(|s| s.condition)
            && matches!(
                expression.kind,
                ExprKind::RecordConstruct { .. } | ExprKind::ResultConstruct { .. }
            )
        {
            return Err(self.diagnostic(expression.span, "record construction is unsupported in recurring while conditions".into(), "construction produces a non-Copy record; use construction only outside the recurring condition", None));
        }
        if self.loop_scope.is_some() {
            self.loop_type(expression.ty, expression.span)?;
        }
        match &expression.kind {
            ExprKind::ResultConstruct {
                result,
                tag,
                payload,
                ..
            } => {
                self.witness(
                    expression.span,
                    outcomes::Action::Construct {
                        result: *result,
                        tag: *tag,
                        payload: payload.span,
                    },
                )?;
                self.expression(payload, Transfer::ResultPayload(*result, *tag))?;
                Ok(None)
            }
            ExprKind::RecordConstruct { fields, .. } => {
                for entry in fields {
                    self.expression(&entry.value, transfer)?;
                }
                Ok(None) // Fresh record; no loan, harness binding or anonymous local.
            }
            ExprKind::CopyReferenceFieldRead {
                reference, record, ..
            } => {
                self.reference_field_read(*reference, *record, expression.span)?;
                Ok(None) // Copied range, never a reference-related receiving hold.
            }
            ExprKind::CopyFieldRead { owner, .. } => {
                self.copy_field_read(*owner, expression.span)?;
                Ok(None)
            }
            ExprKind::IfValue {
                condition,
                then_value,
                else_value,
            } => {
                self.expression(condition, transfer)?;
                self.expire();
                let mut then_uses = FutureUses::new();
                let mut else_uses = FutureUses::new();
                count_expression(then_value, &mut then_uses);
                count_expression(else_value, &mut else_uses);
                let (mut then_state, mut else_state, continuation) =
                    self.split(&then_uses, &else_uses);
                // Every leaf transfers into the same source local. Reference
                // results have already been rejected by typing.
                assert!(then_state.expression(then_value, transfer)?.is_none());
                then_state.expire();
                assert!(else_state.expression(else_value, transfer)?.is_none());
                else_state.expire();
                self.join(then_state, else_state, continuation, expression.span)?;
                Ok(None)
            }
            ExprKind::Parameter(id) => {
                self.consume(Place::Parameter(*id), expression.span, transfer)
            }
            ExprKind::Local(id) => self.consume(Place::Local(*id), expression.span, transfer),
            ExprKind::Deref { reference } => {
                self.borrowed_access(*reference, expression.span, false)?;
                self.expire();
                // The result is a Copy referent, not a reference argument/handle.
                Ok(None)
            }
            ExprKind::Borrow { kind, place } => {
                self.borrow(*place, *kind, expression.span).map(Some)
            }
            ExprKind::Call {
                function,
                arguments,
            } => {
                self.call_arguments(*function, arguments)?;
                // Reference returns are forbidden: call results are fresh owned values.
                Ok(None)
            }
            ExprKind::TrustedCall {
                operation,
                arguments,
            } => {
                self.held_arguments(arguments, |index| {
                    Transfer::TrustedArgument(*operation, index)
                })?;
                if self
                    .program
                    .trusted
                    .as_ref()
                    .and_then(|c| c.operation(*operation))
                    .is_some_and(|op| op.class == crate::trusted::OperationClass::Completion)
                {
                    if let Some(custody) = &mut self.cleanup_custody {
                        custody.completion(*operation, expression.span);
                    }
                }
                Ok(None)
            }
            ExprKind::Unary { operand, .. } => {
                self.expression(operand, transfer)?;
                Ok(None)
            }
            ExprKind::Binary { op, left, right } => {
                self.expression(left, transfer)?;
                let skip_outcomes = self.outcomes.clone();
                let skip = self.cleanup_custody.clone();
                self.expression(right, transfer)?;
                if matches!(op, crate::hir::BinaryOp::And | crate::hir::BinaryOp::Or) {
                    for (place, mut fact) in skip_outcomes {
                        if !self.outcomes.contains_key(&place) {
                            fact.conditional = Some(expression.span);
                        }
                        self.outcomes.insert(place, fact);
                    }
                    if let (Some(custody), Some(skip)) = (&mut self.cleanup_custody, &skip) {
                        custody.retain_skip(skip);
                    }
                }
                Ok(None)
            }
            ExprKind::BoolLiteral(_)
            | ExprKind::IntegerLiteral(_)
            | ExprKind::FormedRangeLiteral(_)
            | ExprKind::FieldAccess(_)
            | ExprKind::OldField(_) => Ok(None),
        }
    }
    fn copy_field_read(&mut self, owner: Place, span: Span) -> OwnershipResult {
        self.available(owner, span)?;
        // Unlike whole-record consume, immutable Copy access permits shared loans.
        if let Some(loan) = self.conflict(owner, Some(BorrowKind::Shared)) {
            return Err(self.diagnostic(span, format!("cannot read field of `{}` while it is exclusively borrowed", self.metadata(owner).0), "a Copy-field read requires an available owned root with no active exclusive whole-record loan", Some(loan.span)));
        }
        if let Some(remaining) = self.remaining.get_mut(&owner) {
            *remaining -= 1;
        }
        Ok(())
    }
    fn borrowed_access(
        &mut self,
        reference: Place,
        span: Span,
        write: bool,
    ) -> OwnershipResult<LoanId> {
        self.loop_handle(reference, span)?;
        self.available(reference, span)?;
        let referent = match self.metadata(reference).1 {
            ValueType::SharedRef(ty) | ValueType::MutableRef(ty) => ty.value_type(),
            _ => unreachable!("typed borrowed access uses reference values"),
        };
        if ownership_kind(referent) == OwnershipKind::Move {
            let ValueType::Record(id) = referent else {
                unreachable!("non-Copy referents are records")
            };
            let name = &self.program.record(id).name;
            let message = if write {
                format!("cannot replace non-Copy value `{name}` through mutable reference before destruction semantics are defined")
            } else {
                format!("cannot move non-Copy value `{name}` out through borrowed reference")
            };
            return Err(self.diagnostic(span, message, "only Copy referents may be materialized or replaced through references; no cloning, displacement, or destruction is implicit", None));
        }
        let id = self.live_handle(reference, span)?;
        if let Some(remaining) = self.remaining.get_mut(&reference) {
            *remaining -= 1;
        }
        Ok(id)
    }
    fn reference_field_read(
        &mut self,
        reference: Place,
        record: RecordId,
        span: Span,
    ) -> OwnershipResult {
        self.loop_handle(reference, span)?;
        let ty = self
            .checked_type(reference)
            .ok_or_else(|| self.provenance_error(span))?;
        if !matches!(ty, ValueType::SharedRef(ReferentType::Record(id)) | ValueType::MutableRef(ReferentType::Record(id)) if id == record)
        {
            return Err(self.provenance_error(span));
        }
        if self
            .program
            .records()
            .get(record.0)
            .filter(|r| r.id == record)
            .is_none()
        {
            return Err(self.provenance_error(span));
        }
        self.available(reference, span)?;
        self.live_handle(reference, span)?;
        if let Some(remaining) = self.remaining.get_mut(&reference) {
            *remaining -= 1;
        }
        self.expire();
        Ok(())
    }
    fn checked_type(&self, place: Place) -> Option<ValueType> {
        checked_place_type(self.program, self.function, place)
    }
    fn provenance_error(&self, span: Span) -> Box<Diagnostic> {
        self.diagnostic(span, "reference access has invalid loan provenance or lineage".into(),
            "the canonical handle must retain its original referent, shared/exclusive permission and borrow origin; compatible loans are not interchangeable", None)
    }
    /// Validate usable incoming references before convergence erases evidence.
    /// The entry mapping selects relevant historical handles only; it never
    /// supplies or replaces the candidate's authoritative active LoanId.
    fn validate_incoming_lineage(&self, entry: &Self, span: Span) -> OwnershipResult {
        let places: BTreeSet<_> = self
            .handles
            .keys()
            .chain(entry.handles.keys())
            .copied()
            .collect();
        for place in places {
            // Copy handles have no availability entry. Moved/conditional handles
            // remain historical lineage, not independently usable references.
            if matches!(
                self.states.get(&place),
                Some(State::Moved { .. } | State::ConditionalMove { .. })
            ) {
                continue;
            }
            let active = |id: &LoanId| self.loans.contains_key(id);
            if self.handles.get(&place).is_some_and(active)
                || entry.handles.get(&place).is_some_and(active)
                || self.remaining.get(&place).copied().unwrap_or(0) > 0
            {
                self.live_handle(place, span)?;
            }
        }
        Ok(())
    }
    fn live_handle(&self, place: Place, span: Span) -> OwnershipResult<LoanId> {
        let id = self.handles.get(&place).copied().filter(|id| id.0 < self.next_loan && self.loans.contains_key(id))
            .ok_or_else(|| self.diagnostic(span, format!("reference `{}` no longer has an active loan", match place { Place::Parameter(id) => self.program.parameters().get(id.0).map(|p| p.name.as_str()), Place::Local(id) => self.program.locals().get(id.0).map(|p| p.name.as_str()) }.unwrap_or("invalid reference")), "reference access requires continuously active original provenance; loans cannot resurrect", None))?;
        let loan = &self.loans[&id];
        let (kind, referent) = match self.checked_type(place) {
            Some(ValueType::SharedRef(ty)) => (BorrowKind::Shared, ty),
            Some(ValueType::MutableRef(ty)) => (BorrowKind::Mutable, ty),
            _ => return Err(self.provenance_error(span)),
        };
        if loan.kind != kind {
            return Err(self.provenance_error(span));
        }
        // Inspect the existing canonical initializer lineage. This is not a
        // second expected-LoanId map: handles remains the active-loan authority.
        let mut seen = BTreeSet::new();
        let (owner, origin, origin_span) = self
            .reference_origin(place, id, &mut seen)
            .ok_or_else(|| self.provenance_error(span))?;
        if loan.owner != owner || loan.origin != origin || loan.span != origin_span {
            return Err(self.provenance_error(span));
        }
        if let Some(owner) = owner {
            if self.checked_type(owner) != Some(referent.value_type()) {
                return Err(self.provenance_error(span));
            }
            self.available(owner, span)?;
        } else {
            let LoanOrigin::ExternalParameter(origin) = origin else {
                return Err(self.provenance_error(span));
            };
            if self.checked_type(Place::Parameter(origin))
                != Some(match kind {
                    BorrowKind::Shared => ValueType::SharedRef(referent),
                    BorrowKind::Mutable => ValueType::MutableRef(referent),
                })
            {
                return Err(self.provenance_error(span));
            }
            // A transferred external mutable parameter need not remain available.
        }
        Ok(id)
    }
    fn reference_origin(
        &self,
        place: Place,
        loan: LoanId,
        seen: &mut BTreeSet<Place>,
    ) -> Option<(Option<Place>, LoanOrigin, Span)> {
        #[cfg(test)]
        initializer_probe::update(|c| {
            c.origin_calls += 1;
            c.max_depth = c.max_depth.max(seen.len() + 1);
        });
        if self.context.is_some_and(|c| {
            !c.lineage(seen.len() + 1, self.program.function(self.function).span())
        }) {
            return None;
        }
        if self.initializers.invalid.is_some() {
            return None;
        }
        if !seen.insert(place) || self.handles.get(&place) != Some(&loan) {
            return None;
        }
        let ty = self.checked_type(place)?;
        match place {
            Place::Parameter(id) => Some((
                None,
                LoanOrigin::ExternalParameter(id),
                self.program.parameters().get(id.0)?.span,
            )),
            Place::Local(id) => {
                let value = self.initializers.get(self.function, id)?;
                if value.ty != ExprType::from(ty) {
                    return None;
                }
                let source = match value.kind {
                    ExprKind::Borrow { kind, place: owner } => {
                        let referent = match self.checked_type(owner)? {
                            ValueType::Bool => ReferentType::Bool,
                            ValueType::Range(id) => ReferentType::Range(id),
                            ValueType::Record(id) => ReferentType::Record(id),
                            ValueType::Resource(id) => ReferentType::Resource(id),
                            _ => return None,
                        };
                        if kind == BorrowKind::Mutable
                            && !matches!(owner, Place::Local(id) if self.program.locals().get(id.0)?.mutable)
                        {
                            return None;
                        }
                        let expected = match kind {
                            BorrowKind::Shared => ValueType::SharedRef(referent),
                            BorrowKind::Mutable => ValueType::MutableRef(referent),
                        };
                        if ty != expected {
                            return None;
                        }
                        return Some((Some(owner), LoanOrigin::LocalBinding(id), value.span));
                    }
                    ExprKind::Parameter(source) => Place::Parameter(source),
                    ExprKind::Local(source) => Place::Local(source),
                    _ => return None,
                };
                if self.checked_type(source) != Some(ty) {
                    return None;
                }
                if matches!(ty, ValueType::SharedRef(_))
                    && matches!(
                        self.states.get(&source),
                        Some(State::Moved { .. } | State::ConditionalMove { .. })
                    )
                {
                    return None;
                }
                if matches!(ty, ValueType::MutableRef(_))
                    && !matches!(self.states.get(&source), Some(State::Moved { transfer: Transfer::Local(target), .. }) if *target == id)
                {
                    return None;
                }
                self.reference_origin(source, loan, seen)
            }
        }
    }
    fn metadata(&self, place: Place) -> (&str, ValueType) {
        match place {
            Place::Parameter(id) => {
                let parameter = self.program.parameter(id);
                let ParameterType::Value(ty) = parameter.ty else {
                    unreachable!("ordinary places use value parameters")
                };
                (&parameter.name, ty)
            }
            Place::Local(id) => {
                let local = self.program.local(id);
                (&local.name, local.ty)
            }
        }
    }
    fn add_loan(&mut self, owner: Option<Place>, kind: BorrowKind, span: Span) -> LoanId {
        let id = LoanId(self.next_loan);
        self.next_loan += 1;
        self.loans.insert(
            id,
            Loan {
                owner,
                origin: LoanOrigin::Unbound,
                kind,
                span,
                held: 0,
            },
        );
        id
    }
    fn expire(&mut self) {
        let handles = &self.handles;
        let remaining = &self.remaining;
        let states = &self.states;
        let recurrent = &self.recurrent;
        self.loans.retain(|id, loan| {
            loan.held > 0
                || recurrent.iter().any(|frame| frame.contains(id))
                || handles.iter().any(|(place, handle)| {
                    handle == id
                        && remaining.get(place).copied().unwrap_or(0) > 0
                        // ConditionalMove is unusable, but can carry an active
                        // loan on a path where the handle was not transferred.
                        && !matches!(
                            states.get(place),
                            Some(State::Moved { .. })
                        )
                })
        });
    }
    fn conflict(&self, place: Place, requested: Option<BorrowKind>) -> Option<&Loan> {
        let copy = ownership_kind(self.metadata(place).1) == OwnershipKind::Copy;
        self.loans.values().find(|loan| {
            loan.owner == Some(place)
                && match requested {
                    Some(BorrowKind::Shared) => loan.kind == BorrowKind::Mutable,
                    Some(BorrowKind::Mutable) => true,
                    None => !copy || loan.kind == BorrowKind::Mutable,
                }
        })
    }
    fn borrow(&mut self, place: Place, kind: BorrowKind, span: Span) -> OwnershipResult<LoanId> {
        if self
            .loop_scope
            .as_ref()
            .is_some_and(|scope| scope.condition)
        {
            return Err(self.loop_reference_error(span));
        }
        self.available(place, span)?;
        if kind == BorrowKind::Mutable
            && !matches!(place, Place::Local(id) if self.program.local(id).mutable)
        {
            return Err(self.diagnostic(span, format!("cannot mutably borrow immutable value `{}`", self.metadata(place).0), "exclusive borrowing requires a mutable owned local declared with let mut; ordinary owned parameters are immutable", None));
        }
        if let Some(loan) = self.conflict(place, Some(kind)) {
            let name = self.metadata(place).0;
            let message = match (kind, loan.kind) {
                (BorrowKind::Mutable, BorrowKind::Shared) => format!(
                    "cannot create exclusive borrow of `{name}` while shared borrow is active"
                ),
                (BorrowKind::Shared, BorrowKind::Mutable) => format!(
                    "cannot create shared borrow of `{name}` while exclusive borrow is active"
                ),
                (BorrowKind::Mutable, BorrowKind::Mutable) => {
                    format!("cannot create a second exclusive borrow of `{name}`")
                }
                _ => unreachable!("shared loans may overlap"),
            };
            return Err(self.diagnostic(
                span,
                message,
                "exclusive loans conflict with every overlapping loan",
                Some(loan.span),
            ));
        }
        Ok(self.add_loan(Some(place), kind, span))
    }
    fn consume(
        &mut self,
        place: Place,
        span: Span,
        transfer: Transfer,
    ) -> OwnershipResult<Option<LoanId>> {
        if self.replacement_target == Some(place) {
            return Err(self.diagnostic(span, format!("cannot consume replacement target `{}` inside its own RHS", self.metadata(place).0), "the displaced whole owner must remain available throughout RHS evaluation; self-consuming transformations require a future take/swap facility", None));
        }
        if matches!(
            self.metadata(place).1,
            ValueType::SharedRef(_) | ValueType::MutableRef(_)
        ) {
            self.loop_handle(place, span)?;
            if matches!(self.metadata(place).1, ValueType::MutableRef(_))
                && self.loop_scope.as_ref().is_some_and(|scope| {
                    scope.header_handles.contains(&place)
                        || self
                            .handles
                            .get(&place)
                            .is_some_and(|id| scope.header_loans.contains(id))
                })
            {
                return Err(self.diagnostic(span, format!("cannot move loop-carried mutable reference `{}`", self.metadata(place).0), "a carried mutable handle must preserve pre-existing availability and unchanged provenance on loop backedges and canonical exits; by-value arguments remain moves, not implicit reborrows", None));
            }
        }
        if self
            .loop_scope
            .as_ref()
            .is_some_and(|scope| scope.header_places.contains(&place))
            && matches!(
                self.metadata(place).1,
                ValueType::Record(_) | ValueType::Resource(_) | ValueType::Result(_)
            )
            && !self.terminal_return
        {
            return Err(self.diagnostic(span, format!("cannot move pre-existing non-Copy value `{}` in while loop", self.metadata(place).0), "the conservative pre-existing-state boundary applies to loop backedges and canonical exits alike; terminal break paths do not relax it; outside the actual return-expression tree, only iteration-local Move values may transfer in the loop body; future generalized cyclic ownership analysis remains unresolved", None));
        }
        self.available(place, span)?;
        if matches!(
            self.metadata(place).1,
            ValueType::SharedRef(_) | ValueType::MutableRef(_)
        ) {
            self.live_handle(place, span)?;
        }
        if let Some(loan) = self.conflict(place, None) {
            let name = self.metadata(place).0;
            let message = if loan.kind == BorrowKind::Mutable {
                format!("cannot use `{name}` while it is exclusively borrowed")
            } else {
                format!("cannot move `{name}` while it is shared-borrowed")
            };
            return Err(self.diagnostic(span, message, "non-Copy owners cannot move while borrowed; exclusive loans also prevent direct Copy owner access", Some(loan.span)));
        }
        if ownership_kind(self.metadata(place).1) == OwnershipKind::Move {
            self.states.insert(place, State::Moved { span, transfer });
            if matches!(self.metadata(place).1, ValueType::Result(_)) {
                let ty = match self.metadata(place).1 {
                    ValueType::Result(ty) => ty,
                    _ => unreachable!(),
                };
                self.witness(
                    span,
                    outcomes::Action::Transfer {
                        place,
                        result: ty,
                        destination: transfer,
                    },
                )?;
                if self.outcomes.remove(&place).is_none() {
                    return Err(self.diagnostic(
                        span,
                        "missing Result outcome authority".into(),
                        "a successful transfer must carry its independent unresolved duty",
                        None,
                    ));
                }
            }
            if let Some(custody) = &mut self.cleanup_custody {
                custody.transfer(place);
            }
        }
        if let Some(remaining) = self.remaining.get_mut(&place) {
            *remaining -= 1;
        }
        Ok(self.handles.get(&place).copied())
    }
    fn diagnostic(
        &self,
        span: Span,
        message: String,
        required: &str,
        earlier: Option<Span>,
    ) -> Box<Diagnostic> {
        Diagnostic {
            message,
            span,
            required: required.into(),
            known: earlier
                .map(|s| {
                    format!(
                        "the conflicting borrow began at line {}, column {}",
                        s.line, s.column
                    )
                })
                .into_iter()
                .collect(),
            conclusion: "ownership checking stopped; this program has not been established valid"
                .into(),
        }
        .into()
    }
    fn available(&self, place: Place, span: Span) -> OwnershipResult {
        let name = self.metadata(place).0;
        if let Some(State::ConditionalMove { span: earlier }) = self.states.get(&place) {
            return Err(
                Diagnostic {
                    message: format!(
                        "value `{name}` may have been moved on a previous control-flow path"
                    ),
                    span,
                    required: "a non-Copy place must remain available on every incoming branch"
                        .into(),
                    known: vec![format!(
                        "a move occurred at line {}, column {}",
                        earlier.line, earlier.column
                    )],
                    conclusion:
                        "ownership checking stopped; this program has not been established valid"
                            .into(),
                }
                .into(),
            );
        }
        if let Some(State::Moved {
            span: earlier,
            transfer: previous,
        }) = self.states.get(&place)
        {
            let destination = match previous {
                Transfer::Match => "by exhaustive Result match".into(),
                Transfer::Propagate(local) => format!(
                    "by or return into success local `{}`",
                    self.program.local(*local).name
                ),
                Transfer::ResultPayload(_, _) => "into a Result payload".into(),
                Transfer::WriteThrough(reference) => {
                    format!("through reference `{}`", self.metadata(*reference).0)
                }
                Transfer::Local(id) => format!("into local `{}`", self.program.local(*id).name),
                Transfer::Argument(id) => format!(
                    "into by-value parameter `{}`",
                    self.program.parameter(*id).name
                ),
                Transfer::TrustedArgument(_, index) => {
                    format!("into trusted argument #{}", index + 1)
                }
                Transfer::Return(id) => format!(
                    "by return from function `{}`",
                    self.program.function(*id).name()
                ),
            };
            return Err(
                Diagnostic {
                    message: format!("use of moved value `{name}`"),
                    span,
                    required: "a non-Copy value cannot be reused after transfer; use its new owner"
                        .into(),
                    known: vec![format!(
                        "`{name}` was previously moved {destination} at line {}, column {}",
                        earlier.line, earlier.column
                    )],
                    conclusion:
                        "ownership checking stopped; this program has not been established valid"
                            .into(),
                }
                .into(),
            );
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{compile_source, parse_source, resolve, types};
    const DECLARATIONS: &str = "type Percent = range 0..100; record Ticket { value: Percent } record Other { value: Percent }";
    fn typed(body: &str) -> Program {
        let ast = parse_source(&format!("{DECLARATIONS} {body}")).unwrap();
        types::check(resolve::resolve(&ast).unwrap()).unwrap()
    }
    #[test]
    fn replacement_transfers_rhs_and_preserves_target_availability() {
        let source = "type Percent = range 0..100; record Ticket { value: Percent } fn f(first: Ticket, second: Ticket) -> Ticket { let mut current = first; current = second; return current; }";
        let program =
            types::check(resolve::resolve(&parse_source(source).unwrap()).unwrap()).unwrap();
        let function = program.functions()[0].as_ordinary().unwrap();
        let mut checker = Checker::new(&program, function);
        checker
            .statements(&function.body[..1], function.id)
            .unwrap();
        checker
            .statements(&function.body[1..2], function.id)
            .unwrap();
        assert_eq!(
            checker.states[&Place::Local(program.locals()[0].id)],
            State::Available
        );
        assert!(matches!(
            checker.states[&Place::Parameter(function.parameters[1])],
            State::Moved { .. }
        ));
        assert_eq!(checker.replacement_target, None);
        assert!(checker.loans.is_empty());
    }

    #[test]
    fn replacement_failure_restores_complete_prepared_state() {
        let p = typed("fn receive(a: Ticket,v: &bool,b: Ticket) -> Ticket { return a; } fn f(first: Ticket,other: Ticket,flag: bool) -> Ticket { let mut target=first; target=receive(other,&flag,target); return target; }");
        let f = p.functions().last().unwrap().as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::Assign { local, value, span } = &f.body[1] else {
            panic!()
        };
        let before = c.clone();
        assert!(c.replace_owned(*local, value, *span).is_err());
        assert_atomic(&c, &before);
    }

    #[test]
    fn replacement_final_loan_failure_rolls_back_successful_rhs_transfer() {
        let p = typed("fn receive(a: Ticket,v: &Ticket) -> Ticket { return a; } fn inspect(v: &Ticket) {} fn f(first: Ticket,other: Ticket) -> Ticket { let mut target=first; let view=&target; target=receive(other,view); inspect(view); return target; }");
        let f = p.functions().last().unwrap().as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::Assign { local, value, span } = &f.body[2] else {
            panic!()
        };
        let before = c.clone();
        let mut prefix = c.clone();
        prefix.expression(value, Transfer::Local(*local)).unwrap();
        assert_ne!(prefix.states, before.states);
        assert!(c
            .replace_owned(*local, value, *span)
            .unwrap_err()
            .message
            .contains("while it is borrowed"));
        assert_atomic(&c, &before);
    }

    #[test]
    fn classification_uses_canonical_value_types_not_aggregate_fields_or_names() {
        let mut program = compile_source(DECLARATIONS).unwrap();
        for range in &mut program.ranges {
            range.name = "display".into();
        }
        for record in &mut program.records {
            record.name = "display".into();
        }
        assert_eq!(ownership_kind(ValueType::Bool), OwnershipKind::Copy);
        assert_eq!(
            ownership_kind(ValueType::Range(program.ranges()[0].id)),
            OwnershipKind::Copy
        );
        for record in program.records() {
            assert_eq!(
                ownership_kind(ValueType::Record(record.id)),
                OwnershipKind::Move
            );
        }
    }
    #[test]
    fn parameters_start_available_and_return_consumes_the_new_local_owner() {
        let program = typed("fn forward(ticket: Ticket, unused: Ticket) -> Ticket { let next = ticket; return next; }");
        let f = program.functions()[0].as_ordinary().unwrap();
        let mut checker = Checker::new(&program, f);
        assert_eq!(
            checker.states[&Place::Parameter(f.parameters[0])],
            State::Available
        );
        assert_eq!(
            checker.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
        checker.body(f).unwrap();
        let local = program.locals()[0].id;
        assert!(
            matches!(checker.states[&Place::Parameter(f.parameters[0])], State::Moved { transfer: Transfer::Local(id), .. } if id == local)
        );
        assert!(
            matches!(checker.states[&Place::Local(local)], State::Moved { transfer: Transfer::Return(id), .. } if id == f.id)
        );
        assert_eq!(
            checker.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
    }
    #[test]
    fn ownership_state_is_isolated_to_each_function_and_copy_values_are_not_consumed() {
        let program = typed("fn first(ticket: Ticket) -> Ticket { return ticket; } fn second(ticket: Ticket, value: Percent) -> Ticket { let first = value; let second = value; return ticket; }");
        let first = program.functions()[0].as_ordinary().unwrap();
        let second = program.functions()[1].as_ordinary().unwrap();
        let mut checker = Checker::new(&program, first);
        checker.body(first).unwrap();
        let mut checker = Checker::new(&program, second);
        assert!(!checker
            .states
            .contains_key(&Place::Parameter(first.parameters[0])));
        checker.body(second).unwrap();
        assert_eq!(checker.states.len(), 1);
        assert!(!checker
            .states
            .contains_key(&Place::Parameter(second.parameters[1])));
        for local in program.locals() {
            assert!(!checker.states.contains_key(&Place::Local(local.id)));
        }
    }
    #[test]
    fn parameter_local_function_and_type_metadata_renaming_preserves_ownership() {
        fn rename(program: &mut Program) {
            for range in &mut program.ranges {
                range.name = "display".into();
            }
            for record in &mut program.records {
                record.name = "display".into();
            }
            for parameter in &mut program.parameters {
                parameter.name = "display".into();
            }
            for local in &mut program.locals {
                local.name = "display".into();
            }
            for f in &mut program.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        let identity = "fn identity(ticket: Ticket) -> Ticket { return ticket; }";
        let valid = format!("{identity} fn forward(ticket: Ticket) -> Ticket {{ let first = identity(ticket); let next = first; return next; }}");
        let mut program = typed(&valid);
        rename(&mut program);
        check(&program).unwrap();
        for body in ["fn bad(ticket: Ticket) -> Ticket { let next = ticket; return ticket; }", "fn bad(ticket: Ticket) -> Ticket { let first = ticket; let next = first; return first; }", "fn bad(ticket: Ticket) -> Ticket { let next = identity(ticket); return ticket; }"] {
            let mut program = typed(&format!("{identity} {body}"));
            let original = check(&program).unwrap_err();
            rename(&mut program);
            let renamed = check(&program).unwrap_err();
            assert_eq!(renamed.len(), 1);
            assert_eq!(original[0].span, renamed[0].span);
            assert_eq!(renamed[0].message, "use of moved value `display`");
            assert!(renamed[0].known[0].contains("`display`"));
        }
    }
    #[test]
    fn loan_provenance_liveness_and_conflicts_ignore_all_display_names() {
        fn rename(program: &mut Program) {
            for range in &mut program.ranges {
                range.name = "display".into();
            }
            for record in &mut program.records {
                record.name = "display".into();
            }
            for field in &mut program.fields {
                field.name = "display".into();
            }
            for parameter in &mut program.parameters {
                parameter.name = "display".into();
            }
            for local in &mut program.locals {
                local.name = "display".into();
            }
            for f in &mut program.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        let declarations = "fn inspect(value: &Ticket) -> bool { return true; } fn exclusive(value: &mut Ticket) -> bool { return true; }";
        for body in [
            "let view = &ticket; let first = view; let second = view; let a = inspect(first); let b = inspect(second); return ticket;",
            "let mut owned = ticket; let first = &mut owned; let second = first; let a = exclusive(second); let b = exclusive(&mut owned); return owned;",
            "let view = &ticket; let unused = view; return ticket;",
        ] {
            let mut program = typed(&format!("{declarations} fn example(ticket: Ticket) -> Ticket {{ {body} }}"));
            let original = program.functions()[2].as_ordinary().unwrap().body.clone();
            check(&program).unwrap();
            rename(&mut program);
            assert_eq!(program.functions()[2].as_ordinary().unwrap().body, original);
            check(&program).unwrap();
        }
        for body in [
            "let view = &ticket; let alias = view; let moved = ticket; let predecessor_ok = inspect(alias); return moved;",
            "let mut owned = ticket; let access = &mut owned; let next = access; let view = &owned; let predecessor_ok = exclusive(next); return owned;",
            "let mut owned = ticket; let access = &mut owned; let predecessor_ok = exclusive(access); let bad = exclusive(access); return owned;",
        ] {
            let mut program = typed(&format!("{declarations} fn example(ticket: Ticket) -> Ticket {{ {body} }}"));
            let original = check(&program).unwrap_err();
            rename(&mut program);
            let renamed = check(&program).unwrap_err();
            assert_eq!(original[0].span, renamed[0].span);
            assert!(renamed[0].message.contains("`display`"));
            assert!(!renamed[0].message.contains("LoanId"));
            assert_eq!(original[0].known.len(), renamed[0].known.len());
        }
    }
    #[test]
    fn reference_classification_and_external_provenance_are_function_local() {
        let program = typed("fn sink(value: &mut Ticket) -> bool { return true; } fn relay(value: &mut Ticket) -> bool { let next = value; return sink(next); } fn shared(value: &Ticket) -> bool { let first = value; let second = value; return true; }");
        let referent = crate::hir::ReferentType::Record(program.records()[0].id);
        assert_eq!(
            ownership_kind(ValueType::SharedRef(referent)),
            OwnershipKind::Copy
        );
        assert_eq!(
            ownership_kind(ValueType::MutableRef(referent)),
            OwnershipKind::Move
        );
        let f = program.functions()[1].as_ordinary().unwrap();
        let mut checker = Checker::new(&program, f);
        assert_eq!(checker.loans.len(), 1);
        assert!(checker.loans.values().all(|loan| loan.owner.is_none()));
        checker.body(f).unwrap();
        assert!(checker.loans.is_empty());
        assert!(matches!(
            checker.states[&Place::Parameter(f.parameters[0])],
            State::Moved { .. }
        ));
        let f = program.functions()[2].as_ordinary().unwrap();
        let mut checker = Checker::new(&program, f);
        assert!(checker.states.is_empty());
        checker.body(f).unwrap();
        assert!(checker.states.is_empty());
        assert!(checker.loans.is_empty());
    }
    #[test]
    fn dereference_and_write_identity_remain_independent_of_diagnostic_names() {
        fn rename(program: &mut Program) {
            for range in &mut program.ranges {
                range.name = "display".into();
            }
            for record in &mut program.records {
                record.name = "display".into();
            }
            for field in &mut program.fields {
                field.name = "display".into();
            }
            for parameter in &mut program.parameters {
                parameter.name = "display".into();
            }
            for local in &mut program.locals {
                local.name = "display".into();
            }
            for f in &mut program.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        for body in [
            "fn good(value: Percent, replacement: Percent) -> Percent { let mut owned = value; let view = &owned; let alias = view; let before = *view; let after = *alias; let access = &mut owned; *access = replacement; *access = *access; return owned; }",
            "fn good(value: &mut Percent) -> Percent { let first = value; let observed = *first; return *first; }",
        ] {
            let mut program = typed(body);
            let original = program.functions()[0].as_ordinary().unwrap().body.clone();
            check(&program).unwrap();
            rename(&mut program);
            assert_eq!(original, program.functions()[0].as_ordinary().unwrap().body);
            check(&program).unwrap();
        }
        for body in [
            "fn bad(value: &Ticket) -> Ticket { return *value; }",
            "fn bad(value: &mut Ticket, replacement: Ticket) -> bool { *value = replacement; return true; }",
            "fn bad(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; let next = access; return *access; }",
            "fn bad(value: Percent) -> Percent { let mut owned = value; let access = &mut owned; *access = owned; return owned; }",
            "fn bad(value: Percent) -> Percent { let mut owned = value; let view = &owned; let access = &mut owned; return *view; }",
        ] {
            let mut program = typed(body);
            let original = check(&program).unwrap_err();
            rename(&mut program);
            let renamed = check(&program).unwrap_err();
            assert_eq!(original[0].span, renamed[0].span);
            assert!(renamed[0].message.contains("`display`"));
            assert_eq!(original[0].known.len(), renamed[0].known.len());
            assert!(!renamed[0].message.contains("LoanId"));
        }
    }
    #[test]
    fn borrowed_copy_access_preserves_handle_availability_and_loan_identity() {
        let program = typed("fn update(value: &mut Percent, replacement: Percent) -> Percent { let before = *value; *value = replacement; return *value; }");
        let f = program.functions()[0].as_ordinary().unwrap();
        let reference = Place::Parameter(f.parameters[0]);
        let mut checker = Checker::new(&program, f);
        let original_loan = checker.handles[&reference];
        let ValueStatement::Let {
            local, initializer, ..
        } = &f.body[0]
        else {
            panic!("let")
        };
        assert_eq!(
            checker
                .expression(initializer, Transfer::Local(*local))
                .unwrap(),
            None
        );
        assert_eq!(checker.states[&reference], State::Available);
        assert_eq!(checker.handles[&reference], original_loan);
        assert_eq!(checker.loans.len(), 1);
        assert_eq!(checker.next_loan, 1);
        // Full read/write/read leaves the handle available and releases the original
        // external loan after last use; no replacement/destruction state is added.
        let mut checker = Checker::new(&program, f);
        checker.body(f).unwrap();
        assert_eq!(checker.states[&reference], State::Available);
        assert_eq!(checker.next_loan, 1);
        assert!(checker.loans.is_empty());
    }
    #[test]
    fn possible_joined_loan_survives_conditional_handle_unavailability() {
        let p = typed("fn sink(access: &mut Percent) -> bool { return true; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let done = sink(access); } let observed = owned; return *access; }");
        let f = p.functions()[1].as_ordinary().unwrap();
        let mut checker = Checker::new(&p, f);
        checker.statements(&f.body[..3], f.id).unwrap();
        let access = Place::Local(p.locals()[1].id);
        assert!(matches!(
            checker.states[&access],
            State::ConditionalMove { .. }
        ));
        let loan = checker.handles[&access];
        assert!(checker.loans.contains_key(&loan));
        checker.expire();
        assert!(checker.loans.contains_key(&loan));
        assert!(checker.loans.values().all(|loan| loan.held == 0));
        let error = checker.statements(&f.body[3..], f.id).unwrap_err();
        assert!(error.message.contains("exclusively borrowed"));
    }
    #[test]
    fn all_path_expiry_does_not_restore_the_moved_handle_or_export_branch_loans() {
        let p = typed("fn sink(access: &mut Percent) -> bool { return true; } fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let next = access; let done = sink(next); } else { let other = &mut owned; *other = value; } return owned; }");
        let f = p.functions()[1].as_ordinary().unwrap();
        let mut checker = Checker::new(&p, f);
        checker.statements(&f.body[..3], f.id).unwrap();
        assert!(matches!(
            checker.states[&Place::Local(p.locals()[1].id)],
            State::ConditionalMove { .. }
        ));
        assert!(checker.loans.is_empty());
        assert_eq!(checker.handles.len(), 1);
        assert!(!checker
            .handles
            .contains_key(&Place::Local(p.locals()[2].id)));
        assert!(!checker
            .handles
            .contains_key(&Place::Local(p.locals()[4].id)));
        checker.statements(&f.body[3..], f.id).unwrap();
    }
    #[test]
    fn future_use_decomposition_preserves_suffix_and_excludes_sibling_counts() {
        let p = typed("fn example(flag: bool, view: &Percent, access: &mut Percent) -> Percent { if flag { let observed = *view; } else { let observed = *access; } return *view; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let ValueStatement::If {
            condition,
            then_body,
            else_body,
            ..
        } = &f.body[0]
        else {
            panic!()
        };
        let mut all = future_uses(&f.body);
        let mut condition_uses = FutureUses::new();
        count_expression(condition, &mut condition_uses);
        subtract_uses(&mut all, &condition_uses);
        let then_uses = future_uses(then_body);
        let else_uses = future_uses(else_body);
        subtract_uses(&mut all, &then_uses);
        subtract_uses(&mut all, &else_uses);
        let view = Place::Parameter(f.parameters[1]);
        let access = Place::Parameter(f.parameters[2]);
        let then_future = with_branch_uses(&all, &then_uses);
        let else_future = with_branch_uses(&all, &else_uses);
        assert_eq!(then_future[&view], 2);
        assert_eq!(then_future[&access], 0);
        assert_eq!(else_future[&view], 1);
        assert_eq!(else_future[&access], 1);
    }
    #[test]
    fn path_liveness_outcomes_ignore_all_display_names() {
        fn rename(p: &mut Program) {
            for r in &mut p.ranges {
                r.name = "display".into();
            }
            for r in &mut p.records {
                r.name = "display".into();
            }
            for field in &mut p.fields {
                field.name = "display".into();
            }
            for parameter in &mut p.parameters {
                parameter.name = "display".into();
            }
            for local in &mut p.locals {
                local.name = "display".into();
            }
            for f in &mut p.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        for body in [
            "fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let alias = view; let observed = *alias; } else { let access = &mut owned; *access = value; } return owned; }",
            "fn example(flag: bool, value: Percent) -> Percent { let mut owned = value; let first = &mut owned; if flag { let second = first; *second = value; } else { let observed = owned; } return owned; }",
        ] {
            let mut p = typed(body);
            check(&p).unwrap();
            let before = crate::mir::lower(&p);
            rename(&mut p);
            check(&p).unwrap();
            assert_eq!(before, crate::mir::lower(&p));
        }
        for body in [
            "fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; if flag { let observed = *view; } else { let access = &mut owned; } return *view; }",
            "fn sink(access: &mut Percent) -> bool { return true; } fn bad(flag: bool, value: Percent) -> Percent { let mut owned = value; let access = &mut owned; if flag { let done = sink(access); } let observed = owned; return *access; }",
        ] {
            let mut p = typed(body);
            let before = check(&p).unwrap_err();
            rename(&mut p);
            let after = check(&p).unwrap_err();
            assert_eq!(before[0].span, after[0].span);
            assert_eq!(before[0].known, after[0].known);
            assert!(after[0].message.contains("display"));
        }
    }
    #[test]
    fn loops_preserve_header_state_and_scope_nested_copy_locals() {
        let p = typed("fn f(flag: bool, value: Percent, ticket: Ticket) -> Percent { let mut owned = value; let view = &owned; while flag { let copied = owned; if flag { let observed = copied; } while flag { let nested = copied; } } return *view; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        c.expire();
        let header = c.clone();
        assert_eq!(header.loans.len(), 1);
        c.statements(&f.body[2..3], f.id).unwrap();
        assert!(c.same_loop_state(&header));
        assert_ne!(c.remaining, header.remaining);
        assert!(c.loop_scope.is_none());
        assert_eq!(c.states.len(), 1); // Untouched record parameter only.
        assert_eq!(c.handles.len(), 1); // No body-local provenance escapes.
        c.statements(&f.body[3..], f.id).unwrap();
        assert!(c.loans.is_empty());
    }
    #[test]
    fn expired_loans_do_not_resurrect_at_backedges() {
        let p = typed("fn f(flag: bool, value: Percent) -> Percent { let mut owned = value; let view = &owned; let observed = *view; while flag { owned = value; while flag { owned = owned; } } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..3], f.id).unwrap();
        assert!(c.loans.is_empty());
        let header = c.clone();
        c.statements(&f.body[3..4], f.id).unwrap();
        assert!(c.same_loop_state(&header));
        assert!(c.loans.is_empty());
    }
    #[test]
    fn forbidden_loop_activity_cannot_silently_change_outer_state() {
        for (body, message) in [
            ("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let moved = ticket; } return ticket; }", "pre-existing non-Copy value"),
            ("fn f(flag: bool, access: &mut Percent, value: Percent) -> Percent { while flag { let moved = access; } return value; }", "loop-carried mutable reference"),
        ] {
            let p = typed(body);
            let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            c.expire();
            let header = c.clone();
            let error = c.statements(&f.body[..1], f.id).unwrap_err();
            assert!(error.message.contains(message));
            assert!(error.required.contains("unchanged provenance") || error.required.contains("future generalized cyclic"));
            assert!(c.same_loop_state(&header));
        }
    }
    #[test]
    fn loop_state_equality_detects_drift_but_ignores_display_metadata() {
        let mut p = typed("fn f(flag: bool, value: Percent, ticket: Ticket) -> Percent { let mut owned = value; let view = &owned; while flag { let copied = owned; } return *view; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let mut drift = c.clone();
        drift.states.insert(
            Place::Parameter(f.parameters[2]),
            State::ConditionalMove { span: f.span },
        );
        assert!(!drift.same_loop_state(&c));
        let mut drift = c.clone();
        drift.handles.clear();
        assert!(!drift.same_loop_state(&c));
        let mut drift = c.clone();
        drift.loans.clear();
        assert!(!drift.same_loop_state(&c));
        let mut drift = c.clone();
        drift.next_loan += 1;
        assert!(!drift.same_loop_state(&c));
        let mut metadata = c.clone();
        for loan in metadata.loans.values_mut() {
            loan.span = Span {
                start: 0,
                end: 0,
                ..f.span
            };
        }
        assert!(metadata.same_loop_state(&c));
        let owner = Place::Parameter(f.parameters[2]);
        metadata.states.insert(
            owner,
            State::Moved {
                span: f.span,
                transfer: Transfer::Return(f.id),
            },
        );
        let mut same_availability = metadata.clone();
        same_availability.states.insert(
            owner,
            State::Moved {
                span: Span {
                    start: 0,
                    end: 0,
                    ..f.span
                },
                transfer: Transfer::Local(p.locals()[0].id),
            },
        );
        assert!(same_availability.same_loop_state(&metadata));
        let before = crate::mir::lower(&p);
        check(&p).unwrap();
        for local in &mut p.locals {
            local.name = "display".into();
        }
        for param in &mut p.parameters {
            param.name = "display".into();
        }
        for range in &mut p.ranges {
            range.name = "display".into();
        }
        for f in &mut p.functions {
            if let crate::hir::Function::Ordinary(f) = f {
                f.name = "display".into();
            }
        }
        check(&p).unwrap();
        assert_eq!(before, crate::mir::lower(&p));
    }
    #[test]
    fn iteration_local_move_states_evolve_then_leave_header_unchanged() {
        let p = typed("fn fresh() -> Ticket { return fresh(); } fn consume(value: Ticket) -> bool { return true; } fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let first = fresh(); let second = first; let unused = fresh(); let done = consume(second); } return ticket; }");
        let f = p.functions()[2].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.expire();
        let header = c.clone();
        let ValueStatement::While {
            condition, body, ..
        } = &f.body[0]
        else {
            panic!()
        };
        let mut iteration = c.clone();
        iteration.loop_scope = Some(LoopScope::new(&header, true));
        iteration
            .expression(condition, Transfer::Return(f.id))
            .unwrap();
        iteration.loop_scope.as_mut().unwrap().condition = false;
        iteration.statements(body, f.id).unwrap();
        let places: Vec<_> = body
            .iter()
            .map(|s| match s {
                ValueStatement::Let { local, .. } => Place::Local(*local),
                _ => panic!(),
            })
            .collect();
        assert!(matches!(iteration.states[&places[0]], State::Moved { .. }));
        assert!(matches!(iteration.states[&places[1]], State::Moved { .. }));
        assert_eq!(iteration.states[&places[2]], State::Available);
        assert!(!iteration.same_loop_state(&header));
        iteration.scope_loop_locals(&header);
        assert!(iteration.same_loop_state(&header));
        assert_eq!(iteration.states.len(), 1);
        // Actual loop checking uses the same cleanup, including owners not consumed
        // explicitly; scope exit does not imply runtime destruction or cleanup code.
        c.statements(&f.body[..1], f.id).unwrap();
        assert!(c.same_loop_state(&header));
        assert!(c.loop_scope.is_none());
        c.statements(&f.body[1..], f.id).unwrap();
    }
    #[test]
    fn carried_move_and_replacement_failures_preserve_original_checker_state() {
        for body in [
            "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let moved = ticket; } return ticket; }",
            "fn consume(value: Ticket) -> bool { return true; } fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let done = consume(ticket); } return ticket; }",
            "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { if flag { let moved = ticket; } } return ticket; }",
            "fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { while flag { let moved = ticket; } } return ticket; }",
            "fn fresh() -> Ticket { return fresh(); } fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let mut local = fresh(); local = ticket; } return ticket; }",
            "fn fresh() -> Ticket { return fresh(); } fn pair(first: Ticket, second: Ticket) -> bool { return true; } fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let local = fresh(); let done = pair(local, ticket); } return ticket; }",
        ] {
            let p = typed(body);
            let f = p.functions().last().unwrap().as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            c.expire();
            let header = c.clone();
            let error = c.statements(&f.body[..1], f.id).unwrap_err();
            assert!(error.message.contains("pre-existing non-Copy") || error.message.contains("cannot replace non-Copy"));
            assert!(c.same_loop_state(&header));
            assert_eq!(c.remaining, header.remaining);
            // The transfer guard itself is atomic, independently of loop cloning.
            c.loop_scope = Some(LoopScope::new(&header, false));
            assert!(c.consume(Place::Parameter(f.parameters[1]), f.span, Transfer::Return(f.id)).is_err());
            assert!(c.same_loop_state(&header));
            assert_eq!(c.remaining, header.remaining);
        }
    }
    #[test]
    fn nested_loop_header_protects_outer_iteration_owner_without_mutation() {
        let p = typed("fn fresh() -> Ticket { return fresh(); } fn f(flag: bool) -> bool { while flag { let outer = fresh(); while flag { let moved = outer; } } return flag; }");
        let f = p.functions()[1].as_ordinary().unwrap();
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        let mut c = Checker::new(&p, f);
        c.loop_scope = Some(LoopScope::new(&c, false));
        c.statements(&body[..1], f.id).unwrap();
        let ValueStatement::Let { local, .. } = body[0] else {
            panic!()
        };
        let owner = Place::Local(local);
        assert!(!c
            .loop_scope
            .as_ref()
            .unwrap()
            .header_places
            .contains(&owner));
        let outer = c.clone();
        let error = c.statements(&body[1..], f.id).unwrap_err();
        assert!(error
            .message
            .contains("pre-existing non-Copy value `outer`"));
        assert_eq!(c.states[&owner], State::Available);
        assert!(c.same_loop_state(&outer));
        assert_eq!(c.remaining, outer.remaining);
    }
    #[test]
    fn lexical_cleanup_cannot_hide_drift_of_a_header_owner() {
        let p = typed("fn fresh() -> Ticket { return fresh(); } fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { let local = fresh(); } return ticket; }");
        let f = p.functions()[1].as_ordinary().unwrap();
        let c = Checker::new(&p, f);
        let mut drift = c.clone();
        drift.states.insert(
            Place::Parameter(f.parameters[1]),
            State::Moved {
                span: f.span,
                transfer: Transfer::Return(f.id),
            },
        );
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        let ValueStatement::Let { local, .. } = body[0] else {
            panic!()
        };
        drift.states.insert(Place::Local(local), State::Available);
        drift.scope_loop_locals(&c);
        assert!(!drift.states.contains_key(&Place::Local(local)));
        assert!(!drift.same_loop_state(&c));
    }
    #[test]
    fn iteration_local_acceptance_and_carried_rejections_ignore_display_names() {
        fn rename(p: &mut Program) {
            for r in &mut p.ranges {
                r.name = "display".into();
            }
            for r in &mut p.records {
                r.name = "display".into();
            }
            for field in &mut p.fields {
                field.name = "display".into();
            }
            for parameter in &mut p.parameters {
                parameter.name = "display".into();
            }
            for local in &mut p.locals {
                local.name = "display".into();
            }
            for function in &mut p.functions {
                if let crate::hir::Function::Ordinary(f) = function {
                    f.name = "display".into();
                }
            }
        }
        for body in [
            "fn fresh() -> Ticket { return fresh(); } fn consume(ticket: Ticket) -> bool { return true; } fn f(flag: bool) -> bool { while flag { let first = fresh(); let second = first; if flag { let done = consume(second); } else { let done = consume(second); } } return flag; }",
            "fn fresh() -> Ticket { return fresh(); } fn consume(ticket: Ticket) -> bool { return true; } fn f(flag: bool) -> bool { while flag { let first = fresh(); while flag { let second = fresh(); let done = consume(second); } let done = consume(first); } return flag; }",
        ] {
            let mut p = typed(body);
            check(&p).unwrap();
            let before = crate::mir::lower(&p);
            rename(&mut p);
            check(&p).unwrap();
            assert_eq!(before, crate::mir::lower(&p));
        }
        for body in [
            "fn f(flag: bool, ticket: Ticket) -> bool { while flag { let moved = ticket; } return flag; }",
            "fn fresh() -> Ticket { return fresh(); } fn f(flag: bool) -> bool { while flag { let outer = fresh(); while flag { let moved = outer; } } return flag; }",
        ] {
            let mut p = typed(body);
            let before = check(&p).unwrap_err();
            rename(&mut p);
            let after = check(&p).unwrap_err();
            assert_eq!(before[0].span, after[0].span);
            assert_eq!(before[0].required, after[0].required);
            assert_eq!(before[0].known, after[0].known);
            assert!(after[0].message.contains("pre-existing non-Copy value `display`"));
        }
    }
    #[test]
    fn header_handle_transfer_and_condition_guards_are_atomic() {
        let p = typed("fn f(flag: bool, access: &mut Percent, value: Percent) -> Percent { while flag { let next = access; } return value; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.expire();
        let header = c.clone();
        let reference = Place::Parameter(f.parameters[1]);
        c.loop_scope = Some(LoopScope::new(&header, false));
        assert!(c
            .consume(reference, f.span, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("loop-carried mutable reference"));
        assert!(c.same_loop_state(&header));
        assert_eq!(c.remaining, header.remaining);
        c.loop_scope.as_mut().unwrap().condition = true;
        for write in [false, true] {
            assert!(c
                .borrowed_access(reference, f.span, write)
                .unwrap_err()
                .message
                .contains("while conditions"));
        }
        assert!(c.same_loop_state(&header));
        assert_eq!(c.remaining, header.remaining);
        c.assert_no_holds();
    }
    #[test]
    fn body_loans_and_aliases_evolve_then_cleanup_restores_header_allocator() {
        let p = typed("fn f(flag: bool, value: Percent) -> Percent { let mut current = value; let outside = &value; while flag { let view = &value; let alias = view; let observed = *alias; let access = &mut current; *access = observed; } return *outside; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let header = c.clone();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let mut iteration = c.clone();
        iteration.loop_scope = Some(LoopScope::new(&header, false));
        iteration.statements(&body[..2], f.id).unwrap();
        let ValueStatement::Let { local: view, .. } = body[0] else {
            panic!()
        };
        let ValueStatement::Let { local: alias, .. } = body[1] else {
            panic!()
        };
        let provenance = iteration.handles[&Place::Local(view)];
        assert_eq!(provenance, iteration.handles[&Place::Local(alias)]);
        assert_eq!(provenance.0, header.next_loan);
        assert!(iteration.loans.contains_key(&provenance));
        iteration.statements(&body[2..], f.id).unwrap();
        assert!(iteration.next_loan > header.next_loan);
        assert!(iteration.handles.len() > header.handles.len());
        iteration.finish_loop_provenance(&header, *span).unwrap();
        assert!(iteration.same_loop_state(&header));
        assert_eq!(iteration.next_loan, header.next_loan);
        c.statements(&f.body[2..3], f.id).unwrap();
        assert!(c.same_loop_state(&header));
    }
    #[test]
    fn allocator_normalization_cannot_hide_surviving_loan_or_handle() {
        let p = typed("fn f(view: &Percent, value: Percent) -> Percent { return *view; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let header = Checker::new(&p, f);
        let reference = Place::Parameter(f.parameters[0]);
        let owner = Place::Parameter(f.parameters[1]);
        for dangling in [false, true] {
            let mut broken = header.clone();
            let new = broken.add_loan(Some(owner), BorrowKind::Shared, f.span);
            // Fault injection: a header handle must never acquire new provenance.
            broken.handles.insert(reference, new);
            if dangling {
                broken.loans.remove(&new);
            }
            let allocated = broken.next_loan;
            let error = broken.finish_loop_provenance(&header, f.span).unwrap_err();
            assert!(error
                .message
                .contains("iteration-created reference state survives"));
            assert_eq!(broken.next_loan, allocated);
            assert_eq!(broken.handles[&reference], new);
        }
        let mut broken = header.clone();
        let new = broken.add_loan(Some(owner), BorrowKind::Mutable, f.span);
        broken.loans.get_mut(&new).unwrap().held = 1; // Escaped operation hold.
        let allocated = broken.next_loan;
        assert!(broken.finish_loop_provenance(&header, f.span).is_err());
        assert_eq!(broken.next_loan, allocated);
        assert!(broken.loans.contains_key(&new));
        let mut broken = header.clone();
        broken.next_loan -= 1;
        assert!(broken
            .finish_loop_provenance(&header, f.span)
            .unwrap_err()
            .message
            .contains("allocation state"));
        assert_ne!(broken.next_loan, header.next_loan);
    }
    #[test]
    fn lexical_handle_cleanup_expires_loan_before_safe_allocator_reuse() {
        let p = typed(
            "fn f(value: Percent) -> Percent { while false { let view = &value; } return value; }",
        );
        let f = p.functions()[0].as_ordinary().unwrap();
        let header = Checker::new(&p, f);
        let ValueStatement::While { body, span, .. } = &f.body[0] else {
            panic!()
        };
        let ValueStatement::Let { local, .. } = body[0] else {
            panic!()
        };
        let mut iteration = header.clone();
        let new = iteration.add_loan(
            Some(Place::Parameter(f.parameters[0])),
            BorrowKind::Shared,
            *span,
        );
        iteration.handles.insert(Place::Local(local), new);
        // A branch/body scoped handle cannot escape even with stale suffix counts.
        iteration.remaining.insert(Place::Local(local), 1);
        iteration.expire();
        assert!(iteration.loans.contains_key(&new));
        iteration.finish_loop_provenance(&header, *span).unwrap();
        assert!(!iteration.handles.contains_key(&Place::Local(local)));
        assert!(!iteration.loans.contains_key(&new));
        assert!(iteration.same_loop_state(&header));
    }
    #[test]
    fn post_loop_loan_reuse_cannot_alias_transient_provenance_or_header_loan() {
        let p = typed("fn f(flag: bool, value: Percent) -> Percent { let mut first = value; let mut second = value; let outside = &first; while flag { let view = &second; let before = *view; } let access = &mut second; *access = value; return *outside; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..3], f.id).unwrap();
        let header = c.clone();
        c.statements(&f.body[3..4], f.id).unwrap();
        assert!(c.same_loop_state(&header));
        let reused = LoanId(header.next_loan);
        assert!(!c.loans.contains_key(&reused));
        c.statements(&f.body[4..5], f.id).unwrap();
        let ValueStatement::Let { local: access, .. } = f.body[4] else {
            panic!()
        };
        let ValueStatement::Let { local: second, .. } = f.body[1] else {
            panic!()
        };
        assert_eq!(c.handles[&Place::Local(access)], reused);
        assert_eq!(c.loans[&reused].owner, Some(Place::Local(second)));
        assert_eq!(c.loans[&reused].kind, BorrowKind::Mutable);
        assert_eq!(c.loans.len(), 2);
        for (place, id) in &header.handles {
            assert_eq!(c.handles[place], *id);
        }
        c.statements(&f.body[5..], f.id).unwrap();
        assert!(c.loans.is_empty());
    }
    #[test]
    fn nested_loops_preserve_outer_provenance_and_restore_relative_boundary() {
        let p = typed("fn f(flag: bool, value: Percent) -> Percent { while flag { let outer = &value; while flag { let inner = &value; let observed = *inner; } let later = *outer; } return value; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[0] else {
            panic!()
        };
        let mut c = Checker::new(&p, f);
        let header = c.clone();
        c.loop_scope = Some(LoopScope::new(&header, false));
        c.statements(&body[..1], f.id).unwrap();
        let ValueStatement::Let { local, .. } = body[0] else {
            panic!()
        };
        let outer = Place::Local(local);
        let provenance = c.handles[&outer];
        let before_inner = c.clone();
        c.statements(&body[1..2], f.id).unwrap();
        assert!(c.same_loop_state(&before_inner));
        assert!(c.loop_handle(outer, *span).is_ok()); // Restored outer boundary.
        let mut inner = c.clone();
        inner.loop_scope = Some(LoopScope::new(&c, false));
        assert!(inner.loop_handle(outer, *span).is_ok());
        assert!(inner
            .loop_scope
            .as_ref()
            .unwrap()
            .header_handles
            .contains(&outer));
        assert!(c.loans.contains_key(&provenance));
        c.statements(&body[2..], f.id).unwrap();
        c.finish_loop_provenance(&header, *span).unwrap();
        assert!(c.same_loop_state(&header));
    }
    #[test]
    fn untouched_exclusive_header_loan_remains_semantically_identical() {
        let p = typed("fn f(flag: bool, value: Percent) -> Percent { let mut first = value; let mut second = value; let outside = &mut first; while flag { let inside = &mut second; *inside = value; } return *outside; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..3], f.id).unwrap();
        let header = c.clone();
        c.statements(&f.body[3..4], f.id).unwrap();
        assert!(c.same_loop_state(&header));
        assert_eq!(c.loans.len(), 1);
        assert_eq!(c.loans.values().next().unwrap().kind, BorrowKind::Mutable);
    }
    #[test]
    fn iteration_borrowing_uses_identity_independent_of_display_metadata() {
        fn rename(p: &mut Program) {
            for r in &mut p.ranges {
                r.name = "display".into();
            }
            for r in &mut p.records {
                r.name = "display".into();
            }
            for f in &mut p.fields {
                f.name = "display".into();
            }
            for v in &mut p.parameters {
                v.name = "display".into();
            }
            for v in &mut p.locals {
                v.name = "display".into();
            }
            for f in &mut p.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        for body in [
            "fn f(flag: bool, value: Percent) -> Percent { let mut current = value; while flag { let view = &current; if flag { let alias = view; let before = *alias; } else { let access = &mut current; *access = value; } } return current; }",
            "fn f(flag: bool, value: Percent) -> Percent { while flag { let outer = &value; while flag { let inner = &value; let before = *inner; } let after = *outer; } return value; }",
        ] {
            let mut p = typed(body);
            check(&p).unwrap();
            let before = crate::mir::lower(&p);
            rename(&mut p);
            check(&p).unwrap();
            assert_eq!(before, crate::mir::lower(&p));
        }
        for body in [
            "fn f(flag: bool, access: &mut Percent, value: Percent) -> Percent { while flag { let moved = access; } return value; }",
            "fn f(flag: bool, value: Percent) -> Percent { let mut owned = value; while flag { let access = &mut owned; while flag { let moved = access; } } return value; }",
        ] {
            let mut p = typed(body);
            let before = check(&p).unwrap_err();
            rename(&mut p);
            let after = check(&p).unwrap_err();
            assert_eq!(before[0].span, after[0].span);
            assert_eq!(before[0].required, after[0].required);
            assert_eq!(before[0].known, after[0].known);
            assert!(after[0].message.contains("loop-carried mutable reference `display`"));
        }
    }
    fn assert_atomic(actual: &Checker<'_>, before: &Checker<'_>) {
        assert_eq!(actual.function, before.function);
        assert_eq!(actual.states, before.states);
        assert_eq!(actual.handles, before.handles);
        assert_eq!(actual.remaining, before.remaining);
        assert_eq!(actual.next_loan, before.next_loan);
        assert_eq!(actual.terminal_return, before.terminal_return);
        assert_eq!(actual.replacement_target, before.replacement_target);
        assert_eq!(actual.recurrent, before.recurrent);
        assert_eq!(actual.targets.len(), before.targets.len());
        for (actual, before) in actual.targets.iter().zip(&before.targets) {
            assert_eq!(actual.outside, before.outside);
            assert!(Rc::ptr_eq(&actual.header, &before.header));
            assert!(Rc::ptr_eq(&actual.exit, &before.exit));
        }
        let loans = |c: &Checker<'_>| {
            c.loans
                .iter()
                .map(|(id, l)| (*id, l.owner, l.origin, l.kind, l.span, l.held))
                .collect::<Vec<_>>()
        };
        assert_eq!(loans(actual), loans(before));
        let scope = |c: &Checker<'_>| {
            c.loop_scope.as_ref().map(|s| {
                (
                    s.header_places.clone(),
                    s.header_handles.clone(),
                    s.header_loans.clone(),
                    s.condition,
                )
            })
        };
        assert_eq!(scope(actual), scope(before));
    }
    fn pin_body<'a>(c: &mut Checker<'a>, body: &[ValueStatement]) -> Checker<'a> {
        let finite = c.remaining.clone();
        let frame = reference_uses(body)
            .iter()
            .filter_map(|p| c.handles.get(p).copied())
            .collect();
        c.recurrent.push(frame);
        assert_eq!(finite, c.remaining); // Obligations never encode or repair counts.
        c.expire();
        let header = c.clone();
        c.loop_scope = Some(LoopScope::new(&header, false));
        header
    }
    #[test]
    fn recurrent_shared_identity_survives_reads_aliases_and_cleanup() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let first = *view; let alias = view; let second = *alias; let last = *view; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        let id = *header.loans.keys().next().unwrap();
        for statement in body {
            c.statements(std::slice::from_ref(statement), f.id).unwrap();
            assert_eq!(c.loans.len(), 1);
            assert!(c.loans.contains_key(&id));
            assert_eq!(c.loans[&id].owner, header.loans[&id].owner);
        }
        assert!(c.handles.len() > header.handles.len());
        c.validate_loop_backedge(&header, *span).unwrap();
        assert!(c.same_loop_state(&header));
        assert_eq!(c.handles, header.handles); // Alias removed, original still live.
        assert!(c.loans.contains_key(&id)); // Checked before obligation discharge.
        c.recurrent.pop();
        c.expire();
        assert!(c.loans.is_empty());
    }
    #[test]
    fn recurrent_mutable_reads_and_copy_writes_preserve_exact_provenance() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let first = *access; *access = *access; *access = true; let second = *access; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        for statement in body {
            c.statements(std::slice::from_ref(statement), f.id).unwrap();
            assert_eq!(c.handles, header.handles);
            assert_eq!(c.states, header.states);
            assert_eq!(c.loans.len(), 1);
            c.assert_no_holds();
        }
        c.validate_loop_backedge(&header, *span).unwrap();
        assert!(c.same_loop_state(&header));
    }
    #[test]
    fn skipped_branch_and_zero_finite_counts_cannot_expire_recurrent_loan() {
        let p = typed("fn f(flag: bool, choice: bool) -> bool { let owned = false; let view = &owned; while flag { if choice { let seen = *view; } } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        let ValueStatement::If {
            condition,
            then_body,
            else_body,
            ..
        } = &body[0]
        else {
            panic!()
        };
        c.expression(condition, Transfer::Return(f.id)).unwrap();
        let (mut yes, mut no, suffix) = c.split(&future_uses(then_body), &future_uses(else_body));
        no.remaining.clear();
        no.expire();
        assert_eq!(no.loans.len(), 1); // Skipped use cannot open a protection gap.
        yes.statements(then_body, f.id).unwrap();
        assert_eq!(yes.loans.len(), 1);
        c.join(yes, no, suffix, f.span).unwrap();
        assert_eq!(c.loans.len(), 1);
        c.validate_loop_backedge(&header, *span).unwrap();
    }
    #[test]
    fn false_exit_uses_suffix_not_backedge_liveness() {
        for later in [false, true] {
            let source = if later {
                "fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; } return *view; }"
            } else {
                "fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; } return owned; }"
            };
            let p = typed(source);
            let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..2], f.id).unwrap();
            let id = *c.loans.keys().next().unwrap();
            c.statements(&f.body[2..3], f.id).unwrap();
            assert!(c.recurrent.is_empty());
            assert_eq!(c.loans.contains_key(&id), later);
            c.statements(&f.body[3..], f.id).unwrap();
            assert!(c.loans.is_empty());
        }
    }
    #[test]
    fn inner_exit_keeps_enclosing_obligation_even_without_finite_uses() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { while flag { let seen = *view; } } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        c.statements(body, f.id).unwrap();
        assert_eq!(c.recurrent.len(), 1);
        assert_eq!(c.recurrent, header.recurrent);
        assert_eq!(c.loans.len(), 1);
        for place in header.handles.keys() {
            assert_eq!(c.remaining[place], 0);
        }
        c.validate_loop_backedge(&header, *span).unwrap();
        c.recurrent.pop();
        c.expire();
        assert!(c.loans.is_empty());
    }
    #[test]
    fn outer_iteration_local_provenance_carries_inner_then_dies_before_outer_backedge() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; while flag { let view = &owned; while flag { let seen = *view; } let after = *view; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        assert!(header.recurrent[0].is_empty());
        c.statements(&body[..1], f.id).unwrap();
        let id = *c.loans.keys().next().unwrap();
        let inner_header = c.clone();
        c.statements(&body[1..2], f.id).unwrap();
        assert!(c.same_loop_state(&inner_header));
        assert!(c.loans.contains_key(&id));
        c.statements(&body[2..], f.id).unwrap();
        assert!(c.loans.is_empty());
        c.validate_loop_backedge(&header, *span).unwrap();
        assert_eq!(c.next_loan, header.next_loan);
        assert!(c.handles.is_empty());
    }
    #[test]
    fn expired_carried_provenance_cannot_be_resurrected_at_loop_entry() {
        let p = typed("fn f(flag: bool, view: &bool) -> bool { while flag { let seen = *view; } return flag; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.loans.clear();
        let before = c.clone();
        let ValueStatement::While {
            condition,
            body,
            span,
        } = &f.body[0]
        else {
            panic!()
        };
        assert!(c
            .while_loop(condition, body, *span, f.id)
            .unwrap_err()
            .message
            .contains("no longer active"));
        assert_atomic(&c, &before);
        let place = Place::Parameter(f.parameters[1]);
        assert!(c.borrowed_access(place, *span, false).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn backedge_validation_rejects_identity_owner_kind_and_obligation_drift() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        c.statements(body, f.id).unwrap();
        let id = *header.loans.keys().next().unwrap();
        let mut missing = c.clone();
        missing.loans.remove(&id);
        assert!(missing.validate_loop_backedge(&header, *span).is_err());
        let mut changed_owner = c.clone();
        changed_owner.loans.get_mut(&id).unwrap().owner = None;
        assert!(changed_owner
            .validate_loop_backedge(&header, *span)
            .is_err());
        let mut changed_kind = c.clone();
        changed_kind.loans.get_mut(&id).unwrap().kind = BorrowKind::Mutable;
        assert!(changed_kind.validate_loop_backedge(&header, *span).is_err());
        let mut released = c.clone();
        released.recurrent.pop();
        released.expire();
        assert!(released.loans.is_empty());
        assert!(released.validate_loop_backedge(&header, *span).is_err());
        let mut refreshed = c.clone();
        let loan = refreshed.add_loan(header.loans[&id].owner, BorrowKind::Shared, *span);
        *refreshed.handles.values_mut().next().unwrap() = loan;
        let allocation = refreshed.next_loan;
        assert!(refreshed
            .validate_loop_backedge(&header, *span)
            .unwrap_err()
            .message
            .contains("survives while backedge"));
        assert_eq!(refreshed.next_loan, allocation); // Normalization cannot hide refresh.
    }
    #[test]
    fn failed_carried_transfer_and_write_rhs_are_fully_atomic() {
        let p = typed("fn sink(access: &mut bool) -> bool { return false; } fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = sink(access); } return owned; }");
        let f = p.functions()[1].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        pin_body(&mut c, body);
        let before = c.clone();
        let reference = *c.handles.keys().next().unwrap();
        assert!(c
            .consume(reference, f.span, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("loop-carried mutable"));
        assert_atomic(&c, &before);
        assert!(c
            .statements(body, f.id)
            .unwrap_err()
            .message
            .contains("loop-carried mutable"));
        assert_atomic(&c, &before);
        c.assert_no_holds();
        let ValueStatement::DerefAssign { value, .. } = &body[0] else {
            panic!()
        };
        let id = c.handles[&reference];
        c.loans.get_mut(&id).unwrap().held = 1;
        let held_before = c.clone();
        assert!(c
            .expression(value, Transfer::WriteThrough(reference))
            .is_err());
        assert_atomic(&c, &held_before); // Failed RHS preserves its caller's existing hold.
    }
    #[test]
    fn nested_call_failure_restores_earlier_argument_counts_and_holds() {
        let p = typed("fn sink(access: &mut bool) -> bool { return false; } fn both(view: &bool, value: bool) -> bool { return value; } fn f(flag: bool) -> bool { let first = false; let view = &first; let mut second = false; let access = &mut second; while flag { let done = both(view, sink(access)); } return false; }");
        let f = p.functions()[2].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..4], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[4] else {
            panic!()
        };
        pin_body(&mut c, body);
        let before = c.clone();
        let ValueStatement::Let { initializer, .. } = &body[0] else {
            panic!()
        };
        assert!(c.expression(initializer, Transfer::Return(f.id)).is_err());
        assert_atomic(&c, &before);
        c.assert_no_holds();
    }
    #[test]
    fn failed_recurrent_owner_assignment_restores_rhs_liveness() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { owned = *view; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        pin_body(&mut c, body);
        let before = c.clone();
        assert!(c
            .statements(body, f.id)
            .unwrap_err()
            .message
            .contains("cannot assign"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn recurrent_conflicting_borrow_rejects_without_allocator_or_hold_changes() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { let other = &owned; *access = true; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        pin_body(&mut c, body);
        let before = c.clone();
        let owner = c.loans.values().next().unwrap().owner.unwrap();
        assert!(c
            .borrow(owner, BorrowKind::Shared, f.span)
            .unwrap_err()
            .message
            .contains("exclusive borrow"));
        assert_atomic(&c, &before);
        assert!(c.statements(&body[..1], f.id).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn post_loop_new_loan_cannot_alias_carried_or_transient_provenance() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let seen = *view; let temporary = &owned; let second = *temporary; } let access = &mut owned; *access = true; return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let carried = *c.loans.keys().next().unwrap();
        let allocator = c.next_loan;
        c.statements(&f.body[2..3], f.id).unwrap();
        assert!(c.loans.is_empty());
        assert_eq!(c.next_loan, allocator);
        c.statements(&f.body[3..4], f.id).unwrap();
        let fresh = *c.loans.keys().next().unwrap();
        assert_ne!(carried, fresh);
        assert_eq!(fresh, LoanId(allocator));
        assert_eq!(c.loans[&fresh].kind, BorrowKind::Mutable);
        c.statements(&f.body[4..], f.id).unwrap();
        assert!(c.loans.is_empty());
    }
    #[test]
    fn recurrent_semantics_and_mir_ignore_all_diagnostic_display_names() {
        fn rename(p: &mut Program) {
            for r in &mut p.ranges {
                r.name = "display".into();
            }
            for r in &mut p.records {
                r.name = "display".into();
            }
            for field in &mut p.fields {
                field.name = "display".into();
            }
            for parameter in &mut p.parameters {
                parameter.name = "display".into();
            }
            for local in &mut p.locals {
                local.name = "display".into();
            }
            for f in &mut p.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        let mut p = typed("fn f(flag: bool, value: Percent) -> Percent { let view = &value; while flag { let alias = view; while flag { let seen = *alias; } } return value; }");
        check(&p).unwrap();
        let before = crate::mir::lower(&p);
        rename(&mut p);
        check(&p).unwrap();
        assert_eq!(before, crate::mir::lower(&p));
        let mut p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let seen = *view; owned = true; } return owned; }");
        let before = check(&p).unwrap_err();
        rename(&mut p);
        let after = check(&p).unwrap_err();
        assert_eq!(before[0].span, after[0].span);
        assert_eq!(before[0].required, after[0].required);
        assert_eq!(before[0].known, after[0].known);
        assert!(after[0].message.contains("display"));
    }

    fn install_continue_target<'a>(
        c: &mut Checker<'a>,
        body: &[ValueStatement],
        outside: FutureUses,
    ) -> Checker<'a> {
        let header = pin_body(c, body);
        let exit = Rc::new(header.false_exit(&outside, header.loop_scope.clone()));
        c.targets.push(LoopTarget {
            header: Rc::new(header.clone()),
            exit,
            outside,
        });
        header
    }
    #[test]
    fn continue_preserves_recurrent_only_shared_and_mutable_provenance() {
        for source in [
            "fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; continue; } return owned; }",
            "fn f(flag: bool) -> bool { let mut owned = false; let access = &mut owned; while flag { *access = true; continue; } return owned; }",
        ] {
            let p = typed(source); let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f); c.statements(&f.body[..2], f.id).unwrap();
            let ValueStatement::While { body, span, .. } = &f.body[2] else { panic!() };
            let header = install_continue_target(&mut c, body, FutureUses::new());
            let id = *c.loans.keys().next().unwrap();
            c.remaining.clear(); c.expire();
            assert!(c.loans.contains_key(&id)); assert_eq!(c.loans[&id].held, 0);
            assert_eq!(c.continue_edge(*span).unwrap(), Flow::Continue);
            assert!(c.same_loop_state(&header));
            assert_eq!(c.loans[&id].owner, header.loans[&id].owner);
            assert_eq!(c.loans[&id].kind, header.loans[&id].kind);
            assert_eq!(c.recurrent, header.recurrent); // Continue never discharges a frame.
            assert_eq!(c.targets.len(), 1);
        }
    }
    #[test]
    fn continue_validation_cleanup_failure_restores_every_state_field() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let alias = view; let seen = *alias; continue; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        c.statements_with(&body[..2], f.id, &FutureUses::new())
            .unwrap();
        let carried = *header.loans.keys().next().unwrap();
        for fault in 0..5 {
            let mut broken = c.clone();
            match fault {
                0 => broken.loans.get_mut(&carried).unwrap().owner = None,
                1 => broken.loans.get_mut(&carried).unwrap().kind = BorrowKind::Mutable,
                2 => {
                    broken.loans.remove(&carried);
                }
                3 => {
                    broken.recurrent.clear();
                }
                _ => {
                    *broken.handles.values_mut().next().unwrap() = LoanId(999);
                }
            }
            let before = broken.clone();
            let e = broken.continue_edge(*span).unwrap_err();
            assert!(e.message.contains("continue backedge rejected"));
            assert_atomic(&broken, &before); // Alias cleanup must also roll back.
            assert!(broken.handles.len() > header.handles.len());
        }
    }
    #[test]
    fn continue_allocator_cannot_hide_surviving_fresh_loan_or_handle() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; continue; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        for surviving_handle in [false, true] {
            let mut broken = c.clone();
            let fresh = broken.add_loan(None, BorrowKind::Shared, *span);
            if surviving_handle {
                let place = *header.handles.keys().next().unwrap();
                broken.handles.insert(place, fresh);
                broken.remaining.insert(place, 1);
            } else {
                broken.loans.get_mut(&fresh).unwrap().held = 1;
            }
            let before = broken.clone();
            assert!(broken
                .continue_edge(*span)
                .unwrap_err()
                .message
                .contains("iteration-created reference state"));
            assert_atomic(&broken, &before);
            assert!(broken.next_loan > header.next_loan);
            assert!(broken.loans.contains_key(&fresh));
        }
    }
    #[test]
    fn continue_rejects_injected_carried_availability_change_atomically() {
        let p = typed(
            "fn f(flag: bool, ticket: Ticket) -> bool { while flag { continue; } return flag; }",
        );
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, span, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        c.states.insert(
            Place::Parameter(f.parameters[1]),
            State::Moved {
                span: *span,
                transfer: Transfer::Return(f.id),
            },
        );
        let before = c.clone();
        assert!(c
            .continue_edge(*span)
            .unwrap_err()
            .message
            .contains("does not preserve"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn continue_cleans_transient_local_provenance_before_allocator_reset() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; while flag { let access = &mut owned; *access = true; continue; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[1] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        assert_eq!(
            c.statements_with(body, f.id, &FutureUses::new()).unwrap(),
            Flow::Continue
        );
        assert!(c.same_loop_state(&header));
        assert!(c.handles.is_empty());
        assert!(c.loans.is_empty());
        assert_eq!(c.next_loan, header.next_loan);
    }
    #[test]
    fn continue_edge_future_is_known_before_conflicting_owner_operation() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; while flag { let view = &owned; if flag { owned = true; continue; } let seen = *view; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[1] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let ValueStatement::If { then_body, .. } = &body[1] else {
            panic!()
        };
        let after = future_uses(&body[2..]);
        let before = block_future(then_body, &after, &FutureUses::new());
        let ValueStatement::Let { local, .. } = &body[0] else {
            panic!()
        };
        assert_eq!(before.get(&Place::Local(*local)).copied().unwrap_or(0), 0);
        assert!(after[&Place::Local(*local)] > 0);
        assert_eq!(
            c.statements_with(body, f.id, &FutureUses::new()).unwrap(),
            Flow::Fallthrough
        );
        assert!(c.loans.is_empty());
    }
    #[test]
    fn all_continue_false_exit_is_independent_and_retains_post_loop_suffix_only() {
        for post_use in [false, true] {
            let suffix = if post_use {
                "return *view;"
            } else {
                "return owned;"
            };
            let p = typed(&format!("fn f(flag: bool) -> bool {{ let owned = false; let view = &owned; while flag {{ if flag {{ let seen = *view; continue; }} else {{ continue; }} }} {suffix} }}"));
            let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..2], f.id).unwrap();
            let id = *c.loans.keys().next().unwrap();
            c.statements(&f.body[2..3], f.id).unwrap();
            assert!(c.targets.is_empty());
            assert!(c.recurrent.is_empty());
            assert!(c.loop_scope.is_none());
            assert_eq!(c.loans.contains_key(&id), post_use);
            c.statements(&f.body[3..], f.id).unwrap();
            assert!(c.loans.is_empty());
        }
    }
    #[test]
    fn nested_all_continue_false_exit_preserves_outer_frame_and_target() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { while flag { if flag { let seen = *view; continue; } else { continue; } } continue; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let outer = install_continue_target(&mut c, body, FutureUses::new());
        c.statements_with(&body[..1], f.id, &FutureUses::new())
            .unwrap();
        assert_eq!(c.recurrent, outer.recurrent);
        assert_eq!(c.targets.len(), 1);
        assert!(c.same_loop_state(&outer));
        c.remaining.clear();
        c.expire();
        assert_eq!(c.loans.len(), 1);
        assert_eq!(c.continue_edge(*span).unwrap(), Flow::Continue);
        assert!(c.same_loop_state(&outer));
    }
    #[test]
    fn failed_nested_continue_operation_restores_parent_target_and_summary() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { while flag { owned = true; let seen = *view; continue; } continue; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let before = c.clone();
        assert!(c
            .statements_with(body, f.id, &FutureUses::new())
            .unwrap_err()
            .message
            .contains("cannot assign"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn continue_metadata_renaming_preserves_identity_cfg_and_ownership() {
        let mut p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { if flag { let alias = view; let seen = *alias; continue; } else { continue; } } return owned; }");
        check(&p).unwrap();
        let before = crate::mir::lower(&p);
        for local in &mut p.locals {
            local.name = "renamed".into();
        }
        for parameter in &mut p.parameters {
            parameter.name = "renamed".into();
        }
        check(&p).unwrap();
        assert_eq!(before, crate::mir::lower(&p));
    }

    #[test]
    fn outer_local_inner_continue_retains_original_id_then_outer_cleanup_reclaims_it() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; while flag { let view = &owned; while flag { let seen = *view; continue; } continue; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let outer = install_continue_target(&mut c, body, FutureUses::new());
        let uses = future_uses(&body[1..]);
        c.statements_with(&body[..1], f.id, &uses).unwrap();
        let id = *c.loans.keys().next().unwrap();
        let ValueStatement::While {
            body: inner_body,
            span: inner_span,
            ..
        } = &body[1]
        else {
            panic!()
        };
        let inner = install_continue_target(&mut c, inner_body, FutureUses::new());
        c.remaining.clear();
        c.expire();
        assert_eq!(c.recurrent.len(), 2);
        assert_eq!(c.continue_edge(*inner_span).unwrap(), Flow::Continue);
        assert!(c.same_loop_state(&inner));
        assert!(c.loans.contains_key(&id));
        assert_eq!(c.targets.len(), 2);
        assert_eq!(c.recurrent.len(), 2);
        // Emulate the checked inner false edge: only its frame/context ends.
        c.recurrent.pop();
        c.targets.pop();
        c.loop_scope = Some(LoopScope::new(&outer, false));
        c.expire();
        assert!(c.loans.is_empty());
        assert_eq!(c.continue_edge(*span).unwrap(), Flow::Continue);
        assert!(c.same_loop_state(&outer));
        assert_eq!(c.next_loan, outer.next_loan);
        assert!(c.handles.is_empty());
    }

    // Inspect the actual normalized edge before while_loop consumes it. Public
    // acceptance alone could otherwise conceal an unvalidated break candidate.
    #[test]
    fn break_recurrent_only_candidate_reduces_to_independent_false_exit() {
        for handle in ["let view = &owned;", "let view = &mut owned;"] {
            let p = typed(&format!("fn f(flag: bool) -> bool {{ let mut owned = false; {handle} while flag {{ let seen = *view; break; }} return owned; }}"));
            let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..2], f.id).unwrap();
            let ValueStatement::While { body, span, .. } = &f.body[2] else {
                panic!()
            };
            let header = install_continue_target(&mut c, body, FutureUses::new());
            let target = c.targets.last().unwrap().clone();
            let canonical = (*target.exit).clone();
            let id = *c.loans.keys().next().unwrap();
            c.remaining.clear();
            c.expire();
            assert!(c.recurrent[0].contains(&id));
            assert!(c.loans.contains_key(&id));
            assert_eq!(c.break_edge(*span).unwrap(), Flow::Break);
            assert!(c.same_loop_state(&canonical));
            assert!(!c.same_loop_state(&header)); // Legitimate exit reduction, not a backedge.
            assert!(c.recurrent.is_empty());
            assert!(c.targets.is_empty());
            assert!(c.loop_scope.is_none());
            assert!(c.loans.is_empty());
            assert_eq!(c.handles, header.handles); // Dead spelling does not recreate a loan.
            assert_eq!(c.next_loan, header.next_loan);
            assert_atomic(&target.exit, &canonical); // Candidate never defines/replaces it.
        }
    }
    #[test]
    fn break_installs_outside_before_expiry_and_preserves_original_provenance() {
        for mutable in [false, true] {
            for recurrent in [false, true] {
                let binding = if mutable {
                    "let view = &mut owned;"
                } else {
                    "let view = &owned;"
                };
                let use_in_body = if recurrent { "let seen = *view;" } else { "" };
                let use_after = if mutable {
                    "*view = true;"
                } else {
                    "let later = *view;"
                };
                let p = typed(&format!("fn f(flag: bool) -> bool {{ let mut owned = false; {binding} while flag {{ {use_in_body} break; }} {use_after} return owned; }}"));
                let f = p.functions()[0].as_ordinary().unwrap();
                let mut c = Checker::new(&p, f);
                c.statements(&f.body[..2], f.id).unwrap();
                let ValueStatement::While { body, span, .. } = &f.body[2] else {
                    panic!()
                };
                let outside = future_uses(&f.body[3..]);
                let header = install_continue_target(&mut c, body, outside.clone());
                let target = c.targets.last().unwrap().clone();
                let id = *header.loans.keys().next().unwrap();
                // Simulate exhausted edge bookkeeping without expiring evidence.
                c.remaining.clear();
                assert_eq!(c.break_edge(*span).unwrap(), Flow::Break);
                assert_eq!(c.remaining, outside);
                assert!(c.same_loop_state(&target.exit));
                assert_eq!(c.handles, header.handles);
                assert_eq!(c.next_loan, header.next_loan);
                assert_eq!(c.loans[&id].owner, header.loans[&id].owner);
                assert_eq!(c.loans[&id].kind, header.loans[&id].kind);
                assert_eq!(c.loans[&id].held, 0);
                assert!(c.recurrent.is_empty());
                c.statements(&f.body[3..], f.id).unwrap();
                assert!(c.loans.is_empty());
            }
        }
    }
    #[test]
    fn break_gate_rejects_frame_drift_before_legitimate_pop_hides_it() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        for frame in [BTreeSet::new(), BTreeSet::from([LoanId(999)])] {
            let mut broken = c.clone();
            *broken.recurrent.last_mut().unwrap() = frame;
            let before = broken.clone();
            let target = broken.targets.last().unwrap().clone();
            let mut erased = broken.clone();
            erased.project_break_exit(&target, *span).unwrap();
            assert!(erased.same_loop_state(&target.exit)); // Cleanup would erase the defect.
            assert!(broken
                .break_edge(*span)
                .unwrap_err()
                .message
                .contains("recurrent-obligation integrity"));
            assert_atomic(&broken, &before); // Restore injected state, not the header.
        }
    }
    #[test]
    fn break_gate_rejects_carried_corruption_before_recurrent_only_expiry() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        let id = *header.loans.keys().next().unwrap();
        for fault in 0..3 {
            let mut broken = c.clone();
            match fault {
                0 => broken.loans.get_mut(&id).unwrap().owner = None,
                1 => broken.loans.get_mut(&id).unwrap().kind = BorrowKind::Mutable,
                _ => {
                    broken.loans.remove(&id);
                }
            }
            let before = broken.clone();
            let target = broken.targets.last().unwrap().clone();
            let mut erased = broken.clone();
            erased.project_break_exit(&target, *span).unwrap();
            assert!(erased.same_loop_state(&target.exit)); // All evidence legitimately expires.
            assert!(broken
                .break_edge(*span)
                .unwrap_err()
                .message
                .contains("carried-loan integrity"));
            assert_atomic(&broken, &before);
        }
    }
    #[test]
    fn break_gate_never_clears_unbalanced_carried_or_fresh_holds() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        for fresh in [false, true] {
            let mut broken = c.clone();
            let id = if fresh {
                broken.add_loan(None, BorrowKind::Shared, *span)
            } else {
                *broken.loans.keys().next().unwrap()
            };
            broken.loans.get_mut(&id).unwrap().held = 1;
            let before = broken.clone();
            assert!(broken
                .break_edge(*span)
                .unwrap_err()
                .message
                .contains("unbalanced operation holds"));
            assert_atomic(&broken, &before);
            assert_eq!(broken.loans[&id].held, 1);
        }
    }
    #[test]
    fn break_gate_rejects_fresh_or_missing_carried_handle_without_allocator_reset() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        let place = *header.handles.keys().next().unwrap();
        for fault in 0..3 {
            let mut broken = c.clone();
            match fault {
                0 => {
                    broken.handles.remove(&place);
                }
                1 => {
                    broken.handles.insert(place, LoanId(999));
                }
                _ => {
                    let id = broken.add_loan(None, BorrowKind::Shared, *span);
                    broken.handles.insert(place, id);
                }
            }
            let before = broken.clone();
            assert!(broken
                .break_edge(*span)
                .unwrap_err()
                .message
                .contains("carried reference provenance"));
            assert_atomic(&broken, &before);
        }
    }
    #[test]
    fn break_rejects_availability_drift_after_cleanup_and_rolls_back_every_field() {
        let p = typed("fn f(flag: bool, ticket: Ticket) -> bool { let owned = false; let view = &owned; while flag { let alias = view; let seen = *alias; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        c.statements_with(&body[..2], f.id, &FutureUses::new())
            .unwrap();
        let ticket = Place::Parameter(f.parameters[1]);
        c.states.insert(
            ticket,
            State::Moved {
                span: *span,
                transfer: Transfer::Return(f.id),
            },
        );
        let before = c.clone();
        let target = c.targets.last().unwrap().clone();
        c.break_integrity(&target, *span).unwrap(); // No raw whole-state header comparison.
        let mut projected = c.clone();
        projected.project_break_exit(&target, *span).unwrap();
        assert_eq!(projected.states[&ticket], before.states[&ticket]); // Never repair availability.
        assert_eq!(projected.handles, header.handles);
        assert!(projected.loans.is_empty());
        assert!(projected.recurrent.is_empty());
        assert!(projected.targets.is_empty());
        assert!(!projected.same_loop_state(&target.exit));
        assert!(c
            .break_edge(*span)
            .unwrap_err()
            .message
            .contains("canonical loop-exit"));
        assert_atomic(&c, &before); // Alias and frame cleanup is transactional too.
    }
    #[test]
    fn break_allocator_proof_precedes_reset_and_failure_is_atomic() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { let seen = *view; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        let mut broken = c.clone();
        broken.next_loan = 0;
        let before = broken.clone();
        assert!(broken
            .break_edge(*span)
            .unwrap_err()
            .message
            .contains("allocation state"));
        assert_atomic(&broken, &before); // Failed after frame pop and expiry, yet exact rollback.
        for handle in [false, true] {
            let mut broken = header.clone();
            let id = broken.add_loan(None, BorrowKind::Shared, *span);
            if handle {
                broken.loans.remove(&id);
                broken
                    .handles
                    .insert(*header.handles.keys().next().unwrap(), id);
            }
            let before = broken.clone();
            assert!(broken
                .normalize_loop_allocator(&header, *span, "break exit")
                .unwrap_err()
                .message
                .contains("survives break exit"));
            assert_atomic(&broken, &before);
        }
        let mut sibling = c.clone();
        assert_eq!(sibling.break_edge(*span).unwrap(), Flow::Break);
        assert_eq!(c.next_loan, header.next_loan);
        assert_eq!(c.targets.len(), 1);
    }
    #[test]
    fn break_cleans_real_fresh_local_provenance_before_allocator_reuse() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; while flag { let access = &mut owned; let second = access; *second = true; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        let target = c.targets.last().unwrap().clone();
        c.statements_with(&body[..3], f.id, &FutureUses::new())
            .unwrap();
        assert!(c.next_loan > header.next_loan);
        assert!(!c.handles.is_empty());
        assert_eq!(c.break_edge(*span).unwrap(), Flow::Break);
        assert!(c.same_loop_state(&target.exit));
        assert!(c.handles.is_empty());
        assert!(c.loans.is_empty());
        assert_eq!(c.next_loan, header.next_loan);
        assert!(c.states.is_empty()); // Branch-local Move handles were scoped, not repaired.
    }
    #[test]
    fn inner_break_pops_exactly_one_frame_even_when_empty_and_preserves_outer_context() {
        for inner_use in ["", "let inside = *view;"] {
            let p = typed(&format!("fn f(flag: bool) -> bool {{ let owned = false; let view = &owned; while flag {{ while flag {{ {inner_use} break; }} let seen = *view; continue; }} return owned; }}"));
            let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..2], f.id).unwrap();
            let ValueStatement::While { body, span, .. } = &f.body[2] else {
                panic!()
            };
            let outer = install_continue_target(&mut c, body, FutureUses::new());
            let outer_target = c.targets.last().unwrap().clone();
            let ValueStatement::While {
                body: inner_body,
                span: inner_span,
                ..
            } = &body[0]
            else {
                panic!()
            };
            let inner = install_continue_target(&mut c, inner_body, future_uses(&body[1..]));
            let inner_target = c.targets.last().unwrap().clone();
            let id = *c.loans.keys().next().unwrap();
            c.remaining.clear();
            c.expire();
            assert_eq!(c.recurrent.len(), 2);
            assert_eq!(c.recurrent[0], BTreeSet::from([id]));
            assert_eq!(c.break_edge(*inner_span).unwrap(), Flow::Break);
            assert!(c.same_loop_state(&inner_target.exit));
            assert_eq!(c.recurrent, outer.recurrent);
            assert_eq!(c.loans[&id].owner, inner.loans[&id].owner);
            assert_eq!(c.loans[&id].kind, inner.loans[&id].kind);
            assert_eq!(c.targets.len(), 1);
            assert!(Rc::ptr_eq(&c.targets[0].header, &outer_target.header));
            assert!(Rc::ptr_eq(&c.targets[0].exit, &outer_target.exit));
            assert_eq!(c.loop_scope, Some(LoopScope::new(&outer, false)));
            assert_eq!(c.continue_edge(*span).unwrap(), Flow::Continue);
            assert!(c.same_loop_state(&outer));
        }
    }
    #[test]
    fn inner_only_recurrence_can_end_without_popping_empty_outer_frame() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; while flag { let view = &owned; while flag { let seen = *view; break; } owned = true; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let outer = install_continue_target(&mut c, body, FutureUses::new());
        c.statements_with(&body[..1], f.id, &future_uses(&body[1..]))
            .unwrap();
        let ValueStatement::While {
            body: inner_body,
            span: inner_span,
            ..
        } = &body[1]
        else {
            panic!()
        };
        install_continue_target(&mut c, inner_body, future_uses(&body[2..]));
        let id = *c.loans.keys().next().unwrap();
        assert!(c.recurrent[1].contains(&id));
        assert_eq!(c.break_edge(*inner_span).unwrap(), Flow::Break);
        assert_eq!(c.recurrent, vec![BTreeSet::new()]);
        assert!(c.loans.is_empty());
        assert_eq!(c.targets.len(), 1);
        assert!(c.handles.values().any(|saved| *saved == id));
        c.statements_with(&body[2..3], f.id, &FutureUses::new())
            .unwrap();
        assert_eq!(c.break_edge(*span).unwrap(), Flow::Break);
        assert_eq!(c.next_loan, outer.next_loan);
        assert!(c.handles.is_empty());
        assert!(c.recurrent.is_empty());
        assert!(c.targets.is_empty());
    }
    #[test]
    fn nested_break_failure_preserves_injected_frames_scope_targets_and_sibling() {
        let p = typed("fn f(flag: bool, ticket: Ticket) -> bool { let owned = false; let view = &owned; while flag { while flag { break; } let seen = *view; break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let ValueStatement::While {
            body: inner_body,
            span: inner_span,
            ..
        } = &body[0]
        else {
            panic!()
        };
        install_continue_target(&mut c, inner_body, future_uses(&body[1..]));
        for fault in 0..6 {
            let mut broken = c.clone();
            match fault {
                0 => broken.recurrent.swap(0, 1),
                1 => broken.recurrent[0].clear(),
                2 => broken.loop_scope.as_mut().unwrap().condition = true,
                3 => {
                    broken.targets[0].outside.clear();
                }
                4 => {
                    broken.states.insert(
                        Place::Parameter(f.parameters[1]),
                        State::Moved {
                            span: *inner_span,
                            transfer: Transfer::Return(f.id),
                        },
                    );
                }
                _ => broken.next_loan = 0,
            }
            // Ensure the last fault is observable even if the original outer map was empty.
            if fault == 3 {
                broken.targets[0]
                    .outside
                    .insert(Place::Parameter(f.parameters[0]), 99);
            }
            let before = broken.clone();
            assert!(broken.break_edge(*inner_span).is_err());
            assert_atomic(&broken, &before);
            let mut sibling = c.clone();
            assert_eq!(sibling.break_edge(*inner_span).unwrap(), Flow::Break);
            assert_eq!(sibling.targets.len(), 1);
            assert_eq!(sibling.break_edge(*span).unwrap(), Flow::Break);
            assert!(sibling.targets.is_empty());
            assert!(sibling.recurrent.is_empty());
        }
    }
    #[test]
    fn break_rejects_recreated_dead_carried_loan_before_exit_erases_it() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { break; } return owned; }");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        assert!(header.loans.is_empty());
        let id = *header.handles.values().next().unwrap();
        c.loans.insert(
            id,
            Loan {
                owner: None,
                origin: LoanOrigin::Unbound,
                kind: BorrowKind::Shared,
                span: *span,
                held: 0,
            },
        );
        let before = c.clone();
        assert!(c
            .break_edge(*span)
            .unwrap_err()
            .message
            .contains("resurrected"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn all_terminal_mixed_edges_have_no_survivor_or_fabricated_suffix_join() {
        for arms in [
            "if flag { break; } else { continue; }",
            "if flag { continue; } else { break; }",
        ] {
            let p = typed(&format!(
                "fn f(flag: bool) -> bool {{ while flag {{ {arms} }} return true; }}"
            ));
            let f = p.functions()[0].as_ordinary().unwrap();
            let mut c = Checker::new(&p, f);
            let ValueStatement::While { body, .. } = &f.body[0] else {
                panic!()
            };
            install_continue_target(&mut c, body, FutureUses::new());
            assert_eq!(
                c.statements_with(body, f.id, &FutureUses::new()).unwrap(),
                Flow::NoFallthrough
            );
            // The enclosing state is not replaced by either terminal arm.
            assert_eq!(c.recurrent.len(), 1);
            assert_eq!(c.targets.len(), 1);
        }
    }
    #[test]
    fn break_canonical_ids_ignore_all_diagnostic_display_names() {
        let rename = |p: &mut Program| {
            for range in &mut p.ranges {
                range.name = "display".into();
            }
            for record in &mut p.records {
                record.name = "display".into();
            }
            for function in &mut p.functions {
                match function {
                    crate::hir::Function::Ordinary(f) => f.name = "display".into(),
                    crate::hir::Function::Verified(f) => f.name = "display".into(),
                }
            }
            for parameter in &mut p.parameters {
                parameter.name = "display".into();
            }
            for local in &mut p.locals {
                local.name = "display".into();
            }
        };
        let mut p = typed("fn f(flag: bool) -> bool { let mut owned = false; while flag { let view = &owned; if flag { owned = true; break; } let seen = *view; } return owned; }");
        check(&p).unwrap();
        let before = crate::mir::lower(&p);
        rename(&mut p);
        check(&p).unwrap();
        assert_eq!(before, crate::mir::lower(&p));
        let mut p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { if flag { owned = true; break; } let seen = *view; } return owned; }");
        let before = check(&p).unwrap_err();
        rename(&mut p);
        let after = check(&p).unwrap_err();
        assert_eq!(before[0].span, after[0].span);
        assert_eq!(before[0].required, after[0].required);
        assert!(after[0].message.contains("display"));
    }

    // Frozen BED-018 return-entry state: summaries/expiry are static inputs,
    // and the terminal operand is the only transactional operation.
    fn return_entry<'a>(c: &Checker<'a>, value: &TypedExpr) -> Checker<'a> {
        let mut entry = c.clone();
        entry.remaining.clear();
        count_expression(value, &mut entry.remaining);
        entry.expire();
        entry
    }
    fn operand(statement: &ValueStatement) -> (&TypedExpr, Span) {
        let ValueStatement::Return { value, span } = statement else {
            panic!("return operand")
        };
        (value, *span)
    }
    const RETURN_HELPERS: &str = "fn take(ticket: Ticket) -> Ticket { return ticket; } fn inspect(view: &Ticket) -> bool { return true; } fn holding(view: &Ticket, ticket: Ticket) -> Ticket { return ticket; } fn after(seen: bool, ticket: Ticket) -> Ticket { return ticket; } fn combine(first: Ticket, view: &bool, second: Ticket) -> Ticket { return second; } fn sink(access: &mut bool) -> bool { return true; } fn fresh() -> Ticket { return fresh(); }";
    fn return_typed(body: &str) -> Program {
        typed(&format!("{RETURN_HELPERS} {body}"))
    }
    fn return_function(p: &Program) -> &ValueFunction {
        p.functions().last().unwrap().as_ordinary().unwrap()
    }

    #[test]
    fn return_finite_summary_is_operand_only_even_with_all_skipped_contexts() {
        let p = typed("fn f(flag: bool, value: bool) -> bool { return value; }");
        let f = return_function(&p);
        let (value, _) = operand(&f.body[0]);
        let outside = BTreeMap::from([(Place::Parameter(f.parameters[0]), 9)]);
        let expected = BTreeMap::from([(Place::Parameter(f.parameters[1]), 1)]);
        assert_eq!(statement_future(&f.body[0], &outside, &outside), expected);
        assert_eq!(block_future(&f.body, &outside, &outside), expected);
        let mut c = Checker::new(&p, f);
        c.remaining = outside;
        let entry = return_entry(&c, value);
        assert_eq!(entry.remaining, expected);
    }
    #[test]
    fn return_entry_expiry_is_not_rolled_back_to_old_outside_summary() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { let view = &ticket; while flag { return combine(ticket, &flag, ticket); } return after(inspect(view), ticket); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[1] else {
            panic!()
        };
        let outside = future_uses(&f.body[2..]);
        install_continue_target(&mut c, body, outside.clone());
        assert_eq!(c.loans.len(), 1);
        let (value, span) = operand(&body[0]);
        let entry = return_entry(&c, value);
        assert!(entry.loans.is_empty());
        assert!(!entry
            .remaining
            .contains_key(&Place::Local(p.locals()[0].id)));
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &entry);
        // The statement-list wrapper must preserve exactly that same boundary.
        let mut direct = entry.clone();
        direct.remaining = outside.clone();
        assert!(direct.statements_with(body, f.id, &outside).is_err());
        assert_atomic(&direct, &entry);
        assert!(!c.terminal_return);
    }
    #[test]
    fn terminal_permission_is_not_the_return_transfer_tag_or_while_condition_tag() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return take(take(ticket)); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (value, span) = operand(&body[0]);
        c = return_entry(&c, value);
        let entry = c.clone();
        assert!(c
            .expression(value, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("pre-existing non-Copy"));
        assert_atomic(&c, &entry);
        assert_eq!(c.return_edge(value, f.id, span).unwrap(), Flow::Return);
        assert!(matches!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Moved { .. }
        ));
        assert!(!c.terminal_return);
        assert_eq!(c.targets.len(), 1);
        assert_eq!(c.recurrent.len(), 1);
    }
    #[test]
    fn return_hold_conflict_restores_created_provenance_and_permission() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return holding(&ticket, ticket); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (value, span) = operand(&body[0]);
        let entry = return_entry(&c, value);
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("shared-borrowed"));
        assert_atomic(&c, &entry);
        assert_eq!(c.next_loan, 0);
        assert!(!c.terminal_return);
        // A failed operand cannot grant a later ordinary expression permission.
        let ticket = TypedExpr {
            kind: ExprKind::Parameter(f.parameters[1]),
            ty: value.ty,
            span,
        };
        assert!(c
            .expression(&ticket, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("pre-existing non-Copy"));
    }
    #[test]
    fn late_nested_return_argument_rolls_back_earlier_move_borrow_counts_and_holds() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return combine(take(ticket), &flag, ticket); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (value, span) = operand(&body[0]);
        let entry = return_entry(&c, value);
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &entry);
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
        assert!(c.loans.is_empty());
    }
    #[test]
    fn fallible_final_return_validation_restores_successful_operand_effects() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return take(ticket); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (value, span) = operand(&body[0]);
        let held = c.add_loan(None, BorrowKind::Shared, span);
        c.loans.get_mut(&held).unwrap().held = 1; // Invalid escaped operation hold.
        let entry = return_entry(&c, value);
        let mut successful_operand = entry.clone();
        successful_operand.terminal_return = true;
        successful_operand
            .expression(value, Transfer::Return(f.id))
            .unwrap();
        assert!(matches!(
            successful_operand.states[&Place::Parameter(f.parameters[1])],
            State::Moved { .. }
        ));
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("operation holds"));
        assert_atomic(&c, &entry);
        assert_eq!(c.loans[&held].held, 1); // Rollback, never hold clearing/repair.
    }
    #[test]
    fn return_success_preserves_original_recurrence_and_never_normalizes_local_successor() {
        let p = typed("fn f(flag: bool) -> bool { let mut owned = false; let view = &owned; while flag { let temporary = &flag; return *view; } return owned; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, future_uses(&f.body[3..]));
        let carried = *header.loans.keys().next().unwrap();
        let mut after = FutureUses::new();
        count_expression(operand(&body[1]).0, &mut after);
        c.statements_with(&body[..1], f.id, &after).unwrap();
        let next = c.next_loan;
        assert!(next > header.next_loan);
        let frames = c.recurrent.clone();
        let targets = c.targets.clone();
        let handles = c.handles.clone();
        let (value, span) = operand(&body[1]);
        assert_eq!(c.return_edge(value, f.id, span).unwrap(), Flow::Return);
        assert_eq!(c.next_loan, next); // No header allocator reset.
        assert_eq!(c.handles, handles); // No lexical successor cleanup.
        assert_eq!(c.recurrent, frames); // No frame discharge.
        assert_eq!(c.loans[&carried].owner, header.loans[&carried].owner);
        assert_eq!(c.loans[&carried].kind, header.loans[&carried].kind);
        assert_eq!(c.remaining[&Place::Local(p.locals()[1].id)], 0);
        assert!(Rc::ptr_eq(&c.targets[0].header, &targets[0].header));
        assert!(Rc::ptr_eq(&c.targets[0].exit, &targets[0].exit));
        assert!(c.loans.values().all(|l| l.held == 0));
        assert!(!c.terminal_return);
    }
    #[test]
    fn return_only_reference_use_discovers_original_recurrent_loan() {
        let p = typed("fn f(flag: bool) -> bool { let owned = false; let view = &owned; while flag { return *view; } return owned; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        let carried = c.handles[&Place::Local(p.locals()[1].id)];
        install_continue_target(&mut c, body, FutureUses::new());
        assert_eq!(c.recurrent.last().unwrap(), &BTreeSet::from([carried]));
        let (value, span) = operand(&body[0]);
        c.return_edge(value, f.id, span).unwrap();
        assert!(c.loans.contains_key(&carried));
        assert!(!c.terminal_return);
    }
    #[test]
    fn return_nested_call_does_not_expire_recurrence_after_last_finite_reference_use() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { let view = &ticket; while flag { return after(inspect(view), ticket); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[1] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        let id = *header.loans.keys().next().unwrap();
        let (value, span) = operand(&body[0]);
        let entry = return_entry(&c, value);
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("shared-borrowed"));
        assert_atomic(&c, &entry);
        assert_eq!(c.recurrent[0], BTreeSet::from([id]));
        assert_eq!(c.loans[&id].owner, header.loans[&id].owner);
    }
    #[test]
    fn return_empty_inner_frame_cannot_discharge_enclosing_recurrence() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { let view = &ticket; while flag { while flag { return ticket; } let seen = inspect(view); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body: outer, .. } = &f.body[1] else {
            panic!()
        };
        install_continue_target(&mut c, outer, FutureUses::new());
        let ValueStatement::While { body: inner, .. } = &outer[0] else {
            panic!()
        };
        install_continue_target(&mut c, inner, FutureUses::new());
        assert_eq!(c.recurrent.len(), 2);
        assert!(!c.recurrent[0].is_empty());
        assert!(c.recurrent[1].is_empty());
        let (value, span) = operand(&inner[0]);
        let entry = return_entry(&c, value);
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("shared-borrowed"));
        assert_atomic(&c, &entry);
    }
    #[test]
    fn return_permission_never_authorizes_carried_mutable_handle_transfer() {
        let p = return_typed("fn f(flag: bool, access: &mut bool) -> bool { while flag { return sink(access); } return false; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (value, span) = operand(&body[0]);
        let entry = return_entry(&c, value);
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("loop-carried mutable reference"));
        assert_atomic(&c, &entry);
    }
    #[test]
    fn return_success_balances_nested_direct_borrow_call_holds() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { return after(inspect(&ticket), ticket); } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (value, span) = operand(&body[0]);
        assert_eq!(c.return_edge(value, f.id, span).unwrap(), Flow::Return);
        assert!(c.loans.is_empty());
        assert_eq!(c.next_loan, 1);
        assert!(!c.terminal_return);
    }
    #[test]
    fn return_branch_state_does_not_enter_survivor_or_canonical_false_exit() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { if flag { let temporary = &flag; return ticket; } else { continue; } } return ticket; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, future_uses(&f.body[1..]));
        let before = c.clone();
        assert_eq!(
            c.statements_with(body, f.id, &FutureUses::new()).unwrap(),
            Flow::NoFallthrough
        );
        // Both alternatives terminate locally; neither becomes a merged state.
        assert_eq!(c.states, before.states);
        assert_eq!(c.next_loan, before.next_loan);
        assert!(!c.terminal_return);
        let exit = &c.targets[0].exit;
        assert_eq!(
            exit.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
        assert_eq!(exit.next_loan, header.next_loan);
        assert!(!exit.terminal_return);
    }
    #[test]
    fn return_survivor_keeps_actual_move_effect_without_repair() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { return fresh(); } else { let moved = ticket; } return ticket; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let after = future_uses(&f.body[1..]);
        assert_eq!(
            c.statements_with(&f.body[..1], f.id, &after).unwrap(),
            Flow::Fallthrough
        );
        assert!(matches!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Moved { .. }
        ));
        let (value, span) = operand(&f.body[1]);
        let entry = return_entry(&c, value);
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &entry);
    }
    #[test]
    fn return_callee_checker_never_inherits_callers_terminal_permission() {
        let p = return_typed("fn caller(flag: bool, ticket: Ticket) -> Ticket { while flag { return invalid(flag, ticket); } return fresh(); } fn invalid(flag: bool, ticket: Ticket) -> Ticket { while flag { let moved = ticket; return moved; } return fresh(); }");
        let errors = check(&p).unwrap_err();
        assert!(errors[0].message.contains("pre-existing non-Copy"));
    }
    #[test]
    fn return_metadata_renaming_preserves_ids_cfg_recurrence_and_errors() {
        fn rename(p: &mut Program) {
            for range in &mut p.ranges {
                range.name = "display".into();
            }
            for record in &mut p.records {
                record.name = "display".into();
            }
            for f in &mut p.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
            for parameter in &mut p.parameters {
                parameter.name = "display".into();
            }
            for local in &mut p.locals {
                local.name = "display".into();
            }
        }
        let mut p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { while flag { if flag { return take(ticket); } else { continue; } } return ticket; }");
        check(&p).unwrap();
        let before = crate::mir::lower(&p);
        rename(&mut p);
        check(&p).unwrap();
        assert_eq!(before, crate::mir::lower(&p));
        let mut p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { let view = &ticket; while flag { return after(inspect(view), ticket); } return fresh(); }");
        let before = check(&p).unwrap_err();
        rename(&mut p);
        let after = check(&p).unwrap_err();
        assert_eq!(before[0].span, after[0].span);
        assert_eq!(before[0].required, after[0].required);
        assert!(after[0].message.contains("display"));
    }

    #[test]
    fn return_tagged_while_condition_has_no_terminal_permission() {
        let p = return_typed("fn predicate(ticket: Ticket) -> bool { return true; } fn f(ticket: Ticket) -> Ticket { while predicate(ticket) { return ticket; } return fresh(); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While {
            condition, body, ..
        } = &f.body[0]
        else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        c.loop_scope.as_mut().unwrap().condition = true;
        c.remaining.clear();
        count_expression(condition, &mut c.remaining);
        let before = c.clone();
        assert!(c
            .expression(condition, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("while conditions"));
        assert_atomic(&c, &before);
        assert!(!c.terminal_return);
    }
    #[test]
    fn returning_branch_move_allocator_and_permission_never_enter_real_fallthrough() {
        let p = return_typed("fn f(flag: bool, ticket: Ticket) -> Ticket { if flag { let temporary = &flag; return take(ticket); } let moved = ticket; return moved; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let after = future_uses(&f.body[1..]);
        assert_eq!(
            c.statements_with(&f.body[..1], f.id, &after).unwrap(),
            Flow::Fallthrough
        );
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
        assert_eq!(c.next_loan, 0);
        assert!(c.loans.is_empty());
        assert!(c.handles.is_empty());
        assert!(!c.terminal_return);
        assert_eq!(
            c.statements_with(&f.body[1..], f.id, &FutureUses::new())
                .unwrap(),
            Flow::Return
        );
    }

    // B19-01 tests invoke the statement transaction directly. A surrounding
    // expression/statement-list/conditional rollback cannot mask a partial commit.
    const NO_VALUE_HELPERS: &str = "fn observe(view: &bool) {} fn consume(ticket: Ticket) {} fn receive(first: Ticket, held: &bool, last: Ticket) {} fn nested(first: Ticket, held: &bool, last: bool) {} fn touch(access: &mut bool) -> bool { return true; } fn read(view: &bool) -> bool { return *view; } fn shared_bool(view: &bool, last: bool) {} fn reset(access: &mut bool) {}";
    fn no_value_typed(source: &str) -> Program {
        typed(&format!("{NO_VALUE_HELPERS} {source}"))
    }
    fn statement_call(statement: &ValueStatement) -> (FunctionId, &[TypedExpr], Span) {
        let ValueStatement::CallNoValue {
            function,
            arguments,
            span,
        } = statement
        else {
            panic!("no-value call")
        };
        (*function, arguments, *span)
    }
    fn prepare_call(c: &mut Checker<'_>, arguments: &[TypedExpr]) {
        c.remaining.clear();
        for argument in arguments {
            count_expression(argument, &mut c.remaining);
        }
        c.expire();
    }
    #[test]
    fn no_value_late_argument_failure_restores_direct_transaction_entry() {
        let p =
            no_value_typed("fn f(ticket: Ticket, flag: bool) { receive(ticket, &flag, ticket); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let (callee, args, span) = statement_call(&f.body[0]);
        prepare_call(&mut c, args);
        let entry = c.clone();
        assert!(c
            .call_statement(callee, args, span)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &entry);
        assert_eq!(c.next_loan, 0);
        assert!(c.loans.is_empty());
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[0])],
            State::Available
        );
    }
    #[test]
    fn no_value_nested_hold_failure_restores_prefix_move_and_all_borrow_state() {
        let p=no_value_typed("fn f(ticket: Ticket) { let mut value = false; nested(ticket, &value, touch(&mut value)); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let (callee, args, span) = statement_call(&f.body[1]);
        prepare_call(&mut c, args);
        let entry = c.clone();
        assert!(c
            .call_statement(callee, args, span)
            .unwrap_err()
            .message
            .contains("shared borrow is active"));
        assert_atomic(&c, &entry);
        assert_eq!(c.next_loan, 0);
        assert!(!c.terminal_return);
    }
    #[test]
    fn no_value_final_call_validation_rolls_back_successful_arguments_without_clearing_fault() {
        let p = no_value_typed("fn f(ticket: Ticket) { consume(ticket); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let (callee, args, span) = statement_call(&f.body[0]);
        prepare_call(&mut c, args);
        let defect = c.add_loan(None, BorrowKind::Shared, span);
        c.loans.get_mut(&defect).unwrap().held = 1;
        let entry = c.clone();
        let mut successful = c.clone();
        successful.call_arguments(callee, args).unwrap();
        assert!(matches!(
            successful.states[&Place::Parameter(f.parameters[0])],
            State::Moved { .. }
        ));
        assert!(c
            .call_statement(callee, args, span)
            .unwrap_err()
            .message
            .contains("balanced operation holds"));
        assert_atomic(&c, &entry);
        assert_eq!(c.loans[&defect].held, 1);
    }
    #[test]
    fn no_value_call_error_preserves_nested_loop_context_recurrence_and_finite_continuation() {
        let p=no_value_typed("fn f(flag: bool) { let mut owned = false; let view = &owned; while flag { while flag { shared_bool(view, touch(&mut owned)); return; } observe(view); } }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body: outer, .. } = &f.body[2] else {
            panic!()
        };
        install_continue_target(&mut c, outer, FutureUses::new());
        let ValueStatement::While { body: inner, .. } = &outer[0] else {
            panic!()
        };
        install_continue_target(&mut c, inner, FutureUses::new());
        let (callee, args, span) = statement_call(&inner[0]);
        prepare_call(&mut c, args);
        let entry = c.clone();
        assert_eq!(entry.recurrent.len(), 2);
        assert!(c.call_statement(callee, args, span).is_err());
        assert_atomic(&c, &entry);
    }
    #[test]
    fn no_value_call_never_inherits_value_return_move_permission() {
        let p = no_value_typed(
            "fn f(flag: bool, ticket: Ticket) { while flag { consume(ticket); return; } }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        install_continue_target(&mut c, body, FutureUses::new());
        let (callee, args, span) = statement_call(&body[0]);
        prepare_call(&mut c, args);
        // Even privileged inherited context cannot give a statement the exemption.
        c.terminal_return = true;
        let entry = c.clone();
        assert!(c
            .call_statement(callee, args, span)
            .unwrap_err()
            .message
            .contains("pre-existing non-Copy"));
        assert_atomic(&c, &entry);
    }
    #[test]
    fn no_value_successful_call_has_exact_move_destination_and_balanced_holds() {
        let p = no_value_typed(
            "fn f(ticket: Ticket, flag: bool, second: Ticket) { receive(ticket, &flag, second); }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let (callee, args, span) = statement_call(&f.body[0]);
        prepare_call(&mut c, args);
        c.call_statement(callee, args, span).unwrap();
        let callee = p.function(callee).as_ordinary().unwrap();
        assert!(
            matches!(c.states[&Place::Parameter(f.parameters[0])],State::Moved{transfer:Transfer::Argument(id),..} if id==callee.parameters[0])
        );
        assert!(
            matches!(c.states[&Place::Parameter(f.parameters[2])],State::Moved{transfer:Transfer::Argument(id),..} if id==callee.parameters[2])
        );
        assert!(c.remaining.values().all(|n| *n == 0));
        assert!(c.loans.is_empty());
        assert_eq!(c.next_loan, 1);
        assert!(!c.terminal_return);
    }
    #[test]
    fn no_value_nested_calls_release_only_their_own_holds() {
        let p = no_value_typed(
            "fn f(value: bool) { let view = &value; shared_bool(view, read(view)); }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let (callee, args, span) = statement_call(&f.body[1]);
        prepare_call(&mut c, args);
        let id = c.handles[&Place::Local(p.locals()[0].id)];
        // Inspect the receiving call's prefix independently of the shared helper.
        let loan = c
            .expression(
                &args[0],
                Transfer::Argument(p.function(callee).as_ordinary().unwrap().parameters[0]),
            )
            .unwrap()
            .unwrap();
        assert_eq!(loan, id);
        c.loans.get_mut(&id).unwrap().held += 1;
        c.expire();
        c.expression(
            &args[1],
            Transfer::Argument(p.function(callee).as_ordinary().unwrap().parameters[1]),
        )
        .unwrap();
        assert_eq!(c.loans[&id].held, 1); // Nested read must not release the receiving hold.
        c.loans.get_mut(&id).unwrap().held -= 1;
        c.expire();
        assert!(c.loans.is_empty());
        let mut full = Checker::new(&p, f);
        full.statements(&f.body[..1], f.id).unwrap();
        prepare_call(&mut full, args);
        full.call_statement(callee, args, span).unwrap();
        assert!(full.loans.is_empty());
    }
    #[test]
    fn no_value_arguments_are_in_finite_and_recurrent_traversal_including_nested_calls() {
        let p=no_value_typed("fn f(flag: bool) { let value = false; let view = &value; while flag { while flag { shared_bool(view, read(view)); } } }");
        let f = return_function(&p);
        let view = Place::Local(p.locals()[1].id);
        let ValueStatement::While { body, .. } = &f.body[2] else {
            panic!()
        };
        assert_eq!(future_uses(body)[&view], 2);
        assert!(reference_uses(body).contains(&view));
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let original = c.handles[&view];
        install_continue_target(&mut c, body, FutureUses::new());
        assert_eq!(c.recurrent[0], BTreeSet::from([original]));
    }
    #[test]
    fn no_value_bare_return_summary_excludes_every_finite_continuation() {
        let p = no_value_typed("fn f(flag: bool) { return; }");
        let f = return_function(&p);
        let outside = BTreeMap::from([(Place::Parameter(f.parameters[0]), 10)]);
        assert!(statement_future(&f.body[0], &outside, &outside).is_empty());
        assert!(block_future(&f.body, &outside, &outside).is_empty());
    }
    #[test]
    fn no_value_bare_return_failed_hold_validation_keeps_prepared_not_old_summary_state() {
        let p = no_value_typed(
            "fn f(flag: bool) { let view = &flag; while flag { return; } observe(view); }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, .. } = &f.body[1] else {
            panic!()
        };
        let ValueStatement::ReturnNoValue { span } = body[0] else {
            panic!()
        };
        let held = c.add_loan(None, BorrowKind::Mutable, span);
        c.loans.get_mut(&held).unwrap().held = 1;
        let old = c.clone();
        let mut entry = c.clone();
        entry.remaining.clear();
        entry.expire();
        assert!(entry.loans.len() < old.loans.len());
        assert!(c
            .bare_return(span)
            .unwrap_err()
            .message
            .contains("balanced operation holds"));
        assert_atomic(&c, &entry);
        let mut wrapper = old;
        let after = wrapper.remaining.clone();
        assert!(wrapper.statements_with(body, f.id, &after).is_err());
        assert_atomic(&wrapper, &entry);
        assert!(!c.terminal_return);
        assert_eq!(c.loans[&held].held, 1);
    }
    #[test]
    fn no_value_bare_return_preserves_original_provenance_all_frames_and_allocator() {
        let p=no_value_typed("fn f(flag: bool) { let value = false; let view = &value; while flag { while flag { return; } observe(view); } }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::While { body: outer, .. } = &f.body[2] else {
            panic!()
        };
        install_continue_target(&mut c, outer, FutureUses::new());
        let ValueStatement::While { body: inner, .. } = &outer[0] else {
            panic!()
        };
        install_continue_target(&mut c, inner, FutureUses::new());
        assert!(!c.recurrent[0].is_empty());
        assert!(c.recurrent[1].is_empty());
        let ValueStatement::ReturnNoValue { span } = inner[0] else {
            panic!()
        };
        let mut entry = c.clone();
        entry.remaining.clear();
        entry.expire();
        assert_eq!(c.bare_return(span).unwrap(), Flow::Return);
        assert_atomic(&c, &entry);
        assert!(!c.terminal_return);
    }
    #[test]
    fn no_value_normal_completion_rejects_unbalanced_holds_without_normalization() {
        let p = no_value_typed("fn f(ticket: Ticket) {}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let held = c.add_loan(None, BorrowKind::Shared, f.span);
        c.loans.get_mut(&held).unwrap().held = 1;
        let entry = c.clone();
        assert!(c
            .complete_no_value(f.span)
            .unwrap_err()
            .message
            .contains("balanced operation holds"));
        assert_atomic(&c, &entry);
        // The function-body path must invoke the fallible completion check too.
        let mut expected = c.clone();
        expected.remaining.clear();
        expected.expire();
        assert!(c.body(f).is_err());
        assert_atomic(&c, &expected);
    }
    #[test]
    fn no_value_normal_completion_does_not_consume_unused_owner_or_normalize_state() {
        let p = no_value_typed("fn f(ticket: Ticket) {}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let entry = c.clone();
        c.complete_no_value(f.span).unwrap();
        assert_atomic(&c, &entry);
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[0])],
            State::Available
        );
        c.body(f).unwrap();
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[0])],
            State::Available
        );
    }
    #[test]
    fn no_value_terminated_move_does_not_enter_fallthrough_sibling() {
        let p=no_value_typed("fn f(flag: bool, ticket: Ticket) { if flag { consume(ticket); return; } consume(ticket); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let after = future_uses(&f.body[1..]);
        assert_eq!(
            c.statements_with(&f.body[..1], f.id, &after).unwrap(),
            Flow::Fallthrough
        );
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
        assert!(!c.terminal_return);
        c.statements_with(&f.body[1..], f.id, &FutureUses::new())
            .unwrap();
        assert!(matches!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Moved { .. }
        ));
    }
    #[test]
    fn no_value_top_level_completion_keeps_conditional_move_and_local_suffix_still_checks_it() {
        let p = no_value_typed("fn f(flag: bool, ticket: Ticket) { if flag { consume(ticket); } }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.body(f).unwrap();
        assert!(matches!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::ConditionalMove { .. }
        ));
        let entry = c.clone();
        c.complete_no_value(f.span).unwrap();
        assert_atomic(&c, &entry);
    }
    #[test]
    fn no_value_call_and_bare_return_outcomes_ignore_all_display_names() {
        fn rename(p: &mut Program) {
            for range in &mut p.ranges {
                range.name = "display".into();
            }
            for record in &mut p.records {
                record.name = "display".into();
            }
            for parameter in &mut p.parameters {
                parameter.name = "display".into();
            }
            for local in &mut p.locals {
                local.name = "display".into();
            }
            for f in &mut p.functions {
                if let crate::hir::Function::Ordinary(f) = f {
                    f.name = "display".into();
                }
            }
        }
        let mut p=no_value_typed("fn f(flag: bool) { let value = false; let view = &value; while flag { observe(view); if flag { return; } } }");
        check(&p).unwrap();
        let original = crate::mir::lower(&p);
        rename(&mut p);
        check(&p).unwrap();
        assert_eq!(original, crate::mir::lower(&p));
        let mut p =
            no_value_typed("fn f(ticket: Ticket, flag: bool) { receive(ticket, &flag, ticket); }");
        let original = check(&p).unwrap_err();
        rename(&mut p);
        let renamed = check(&p).unwrap_err();
        assert_eq!(original[0].span, renamed[0].span);
        assert_eq!(original[0].required, renamed[0].required);
        assert!(renamed[0].message.contains("display"));
    }

    const RECORD_HELPERS: &str = "fn consume(t: Ticket) -> Percent { return t.value; } fn duplicate(a: Ticket,b: Ticket) -> Percent { return a.value; } fn prefix(a: Ticket,view: &bool,b: Ticket) -> Percent { return a.value; } fn receive(b: Other,t: Ticket) {} fn read(view: &Percent) -> Percent { return *view; } fn inspect_mut(access: &mut Ticket) -> Percent { return inspect_mut(access); }";
    fn record_typed(body: &str) -> Program {
        typed(&format!("{RECORD_HELPERS} {body}"))
    }
    fn prepared_expression(c: &mut Checker<'_>, value: &TypedExpr) {
        c.remaining.clear();
        count_expression(value, &mut c.remaining);
        c.expire();
    }
    fn add_test_target(c: &mut Checker<'_>) {
        let header = Rc::new(c.clone());
        c.targets.push(LoopTarget {
            header: header.clone(),
            exit: header,
            outside: c.remaining.clone(),
        });
    }

    #[test]
    fn construction_success_is_fresh_and_has_only_written_initializer_effects() {
        let p = record_typed("fn f(p: Percent) -> Ticket { return Ticket { value: p }; }");
        let f = return_function(&p);
        let (value, _) = operand(&f.body[0]);
        let ExprKind::RecordConstruct { record, fields } = &value.kind else {
            panic!()
        };
        assert_eq!(p.record(*record).fields[0], fields[0].field);
        assert_eq!(p.field(fields[0].field).record, *record);
        let mut c = Checker::new(&p, f);
        prepared_expression(&mut c, value);
        let mut expected = c.clone();
        expected
            .remaining
            .insert(Place::Parameter(f.parameters[0]), 0);
        assert!(c
            .expression(value, Transfer::Return(f.id))
            .unwrap()
            .is_none());
        assert_atomic(&c, &expected);
        assert!(p.bindings().is_empty());
        assert!(p.locals().is_empty());
    }
    #[test]
    fn construction_direct_expression_rolls_back_successful_move_loan_hold_prefix() {
        let p = record_typed(
            "fn f(t: Ticket,flag: bool) -> Other { return Other { value: prefix(t,&flag,t) }; }",
        );
        let f = return_function(&p);
        let (value, _) = operand(&f.body[0]);
        let mut c = Checker::new(&p, f);
        prepared_expression(&mut c, value);
        let before = c.clone();
        let ExprKind::RecordConstruct { fields, .. } = &value.kind else {
            panic!()
        };
        let child = &fields[0].value;
        let ExprKind::Call { arguments, .. } = &child.kind else {
            panic!()
        };
        let mut prefix = c.clone();
        prefix
            .expression(&arguments[0], Transfer::Return(f.id))
            .unwrap();
        let loan = prefix
            .expression(&arguments[1], Transfer::Return(f.id))
            .unwrap()
            .unwrap();
        prefix.loans.get_mut(&loan).unwrap().held += 1;
        assert_ne!(prefix.states, before.states);
        assert_ne!(prefix.next_loan, before.next_loan);
        assert!(prefix.loans[&loan].held > 0);
        assert!(c
            .expression(value, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn construction_failure_does_not_publish_destination_owner() {
        let p =
            record_typed("fn f(t: Ticket) { let destination = Other { value: duplicate(t,t) }; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::Let {
            local, initializer, ..
        } = &f.body[0]
        else {
            panic!()
        };
        prepared_expression(&mut c, initializer);
        let before = c.clone();
        assert!(c.statement(&f.body[0], f.id, &FutureUses::new()).is_err());
        assert_atomic(&c, &before);
        assert!(!c.states.contains_key(&Place::Local(*local)));
    }
    #[test]
    fn construction_rollback_preserves_nested_context_and_recurrent_provenance() {
        let p=record_typed("fn f(t: Ticket,flag: bool) -> Other { let view = &flag; return Other { value: prefix(t,view,t) }; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let loan = *c.loans.keys().next().unwrap();
        c.recurrent.push(BTreeSet::from([loan]));
        add_test_target(&mut c);
        c.recurrent.push(BTreeSet::new());
        add_test_target(&mut c);
        c.loop_scope = Some(LoopScope::new(&c, false));
        let (value, _) = operand(&f.body[1]);
        prepared_expression(&mut c, value);
        c.terminal_return = true;
        let before = c.clone();
        assert!(c.expression(value, Transfer::Return(f.id)).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn construction_inside_kd033_preserves_prepared_return_entry_on_failure() {
        let p =
            record_typed("fn f(t: Ticket) -> Other { return Other { value: duplicate(t,t) }; }");
        let f = return_function(&p);
        let (value, span) = operand(&f.body[0]);
        let mut c = Checker::new(&p, f);
        c.loop_scope = Some(LoopScope::new(&c, false));
        prepared_expression(&mut c, value);
        let before = c.clone();
        assert!(c
            .return_edge(value, f.id, span)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn construction_successful_call_prefix_rolls_back_at_later_argument() {
        let p = record_typed("fn f(t: Ticket) { receive(Other { value: consume(t) },t); }");
        let f = return_function(&p);
        let (function, args, span) = statement_call(&f.body[0]);
        let mut c = Checker::new(&p, f);
        prepare_call(&mut c, args);
        let before = c.clone();
        let mut prefix = c.clone();
        assert!(prefix
            .expression(
                &args[0],
                Transfer::Argument(p.function(function).as_ordinary().unwrap().parameters[0])
            )
            .unwrap()
            .is_none());
        assert_ne!(prefix.states, before.states);
        assert!(c
            .call_statement(function, args, span)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn construction_statement_call_final_validation_rolls_back_successful_initializer() {
        let p=record_typed("fn f(t: Ticket,flag: bool,p: Percent) { receive(Other { value: consume(t) },Ticket { value: p }); }");
        // Use the successful construction only: injected outstanding hold causes final validation.
        let f = return_function(&p);
        let (function, args, span) = statement_call(&f.body[0]);
        let mut c = Checker::new(&p, f);
        let loan = c.add_loan(
            Some(Place::Parameter(f.parameters[1])),
            BorrowKind::Shared,
            f.span,
        );
        c.loans.get_mut(&loan).unwrap().held = 1;
        prepare_call(&mut c, args);
        let before = c.clone();
        let mut success = c.clone();
        success.call_arguments(function, args).unwrap();
        assert_ne!(success.states, before.states);
        assert!(c
            .call_statement(function, args, span)
            .unwrap_err()
            .message
            .contains("balanced operation holds"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn copy_field_read_preserves_available_root_transfer_history_and_all_other_state() {
        let p = record_typed("fn f(t: Ticket) -> Ticket { let seen = t.value; return t; }");
        let f = return_function(&p);
        let ValueStatement::Let { initializer, .. } = &f.body[0] else {
            panic!()
        };
        let mut c = Checker::new(&p, f);
        prepared_expression(&mut c, initializer);
        let mut expected = c.clone();
        expected
            .remaining
            .insert(Place::Parameter(f.parameters[0]), 0);
        let entries = c.states.len();
        assert!(c
            .expression(initializer, Transfer::Return(f.id))
            .unwrap()
            .is_none());
        assert_atomic(&c, &expected);
        assert_eq!(c.states.len(), entries);
        assert!(c.handles.is_empty());
        assert!(c.loans.is_empty());
    }
    #[test]
    fn field_reads_reject_moved_and_conditional_roots_without_partial_entries() {
        let p = record_typed("fn f(t: Ticket) -> Percent { return t.value; }");
        let f = return_function(&p);
        let (value, _) = operand(&f.body[0]);
        let owner = Place::Parameter(f.parameters[0]);
        for state in [
            State::Moved {
                span: f.span,
                transfer: Transfer::Return(f.id),
            },
            State::ConditionalMove { span: f.span },
        ] {
            let mut c = Checker::new(&p, f);
            c.states.insert(owner, state);
            prepared_expression(&mut c, value);
            let before = c.clone();
            assert!(c.expression(value, Transfer::Return(f.id)).is_err());
            assert_atomic(&c, &before);
            assert_eq!(c.states.len(), 1);
        }
    }
    #[test]
    fn shared_whole_record_loan_survives_field_read_with_later_handle_use() {
        let p=record_typed("fn inspect(view: &Ticket) -> Percent { return inspect(view); } fn f(t: Ticket) -> Percent { let view = &t; let seen = t.value; let checked = inspect(view); return seen; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::Let { initializer, .. } = &f.body[1] else {
            panic!()
        };
        let owner = Place::Parameter(f.parameters[0]);
        let before = c.clone();
        let mut expected = before.clone();
        *expected.remaining.get_mut(&owner).unwrap() -= 1;
        c.expression(initializer, Transfer::Return(f.id)).unwrap();
        assert_atomic(&c, &expected);
        assert_eq!(c.loans.len(), 1);
    }
    #[test]
    fn exclusive_read_failure_preserves_finite_required_whole_record_loan() {
        let p=record_typed("fn f(t: Ticket) -> Percent { let mut owned = t; let access = &mut owned; let seen = owned.value; return inspect_mut(access); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::Let { initializer, .. } = &f.body[2] else {
            panic!()
        };
        let before = c.clone();
        assert!(c
            .expression(initializer, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("exclusively borrowed"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn exclusive_recurrence_blocks_field_read_with_no_finite_uses_or_holds() {
        for empty_inner in [false, true] {
            let p=record_typed("fn f(t: Ticket) -> Percent { let mut owned = t; let access = &mut owned; let seen = owned.value; return inspect_mut(access); }");
            let f = return_function(&p);
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..2], f.id).unwrap();
            let loan = *c.loans.keys().next().unwrap();
            c.recurrent.push(BTreeSet::from([loan]));
            add_test_target(&mut c);
            if empty_inner {
                c.recurrent.push(BTreeSet::new());
                add_test_target(&mut c);
            }
            c.loop_scope = Some(LoopScope::new(&c, false));
            c.remaining.clear();
            c.expire();
            let ValueStatement::Let { initializer, .. } = &f.body[2] else {
                panic!()
            };
            let ExprKind::CopyFieldRead { owner, .. } = initializer.kind else {
                panic!()
            };
            let before = c.clone();
            assert_eq!(c.loans[&loan].held, 0);
            assert!(c
                .copy_field_read(owner, initializer.span)
                .unwrap_err()
                .message
                .contains("exclusively borrowed"));
            assert_atomic(&c, &before);
            assert_eq!(c.states.len(), before.states.len());
        }
    }
    #[test]
    fn field_read_never_expires_or_changes_unrelated_protected_loans() {
        let p=record_typed("fn f(t: Ticket,flag: bool) -> Percent { let view = &flag; let seen = t.value; let used = read_flag(view); return seen; } fn read_flag(view: &bool) -> bool { return *view; }");
        let f = p
            .functions()
            .iter()
            .filter_map(|f| f.as_ordinary())
            .find(|f| f.name == "f")
            .unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let loan = *c.loans.keys().next().unwrap();
        c.recurrent.push(BTreeSet::from([loan]));
        let ValueStatement::Let { initializer, .. } = &f.body[1] else {
            panic!()
        };
        let mut expected = c.clone();
        *expected
            .remaining
            .get_mut(&Place::Parameter(f.parameters[0]))
            .unwrap() -= 1;
        c.expression(initializer, Transfer::Return(f.id)).unwrap();
        assert_atomic(&c, &expected);
    }
    #[test]
    fn construction_both_visitors_descend_through_nested_children() {
        let p=record_typed("fn f(flag: bool,p: Percent) -> Ticket { let view = &p; while flag { let b = Ticket { value: read(view) }; } return Ticket { value: p }; }");
        let f = return_function(&p);
        let ValueStatement::While { body, .. } = &f.body[1] else {
            panic!()
        };
        let ValueStatement::Let { initializer, .. } = &body[0] else {
            panic!()
        };
        let ValueStatement::Let { local: view, .. } = f.body[0] else {
            panic!()
        };
        let mut uses = FutureUses::new();
        count_expression(initializer, &mut uses);
        assert_eq!(uses[&Place::Local(view)], 1);
        assert!(reference_uses(body).contains(&Place::Local(view)));
        check(&p).unwrap();
    }
    #[test]
    fn construction_recurrence_preserves_original_shared_provenance_at_backedge() {
        let p=record_typed("fn f(flag: bool,p: Percent) -> Ticket { let view = &p; while flag { let b = Ticket { value: read(view) }; } return Ticket { value: p }; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let loan = *c.loans.keys().next().unwrap();
        let header = pin_body(&mut c, body);
        c.statements(body, f.id).unwrap();
        assert_eq!(c.recurrent.last().unwrap(), &BTreeSet::from([loan]));
        assert!(c.loans.contains_key(&loan));
        assert!(c.states.len() > header.states.len());
        c.validate_loop_backedge(&header, *span).unwrap();
        assert!(c.same_loop_state(&header));
        assert_eq!(c.next_loan, header.next_loan);
        assert_eq!(c.states.len(), header.states.len());
        assert_eq!(c.handles, header.handles);
        assert_eq!(c.loans[&loan].owner, header.loans[&loan].owner);
        assert_eq!(c.loans[&loan].kind, header.loans[&loan].kind);
        assert_eq!(c.recurrent, header.recurrent);
        c.assert_no_holds();
    }
    #[test]
    fn record_identity_and_ownership_ignore_diagnostic_names() {
        let mut p=record_typed("fn f(flag: bool,p: Percent) -> Ticket { let b = if flag { Ticket { value: p } } else { Ticket { value: p } }; let seen = b.value; return b; }");
        check(&p).unwrap();
        let m = crate::mir::lower(&p);
        for r in &mut p.records {
            r.name = "same".into();
        }
        for r in &mut p.ranges {
            r.name = "same".into();
        }
        for f in &mut p.fields {
            f.name = "same".into();
        }
        for v in &mut p.locals {
            v.name = "same".into();
        }
        for v in &mut p.parameters {
            v.name = "same".into();
        }
        p.validate_record_expressions().unwrap();
        check(&p).unwrap();
        assert_eq!(m, crate::mir::lower(&p));
    }
    fn multiple_typed(body: &str) -> Program {
        record_typed(&format!("record Triple {{ first: Percent, middle: Percent, last: Percent }} fn inspect3(view: &Triple) -> Percent {{ return inspect3(view); }} fn observe(view: &bool,p: Percent) -> Percent {{ return p; }} fn receive3(b: Triple,t: Ticket) {{}} fn hold3(view: &bool,b: Triple) {{}} {body}"))
    }
    #[test]
    fn multi_initializer_prefix_move_and_allocator_effects_roll_back_complete_prepared_state() {
        let p=multiple_typed("fn f(t: Ticket,flag: bool,p: Percent,view: &Percent) -> Triple { return Triple { last: consume(t), first: observe(&flag,p), middle: consume(t) }; }");
        let f = return_function(&p);
        let (e, _) = operand(&f.body[0]);
        let mut c = Checker::new(&p, f);
        let carried = *c.loans.keys().next().unwrap();
        c.recurrent.push(BTreeSet::from([carried]));
        c.recurrent.push(BTreeSet::new());
        add_test_target(&mut c);
        c.loop_scope = Some(LoopScope::new(&c, false));
        c.terminal_return = true; // Actual operand context, not retroactive permission.
        c.loans.get_mut(&carried).unwrap().held = 1; // Prepared enclosing operation protection.
        prepared_expression(&mut c, e);
        let before = c.clone();
        let ExprKind::RecordConstruct { fields, .. } = &e.kind else {
            panic!()
        };
        let mut prefix = c.clone();
        prefix
            .expression(&fields[0].value, Transfer::Return(f.id))
            .unwrap();
        prefix
            .expression(&fields[1].value, Transfer::Return(f.id))
            .unwrap();
        assert_ne!(prefix.states, before.states);
        assert!(prefix.next_loan > before.next_loan);
        assert_eq!(prefix.loans[&carried].held, 1);
        assert!(c
            .expression(e, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn multi_initializers_visit_first_middle_final_and_reverse_written_order() {
        for position in 0..3 {
            let names = ["last", "middle", "first"];
            let entries = names
                .iter()
                .enumerate()
                .map(|(i, n)| format!("{n}: {}", if i == position { "consume(t)" } else { "p" }))
                .collect::<Vec<_>>()
                .join(",");
            let p = multiple_typed(&format!(
                "fn f(t: Ticket,p: Percent) -> Triple {{ return Triple {{ {entries} }}; }}"
            ));
            let f = return_function(&p);
            let (e, _) = operand(&f.body[0]);
            let mut c = Checker::new(&p, f);
            prepared_expression(&mut c, e);
            assert!(c.expression(e, Transfer::Return(f.id)).unwrap().is_none());
            assert!(matches!(
                c.states[&Place::Parameter(f.parameters[0])],
                State::Moved { .. }
            ));
            assert_eq!(c.states.len(), 1); // Whole Ticket only; no field/destination owner.
            assert!(c.handles.is_empty());
            assert!(c.loans.is_empty());
            assert!(p.bindings().is_empty());
        }
    }
    #[test]
    fn multi_construction_failure_never_publishes_destination_or_fields() {
        let p=multiple_typed("fn f(t: Ticket,p: Percent) -> Triple { let predecessor_result = Triple { middle: consume(t), first: p, last: consume(t) }; return predecessor_result; }");
        let f = return_function(&p);
        let ValueStatement::Let {
            local, initializer, ..
        } = &f.body[0]
        else {
            panic!()
        };
        let mut c = Checker::new(&p, f);
        prepared_expression(&mut c, initializer);
        let before = c.clone();
        assert!(c.statement(&f.body[0], f.id, &FutureUses::new()).is_err());
        assert!(!c.states.contains_key(&Place::Local(*local)));
        assert_atomic(&c, &before);
    }
    #[test]
    fn multi_construction_complete_call_prefix_is_rolled_back_by_later_argument() {
        let p=multiple_typed("fn f(t: Ticket,p: Percent) { receive3(Triple { last: consume(t), first: p, middle: p },t); }");
        let f = return_function(&p);
        let (function, args, span) = statement_call(&f.body[0]);
        let mut c = Checker::new(&p, f);
        prepare_call(&mut c, args);
        let before = c.clone();
        let mut prefix = c.clone();
        prefix
            .expression(
                &args[0],
                Transfer::Argument(p.function(function).as_ordinary().unwrap().parameters[0]),
            )
            .unwrap();
        assert_ne!(prefix.states, before.states);
        assert!(c.call_statement(function, args, span).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn multi_construction_preserves_legitimate_receiving_call_hold_until_enclosing_release() {
        let p=multiple_typed("fn f(flag: bool,p: Percent) { hold3(&flag,Triple { last: p, first: observe(&flag,p), middle: p }); }");
        let f = return_function(&p);
        let (function, args, _) = statement_call(&f.body[0]);
        let mut c = Checker::new(&p, f);
        prepare_call(&mut c, args);
        // Reproduce the existing call_arguments receiving hold at the exact boundary.
        let params = &p.function(function).as_ordinary().unwrap().parameters;
        let loan = c
            .expression(&args[0], Transfer::Argument(params[0]))
            .unwrap()
            .unwrap();
        c.loans.get_mut(&loan).unwrap().held += 1;
        let entry = c.clone();
        c.expression(&args[1], Transfer::Argument(params[1]))
            .unwrap();
        assert_eq!(c.loans[&loan].held, 1);
        assert_eq!(c.recurrent, entry.recurrent);
        assert_eq!(c.terminal_return, entry.terminal_return);
        assert_eq!(c.next_loan, entry.next_loan + 1); // Nested observe borrow only.
        c.loans.get_mut(&loan).unwrap().held -= 1;
        c.expire();
        assert!(c.loans.is_empty());
        let mut whole = Checker::new(&p, f);
        prepare_call(&mut whole, args);
        whole.call_arguments(function, args).unwrap();
        whole.assert_no_holds();
        assert!(whole.loans.is_empty());
    }
    #[test]
    fn multi_constructor_kd033_return_transaction_restores_real_entry_on_late_failure() {
        let p=multiple_typed("fn f(t: Ticket,p: Percent,flag: bool) -> Triple { while flag { return Triple { middle: consume(t), first: p, last: consume(t) }; } return Triple { first: consume(t), middle: p, last: p }; }");
        let f = return_function(&p);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        let (e, span) = operand(&body[0]);
        let mut c = Checker::new(&p, f);
        c.loop_scope = Some(LoopScope::new(&c, false));
        c.recurrent.push(BTreeSet::new());
        add_test_target(&mut c);
        prepared_expression(&mut c, e);
        let before = c.clone();
        assert!(c.return_edge(e, f.id, span).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn multi_finite_and_recurrent_visitors_reach_every_written_initializer() {
        let p=multiple_typed("fn f(a: &Percent,b: &Percent,c: &Percent) -> Triple { return Triple { last: read(c), first: read(a), middle: read(b) }; }");
        let f = return_function(&p);
        let (e, _) = operand(&f.body[0]);
        let mut uses = FutureUses::new();
        count_expression(e, &mut uses);
        for id in &f.parameters {
            assert_eq!(uses[&Place::Parameter(*id)], 1);
        }
        assert_eq!(
            reference_uses(&f.body),
            f.parameters
                .iter()
                .map(|id| Place::Parameter(*id))
                .collect()
        );
        let mut c = Checker::new(&p, f);
        let frame = c.loans.keys().copied().collect();
        c.recurrent.push(frame);
        add_test_target(&mut c);
        prepared_expression(&mut c, e);
        let before = c.clone();
        c.expression(e, Transfer::Return(f.id)).unwrap();
        assert_eq!(c.recurrent, before.recurrent);
        assert_eq!(c.handles, before.handles);
        assert_eq!(c.loans.len(), 3);
        c.assert_no_holds();
    }
    #[test]
    fn multi_fields_preserve_recurrent_provenance_through_real_backedge_cleanup() {
        let p=multiple_typed("fn f(flag: bool,p: Percent) -> Triple { let view = &p; while flag { let b = Triple { last: read(view), first: read(view), middle: read(view) }; } return Triple { first: p, middle: p, last: p }; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let header = pin_body(&mut c, body);
        let loan = *header.loans.keys().next().unwrap();
        c.statements(body, f.id).unwrap();
        assert!(c.loans.contains_key(&loan));
        c.validate_loop_backedge(&header, *span).unwrap();
        assert!(c.same_loop_state(&header));
        assert_eq!(c.recurrent, header.recurrent);
    }
    #[test]
    fn multiple_field_reads_never_create_partial_state_loan_or_provenance() {
        let p=multiple_typed("fn f(t: Triple) -> Triple { let a = t.first; let b = t.middle; let c = t.last; return t; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let before = c.clone();
        for statement in &f.body[..3] {
            let ValueStatement::Let { initializer, .. } = statement else {
                panic!()
            };
            c.expression(initializer, Transfer::Return(f.id)).unwrap();
        }
        let mut expected = before.clone();
        *expected
            .remaining
            .get_mut(&Place::Parameter(f.parameters[0]))
            .unwrap() -= 3;
        assert_atomic(&c, &expected);
        assert!(c.handles.is_empty());
        assert!(c.loans.is_empty());
        assert_eq!(c.states.len(), 1);
    }
    #[test]
    fn multi_call_failure_restores_real_exclusive_argument_hold_and_allocator() {
        let p=multiple_typed("fn protect(access: &mut Triple,b: Triple) {} fn f(b: Triple,p: Percent) { let mut owned = b; protect(&mut owned,Triple { last: p, first: p, middle: owned.middle }); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let (function, args, span) = statement_call(&f.body[1]);
        prepare_call(&mut c, args);
        let before = c.clone();
        let mut prefix = c.clone();
        let parameter = p.function(function).as_ordinary().unwrap().parameters[0];
        let loan = prefix
            .expression(&args[0], Transfer::Argument(parameter))
            .unwrap()
            .unwrap();
        prefix.loans.get_mut(&loan).unwrap().held += 1;
        assert!(prefix.next_loan > before.next_loan);
        assert_eq!(prefix.loans[&loan].kind, BorrowKind::Mutable);
        assert!(c
            .call_statement(function, args, span)
            .unwrap_err()
            .message
            .contains("exclusively borrowed"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn multi_record_identity_and_ownership_are_independent_of_display_spelling() {
        let mut p=multiple_typed("fn f(t: Ticket,p: Percent) -> Triple { return Triple { last: consume(t), first: p, middle: p }; }");
        check(&p).unwrap();
        let m = crate::mir::lower(&p);
        for r in &mut p.records {
            r.name = "display".into();
        }
        for r in &mut p.ranges {
            r.name = "display".into();
        }
        for field in &mut p.fields {
            field.name = format!("display_{}", field.id.0);
        }
        for parameter in &mut p.parameters {
            parameter.name = "display".into();
        }
        p.validate_record_expressions().unwrap();
        check(&p).unwrap();
        assert_eq!(m, crate::mir::lower(&p));
    }
    #[test]
    fn formed_leaf_preserves_complete_state_holds_targets_and_recurrence() {
        let p = multiple_typed("fn f(p: Percent,view: &Percent) -> Percent {return 80;}");
        let f = return_function(&p);
        let (e, _) = operand(&f.body[0]);
        let mut c = Checker::new(&p, f);
        let loan = *c.loans.keys().next().unwrap();
        c.loans.get_mut(&loan).unwrap().held = 2; // Legitimate inherited operation holds.
        c.recurrent.push(BTreeSet::from([loan]));
        c.recurrent.push(BTreeSet::new());
        c.loop_scope = Some(LoopScope::new(&c, false));
        add_test_target(&mut c);
        for terminal in [false, true] {
            c.terminal_return = terminal;
            let entry = c.clone();
            assert_eq!(c.expression(e, Transfer::Return(f.id)).unwrap(), None);
            assert_atomic(&c, &entry);
        }
        let mut finite = FutureUses::new();
        count_expression(e, &mut finite);
        assert!(finite.is_empty());
        assert!(reference_uses(&f.body).is_empty());
    }
    #[test]
    fn formed_initializer_between_real_effects_retains_complete_expression_rollback() {
        let p=multiple_typed("fn f(t: Ticket,flag: bool,view: &Percent) -> Triple {return Triple {last: consume(t),first: observe(&flag,80),middle: consume(t)};}");
        let f = return_function(&p);
        let (e, span) = operand(&f.body[0]);
        let mut c = Checker::new(&p, f);
        let loan = *c.loans.keys().next().unwrap();
        c.recurrent.push(BTreeSet::from([loan]));
        c.recurrent.push(BTreeSet::new());
        c.loans.get_mut(&loan).unwrap().held = 1;
        c.loop_scope = Some(LoopScope::new(&c, false));
        add_test_target(&mut c);
        prepared_expression(&mut c, e);
        let before = c.clone();
        let ExprKind::RecordConstruct { fields, .. } = &e.kind else {
            panic!()
        };
        let mut prefix = c.clone();
        prefix.terminal_return = true;
        prefix
            .expression(&fields[0].value, Transfer::Return(f.id))
            .unwrap();
        prefix
            .expression(&fields[1].value, Transfer::Return(f.id))
            .unwrap();
        assert_ne!(prefix.states, before.states);
        assert!(prefix.next_loan > before.next_loan);
        let mut complete = c.clone();
        complete.terminal_return = true;
        let expression_entry = complete.clone();
        assert!(complete.expression(e, Transfer::Return(f.id)).is_err());
        assert_atomic(&complete, &expression_entry);
        assert!(c.return_edge(e, f.id, span).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn formed_middle_call_argument_does_not_release_receiving_hold() {
        let p=multiple_typed("fn hold(view: &mut bool,p: Percent,flag: bool) {} fn f(flag: bool) {let mut owned=flag; hold(&mut owned,80,owned);}");
        let f = return_function(&p);
        let (function, args, span) = statement_call(&f.body[1]);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        prepare_call(&mut c, args);
        let before = c.clone();
        assert!(c
            .call_statement(function, args, span)
            .unwrap_err()
            .message
            .contains("exclusively borrowed"));
        assert_atomic(&c, &before);
    }

    #[test]
    fn formed_first_middle_final_leaves_add_no_finite_or_recurrent_uses() {
        for position in 0..3 {
            let entries = ["last", "first", "middle"]
                .iter()
                .enumerate()
                .map(|(i, n)| format!("{n}: {}", if i == position { "80" } else { "read(view)" }))
                .collect::<Vec<_>>()
                .join(",");
            let p = multiple_typed(&format!(
                "fn f(view: &Percent) -> Triple {{return Triple {{{entries}}};}}"
            ));
            let f = return_function(&p);
            let (e, _) = operand(&f.body[0]);
            let mut uses = FutureUses::new();
            count_expression(e, &mut uses);
            let place = Place::Parameter(f.parameters[0]);
            assert_eq!(uses, FutureUses::from([(place, 2)]));
            assert_eq!(reference_uses(&f.body), BTreeSet::from([place]));
            let mut c = Checker::new(&p, f);
            let loan = c.handles[&place];
            c.recurrent.push(BTreeSet::from([loan]));
            prepared_expression(&mut c, e);
            let before = c.clone();
            c.expression(e, Transfer::Return(f.id)).unwrap();
            assert_eq!(c.loans[&loan].owner, before.loans[&loan].owner);
            assert_eq!(c.handles, before.handles);
            assert_eq!(c.recurrent, before.recurrent);
            assert_eq!(c.next_loan, before.next_loan);
            c.assert_no_holds();
        }
    }
    #[test]
    fn formed_field_complete_expression_and_enclosing_call_restore_prepared_state() {
        let p=multiple_typed("fn f(t: Ticket,flag: bool) {receive3(Triple {last: consume(t),first: observe(&flag,80),middle: 100},t);}");
        let f = return_function(&p);
        let (function, args, span) = statement_call(&f.body[0]);
        let mut c = Checker::new(&p, f);
        prepare_call(&mut c, args);
        let before = c.clone();
        let mut prefix = c.clone();
        prefix
            .expression(
                &args[0],
                Transfer::Argument(p.function(function).as_ordinary().unwrap().parameters[0]),
            )
            .unwrap();
        assert_ne!(prefix.states, before.states);
        assert!(prefix.next_loan > before.next_loan);
        assert!(c.call_statement(function, args, span).is_err());
        assert_atomic(&c, &before);
    }
    #[test]
    fn formed_call_arguments_preserve_neighbor_finite_and_recurrent_reference_uses() {
        for position in 0..3 {
            let args = (0..3)
                .map(|i| if i == position { "80" } else { "read(view)" })
                .collect::<Vec<_>>()
                .join(",");
            let p=multiple_typed(&format!("fn tri(a: Percent,b: Percent,c: Percent) -> Percent {{return a;}} fn f(view: &Percent) -> Percent {{return tri({args});}}"));
            let f = return_function(&p);
            let (e, _) = operand(&f.body[0]);
            let place = Place::Parameter(f.parameters[0]);
            let mut finite = FutureUses::new();
            count_expression(e, &mut finite);
            assert_eq!(finite, FutureUses::from([(place, 2)]));
            assert_eq!(reference_uses(&f.body), BTreeSet::from([place]));
            let mut c = Checker::new(&p, f);
            let loan = c.handles[&place];
            c.recurrent.push(BTreeSet::from([loan]));
            prepared_expression(&mut c, e);
            let before = c.clone();
            c.expression(e, Transfer::Return(f.id)).unwrap();
            assert_eq!(c.handles, before.handles);
            assert_eq!(c.recurrent, before.recurrent);
            assert_eq!(c.next_loan, before.next_loan);
            c.assert_no_holds();
        }
    }
    const FIELD_HELPERS: &str = "fn sample(v: &Ticket) -> Percent { return 80; } fn sample_mut(v: &mut Ticket) -> Percent { return 80; } fn hold(v: &Ticket,p: Percent) -> Percent { return p; } fn join(a: Percent,b: Percent,c: Percent) -> Percent { return a; } fn finish(v: &Ticket) {} fn finish_mut(v: &mut Ticket) {}";
    fn field_typed(body: &str) -> Program {
        record_typed(&format!("{FIELD_HELPERS} {body}"))
    }
    fn field_operation(c: &mut Checker<'_>, s: &ValueStatement) -> OwnershipResult {
        let ValueStatement::CopyFieldAssign {
            owner,
            record,
            field,
            value,
            owner_span,
            span,
            ..
        } = s
        else {
            panic!("field assignment")
        };
        c.copy_field_assign(*owner, *record, *field, value, *owner_span, *span)
    }
    #[test]
    fn field_operation_late_root_failure_restores_exact_prepared_entry() {
        let p = field_typed(
            "fn f(t: Ticket,p: Percent) { let mut owned = t; owned.value = consume(owned); }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let s = &f.body[1];
        c.remaining = statement_future(s, &FutureUses::new(), &FutureUses::new());
        c.expire();
        // Preserve legitimate enclosing contexts without treating the target as
        // pre-existing at this synthetic inner header.
        c.loop_scope = Some(LoopScope::new(&c, false));
        let ValueStatement::CopyFieldAssign { owner, .. } = s else {
            panic!()
        };
        c.loop_scope
            .as_mut()
            .unwrap()
            .header_places
            .remove(&Place::Local(*owner));
        c.recurrent.push(BTreeSet::new());
        add_test_target(&mut c);
        let before = c.clone();
        assert!(field_operation(&mut c, s)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &before);
        assert!(!c.terminal_return);
        // Invoke without any statement-list wrapper; final availability, not
        // RHS failure, must have caused the rollback of a successful consume.
        let ValueStatement::CopyFieldAssign { value, .. } = s else {
            panic!()
        };
        c.expression(value, Transfer::Local(*owner)).unwrap();
        assert_ne!(c.states, before.states);
    }
    #[test]
    fn field_operation_nested_rhs_failure_restores_prefix_and_allocator() {
        let p=field_typed("fn f(t: Ticket,other: Ticket) { let mut owned = t; owned.value = hold(&owned,duplicate(other,other)); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let before = c.clone();
        assert!(field_operation(&mut c, &f.body[1])
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_atomic(&c, &before);
        assert_eq!(c.next_loan, 0);
    }
    #[test]
    fn field_operation_final_shared_and_exclusive_conflicts_restore_successful_rhs_effects() {
        for (borrow, finish) in [("&owned", "finish"), ("&mut owned", "finish_mut")] {
            let p=field_typed(&format!("fn f(t: Ticket,other: Ticket) {{ let mut owned = t; let view = {borrow}; owned.value = consume(other); {finish}(view); }}"));
            let f = return_function(&p);
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..2], f.id).unwrap();
            let before = c.clone();
            let ValueStatement::CopyFieldAssign {
                owner, value, span, ..
            } = &f.body[2]
            else {
                panic!()
            };
            // Show that the well-typed RHS succeeds and consumes the unrelated
            // owner before the real helper reaches its final conflict check.
            let mut rhs_only = before.clone();
            rhs_only.expression(value, Transfer::Local(*owner)).unwrap();
            assert_ne!(
                rhs_only.states[&Place::Parameter(f.parameters[1])],
                State::Available
            );
            let error = field_operation(&mut c, &f.body[2]).unwrap_err();
            assert_eq!(
                error.message,
                "cannot assign `owned.value` while it is borrowed"
            );
            assert_eq!(error.span, *span);
            assert!(error.required.contains("field `Ticket.value`"));
            assert!(error
                .required
                .contains("declared named-range type `Percent`"));
            let kind = if borrow == "&owned" {
                "shared"
            } else {
                "exclusive"
            };
            assert!(error
                .required
                .contains(&format!("active {kind} whole-record loan")));
            let origin = before.loans.values().next().unwrap().span;
            assert_eq!(
                error.known,
                vec![format!(
                    "the conflicting borrow began at line {}, column {}",
                    origin.line, origin.column
                )]
            );
            assert_atomic(&c, &before);
            assert_eq!(
                c.states[&Place::Parameter(f.parameters[1])],
                State::Available
            );
        }
    }
    #[test]
    fn field_operation_rollback_is_distinct_from_enclosing_list_entry() {
        let p=field_typed("fn f(t: Ticket,other: Ticket) { let mut owned = t; let unused = &owned; owned.value = consume(owned); }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let broad = c.clone();
        // An earlier statement in this list publishes a handle and advances the
        // loan allocator. Prepared field entry must retain those lawful effects.
        c.statement(&f.body[1], f.id, &FutureUses::new()).unwrap();
        c.remaining = statement_future(&f.body[2], &FutureUses::new(), &FutureUses::new());
        c.expire();
        let prepared = c.clone();
        assert_ne!(prepared.next_loan, broad.next_loan);
        assert!(field_operation(&mut c, &f.body[2]).is_err());
        assert_atomic(&c, &prepared);
        let mut outer = broad.clone();
        assert!(outer
            .statements_with(&f.body[1..], f.id, &FutureUses::new())
            .is_err());
        assert_atomic(&outer, &broad);
    }
    #[test]
    fn field_operation_success_commits_unrelated_move_without_target_transfer_or_field_state() {
        let p = field_typed(
            "fn f(t: Ticket,other: Ticket) { let mut owned = t; owned.value = consume(other); }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let before = c.clone();
        field_operation(&mut c, &f.body[1]).unwrap();
        let ValueStatement::CopyFieldAssign { owner, .. } = f.body[1] else {
            panic!()
        };
        assert_eq!(c.states[&Place::Local(owner)], State::Available);
        assert_ne!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Available
        );
        assert_eq!(c.states.len(), before.states.len());
        assert_eq!(c.handles, before.handles);
        assert_eq!(c.loans.len(), before.loans.len());
        assert_eq!(c.next_loan, before.next_loan);
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[0])],
            before.states[&Place::Parameter(f.parameters[0])]
        );
    }
    #[test]
    fn field_operation_success_commits_temporary_loan_allocation_and_last_use_expiry() {
        for rhs in ["sample(&owned)", "sample_mut(&mut owned)"] {
            let p = field_typed(&format!(
                "fn f(t: Ticket) {{ let mut owned = t; owned.value = {rhs}; }}"
            ));
            let f = return_function(&p);
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..1], f.id).unwrap();
            let before = c.clone();
            field_operation(&mut c, &f.body[1]).unwrap();
            assert_eq!(c.states, before.states);
            assert!(c.handles.is_empty());
            assert!(c.loans.is_empty());
            assert_eq!(c.next_loan, before.next_loan + 1);
        }
        let p = field_typed(
            "fn f(t: Ticket) { let mut owned = t; let view = &owned; owned.value = sample(view); }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        assert_eq!(c.loans.len(), 1);
        let states = c.states.clone();
        let handles = c.handles.clone();
        field_operation(&mut c, &f.body[2]).unwrap();
        assert!(c.loans.is_empty());
        assert_eq!(c.states, states);
        assert_eq!(c.handles, handles);
    }
    #[test]
    fn field_operation_preserves_inherited_holds_recurrence_targets_and_terminal_context() {
        for kind in [BorrowKind::Shared, BorrowKind::Mutable] {
            for protection in ["held", "recurrent"] {
                let p = field_typed(
                    "fn f(t: Ticket,p: Percent) { let mut owned = t; owned.value = p; }",
                );
                let f = return_function(&p);
                let mut c = Checker::new(&p, f);
                c.statements(&f.body[..1], f.id).unwrap();
                let ValueStatement::CopyFieldAssign { owner, .. } = f.body[1] else {
                    panic!()
                };
                let loan = c.add_loan(Some(Place::Local(owner)), kind, f.span);
                if protection == "held" {
                    c.loans.get_mut(&loan).unwrap().held = 2;
                }
                c.recurrent.push(if protection == "recurrent" {
                    BTreeSet::from([loan])
                } else {
                    BTreeSet::new()
                });
                add_test_target(&mut c);
                c.recurrent.push(BTreeSet::new());
                add_test_target(&mut c);
                c.loop_scope = Some(LoopScope::new(&c, false));
                c.terminal_return = true;
                let before = c.clone();
                assert!(field_operation(&mut c, &f.body[1]).is_err());
                assert_atomic(&c, &before);
            }
        }
        // An unrelated inherited operation hold is lawful and not globally balanced away.
        let p = field_typed(
            "fn f(t: Ticket,other: Ticket,p: Percent) { let mut owned = t; owned.value = p; }",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let loan = c.add_loan(
            Some(Place::Parameter(f.parameters[1])),
            BorrowKind::Shared,
            f.span,
        );
        c.loans.get_mut(&loan).unwrap().held = 1;
        let before = c.clone();
        field_operation(&mut c, &f.body[1]).unwrap();
        let mut expected = before;
        *expected
            .remaining
            .get_mut(&Place::Parameter(f.parameters[2]))
            .unwrap() -= 1;
        assert_atomic(&c, &expected);
    }
    #[test]
    fn field_rhs_finite_and_recurrent_visitors_reach_every_nested_child_but_not_target() {
        for position in 0..3 {
            let mut args = ["80", "80", "80"];
            args[position] = "sample(view)";
            let p=field_typed(&format!("fn f(t: Ticket) {{ let mut owned = t; let view = &owned; owned.value = join({}, {}, {}); }}",args[0],args[1],args[2]));
            let f = return_function(&p);
            let s = &f.body[2];
            let view = p.locals[1].id;
            let owner = p.locals[0].id;
            let expected = BTreeMap::from([(Place::Local(view), 1)]);
            assert_eq!(future_uses(std::slice::from_ref(s)), expected);
            assert_eq!(
                reference_uses(std::slice::from_ref(s)),
                BTreeSet::from([Place::Local(view)])
            );
            assert!(!future_uses(std::slice::from_ref(s)).contains_key(&Place::Local(owner)));
            for container in [
                s.clone(),
                ValueStatement::If {
                    condition: TypedExpr {
                        kind: ExprKind::BoolLiteral(true),
                        ty: ExprType::Bool,
                        span: f.span,
                    },
                    then_body: vec![s.clone()],
                    else_body: vec![],
                    span: f.span,
                },
                ValueStatement::While {
                    condition: TypedExpr {
                        kind: ExprKind::BoolLiteral(true),
                        ty: ExprType::Bool,
                        span: f.span,
                    },
                    body: vec![s.clone()],
                    span: f.span,
                },
            ] {
                assert_eq!(
                    reference_uses(&[container.clone()]),
                    BTreeSet::from([Place::Local(view)])
                );
                assert_eq!(future_uses(&[container]), expected);
            }
        }
    }
    #[test]
    fn field_assignment_metadata_spelling_is_not_semantic_identity() {
        let mut p=field_typed("fn f(t: Ticket) -> Ticket { let mut owned = t; owned.value = sample(&owned); return owned; }");
        check(&p).unwrap();
        let before = crate::mir::lower(&p);
        for l in &mut p.locals {
            l.name = "display".into();
        }
        for r in &mut p.records {
            r.name = "display".into();
        }
        for f in &mut p.fields {
            f.name = "display".into();
        }
        check(&p).unwrap();
        assert_eq!(before, crate::mir::lower(&p));
    }
    const REFERENCE_FIELD_HELPERS: &str = "record Triple { first: Percent, middle: Percent, last: Percent } fn three(a: Percent,b: Percent,c: Percent) -> Percent { return a; } fn pair(a: Percent,b: Percent) -> Percent { return a; } fn triple(t: Triple) -> Percent { return t.first; } fn duplicate(a: Ticket,b: Ticket) -> Percent { return a.value; } fn receive(a: Percent,b: Percent) {}";
    fn reference_typed(body: &str) -> Program {
        typed(&format!("{REFERENCE_FIELD_HELPERS} {body}"))
    }
    #[test]
    fn reference_field_shared_and_exclusive_reads_preserve_identity_permission_holds_and_context() {
        for capability in ["&", "&mut "] {
            let p = reference_typed(&format!(
                "fn f(v: {capability}Ticket) -> Percent {{ return v.value; }}"
            ));
            let f = return_function(&p);
            let (value, _) = operand(&f.body[0]);
            let mut c = Checker::new(&p, f);
            let place = Place::Parameter(f.parameters[0]);
            let loan = c.handles[&place];
            c.loans.get_mut(&loan).unwrap().held = 2;
            c.recurrent.push(BTreeSet::from([loan]));
            add_test_target(&mut c);
            c.loop_scope = Some(LoopScope::new(&c, false));
            c.terminal_return = true;
            let before = c.clone();
            assert!(c
                .expression(value, Transfer::Return(f.id))
                .unwrap()
                .is_none());
            assert_eq!(c.remaining[&place], 0);
            c.remaining = before.remaining.clone();
            assert_atomic(&c, &before); // All other semantic state exactly unchanged.
        }
    }
    #[test]
    fn reference_field_final_use_expires_without_allocating_consuming_or_returning_a_loan() {
        for capability in ["&", "&mut "] {
            let p = reference_typed(&format!(
                "fn f(v: {capability}Ticket) -> Percent {{ return v.value; }}"
            ));
            let f = return_function(&p);
            let (value, _) = operand(&f.body[0]);
            let mut c = Checker::new(&p, f);
            let before = c.clone();
            assert!(c
                .expression(value, Transfer::Return(f.id))
                .unwrap()
                .is_none());
            assert!(c.loans.is_empty());
            assert_eq!(c.states, before.states);
            assert_eq!(c.handles, before.handles);
            assert_eq!(c.next_loan, before.next_loan);
            assert!(!c.terminal_return);
        }
    }
    #[test]
    fn reference_field_invalid_external_lineage_and_same_kind_compatible_loan_remapping_reject_atomically(
    ) {
        for capability in ["&", "&mut "] {
            let p=reference_typed(&format!("fn f(a: {capability}Ticket,b: {capability}Ticket) -> Percent {{ let seen = a.value; return b.value; }}"));
            let f = return_function(&p);
            let ValueStatement::Let {
                initializer: value, ..
            } = &f.body[0]
            else {
                panic!()
            };
            let original = Checker::new(&p, f);
            let a = Place::Parameter(f.parameters[0]);
            let b = Place::Parameter(f.parameters[1]);
            let loan = original.handles[&a];
            for defect in 0..9 {
                let mut c = original.clone();
                match defect {
                    0 => {
                        c.handles.remove(&a);
                    }
                    1 => {
                        c.loans.remove(&loan);
                    }
                    2 => {
                        c.handles.insert(a, LoanId(usize::MAX));
                    }
                    3 => {
                        c.handles.insert(a, c.handles[&b]);
                    } // Same record and permission, wrong origin.
                    4 => {
                        c.loans.get_mut(&loan).unwrap().kind = if capability == "&" {
                            BorrowKind::Mutable
                        } else {
                            BorrowKind::Shared
                        };
                    }
                    5 => {
                        c.loans.get_mut(&loan).unwrap().origin =
                            LoanOrigin::ExternalParameter(f.parameters[1]);
                    }
                    6 => {
                        c.loans.get_mut(&loan).unwrap().owner = Some(b);
                    }
                    7 => {
                        c.states
                            .insert(a, State::ConditionalMove { span: value.span });
                    }
                    8 => {
                        let forged = c.loans[&loan].clone();
                        c.loans.insert(LoanId(usize::MAX), forged);
                        c.handles.insert(a, LoanId(usize::MAX));
                    }
                    _ => unreachable!(),
                }
                let before = c.clone();
                assert!(
                    c.expression(value, Transfer::Return(f.id)).is_err(),
                    "{capability}, defect {defect}"
                );
                assert_atomic(&c, &before);
            }
            for invalid in [
                Place::Parameter(ParameterId(usize::MAX)),
                Place::Local(LocalId(usize::MAX)),
            ] {
                let mut value = value.clone();
                let ExprKind::CopyReferenceFieldRead { reference, .. } = &mut value.kind else {
                    panic!()
                };
                *reference = invalid;
                let mut c = original.clone();
                let before = c.clone();
                assert!(c.expression(&value, Transfer::Return(f.id)).is_err());
                assert_atomic(&c, &before);
            }
        }
    }
    #[test]
    fn reference_field_local_origin_alias_transfer_and_owner_integrity_are_checked() {
        for capability in ["&", "&mut "] {
            let p=reference_typed(&format!("fn f(t: Ticket) -> Percent {{ let mut owned = t; let first = {capability}owned; let alias = first; return alias.value; }}"));
            let f = return_function(&p);
            let (value, _) = operand(&f.body[3]);
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..3], f.id).unwrap();
            let ExprKind::CopyReferenceFieldRead { reference, .. } = value.kind else {
                panic!()
            };
            let loan = c.handles[&reference];
            let owner = c.loans[&loan].owner.unwrap();
            assert!(c.clone().expression(value, Transfer::Return(f.id)).is_ok());
            for defect in 0..6 {
                let mut broken = c.clone();
                match defect {
                    0 => {
                        broken.loans.get_mut(&loan).unwrap().owner =
                            Some(Place::Local(LocalId(usize::MAX)));
                    }
                    1 => {
                        broken.loans.get_mut(&loan).unwrap().span.end += 1;
                    }
                    2 => {
                        broken
                            .states
                            .insert(owner, State::ConditionalMove { span: value.span });
                    }
                    3 => {
                        broken.handles.remove(&Place::Local(LocalId(1)));
                    }
                    4 => {
                        broken.loans.get_mut(&loan).unwrap().origin =
                            LoanOrigin::ExternalParameter(f.parameters[0]);
                    }
                    5 => {
                        broken.states.insert(
                            Place::Local(LocalId(1)),
                            if capability == "&" {
                                State::ConditionalMove { span: value.span }
                            } else {
                                State::Available
                            },
                        );
                    }
                    _ => unreachable!(),
                }
                let before = broken.clone();
                assert!(
                    broken.expression(value, Transfer::Return(f.id)).is_err(),
                    "{capability}, defect {defect}"
                );
                assert_atomic(&broken, &before);
            }
        }
    }
    #[test]
    fn reference_field_shared_same_owner_wrong_live_borrow_origin_is_not_interchangeable() {
        let p=reference_typed("fn f(t: Ticket) -> Percent { let first = &t; let second = &t; let observed = first.value; return second.value; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let ValueStatement::Let {
            initializer: value, ..
        } = &f.body[2]
        else {
            panic!()
        };
        let first = Place::Local(LocalId(0));
        let second = Place::Local(LocalId(1));
        assert_ne!(c.handles[&first], c.handles[&second]);
        assert_eq!(c.loans.len(), 2);
        c.handles.insert(first, c.handles[&second]);
        let before = c.clone();
        assert!(c
            .expression(value, Transfer::Return(f.id))
            .unwrap_err()
            .message
            .contains("lineage"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn reference_field_every_call_and_constructor_position_restores_expired_prefix_at_expression_boundary(
    ) {
        for container in ["call", "construction"] {
            for position in 0..3 {
                let mut children = ["80", "80", "80"];
                children[position] = "view.value";
                let first = if container == "call" {
                    format!("three({})", children.join(","))
                } else {
                    format!(
                        "triple(Triple {{last:{},first:{},middle:{}}})",
                        children[0], children[1], children[2]
                    )
                };
                let p=reference_typed(&format!("fn f(b: Ticket,t: Ticket) -> Percent {{ let view = &b; return pair({first},duplicate(t,t)); }}"));
                let f = return_function(&p);
                let (value, _) = operand(&f.body[1]);
                let mut c = Checker::new(&p, f);
                c.statements(&f.body[..1], f.id).unwrap();
                prepared_expression(&mut c, value);
                let before = c.clone();
                let ExprKind::Call { arguments, .. } = &value.kind else {
                    panic!()
                };
                let place = Place::Local(LocalId(0));
                let loan = c.handles[&place];
                let mut prefix = c.clone();
                assert!(prefix
                    .expression(&arguments[0], Transfer::Return(f.id))
                    .unwrap()
                    .is_none());
                assert!(!prefix.loans.contains_key(&loan));
                assert_eq!(prefix.remaining[&place], 0);
                let mut uses = FutureUses::new();
                count_expression(&arguments[0], &mut uses);
                assert_eq!(uses, FutureUses::from([(place, 1)]));
                assert_eq!(
                    reference_uses(&f.body[1..]),
                    BTreeSet::from([place, Place::Parameter(f.parameters[1])])
                );
                assert!(c
                    .expression(value, Transfer::Return(f.id))
                    .unwrap_err()
                    .message
                    .contains("moved value"));
                assert_atomic(&c, &before);
                // Direct expression transaction, not an enclosing statement list.
            }
        }
    }
    #[test]
    fn reference_field_statement_call_and_return_transactions_restore_their_own_prepared_entries() {
        for returned in [false, true] {
            let body = if returned {
                "return pair(view.value,duplicate(t,t));"
            } else {
                "receive(view.value,duplicate(t,t));"
            };
            let p = reference_typed(&format!(
                "fn f(b: Ticket,t: Ticket){} {{ let view = &b; {body} }}",
                if returned { " -> Percent" } else { "" }
            ));
            let f = return_function(&p);
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..1], f.id).unwrap();
            if returned {
                let (value, span) = operand(&f.body[1]);
                prepared_expression(&mut c, value);
                let before = c.clone();
                assert!(c.return_edge(value, f.id, span).is_err());
                assert_atomic(&c, &before);
            } else {
                let (function, args, span) = statement_call(&f.body[1]);
                prepare_call(&mut c, args);
                let before = c.clone();
                assert!(c.call_statement(function, args, span).is_err());
                assert_atomic(&c, &before);
            }
        }
    }
    #[test]
    fn reference_field_recurrence_preserves_external_origin_through_real_backedge_validation() {
        for capability in ["&", "&mut "] {
            let p=reference_typed(&format!("fn f(flag: bool,v: {capability}Ticket) -> Percent {{ while flag {{ let seen = v.value; continue; }} return v.value; }}"));
            let f = return_function(&p);
            let mut c = Checker::new(&p, f);
            let ValueStatement::While { body, span, .. } = &f.body[0] else {
                panic!()
            };
            let header = install_continue_target(&mut c, body, future_uses(&f.body[1..]));
            let original = c.clone();
            c.statements_with(body, f.id, &FutureUses::new()).unwrap();
            assert!(c.same_loop_state(&header));
            assert_eq!(c.recurrent, header.recurrent);
            assert_eq!(
                c.loans.values().next().unwrap().origin,
                original.loans.values().next().unwrap().origin
            );
            let mut corrupted = original.clone();
            corrupted.loans.values_mut().next().unwrap().origin =
                LoanOrigin::ExternalParameter(f.parameters[0]);
            assert!(corrupted.validate_loop_backedge(&header, *span).is_err());
        }
    }
    #[test]
    fn reference_field_metadata_spelling_does_not_affect_ownership_or_mir_identity() {
        let mut p=reference_typed("fn f(flag: bool,v: &Ticket) -> Percent { let seen = if flag {v.value} else {v.value}; return seen; }");
        check(&p).unwrap();
        let before = crate::mir::lower(&p);
        for parameter in &mut p.parameters {
            parameter.name = "metadata".into();
        }
        for local in &mut p.locals {
            local.name = "metadata".into();
        }
        for record in &mut p.records {
            record.name = "metadata".into();
        }
        for field in &mut p.fields {
            field.name = "metadata".into();
        }
        check(&p).unwrap();
        let after = crate::mir::lower(&p);
        assert_eq!(before, after);
        // Names do not appear in canonical MIR expression leaves.
        for f in before.functions() {
            f.validate().unwrap();
        }
    }
    #[test]
    fn reference_field_conditional_expression_failure_restores_entry_without_publishing_destination(
    ) {
        let p=reference_typed("fn f(flag: bool,b: Ticket,t: Ticket) -> Percent { let view = &b; let chosen = if flag {pair(view.value,duplicate(t,t))} else {view.value}; return chosen; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::Let {
            local, initializer, ..
        } = &f.body[1]
        else {
            panic!()
        };
        prepared_expression(&mut c, initializer);
        let before = c.clone();
        assert!(c.expression(initializer, Transfer::Local(*local)).is_err());
        assert_atomic(&c, &before);
        assert!(!c.handles.contains_key(&Place::Local(*local)));
        assert!(!c.states.contains_key(&Place::Local(*local)));
    }
    #[test]
    fn reference_field_local_lineage_is_checked_before_recurrent_frame_discharge_on_break() {
        let p=reference_typed("fn f(flag: bool,t: Ticket) -> Percent { let view = &t; while flag {let seen = view.value; break;} return view.value; }");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        install_continue_target(&mut c, body, future_uses(&f.body[2..]));
        let loan = *c.loans.keys().next().unwrap();
        c.loans.get_mut(&loan).unwrap().origin = LoanOrigin::LocalBinding(LocalId(999));
        let before = c.clone();
        assert!(c
            .break_edge(*span)
            .unwrap_err()
            .message
            .contains("integrity"));
        assert_atomic(&c, &before);
    }
    #[test]
    fn reference_field_join_rejects_changed_incoming_origin_before_erasing_evidence() {
        let p = reference_typed("fn f(a: &Ticket,b: &Ticket) -> Percent {return a.value;}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let before = c.clone();
        for side in [0, 1] {
            let mut yes = c.clone();
            let mut no = c.clone();
            let incoming = if side == 0 { &mut yes } else { &mut no };
            let loan = incoming.handles[&Place::Parameter(f.parameters[0])];
            incoming.loans.get_mut(&loan).unwrap().origin =
                LoanOrigin::ExternalParameter(f.parameters[1]);
            assert!(c.join(yes, no, before.remaining.clone(), f.span).is_err());
            assert_atomic(&c, &before);
        }
    }
    fn join_mapping_fault(side: usize, missing: bool) {
        let p = reference_typed("fn f(a: &Ticket,b: &Ticket) -> Percent {return a.value;}");
        let f = return_function(&p);
        let mut entry = Checker::new(&p, f);
        let before = entry.clone();
        let mut yes = entry.clone();
        let mut no = entry.clone();
        let incoming = if side == 0 { &mut yes } else { &mut no };
        let a = Place::Parameter(f.parameters[0]);
        let b = Place::Parameter(f.parameters[1]);
        if missing {
            incoming.handles.remove(&a);
        } else {
            incoming.handles.insert(a, incoming.handles[&b]);
        }
        assert!(incoming.live_handle(a, f.span).is_err());
        assert!(entry
            .join(yes, no, before.remaining.clone(), f.span)
            .is_err());
        // Rejection precedes publishing entry mappings or discarding candidates.
        assert_atomic(&entry, &before);
    }
    #[test]
    fn join_rejects_then_compatible_live_mapping_substitution() {
        join_mapping_fault(0, false);
    }
    #[test]
    fn join_rejects_else_compatible_live_mapping_substitution() {
        join_mapping_fault(1, false);
    }
    #[test]
    fn join_rejects_missing_then_mapping_before_entry_can_conceal_it() {
        join_mapping_fault(0, true);
    }
    #[test]
    fn join_rejects_missing_else_mapping_before_entry_can_conceal_it() {
        join_mapping_fault(1, true);
    }

    #[test]
    fn join_accepts_one_sided_expiry_without_requiring_dead_historical_handles() {
        let p = reference_typed("fn f(a: &Ticket) -> Percent {return a.value;}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let a = Place::Parameter(f.parameters[0]);
        let id = c.handles[&a];
        let mut yes = c.clone();
        yes.remaining.clear();
        yes.expire();
        assert!(!yes.loans.contains_key(&id));
        let no = c.clone();
        let suffix = c.remaining.clone();
        c.join(yes, no, suffix, f.span).unwrap();
        assert_eq!(c.live_handle(a, f.span).unwrap(), id);
    }
    fn join_legal_lineage(exclusive: bool) {
        let capability = if exclusive { "&mut " } else { "&" };
        let p = reference_typed(&format!(
            "fn f(v: {capability}Ticket) -> Percent {{let alias = v; return alias.value;}}"
        ));
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let alias = Place::Local(LocalId(0));
        let id = c.live_handle(alias, f.span).unwrap();
        if exclusive {
            assert!(matches!(
                c.states[&Place::Parameter(f.parameters[0])],
                State::Moved { .. }
            ));
        }
        let suffix = c.remaining.clone();
        c.join(c.clone(), c.clone(), suffix, f.span).unwrap();
        assert_eq!(c.live_handle(alias, f.span).unwrap(), id);
    }
    #[test]
    fn join_preserves_legitimate_shared_alias() {
        join_legal_lineage(false);
    }
    #[test]
    fn join_preserves_legitimate_exclusive_transfer_from_moved_source() {
        join_legal_lineage(true);
    }
    #[test]
    fn join_preserves_conditional_move_and_branch_local_transfer_cleanup() {
        let p = reference_typed(
            "fn f(v: &mut Ticket) -> Percent {let alias = v; let next = alias; return next.value;}",
        );
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let mut yes = c.clone();
        yes.statements(&f.body[1..2], f.id).unwrap();
        let no = c.clone();
        let alias = Place::Local(LocalId(0));
        c.join(yes, no, FutureUses::from([(alias, 1)]), f.span)
            .unwrap();
        assert!(matches!(c.states[&alias], State::ConditionalMove { .. }));
        assert!(c.available(alias, f.span).is_err());
        assert!(!c.handles.contains_key(&Place::Local(LocalId(1))));
        assert_eq!(c.loans.len(), 1);
    }

    fn loop_lineage_fault(edge: usize, transfer: bool) {
        // There is no outside reference use: discharge would erase the loan.
        let capability = if transfer { "&mut " } else { "&" };
        let p = reference_typed(&format!("fn f(flag: bool,v: {capability}Ticket) -> Percent {{let carried = v; while flag {{let seen = carried.value;}} return 80;}}"));
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        let carried = Place::Local(LocalId(0));
        let id = c.handles[&carried];
        c.remaining.clear();
        c.expire();
        assert!(c.loans.contains_key(&id));
        assert!(c.targets.last().unwrap().exit.loans.is_empty());
        if transfer {
            let State::Moved { transfer, .. } = c
                .states
                .get_mut(&Place::Parameter(f.parameters[1]))
                .unwrap()
            else {
                panic!()
            };
            *transfer = Transfer::Local(LocalId(999));
        } else {
            c.loans.get_mut(&id).unwrap().span = *span;
            assert_ne!(c.loans[&id].span, header.loans[&id].span);
        }
        assert!(c.live_handle(carried, *span).is_err());
        let before = c.clone();
        match edge {
            0 => {
                assert!(c.validate_loop_backedge(&header, *span).is_err());
            }
            1 => {
                assert!(c.continue_edge(*span).is_err());
            }
            2 => {
                assert!(c.break_edge(*span).is_err());
            }
            _ => unreachable!(),
        }
        assert_atomic(&c, &before); // No cleanup, normalization or discharge hid it.
    }
    #[test]
    fn recurrent_backedge_rejects_changed_origin_span() {
        loop_lineage_fault(0, false);
    }
    #[test]
    fn continue_rejects_changed_origin_span() {
        loop_lineage_fault(1, false);
    }
    #[test]
    fn break_rejects_changed_origin_span_before_recurrence_only_discharge() {
        loop_lineage_fault(2, false);
    }
    #[test]
    fn recurrent_backedge_rejects_corrupted_exclusive_transfer_payload() {
        loop_lineage_fault(0, true);
    }
    #[test]
    fn continue_rejects_corrupted_exclusive_transfer_payload() {
        loop_lineage_fault(1, true);
    }
    #[test]
    fn break_rejects_corrupted_exclusive_transfer_before_recurrence_only_discharge() {
        loop_lineage_fault(2, true);
    }

    fn lawful_carried_lineage(exclusive: bool) {
        let capability = if exclusive { "&mut " } else { "&" };
        let p = reference_typed(&format!("fn f(flag: bool,v: {capability}Ticket) -> Percent {{let carried = v; while flag {{let seen = carried.value;}} return 80;}}"));
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let header = install_continue_target(&mut c, body, FutureUses::new());
        c.statements_with(body, f.id, &FutureUses::new()).unwrap();
        let carried = Place::Local(LocalId(0));
        let id = c.live_handle(carried, *span).unwrap();
        assert_eq!(c.remaining.get(&carried).copied().unwrap_or(0), 0);
        let mut back = c.clone();
        back.validate_loop_backedge(&header, *span).unwrap();
        let mut next = c.clone();
        assert_eq!(next.continue_edge(*span).unwrap(), Flow::Continue);
        assert_eq!(c.break_edge(*span).unwrap(), Flow::Break);
        assert!(!c.loans.contains_key(&id));
        assert!(c.recurrent.is_empty());
    }
    #[test]
    fn shared_carried_lineage_accepts_backedge_continue_final_use_and_discharge() {
        lawful_carried_lineage(false);
    }
    #[test]
    fn transferred_exclusive_carried_lineage_accepts_backedge_continue_and_discharge() {
        lawful_carried_lineage(true);
    }
    #[test]
    fn initializer_index_borrows_direct_reference_and_unrelated_initializers() {
        let p = typed("fn f(t: Ticket, flag: bool) -> Percent {let other = flag; let view = &t; let predecessor_result = view.value; return predecessor_result;}");
        let f = p.functions()[0].as_ordinary().unwrap();
        let c = Checker::new(&p, f);
        assert!(c.initializers.invalid.is_none());
        assert_eq!(c.initializers.entries.len(), 3);
        for statement in &f.body {
            if let ValueStatement::Let {
                local, initializer, ..
            } = statement
            {
                assert!(std::ptr::eq(
                    c.initializers.get(f.id, *local).unwrap(),
                    initializer
                ));
            }
        }
        check(&p).unwrap();
    }
    #[test]
    fn initializer_index_covers_then_else_while_and_nested_compositions() {
        let p = typed("fn f(t: Ticket, flag: bool) -> Percent {if flag {let a = &t; while flag {if flag {let b = a; let x = b.value; continue;} else {let c = &t; let y = c.value; break;}}} else {while flag {let d = &t; let z = d.value;}} let e = &t; return e.value;}");
        let f = p.functions()[0].as_ordinary().unwrap();
        let c = Checker::new(&p, f);
        fn inspect(c: &Checker<'_>, body: &[ValueStatement]) -> usize {
            let mut count = 0;
            for statement in body {
                match statement {
                    ValueStatement::Let {
                        local, initializer, ..
                    } => {
                        assert!(std::ptr::eq(
                            c.initializers.get(c.function, *local).unwrap(),
                            initializer
                        ));
                        count += 1;
                    }
                    ValueStatement::If {
                        then_body,
                        else_body,
                        ..
                    } => {
                        count += inspect(c, then_body) + inspect(c, else_body);
                    }
                    ValueStatement::While { body, .. } => count += inspect(c, body),
                    _ => {}
                }
            }
            count
        }
        assert_eq!(inspect(&c, &f.body), p.locals().len());
        assert_eq!(c.initializers.entries.len(), p.locals().len());
        check(&p).unwrap();
    }
    #[test]
    fn initializer_index_lookup_uses_ids_independent_of_unrelated_order_and_function() {
        for declarations in [
            "let first = flag; let second = flag;",
            "let second = flag; let first = flag;",
        ] {
            let p = typed(&format!("fn f(t: Ticket,flag: bool) -> Percent {{{declarations} let view = &t; return view.value;}} fn g(t: Ticket,flag: bool) -> Percent {{let view = &t; return view.value;}}"));
            for f in p.functions().iter().filter_map(|f| f.as_ordinary()) {
                let c = Checker::new(&p, f);
                assert!(c.initializers.invalid.is_none());
                for local in p.locals() {
                    let found = c.initializers.get(f.id, local.id);
                    assert_eq!(found.is_some(), local.function == f.id);
                    assert!(c
                        .initializers
                        .get(FunctionId(usize::MAX), local.id)
                        .is_none());
                }
            }
            check(&p).unwrap();
        }
    }
    fn indexed_chain(n: usize, exclusive: bool) -> Program {
        let capability = if exclusive { "&mut " } else { "&" };
        let mut body = format!("fn f(r0: {capability}Ticket) -> Percent {{");
        for i in 1..=n {
            body.push_str(&format!("let r{i} = r{};", i - 1));
        }
        body.push_str(&format!("return r{n}.value;}}"));
        typed(&body)
    }
    #[test]
    fn initializer_index_deep_shared_chain_retains_one_live_authority() {
        let p = indexed_chain(128, false);
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..128], f.id).unwrap();
        let id = c.handles[&Place::Parameter(f.parameters[0])];
        assert_eq!(c.initializers.entries.len(), 128);
        assert!(c.handles.values().all(|loan| *loan == id));
        assert_eq!(
            c.live_handle(Place::Local(LocalId(127)), f.span).unwrap(),
            id
        );
        check(&p).unwrap();
    }
    #[test]
    fn initializer_index_exclusive_chain_validates_moved_source_edges_live() {
        let p = indexed_chain(64, true);
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..64], f.id).unwrap();
        let last = Place::Local(LocalId(63));
        let id = c.live_handle(last, f.span).unwrap();
        assert!(matches!(
            c.states[&Place::Parameter(f.parameters[0])],
            State::Moved {
                transfer: Transfer::Local(LocalId(0)),
                ..
            }
        ));
        let mut broken = c.clone();
        let State::Moved { transfer, .. } =
            broken.states.get_mut(&Place::Local(LocalId(30))).unwrap()
        else {
            panic!()
        };
        *transfer = Transfer::Local(LocalId(999));
        assert!(broken.live_handle(last, f.span).is_err());
        assert_eq!(c.live_handle(last, f.span).unwrap(), id);
        assert!(Rc::ptr_eq(&c.initializers, &broken.initializers));
        check(&p).unwrap();
    }
    #[test]
    fn initializer_index_is_shared_across_snapshots_and_expression_rollback() {
        let p = reference_typed("fn f(b: Ticket,t: Ticket) -> Percent {let view = &b; return pair(view.value,duplicate(t,t));}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let (value, _) = operand(&f.body[1]);
        prepared_expression(&mut c, value);
        let before = c.clone();
        initializer_probe::reset();
        for _ in 0..100 {
            assert!(Rc::ptr_eq(&c.initializers, &c.clone().initializers));
        }
        assert_eq!(initializer_probe::counts().construction_visits, 0);
        assert!(c.expression(value, Transfer::Return(f.id)).is_err());
        assert_atomic(&c, &before);
        assert!(Rc::ptr_eq(&c.initializers, &before.initializers));
        assert_eq!(initializer_probe::counts().construction_visits, 0);
    }
    #[test]
    fn initializer_index_faults_fail_closed_before_ownership_without_panic() {
        for fault in 0..11 {
            let mut p = typed("fn f(t: Ticket) -> Percent {let view = &t; return view.value;} fn g(t: Ticket) -> Percent {let view = &t; return view.value;}");
            let foreign = p.locals()[1].id;
            match fault {
                0 => {
                    let crate::hir::Function::Ordinary(f) = &mut p.functions[0] else {
                        panic!()
                    };
                    f.body.remove(0);
                }
                1 => {
                    let crate::hir::Function::Ordinary(f) = &mut p.functions[0] else {
                        panic!()
                    };
                    f.body.insert(1, f.body[0].clone());
                }
                2 | 3 => {
                    let crate::hir::Function::Ordinary(f) = &mut p.functions[0] else {
                        panic!()
                    };
                    let ValueStatement::Let { local, .. } = &mut f.body[0] else {
                        panic!()
                    };
                    *local = if fault == 2 {
                        LocalId(usize::MAX)
                    } else {
                        foreign
                    };
                }
                4 => p.locals[0].function = FunctionId(1),
                5 => p.locals[0].id = LocalId(999),
                6 => {
                    let crate::hir::Function::Ordinary(f) = &mut p.functions[0] else {
                        panic!()
                    };
                    let ValueStatement::Let { initializer, .. } = &mut f.body[0] else {
                        panic!()
                    };
                    initializer.ty = ExprType::Bool;
                }
                7 => {
                    let mut orphan = p.locals[0].clone();
                    orphan.id = LocalId(p.locals.len());
                    p.locals.push(orphan);
                }
                8..=10 => {
                    let crate::hir::Function::Ordinary(f) = &mut p.functions[0] else {
                        panic!()
                    };
                    let ValueStatement::Let { initializer, .. } = &mut f.body[0] else {
                        panic!()
                    };
                    initializer.kind = match fault {
                        8 => ExprKind::Local(LocalId(usize::MAX)),
                        9 => ExprKind::Local(foreign),
                        _ => ExprKind::Parameter(ParameterId(usize::MAX)),
                    };
                }
                _ => unreachable!(),
            }
            let f = p.functions()[0].as_ordinary().unwrap();
            let c = Checker::new(&p, f);
            assert!(c.initializers.invalid.is_some(), "fault {fault}");
            assert!(c.initializers.get(f.id, LocalId(0)).is_none());
            let errors = check(&p).unwrap_err();
            assert!(
                errors[0]
                    .message
                    .contains("canonical local initializer index"),
                "fault {fault}"
            );
        }
    }
    #[test]
    fn initializer_index_missing_entry_never_falls_back_to_body_scan() {
        let p = typed("fn f(t: Ticket) -> Percent {let view = &t; return view.value;}");
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..1], f.id).unwrap();
        let view = Place::Local(LocalId(0));
        c.live_handle(view, f.span).unwrap();
        // Replace structural metadata only in this injected private fixture.
        c.initializers = Rc::new(Initializers {
            function: f.id,
            entries: BTreeMap::new(),
            arm_bindings: BTreeSet::new(),
            invalid: None,
        });
        initializer_probe::reset();
        assert!(c.live_handle(view, f.span).is_err());
        assert_eq!(initializer_probe::counts().lookups, 1);
        assert_eq!(initializer_probe::counts().construction_visits, 0);
    }
    #[test]
    fn initializer_index_rejects_same_typed_but_malformed_initializer_lineage() {
        let mut p = indexed_chain(2, false);
        let crate::hir::Function::Ordinary(f) = &mut p.functions[0] else {
            panic!()
        };
        let ValueStatement::Let { initializer, .. } = &mut f.body[1] else {
            panic!()
        };
        initializer.kind = ExprKind::Local(LocalId(1)); // Self-cycle, exact ref type.
        let f = p.functions()[0].as_ordinary().unwrap();
        let mut c = Checker::new(&p, f);
        assert!(c.initializers.invalid.is_none()); // Location is sound; lineage is not cached.
        c.handles.insert(
            Place::Local(LocalId(1)),
            c.handles[&Place::Parameter(f.parameters[0])],
        );
        assert!(c.live_handle(Place::Local(LocalId(1)), f.span).is_err());
    }
    #[test]
    fn initializer_index_actual_events_separate_construction_lookup_lineage_and_depth() {
        for n in [0, 1, 10, 25, 50] {
            for family in ["shared", "exclusive", "fanout", "scalar"] {
                let p = if family == "shared" || family == "exclusive" {
                    indexed_chain(n, family == "exclusive")
                } else {
                    let mut body = if family == "fanout" {
                        "fn f(r0: &Ticket) -> Percent {".to_owned()
                    } else {
                        "fn f(r0: Percent) -> Percent {".to_owned()
                    };
                    for i in 1..=n {
                        body.push_str(&format!(
                            "let r{i} = r{};",
                            if family == "fanout" { 0 } else { i - 1 }
                        ));
                    }
                    body.push_str(&format!(
                        "return r{n}{};}}",
                        if family == "fanout" { ".value" } else { "" }
                    ));
                    typed(&body)
                };
                let f = p.functions()[0].as_ordinary().unwrap();
                initializer_probe::reset();
                let mut c = Checker::new(&p, f);
                let construction = initializer_probe::counts();
                assert_eq!(construction.construction_visits, n + 1);
                assert_eq!(construction.lookups, 0);
                assert_eq!(c.initializers.entries.len(), n);
                let snapshot = c.clone();
                assert!(Rc::ptr_eq(&c.initializers, &snapshot.initializers));
                c.body(f).unwrap();
                let actual = initializer_probe::counts();
                let (lookups, calls, depth) = match family {
                    "shared" | "exclusive" => (n * (n + 1) / 2, (n + 1) * (n + 2) / 2, n + 1),
                    "fanout" => (
                        usize::from(n > 0),
                        n + 1 + usize::from(n > 0),
                        if n > 0 { 2 } else { 1 },
                    ),
                    _ => (0, 0, 0),
                };
                assert_eq!(actual.construction_visits, n + 1); // No per-ancestor statement scans.
                assert_eq!(
                    (actual.lookups, actual.origin_calls, actual.max_depth),
                    (lookups, calls, depth),
                    "{family} n={n}"
                );
            }
        }
    }
    fn reference_write(c: &mut Checker<'_>, s: &ValueStatement) -> OwnershipResult {
        let ValueStatement::CopyReferenceFieldAssign {
            reference,
            record,
            field,
            value,
            reference_span,
            span,
            ..
        } = s
        else {
            panic!()
        };
        c.copy_reference_field_assign(*reference, *record, *field, value, *reference_span, *span)
    }
    fn prepare_reference_write(c: &mut Checker<'_>, s: &ValueStatement) {
        c.remaining = future_uses(std::slice::from_ref(s));
        c.expire();
    }
    fn assert_reference_write_atomic(c: &Checker<'_>, before: &Checker<'_>) {
        assert_atomic(c, before);
        assert!(Rc::ptr_eq(&c.initializers, &before.initializers));
    }
    #[test]
    fn reference_write_inert_rhs_has_no_intrinsic_transfer_or_allocation_and_only_its_own_hold() {
        let p = field_typed("fn f(v: &mut Ticket) {v.value=80;}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let target = Place::Parameter(f.parameters[0]);
        let id = c.handles[&target];
        c.loans.get_mut(&id).unwrap().held = 2;
        c.recurrent.push(BTreeSet::from([id]));
        add_test_target(&mut c);
        c.loop_scope = Some(LoopScope::new(&c, false));
        c.terminal_return = true;
        prepare_reference_write(&mut c, &f.body[0]);
        let before = c.clone();
        reference_write(&mut c, &f.body[0]).unwrap();
        assert_eq!(c.remaining[&target], 0);
        c.remaining = before.remaining.clone();
        assert_reference_write_atomic(&c, &before);
    }
    #[test]
    fn reference_write_final_use_expires_only_after_complete_write() {
        let p = field_typed("fn f(v: &mut Ticket) {v.value=v.value;}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let before = c.clone();
        let target = Place::Parameter(f.parameters[0]);
        assert_eq!(c.remaining[&target], 2);
        reference_write(&mut c, &f.body[0]).unwrap();
        assert_eq!(c.remaining[&target], 0);
        assert!(c.loans.is_empty());
        assert_eq!(c.handles, before.handles);
        assert_eq!(c.states, before.states);
        assert_eq!(c.next_loan, before.next_loan);
    }
    #[test]
    fn reference_write_success_retains_lawful_temporary_allocator_advancement() {
        for rhs in [
            "sample(&other)",
            "sample_mut(&mut other)",
            "hold(&other,sample(&other))",
        ] {
            let p = field_typed(&format!(
                "fn f(v: &mut Ticket,t: Ticket) {{let mut other=t; v.value={rhs};}}"
            ));
            let f = return_function(&p);
            let mut c = Checker::new(&p, f);
            c.statements(&f.body[..1], f.id).unwrap();
            prepare_reference_write(&mut c, &f.body[1]);
            let before = c.clone();
            reference_write(&mut c, &f.body[1]).unwrap();
            assert!(c.next_loan > before.next_loan);
            assert_eq!(c.states, before.states);
            assert_eq!(c.handles, before.handles);
            assert!(c.loans.is_empty());
            c.assert_no_holds();
        }
    }
    #[test]
    fn reference_write_success_commits_unrelated_owner_transfer() {
        let p = field_typed("fn f(v: &mut Ticket,t: Ticket) {v.value=consume(t);}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let before = c.clone();
        reference_write(&mut c, &f.body[0]).unwrap();
        assert!(matches!(
            c.states[&Place::Parameter(f.parameters[1])],
            State::Moved { .. }
        ));
        assert_eq!(
            c.states[&Place::Parameter(f.parameters[0])],
            State::Available
        );
        assert_eq!(c.next_loan, before.next_loan);
        c.assert_no_holds();
    }
    #[test]
    fn reference_write_late_rhs_failure_restores_prepared_operation_not_list_entry() {
        let p=field_typed("fn fresh() -> Ticket {return fresh();} fn f(flag: bool,v: &mut Ticket) {while flag {let t=fresh(); let other=fresh(); v.value=join(consume(t),sample(&other),duplicate(other,other));}}");
        let f = return_function(&p);
        let ValueStatement::While { body, .. } = &f.body[0] else {
            panic!()
        };
        let s = &body[2];
        let mut c = Checker::new(&p, f);
        install_continue_target(&mut c, body, FutureUses::new());
        c.statements_with(&body[..2], f.id, &future_uses(&body[2..]))
            .unwrap();
        let target = Place::Parameter(f.parameters[1]);
        let id = c.handles[&target];
        assert!(c.loop_scope.is_some());
        assert!(!c.targets.is_empty());
        assert!(!c.recurrent.is_empty());
        c.terminal_return = true;
        prepare_reference_write(&mut c, s);
        let before = c.clone();
        let ValueStatement::CopyReferenceFieldAssign { value, .. } = s else {
            panic!()
        };
        let ExprKind::Call { arguments, .. } = &value.kind else {
            panic!()
        };
        let mut prefix = c.clone();
        prefix.loans.get_mut(&id).unwrap().held += 1;
        prefix
            .expression(&arguments[0], Transfer::WriteThrough(target))
            .unwrap();
        prefix
            .expression(&arguments[1], Transfer::WriteThrough(target))
            .unwrap();
        assert_ne!(prefix.states, before.states);
        assert!(prefix.next_loan > before.next_loan);
        assert!(prefix
            .expression(&arguments[2], Transfer::WriteThrough(target))
            .is_err());
        assert!(reference_write(&mut c, s)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_reference_write_atomic(&c, &before);
    }
    #[test]
    fn reference_write_successful_rhs_transfer_fails_at_final_gate_and_restores_exact_entry() {
        let p = field_typed("fn f(v: &mut Ticket) {v.value=inspect_mut(v);}");
        let f = return_function(&p);
        let s = &f.body[0];
        let mut c = Checker::new(&p, f);
        prepare_reference_write(&mut c, s);
        let before = c.clone();
        let target = Place::Parameter(f.parameters[0]);
        let id = c.handles[&target];
        let ValueStatement::CopyReferenceFieldAssign { value, .. } = s else {
            panic!()
        };
        // Independently prove that the entire RHS succeeds; failure is final availability.
        let mut rhs = c.clone();
        *rhs.remaining.get_mut(&target).unwrap() -= 1;
        rhs.loans.get_mut(&id).unwrap().held += 1;
        rhs.expression(value, Transfer::WriteThrough(target))
            .unwrap();
        assert!(matches!(rhs.states[&target], State::Moved { .. }));
        assert!(rhs.loans.contains_key(&id));
        assert_eq!(rhs.loans[&id].held, 1);
        assert!(rhs.available(target, value.span).is_err());
        assert!(reference_write(&mut c, s)
            .unwrap_err()
            .message
            .contains("moved value"));
        assert_reference_write_atomic(&c, &before);
    }
    #[test]
    fn reference_write_nested_calls_cannot_expire_or_release_target_retention() {
        let p=field_typed("fn f(t: Ticket,other: Ticket) {let mut owned=t; let v=&mut owned; v.value=hold(&other,join(v.value,sample(&other),owned.value));}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        prepare_reference_write(&mut c, &f.body[2]);
        let before = c.clone();
        assert!(reference_write(&mut c, &f.body[2])
            .unwrap_err()
            .message
            .contains("exclusively borrowed"));
        assert_reference_write_atomic(&c, &before);
        let p=field_typed("fn f(t: Ticket,other: Ticket) {let mut owned=t; let v=&mut owned; v.value=hold(&other,join(v.value,sample(&other),80));}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..2], f.id).unwrap();
        let target = Place::Local(p.locals()[1].id);
        let id = c.handles[&target];
        c.loans.get_mut(&id).unwrap().held = 2;
        prepare_reference_write(&mut c, &f.body[2]);
        reference_write(&mut c, &f.body[2]).unwrap();
        assert_eq!(c.loans[&id].held, 2);
    }
    #[test]
    fn reference_write_faults_reject_live_mapping_origin_owner_span_kind_and_availability_atomically(
    ) {
        let p = field_typed("fn f(a: &mut Ticket,b: &mut Ticket) {a.value=80; b.value=80;}");
        let f = return_function(&p);
        let c = Checker::new(&p, f);
        let a = Place::Parameter(f.parameters[0]);
        let b = Place::Parameter(f.parameters[1]);
        let id = c.handles[&a];
        for defect in 0..8 {
            let mut broken = c.clone();
            match defect {
                0 => {
                    broken.handles.insert(a, broken.handles[&b]);
                }
                1 => {
                    broken.handles.remove(&a);
                }
                2 => broken.loans.get_mut(&id).unwrap().span.end += 1,
                3 => {
                    broken.loans.get_mut(&id).unwrap().origin =
                        LoanOrigin::ExternalParameter(f.parameters[1])
                }
                4 => broken.loans.get_mut(&id).unwrap().owner = Some(b),
                5 => broken.loans.get_mut(&id).unwrap().kind = BorrowKind::Shared,
                6 => {
                    broken
                        .states
                        .insert(a, State::ConditionalMove { span: f.span });
                }
                7 => {
                    broken.loans.remove(&id);
                }
                _ => unreachable!(),
            };
            assert!(broken.live_handle(a, f.span).is_err() || broken.available(a, f.span).is_err());
            let before = broken.clone();
            assert!(
                reference_write(&mut broken, &f.body[0]).is_err(),
                "defect {defect}"
            );
            assert_reference_write_atomic(&broken, &before);
        }
    }
    #[test]
    fn reference_write_transferred_recipient_validates_exact_transfer_payload_without_source_availability(
    ) {
        let p=field_typed("fn f(t: Ticket) {let mut owned=t; let first=&mut owned; let second=first; second.value=80;}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        c.statements(&f.body[..3], f.id).unwrap();
        prepare_reference_write(&mut c, &f.body[3]);
        let valid = c.clone();
        assert!(matches!(
            c.states[&Place::Local(p.locals()[1].id)],
            State::Moved { .. }
        ));
        reference_write(&mut c, &f.body[3]).unwrap();
        let mut broken = valid;
        let State::Moved { transfer, .. } = broken
            .states
            .get_mut(&Place::Local(p.locals()[1].id))
            .unwrap()
        else {
            panic!()
        };
        *transfer = Transfer::Local(LocalId(999));
        let before = broken.clone();
        assert!(reference_write(&mut broken, &f.body[3]).is_err());
        assert_reference_write_atomic(&broken, &before);
    }
    #[test]
    fn reference_write_invalid_target_ids_and_field_metadata_reject_without_panic() {
        let p = field_typed("fn f(v: &mut Ticket) {v.value=80;}");
        let f = return_function(&p);
        for defect in 0..5 {
            let mut s = f.body[0].clone();
            let ValueStatement::CopyReferenceFieldAssign {
                reference,
                record,
                field,
                value,
                ..
            } = &mut s
            else {
                panic!()
            };
            match defect {
                0 => *reference = Place::Parameter(ParameterId(usize::MAX)),
                1 => *reference = Place::Local(LocalId(usize::MAX)),
                2 => *record = RecordId(usize::MAX),
                3 => *field = FieldId(usize::MAX),
                4 => value.ty = ExprType::Range(crate::hir::RangeTypeId(usize::MAX)),
                _ => unreachable!(),
            };
            let mut c = Checker::new(&p, f);
            let before = c.clone();
            assert!(reference_write(&mut c, &s).is_err());
            assert_reference_write_atomic(&c, &before);
        }
    }
    #[test]
    fn reference_write_summaries_count_target_and_every_nested_rhs_child() {
        for construction in [false, true] {
            for pos in 0..3 {
                let mut args = ["80", "80", "80"];
                args[pos] = "v.value";
                let rhs = if construction {
                    format!(
                        "triple(Triple {{last:{},first:{},middle:{}}})",
                        args[0], args[1], args[2]
                    )
                } else {
                    format!("three({})", args.join(","))
                };
                let p = reference_typed(&format!("fn f(v: &mut Ticket) {{v.value={rhs};}}"));
                let f = return_function(&p);
                let target = Place::Parameter(f.parameters[0]);
                assert_eq!(future_uses(&f.body), FutureUses::from([(target, 2)]));
                assert_eq!(reference_uses(&f.body), BTreeSet::from([target]));
                let mut c = Checker::new(&p, f);
                let id = c.handles[&target];
                c.recurrent.push(BTreeSet::from([id]));
                let before = c.clone();
                reference_write(&mut c, &f.body[0]).unwrap();
                assert_eq!(c.remaining[&target], 0);
                assert_eq!(c.next_loan, before.next_loan);
                assert_eq!(c.handles, before.handles);
                assert!(c.loans.contains_key(&id));
                c.assert_no_holds();
                assert!(Rc::ptr_eq(&c.initializers, &before.initializers));
            }
        }
    }
    #[test]
    fn reference_write_join_rejects_mapping_substitution_before_write_authorization() {
        let p = field_typed("fn f(a: &mut Ticket,b: &mut Ticket) {a.value=80; b.value=80;}");
        let f = return_function(&p);
        let c = Checker::new(&p, f);
        let a = Place::Parameter(f.parameters[0]);
        let b = Place::Parameter(f.parameters[1]);
        for side in [false, true] {
            let mut entry = c.clone();
            let mut broken = c.clone();
            broken.handles.insert(a, broken.handles[&b]);
            assert!(reference_write(&mut broken, &f.body[0]).is_err());
            let before = entry.clone();
            let (yes, no) = if side {
                (broken, c.clone())
            } else {
                (c.clone(), broken)
            };
            assert!(entry.join(yes, no, c.remaining.clone(), f.span).is_err());
            assert_reference_write_atomic(&entry, &before);
        }
    }
    #[test]
    fn reference_write_recurrent_edges_validate_lineage_before_discharge() {
        let p = field_typed(
            "fn f(flag: bool,v: &mut Ticket) {let carried=v; while flag {carried.value=80;}}",
        );
        let f = return_function(&p);
        let mut entry = Checker::new(&p, f);
        entry.statements(&f.body[..1], f.id).unwrap();
        let ValueStatement::While { body, span, .. } = &f.body[1] else {
            panic!()
        };
        let header = install_continue_target(&mut entry, body, FutureUses::new());
        let carried = Place::Local(p.locals()[0].id);
        let id = entry.handles[&carried];
        for edge in 0..3 {
            for defect in 0..2 {
                let mut c = entry.clone();
                if defect == 0 {
                    c.loans.get_mut(&id).unwrap().span.end += 1;
                } else {
                    let State::Moved { transfer, .. } = c
                        .states
                        .get_mut(&Place::Parameter(f.parameters[1]))
                        .unwrap()
                    else {
                        panic!()
                    };
                    *transfer = Transfer::Local(LocalId(999));
                }
                let before = c.clone();
                assert!(reference_write(&mut c, &body[0]).is_err());
                assert_reference_write_atomic(&c, &before);
                let result = match edge {
                    0 => c.validate_loop_backedge(&header, *span),
                    1 => c.continue_edge(*span).map(|_| ()),
                    2 => c.break_edge(*span).map(|_| ()),
                    _ => unreachable!(),
                };
                assert!(result.is_err());
                assert_reference_write_atomic(&c, &before);
            }
        }
        for edge in 0..3 {
            let mut c = entry.clone();
            prepare_reference_write(&mut c, &body[0]);
            reference_write(&mut c, &body[0]).unwrap();
            assert!(c.loans.contains_key(&id));
            match edge {
                0 => c.validate_loop_backedge(&header, *span).unwrap(),
                1 => {
                    c.continue_edge(*span).unwrap();
                }
                2 => {
                    c.break_edge(*span).unwrap();
                }
                _ => unreachable!(),
            };
            if edge == 2 {
                assert!(c.loans.is_empty());
            } else {
                assert!(c.loans.contains_key(&id));
            }
            c.assert_no_holds();
        }
    }
    #[test]
    fn reference_write_final_gate_rejects_a_different_live_id_even_with_identical_lineage() {
        let p = field_typed("fn f(v: &mut Ticket) {v.value=80;}");
        let f = return_function(&p);
        let mut c = Checker::new(&p, f);
        let ValueStatement::CopyReferenceFieldAssign {
            reference,
            record,
            reference_span,
            ..
        } = &f.body[0]
        else {
            panic!()
        };
        let original = c.handles[reference];
        c.loans.get_mut(&original).unwrap().held = 1;
        c.validate_reference_write_completion(*reference, *record, original, *reference_span)
            .unwrap();
        let replacement = LoanId(c.next_loan);
        c.next_loan += 1;
        c.loans.insert(replacement, c.loans[&original].clone());
        c.handles.insert(*reference, replacement);
        // The sole active mapping and canonical lineage are valid on their own.
        assert_eq!(
            c.exclusive_record_handle(*reference, *record, *reference_span)
                .unwrap(),
            replacement
        );
        let before = c.clone();
        assert!(c
            .validate_reference_write_completion(*reference, *record, original, *reference_span)
            .is_err());
        assert_reference_write_atomic(&c, &before);
    }
}

#[cfg(test)]
mod cleanup_effect_transaction_tests {
    use super::*;
    use crate::effects::{custody::Custody, Effect, EffectSet};

    #[test]
    fn complete_expression_failure_restores_cleanup_custody_without_an_approval() {
        let mut release = EffectSet::EMPTY;
        release.insert(Effect::Filesystem);
        let p = crate::hir::resources::fixture_effects(
            "fn consume(x:Leaf)->bool effects filesystem{return true;} fn receive(a:bool,b:bool)->bool{return true;} fn f(a:Leaf,b:Leaf)->bool effects filesystem{return receive(consume(a),consume(b));}",
            EffectSet::EMPTY,
            release,
        );
        let f = p.functions()[4].as_ordinary().unwrap();
        let mut checker = Checker::new(&p, f);
        let plans = p.resources.plans(&p).unwrap();
        let summaries = Rc::new(p.resources.summaries(&p, &plans).unwrap());
        let mut custody = Custody::new(f.id, summaries, None);
        for parameter in &f.parameters {
            custody.initialize(Place::Parameter(*parameter), ValueType::Record(RecordId(0)));
        }
        checker.cleanup_custody = Some(custody);
        let ValueStatement::Return { value, .. } = &f.body[0] else {
            unreachable!()
        };
        let mut value = value.clone();
        let ExprKind::Call { arguments, .. } = &mut value.kind else {
            unreachable!()
        };
        let ExprKind::Call { arguments, .. } = &mut arguments[1].kind else {
            unreachable!()
        };
        arguments[0].kind = ExprKind::Parameter(f.parameters[0]);
        let before = checker.clone();
        assert!(checker.expression(&value, Transfer::Return(f.id)).is_err());
        assert_eq!(checker.cleanup_custody, before.cleanup_custody);
        assert_eq!(checker.states, before.states);
        assert_eq!(checker.remaining, before.remaining);
    }
}
