//! Deterministic ordinary MIR execution. The only public preparation starts at
//! bounded source; public HIR and MIR inspection never confer execution authority.
// Keep immediate origins and the wide trap value inline: exhaustion reporting
// must not require an additional allocation merely to return the error itself.
#![allow(clippy::result_large_err)]
mod audit;
pub mod contract;
mod domain;
mod machine;
mod render;

pub use crate::bounded::{AdmissionLimits, ResourceFailure, ResourceKind};
use crate::{
    bounded::{Context, START},
    hir::{self, FunctionId, ValueType},
    lexer::Span,
    mir::{self, Origin},
};
pub use machine::conformance;
pub use machine::trusted::{Event as TrustedEvent, ExecutionSession};
pub use render::write_output;
use std::{io::Read, rc::Rc};
pub fn render_admission_error(
    error: &ExecutionError,
    allowance: usize,
) -> Result<String, ExecutionError> {
    render::error(None, error, allowance)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionLimits {
    pub steps: u64,
    pub call_depth: usize,
    pub expression_depth: usize,
    pub working_storage: usize,
    pub rendered_output: usize,
    pub trace_depth: usize,
}
impl ExecutionLimits {
    pub const MAX: Self = Self {
        steps: 100_000_000,
        call_depth: 256,
        expression_depth: 128,
        working_storage: 268_435_456,
        rendered_output: 1_048_576,
        trace_depth: 64,
    };
    pub fn validate(self) -> Result<(), ResourceFailure> {
        let m = Self::MAX;
        if self.steps == 0 || self.steps > m.steps {
            return Err(ResourceFailure::limit("steps override", START));
        }
        for (v, max, name) in [
            (self.call_depth, m.call_depth, "call depth override"),
            (
                self.expression_depth,
                m.expression_depth,
                "expression depth override",
            ),
            (
                self.working_storage,
                m.working_storage,
                "working storage override",
            ),
            (
                self.rendered_output,
                m.rendered_output,
                "rendered output override",
            ),
            (self.trace_depth, m.trace_depth, "trace depth override"),
        ] {
            if v == 0 || v > max {
                return Err(ResourceFailure::limit(name, START));
            }
        }
        Ok(())
    }
}
impl Default for ExecutionLimits {
    fn default() -> Self {
        Self {
            steps: 1_000_000,
            call_depth: 64,
            expression_depth: 64,
            working_storage: 16_777_216,
            rendered_output: 65_536,
            trace_depth: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeValue {
    pub ty: hir::RangeTypeId,
    pub value: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeValue {
    /// Flat outer-to-inner active chain, with one non-Result leaf payload.
    Result(Vec<ResultLayer>),
    Bool(bool),
    Range(RangeValue),
    Record {
        record: hir::RecordId,
        fields: Vec<(hir::FieldId, RangeValue)>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultLayer {
    pub ty: hir::ResultTypeId,
    pub tag: hir::ResultTag,
    pub payload: Option<ResultPayload>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResultPayload {
    Bool(bool),
    Range(RangeValue),
    Record {
        record: hir::RecordId,
        fields: Vec<(hir::FieldId, RangeValue)>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionResult {
    Value(RuntimeValue),
    NoValue,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Frontend {
        phase: &'static str,
        message: String,
    },
    RangeTrap {
        ty: hir::RangeTypeId,
        min: i64,
        max: i64,
        computed: i128,
    },
    ConfiguredLimit(&'static str),
    Integrity(&'static str),
    Host(&'static str),
    /// Private synthetic construction control, not a recoverable source error.
    SyntheticConstructionFailure,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallTrace {
    pub function: FunctionId,
    pub call_site: Span,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionError {
    pub kind: ErrorKind,
    pub origin: Span,
    pub expression: Option<Span>,
    pub trace: Vec<CallTrace>,
    pub trace_truncated: bool,
}
impl ExecutionError {
    pub(crate) fn new(kind: ErrorKind, origin: Span) -> Self {
        Self {
            kind,
            origin,
            expression: None,
            trace: Vec::new(),
            trace_truncated: false,
        }
    }
    pub(crate) fn integrity(message: &'static str) -> Self {
        Self::new(ErrorKind::Integrity(message), START)
    }
}
impl From<ResourceFailure> for ExecutionError {
    fn from(f: ResourceFailure) -> Self {
        Self::new(
            match f.kind {
                ResourceKind::Limit => ErrorKind::ConfiguredLimit(f.resource),
                ResourceKind::Host => ErrorKind::Host(f.resource),
            },
            f.span,
        )
    }
}
impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} at {}:{}",
            self.kind, self.origin.line, self.origin.column
        )
    }
}
impl std::error::Error for ExecutionError {}

// An immutable witness retained only after ownership succeeds. Exact comparison
// catches compiler-internal replacement with a merely typed program as well as
// numeric-ID-aligned substitutions; this never re-runs ownership at execution.
struct Approval {
    schema: contract::Schema,
    schedule: contract::Schedule,
    outcomes: Vec<crate::ownership::outcomes::Witness>,
    program: hir::Program,
    cleanup_sites: Vec<crate::effects::custody::Site>,
    domain: domain::Domain,
    trusted: Option<crate::trusted::approval::Approved>,
}

/// No public fields, constructors, mutation, or independent HIR/MIR pairing.
/// ```compile_fail
/// use bowfin_compiler::{compile_source, execution::CheckedExecutable};
/// let program = compile_source("fn demo() {}").unwrap();
/// let executable = CheckedExecutable { program };
/// ```
pub struct CheckedExecutable {
    outcomes: Vec<crate::ownership::outcomes::Witness>,
    program: hir::Program,
    mir: mir::MirProgram,
    origins: Vec<Vec<Origin>>,
    // Private preparation identity binds three independently stored components.
    // It is neither a numeric source identity nor global runtime state.
    approval: Rc<Approval>,
    mir_approval: Rc<Approval>,
    origin_approval: Rc<Approval>,
    limits: AdmissionLimits,
    cleanup_plans: Vec<Option<hir::resources::CleanupPlan>>,
    cleanup_effects: Vec<crate::effects::EffectSet>,
    result_plans: Vec<hir::results::VariantPlan>,
    domain: domain::Domain,
    trusted: Option<crate::trusted::approval::Approved>,
}
impl CheckedExecutable {
    /// Static rooted authority, requiring no binding invocation or source lookup.
    pub fn render_authority(&self, limits: ExecutionLimits) -> Result<String, ExecutionError> {
        audit::render(self, limits, 0)
    }
    pub fn entry(&self, name: &str) -> Result<FunctionId, ExecutionError> {
        let f = self
            .program
            .functions()
            .iter()
            .find(|f| f.name() == name)
            .ok_or_else(|| ExecutionError::integrity("selected entry does not exist"))?;
        let ordinary = f.as_ordinary().ok_or_else(|| {
            ExecutionError::new(
                ErrorKind::Integrity("selected entry is verified, not ordinary"),
                f.span(),
            )
        })?;
        if !ordinary.parameters.is_empty() {
            return Err(ExecutionError::new(
                ErrorKind::Integrity("selected entry must have zero parameters"),
                ordinary.span,
            ));
        }
        if let Some(trusted) = &self.trusted {
            if !trusted.roots.contains(&ordinary.id) {
                return Err(ExecutionError::integrity(
                    "selected entry is outside the sealed trusted roots",
                ));
            }
        }
        Ok(ordinary.id)
    }
    fn function(&self, id: FunctionId) -> Result<&mir::MirFunction, ExecutionError> {
        let mut found = self.mir.functions().iter().filter(|f| f.function() == id);
        let f = found.next().ok_or_else(|| {
            ExecutionError::integrity("canonical ordinary MIR function is missing")
        })?;
        if found.next().is_some() {
            return Err(ExecutionError::integrity(
                "duplicate canonical MIR function",
            ));
        }
        Ok(f)
    }
    fn origin(&self, id: FunctionId, block: mir::BasicBlockId) -> Result<Origin, ExecutionError> {
        let index = self
            .mir
            .functions()
            .iter()
            .position(|f| f.function() == id)
            .ok_or_else(|| ExecutionError::integrity("origin function missing"))?;
        let position = self.mir.functions()[index]
            .blocks()
            .position(|(b, _)| b == block)
            .ok_or_else(|| ExecutionError::integrity("origin block missing"))?;
        self.origins
            .get(index)
            .and_then(|v| v.get(position))
            .copied()
            .ok_or_else(|| ExecutionError::integrity("lowering origin missing"))
    }
    fn validate(&self) -> Result<(), ExecutionError> {
        self.approval.schema.validate()?;
        self.approval.schedule.validate()?;
        if !Rc::ptr_eq(&self.approval, &self.mir_approval)
            || !Rc::ptr_eq(&self.approval, &self.origin_approval)
        {
            return Err(ExecutionError::integrity(
                "cross-compilation execution authority",
            ));
        }
        let c = Context::new(self.limits);
        if let Some(catalog) = &self.program.trusted {
            c.reserve_completion_catalog(
                catalog.selection.roots().len(),
                catalog.selection.roots().iter().map(String::len).sum(),
            )?;
        }
        // Admission already bounds this private immutable representation. Check
        // shape before any recursive comparison/re-lowering, including injections.
        validate::program_shape(&self.program, &c)?;
        for trusted in [self.trusted.as_ref(), self.approval.trusted.as_ref()]
            .into_iter()
            .flatten()
        {
            trusted.shape(self.limits)?;
        }
        self.domain.shape(self.limits)?;
        if self.domain != self.approval.domain {
            return Err(ExecutionError::integrity(
                "private execution domain correspondence",
            ));
        }
        if !self.approval.cleanup_sites.is_empty() {
            let holders = self.program.locals.len()
                + self.program.parameters.len()
                + usize::from(self.program.trusted.is_some());
            let nodes = c.resource_nodes.get() + self.program.functions.len() + holders;
            c.reserve_cleanup_effects(nodes, crate::effects::custody::holder_bound(&self.program))?;
            if self.approval.cleanup_sites.len() > nodes.saturating_mul(holders) {
                return Err(ExecutionError::integrity("cleanup effect witness shape"));
            }
        }
        let plans = if self.program.resources.leaves.is_empty() {
            Vec::new()
        } else {
            self.program
                .resources
                .plans(&self.program)
                .map_err(ExecutionError::from)?
        };
        if self.cleanup_plans != plans {
            return Err(ExecutionError::integrity(
                "canonical cleanup plan correspondence",
            ));
        }
        let summaries = self
            .program
            .resources
            .summaries(&self.program, &plans)
            .map_err(ExecutionError::from)?;
        validate::cleanup_site_shape(&self.program, &self.approval.cleanup_sites, &summaries)?;
        if self.cleanup_effects != summaries {
            return Err(ExecutionError::integrity(
                "canonical cleanup effect summary correspondence",
            ));
        }
        if self.result_plans
            != self
                .program
                .variant_plans(&plans, &summaries)
                .map_err(ExecutionError::from)?
        {
            return Err(ExecutionError::integrity(
                "canonical Result variant plan/effect correspondence",
            ));
        }
        if !self.program.result_types().is_empty() {
            c.reserve_outcomes(
                self.program.locals.len()
                    + self.program.parameters.len()
                    + self.program.result_types().len()
                    + self.program.functions.len(),
            )?;
        }
        if self.outcomes.len() > self.limits.tokens.saturating_mul(self.limits.tokens)
            || self.outcomes != self.approval.outcomes
        {
            return Err(ExecutionError::integrity(
                "immutable Result outcome witness correspondence",
            ));
        }
        if self.program != self.approval.program {
            return Err(ExecutionError::integrity(
                "frontend ownership approval mismatch",
            ));
        }
        if self.mir.functions().len() != self.origins.len() {
            return Err(ExecutionError::integrity("origin/function correspondence"));
        }
        validate::mir_shape(&self.program, &self.mir, &c)?;
        if self.trusted != self.approval.trusted
            || self.trusted
                != crate::trusted::approval::Approved::derive(
                    &self.program,
                    &self.mir,
                    self.limits,
                )?
        {
            return Err(ExecutionError::integrity(
                "trusted runtime/invocation/root authority correspondence",
            ));
        }
        for f in self.mir.functions() {
            f.validate_bounded().map_err(ExecutionError::from)?;
        }
        // Same deterministic lowerer, not a second semantic or ownership pipeline.
        let (mut expected, origins) =
            mir::lower_with_origins(&self.program).map_err(ExecutionError::from)?;
        mir::attach_cleanup_effects(&mut expected, &self.approval.cleanup_sites)
            .map_err(ExecutionError::from)?;
        if self.mir != expected || self.origins != origins {
            return Err(ExecutionError::integrity(
                "exact HIR/MIR/origin correspondence",
            ));
        }
        self.domain.validate(self)?;
        Ok(())
    }
    pub fn render_result(
        &self,
        result: &ExecutionResult,
        allowance: usize,
    ) -> Result<String, ExecutionError> {
        render::result(self, result, allowance)
    }
    pub fn render_error(
        &self,
        error: &ExecutionError,
        allowance: usize,
    ) -> Result<String, ExecutionError> {
        render::error(Some(self), error, allowance)
    }
}

pub fn prepare_execution(
    source: &str,
    limits: AdmissionLimits,
) -> Result<CheckedExecutable, ExecutionError> {
    prepare_selected(source, limits, None)
}

/// Explicit selection uses the same bounded frontend and opaque preparation.
/// No profile is implicitly selected by the resource-free convenience API.
pub fn prepare_execution_with_profile(
    source: &str,
    limits: AdmissionLimits,
    selection: &crate::trusted::Selection,
) -> Result<CheckedExecutable, ExecutionError> {
    prepare_selected(source, limits, Some(selection))
}

fn prepare_selected(
    source: &str,
    limits: AdmissionLimits,
    selection: Option<&crate::trusted::Selection>,
) -> Result<CheckedExecutable, ExecutionError> {
    limits.validate()?;
    if source.len() > limits.source_bytes {
        return Err(ResourceFailure::limit("source bytes", START).into());
    }
    // Reserve lexer input/token capacity before its first source-directed growth.
    if source
        .len()
        .checked_mul(128)
        .map_or(true, |v| v > limits.working_storage)
    {
        return Err(ResourceFailure::limit("lexer storage reservation", START).into());
    }
    if source.len() as u64 > limits.frontend_work {
        return Err(ResourceFailure::limit("lexing work", START).into());
    }
    let context = Context::new(limits);
    let tokens = crate::lexer::lex_bounded(source, &context).map_err(|e| {
        context
            .failure
            .get()
            .map(ExecutionError::from)
            .unwrap_or_else(|| {
                ExecutionError::new(
                    ErrorKind::Frontend {
                        phase: "lex",
                        message: e.message,
                    },
                    e.span,
                )
            })
    })?;
    context.reserve_frontend(tokens.len(), source.len())?;
    let ast = crate::parser::parse_bounded(&tokens, &context).map_err(|e| {
        context
            .failure
            .get()
            .map(ExecutionError::from)
            .unwrap_or_else(|| {
                ExecutionError::new(
                    ErrorKind::Frontend {
                        phase: "parse",
                        message: e.message,
                    },
                    e.span,
                )
            })
    })?;
    let resolved = match selection {
        Some(selection) => crate::resolve::resolve_profile(&ast, &context, selection),
        None => crate::resolve::resolve_bounded(&ast, &context),
    }
    .map_err(|e| frontend("resolve", e, &context))?;
    let program = crate::types::check(resolved).map_err(|e| frontend("type", e, &context))?;
    let (cleanup_sites, outcomes) = crate::ownership::evidence_bounded(&program, &context)
        .map_err(|e| frontend("ownership", e, &context))?;
    if let Some(e) = context.failure.get() {
        return Err(e.into());
    }
    crate::effects::calls::check(&program).map_err(|e| frontend("effects", e, &context))?;
    crate::effects::custody::check(&program, &cleanup_sites)
        .map_err(|e| frontend("effects", e, &context))?;
    seal(program, limits, cleanup_sites, outcomes)
}
impl From<hir::resources::PlanError> for ExecutionError {
    fn from(error: hir::resources::PlanError) -> Self {
        use hir::resources::PlanError;
        match error {
            PlanError::Integrity(m) => Self::integrity(m),
            PlanError::Limit(m) => Self::new(
                ErrorKind::ConfiguredLimit(if m == "cleanup type graph depth exceeded" {
                    "cleanup type graph depth"
                } else {
                    m
                }),
                START,
            ),
            PlanError::Host(m) => Self::new(ErrorKind::Host(m), START),
        }
    }
}
fn seal(
    program: hir::Program,
    limits: AdmissionLimits,
    cleanup_sites: Vec<crate::effects::custody::Site>,
    outcomes: Vec<crate::ownership::outcomes::Witness>,
) -> Result<CheckedExecutable, ExecutionError> {
    seal_in_domain(
        program,
        limits,
        cleanup_sites,
        outcomes,
        domain::Request::Ordinary,
    )
}
fn seal_in_domain(
    program: hir::Program,
    limits: AdmissionLimits,
    cleanup_sites: Vec<crate::effects::custody::Site>,
    outcomes: Vec<crate::ownership::outcomes::Witness>,
    request: domain::Request,
) -> Result<CheckedExecutable, ExecutionError> {
    let (mut mir, origins) = mir::lower_with_origins(&program).map_err(ExecutionError::from)?;
    mir::attach_cleanup_effects(&mut mir, &cleanup_sites).map_err(ExecutionError::from)?;
    let trusted = crate::trusted::approval::Approved::derive(&program, &mir, limits)?;
    let cleanup_plans = if program.resources.leaves.is_empty() {
        Vec::new()
    } else {
        program
            .resources
            .plans(&program)
            .map_err(ExecutionError::from)?
    };
    let cleanup_effects = program
        .resources
        .summaries(&program, &cleanup_plans)
        .map_err(ExecutionError::from)?;
    let domain = domain::Domain::admit(
        &program,
        &mir,
        &cleanup_plans,
        &cleanup_effects,
        limits,
        request,
    )?;
    let result_plans = program
        .variant_plans(&cleanup_plans, &cleanup_effects)
        .map_err(ExecutionError::from)?;
    #[cfg(test)]
    if !outcomes.is_empty() && crate::ownership::outcomes::allocation_fails() {
        return Err(ExecutionError::new(
            ErrorKind::Host("immutable Result witness allocation"),
            START,
        ));
    }
    let mut approved_outcomes = Vec::new();
    approved_outcomes
        .try_reserve_exact(outcomes.len())
        .map_err(|_| {
            ExecutionError::new(
                ErrorKind::Host("immutable Result witness allocation"),
                START,
            )
        })?;
    approved_outcomes.extend_from_slice(&outcomes);
    let approval = Rc::new(Approval {
        schema: contract::Schema::V1,
        schedule: contract::Schedule::reference(),
        outcomes: approved_outcomes,
        program: program.clone(),
        cleanup_sites,
        domain: domain.clone(),
        trusted: trusted.clone(),
    });
    let executable = CheckedExecutable {
        outcomes,
        program,
        mir,
        origins,
        mir_approval: approval.clone(),
        origin_approval: approval.clone(),
        approval,
        limits,
        cleanup_plans,
        cleanup_effects,
        result_plans,
        domain,
        trusted,
    };
    executable.validate()?;
    Ok(executable)
}
fn frontend(
    phase: &'static str,
    errors: Vec<crate::diagnostics::Diagnostic>,
    context: &Context,
) -> ExecutionError {
    if let Some(e) = context.failure.get() {
        return e.into();
    }
    match errors.into_iter().next() {
        Some(e) => ExecutionError::new(
            ErrorKind::Frontend {
                phase,
                message: e.message,
            },
            e.span,
        ),
        None => ExecutionError::integrity("frontend returned no diagnostic"),
    }
}

/// Reads at most source_bytes + one lookahead byte; oversized input is never
/// accumulated. The fixed read buffer also bounds rejected-input housekeeping.
pub fn prepare_execution_from_reader(
    input: impl Read,
    limits: AdmissionLimits,
) -> Result<CheckedExecutable, ExecutionError> {
    prepare_from_reader(input, limits, None)
}

/// The same bounded input path with one explicit immutable profile selection.
pub fn prepare_execution_from_reader_with_profile(
    input: impl Read,
    limits: AdmissionLimits,
    selection: &crate::trusted::Selection,
) -> Result<CheckedExecutable, ExecutionError> {
    prepare_from_reader(input, limits, Some(selection))
}
fn prepare_from_reader(
    mut input: impl Read,
    limits: AdmissionLimits,
    selection: Option<&crate::trusted::Selection>,
) -> Result<CheckedExecutable, ExecutionError> {
    limits.validate()?;
    let mut source = Vec::new();
    let mut buffer = [0u8; 4096];
    loop {
        let available = limits
            .source_bytes
            .checked_sub(source.len())
            .ok_or_else(|| ExecutionError::integrity("input accounting"))?;
        let count = input
            .read(&mut buffer[..available.saturating_add(1).min(4096)])
            .map_err(|_| ExecutionError::new(ErrorKind::Host("source input"), START))?;
        if count == 0 {
            break;
        }
        if count > available {
            return Err(ResourceFailure::limit("source bytes", START).into());
        }
        let needed = source
            .len()
            .checked_add(count)
            .and_then(|n| n.checked_mul(2));
        if needed.map_or(true, |n| n > limits.working_storage) {
            return Err(ResourceFailure::limit("source storage", START).into());
        }
        source
            .try_reserve_exact(count)
            .map_err(|_| ExecutionError::new(ErrorKind::Host("source allocation"), START))?;
        source.extend_from_slice(&buffer[..count]);
    }
    let source = std::str::from_utf8(&source)
        .map_err(|_| ExecutionError::new(ErrorKind::Host("source UTF-8 decoding"), START))?;
    match selection {
        Some(selection) => prepare_execution_with_profile(source, limits, selection),
        None => prepare_execution(source, limits),
    }
}

pub fn execute(
    executable: &CheckedExecutable,
    entry: FunctionId,
    limits: ExecutionLimits,
) -> Result<ExecutionResult, ExecutionError> {
    checked_entry(executable, entry, limits)?;
    if executable.program.trusted.is_some() {
        return Err(ExecutionError::integrity(
            "selected profile execution requires persistent session custody",
        ));
    }
    machine::Machine::new(executable, limits).run(entry)
}
fn checked_entry(
    executable: &CheckedExecutable,
    entry: FunctionId,
    limits: ExecutionLimits,
) -> Result<(), ExecutionError> {
    limits.validate()?;
    executable.validate()?;
    let f = executable
        .program
        .functions()
        .get(entry.0)
        .filter(|f| f.id() == entry)
        .and_then(|f| f.as_ordinary())
        .ok_or_else(|| ExecutionError::integrity("selected entry is not canonical ordinary"))?;
    if !f.parameters.is_empty() {
        return Err(ExecutionError::new(
            ErrorKind::Integrity("selected entry must have zero parameters"),
            f.span,
        ));
    }
    if executable
        .trusted
        .as_ref()
        .is_some_and(|t| !t.roots.contains(&entry))
    {
        return Err(ExecutionError::integrity(
            "selected entry is outside the sealed trusted roots",
        ));
    }
    if matches!(
        f.return_type,
        Some(ValueType::SharedRef(_) | ValueType::MutableRef(_))
    ) {
        return Err(ExecutionError::new(
            ErrorKind::Integrity("reference result cannot escape"),
            f.span,
        ));
    }
    if let Some(ValueType::Record(id)) = f.return_type {
        if executable
            .cleanup_plans
            .get(id.0)
            .is_some_and(Option::is_some)
        {
            return Err(ExecutionError::integrity(
                "live resource result cannot cross public boundary",
            ));
        }
    }
    if matches!(f.return_type, Some(ValueType::Resource(_))) {
        return Err(ExecutionError::integrity(
            "public live-resource egress is excluded",
        ));
    }
    if let Some(ValueType::Result(id)) = f.return_type {
        if executable
            .result_plans
            .get(id.0)
            .is_some_and(hir::results::VariantPlan::bearing)
        {
            return Err(ExecutionError::integrity(
                "live resource Result cannot cross public boundary",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod completion_authority_tests;
#[cfg(test)]
mod result_admission_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod trusted_authority_tests;
mod validate;

#[cfg(test)]
fn prepare_synthetic(
    program: hir::Program,
    limits: AdmissionLimits,
) -> Result<CheckedExecutable, ExecutionError> {
    // Predecessor state-injection fixtures are explicitly protocol evidence.
    prepare_in_domain(program, limits, domain::Request::Primitive)
}
#[cfg(test)]
pub(crate) fn prepare_ordinary_synthetic(
    program: hir::Program,
    limits: AdmissionLimits,
) -> Result<CheckedExecutable, ExecutionError> {
    prepare_in_domain(program, limits, domain::Request::Ordinary)
}
#[cfg(test)]
fn prepare_in_domain(
    program: hir::Program,
    limits: AdmissionLimits,
    domain: domain::Request,
) -> Result<CheckedExecutable, ExecutionError> {
    limits.validate()?;
    let context = Context::new(limits);
    context.reserve_frontend(
        program.functions.len()
            + program.fields.len()
            + program.locals.len()
            + program.parameters.len()
            + 16,
        0,
    )?;
    validate::program_shape(&program, &context)?;
    let (cleanup_sites, outcomes) = crate::ownership::evidence_bounded(&program, &context)
        .map_err(|e| frontend("ownership", e, &context))?;
    if let Some(e) = context.failure.get() {
        return Err(e.into());
    }
    crate::effects::calls::check(&program).map_err(|e| frontend("effects", e, &context))?;
    crate::effects::custody::check(&program, &cleanup_sites)
        .map_err(|e| frontend("effects", e, &context))?;
    seal_in_domain(program, limits, cleanup_sites, outcomes, domain)
}

#[cfg(test)]
mod effect_authority_tests {
    use super::*;
    use crate::effects::{Effect, EffectSet};

    fn cleanup_fixture(source: &str) -> hir::Program {
        let mut acquire = EffectSet::EMPTY;
        acquire.insert(Effect::Io);
        let mut release = EffectSet::EMPTY;
        release.insert(Effect::Filesystem);
        hir::resources::fixture_effects(source, acquire, release)
    }

    #[test]
    fn protected_return_owner_cannot_gain_cleanup_authority_from_an_unused_superset() {
        let mut e = prepare_ordinary_synthetic(cleanup_fixture("fn make()->Leaf effects io,filesystem{let outgoing=acquire();return outgoing;} fn demo() effects io,filesystem{let received=make();}"), AdmissionLimits::default()).unwrap();
        let f = e.program.functions()[2].as_ordinary().unwrap();
        let local = e
            .program
            .locals()
            .iter()
            .find(|l| l.function == f.id)
            .unwrap()
            .id;
        let hir::ValueStatement::Return { span, .. } = f.body.last().unwrap() else {
            unreachable!()
        };
        let mut effects = EffectSet::EMPTY;
        effects.insert(Effect::Filesystem);
        assert!(e.mir.functions()[2].cleanup_effect_sites.is_empty());
        e.mir.test_functions_mut()[2]
            .cleanup_effect_sites
            .push(crate::effects::custody::Site {
                function: f.id,
                holder: crate::effects::custody::Holder::Place(hir::Place::Local(local)),
                plan: hir::resources::CleanupType::Record(hir::RecordId(0)),
                effects,
                kind: crate::effects::custody::Kind::Return,
                origin: *span,
            });
        assert!(e.validate().is_err());
    }

    #[test]
    fn cleanup_witness_storage_neighbors_and_host_failure_never_grant_authority() {
        let p = cleanup_fixture("fn demo() effects io,filesystem{let x=acquire();}");
        let prepare = |bytes| {
            prepare_ordinary_synthetic(
                p.clone(),
                AdmissionLimits {
                    working_storage: bytes,
                    ..AdmissionLimits::default()
                },
            )
        };
        let mut low = 1;
        let mut high = AdmissionLimits::default().working_storage;
        assert!(prepare(high).is_ok());
        while low < high {
            let middle = low + (high - low) / 2;
            if prepare(middle).is_ok() {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        assert!(prepare(low).is_ok());
        let error = prepare(low - 1).err().unwrap();
        assert!(matches!(error.kind, ErrorKind::ConfiguredLimit(_)));
        struct Restore;
        impl Drop for Restore {
            fn drop(&mut self) {
                crate::effects::custody::FAIL_WITNESS_ALLOCATION.with(|f| f.set(false));
            }
        }
        let _restore = Restore;
        crate::effects::custody::FAIL_WITNESS_ALLOCATION.with(|f| f.set(true));
        let error = prepare(high).err().unwrap();
        assert_eq!(
            error.kind,
            ErrorKind::Host("cleanup effect witness allocation")
        );
    }

    #[test]
    fn every_ordinary_exit_and_displacement_requires_its_cleanup_effect() {
        use crate::effects::custody::Kind;
        for (result, body, kind) in [
            ("", "let x=acquire();", Kind::Completion),
            ("", "let x=acquire();return;", Kind::BareReturn),
            ("->bool", "let x=acquire();return true;", Kind::Return),
            ("", "if true{let x=acquire();}", Kind::Fallthrough),
            (
                "",
                "let mut flag=true;while flag{let x=acquire();flag=false;}",
                Kind::Iteration,
            ),
            ("", "while true{let x=acquire();break;}", Kind::Break),
            (
                "",
                "let mut flag=true;while flag{let x=acquire();flag=false;continue;}",
                Kind::Continue,
            ),
            ("", "let mut x=acquire();x=acquire();", Kind::Replacement),
        ] {
            let bad = cleanup_fixture(&format!("fn demo(){result} effects io{{{body}}}"));
            let error = prepare_ordinary_synthetic(bad, AdmissionLimits::default())
                .err()
                .unwrap();
            assert!(
                matches!(
                    error.kind,
                    ErrorKind::Frontend {
                        phase: "effects",
                        ..
                    }
                ),
                "{error}"
            );
            assert!(error.to_string().contains("filesystem"));
            let good = cleanup_fixture(&format!(
                "fn demo(){result} effects io,filesystem{{{body}}}"
            ));
            let e = prepare_ordinary_synthetic(good, AdmissionLimits::default()).unwrap();
            assert!(e.approval.cleanup_sites.iter().any(|s| s.kind == kind));
            assert!(execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).is_ok());
        }
    }

    #[test]
    fn pass_through_both_path_transfer_and_return_protection_do_not_charge_cleanup() {
        let source = "fn pass(x:Leaf)->Leaf{return x;} fn branches(x:Leaf,flag:bool)->Leaf{if flag{return x;}else{return x;}} fn make()->Leaf effects io{return acquire();} fn mixed(x:Leaf,flag:bool)->Leaf effects io,filesystem{if flag{return x;}let other=acquire();return other;} fn consume(x:Leaf) effects filesystem{} fn both(x:Leaf,flag:bool) effects filesystem{if flag{consume(x);}else{consume(x);}} fn maybe(x:Leaf,flag:bool) effects filesystem{if flag{consume(x);}} fn demo() effects io,filesystem{let a=acquire();let b=pass(a);let c=branches(b,false);let d=mixed(c,true);let e=make();both(acquire(),true);maybe(acquire(),false);}";
        let e = prepare_ordinary_synthetic(cleanup_fixture(source), AdmissionLimits::default())
            .unwrap();
        for name in ["pass", "branches", "make", "both"] {
            let id = e
                .program
                .functions()
                .iter()
                .find(|f| f.name() == name)
                .unwrap()
                .id();
            assert!(
                !e.approval.cleanup_sites.iter().any(|s| s.function == id),
                "{name}"
            );
        }
        for name in ["mixed", "maybe"] {
            let f = e
                .program
                .functions()
                .iter()
                .find(|f| f.name() == name)
                .unwrap()
                .as_ordinary()
                .unwrap();
            assert!(e
                .approval
                .cleanup_sites
                .iter()
                .any(|s| s.function == f.id && s.holder == hir::Place::Parameter(f.parameters[0])));
        }
        assert!(execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).is_ok());
    }

    #[test]
    fn short_circuit_cleanup_custody_keeps_skip_alternatives_separate_from_usability() {
        let source = "fn take(x:Leaf)->bool effects filesystem{return true;} fn skipped(x:Leaf) effects filesystem{let seen=false&&take(x);} fn direct(x:Leaf) effects filesystem{let seen=take(x);} fn demo() effects io,filesystem{skipped(acquire());direct(acquire());}";
        let e = prepare_ordinary_synthetic(cleanup_fixture(source), AdmissionLimits::default())
            .unwrap();
        let find = |name| {
            e.program
                .functions()
                .iter()
                .find(|f| f.name() == name)
                .unwrap()
                .id()
        };
        assert!(e
            .approval
            .cleanup_sites
            .iter()
            .any(|s| s.function == find("skipped")));
        assert!(!e
            .approval
            .cleanup_sites
            .iter()
            .any(|s| s.function == find("direct")));
        assert!(execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).is_ok());
    }

    #[test]
    fn cleanup_witness_site_function_owner_plan_summary_and_origin_mutations_reject() {
        for mutation in 0..11 {
            let mut e = prepare_ordinary_synthetic(
                cleanup_fixture(
                    "fn demo() effects io,filesystem{let a=acquire();let b=acquire();}",
                ),
                AdmissionLimits::default(),
            )
            .unwrap();
            let sites = &mut e
                .mir
                .test_functions_mut()
                .last_mut()
                .unwrap()
                .cleanup_effect_sites;
            assert_eq!(sites.len(), 2);
            match mutation {
                0 => {
                    sites.pop();
                }
                1 => sites.push(sites[0].clone()),
                2 => sites.swap(0, 1),
                3 => sites[0].function = FunctionId(0),
                4 => {
                    sites[0].holder =
                        crate::effects::custody::Holder::Place(hir::Place::Local(hir::LocalId(999)))
                }
                5 => sites[0].plan = hir::resources::CleanupType::Record(hir::RecordId(1)),
                6 => sites[0].effects = EffectSet::EMPTY,
                7 => sites[0].origin.start += 1,
                8 => {
                    sites[0].effects.insert(Effect::Io);
                }
                9 => sites[0].kind = crate::effects::custody::Kind::Replacement,
                _ => sites[0].holder = sites[1].holder,
            }
            assert!(e.validate().is_err(), "mutation {mutation}");
        }
    }

    #[test]
    fn cached_cleanup_effect_summaries_are_bound_to_canonical_derivation() {
        for mutation in 0..5 {
            let mut acquire = EffectSet::EMPTY;
            acquire.insert(Effect::Io);
            let mut release = EffectSet::EMPTY;
            release.insert(Effect::Filesystem);
            release.insert(Effect::Blocking);
            let p = hir::resources::fixture_effects(
                "record Plain{x:R} fn demo() effects io,filesystem,blocking {let x=acquire();}",
                acquire,
                release,
            );
            let mut e = prepare_ordinary_synthetic(p, AdmissionLimits::default()).unwrap();
            assert_eq!(e.cleanup_effects, vec![release, release, EffectSet::EMPTY]);
            assert!(e.validate().is_ok());
            match mutation {
                0 => e.cleanup_effects[0] = EffectSet::EMPTY,
                1 => {
                    e.cleanup_effects[1].insert(Effect::Clock);
                }
                2 => {
                    e.cleanup_effects.pop();
                }
                3 => e.cleanup_effects.push(release),
                _ => e.cleanup_effects.swap(0, 2),
            }
            assert!(e.validate().is_err(), "mutation {mutation}");
        }
    }

    #[test]
    fn direct_effect_sites_reject_independent_authority_mutations() {
        for mutation in 0..9 {
            let mut e = prepare_execution(
                "fn leaf() effects clock {} fn demo() effects io,clock {leaf();leaf();}",
                AdmissionLimits::default(),
            )
            .unwrap();
            let sites = &mut e.mir.test_functions_mut()[1].direct_effects;
            match mutation {
                0 => {
                    sites.pop();
                }
                1 => sites.push(sites[0].clone()),
                2 => sites.swap(0, 1),
                3 => sites[0].function = FunctionId(0),
                4 => sites[0].effects = EffectSet::EMPTY,
                5 => {
                    sites[0].effects.insert(Effect::Io);
                }
                6 => sites[0].origin.start += 1,
                7 => sites[0].ordinal += 10,
                _ => sites[0].callee = crate::effects::calls::Callee::Source(FunctionId(1)),
            }
            assert!(e.validate().is_err(), "mutation {mutation}");
        }
    }
}

#[cfg(test)]
mod temporary_authority_tests {
    use super::*;
    use crate::mir::temporaries::{Destination, RegionId, SiteId};

    fn artifact() -> CheckedExecutable {
        prepare_synthetic(
            hir::resources::fixture_with(
                "fn take(a:Leaf,b:Leaf){}fn demo(){let a=acquire();let b=acquire();take(a,b);}",
            ),
            AdmissionLimits::default(),
        )
        .unwrap()
    }

    #[test]
    fn each_independent_temporary_authority_mutation_is_rejected() {
        for fault in 0..37 {
            let mut e = artifact();
            let f = e.mir.test_functions_mut().last_mut().unwrap();
            if fault == 0 {
                f.temporaries = None;
            } else {
                let flow = f.temporaries.as_mut().unwrap();
                match fault {
                    1 => flow.regions[0].roots.clear(),
                    2 => {
                        let root = flow.regions[0].roots[0];
                        flow.regions[0].roots.push(root);
                    }
                    3 => flow.sites[0].id = SiteId(usize::MAX),
                    4 => {
                        flow.sites[0].plan =
                            Some(hir::resources::CleanupType::Record(hir::RecordId(999)))
                    }
                    5 => flow.sites[0].destination = Destination::Parameter(hir::ParameterId(999)),
                    6 => flow.regions[0].parent = Some(RegionId(0)),
                    7 => flow.regions[0].origin.kind = mir::OriginKind::FunctionBoundary,
                    8 => flow.sites[0].transfer.kind = mir::OriginKind::Statement,
                    9 => flow.sites[0].cleanup.kind = mir::OriginKind::Statement,
                    10 => flow.regions.last_mut().unwrap().roots.swap(0, 1),
                    11 => {
                        flow.points.remove(0);
                    }
                    12 => flow.points.push(flow.points[0].clone()),
                    13 => flow.points[0].block = mir::BasicBlockId(usize::MAX),
                    14 => flow.points[0].statement = usize::MAX,
                    15 => flow.points[0].begin = false,
                    16 => flow.points[0].complete = false,
                    17 => flow.regions[0].protect_result = false,
                    18 => flow.regions[0].retire_before_publication = false,
                    19 => flow.regions[0].destination = Destination::Return,
                    20 => flow.sites.push(flow.sites[0].clone()),
                    21 => flow.regions.push(flow.regions[0].clone()),
                    22 => flow.sites[0].children.push(SiteId(0)),
                    23 => flow.regions[0].evaluation = Some(1),
                    24 => flow.sites[0].origin.span.line = 0,
                    25 => flow.sites[0].plan = None,
                    26 => flow.sites[0].cleanup.span = flow.sites[1].cleanup.span,
                    27 => flow.sites[0].transfer.span = flow.sites[1].transfer.span,
                    28 => flow.regions[0].publication.span = flow.regions[1].publication.span,
                    29 => flow.points[0].region = RegionId(1),
                    30 => flow.regions[1].parent = Some(RegionId(0)),
                    31 => flow.points[0].site = Some(SiteId(999)),
                    32 => flow.sites[0].ty = hir::ExprType::Bool,
                    33 => flow.regions[0].roots[0] = SiteId(999),
                    34 => flow.regions[0].retirement.kind = mir::OriginKind::Statement,
                    35 => flow.regions[0].publication.kind = mir::OriginKind::Statement,
                    36 => flow.sites[0].children.push(SiteId(1)),
                    _ => unreachable!(),
                }
            }
            assert!(
                matches!(e.validate().unwrap_err().kind, ErrorKind::Integrity(_)),
                "fault {fault}"
            );
        }
    }

    #[test]
    fn initializer_points_begin_once_and_complete_only_on_selected_leaf() {
        let e = prepare_synthetic(
            hir::resources::fixture_with(
                "fn demo(){let a=acquire();let chosen=if true{a}else{acquire()};}",
            ),
            AdmissionLimits::default(),
        )
        .unwrap();
        let flow = e
            .mir
            .functions()
            .last()
            .unwrap()
            .temporaries
            .as_ref()
            .unwrap();
        let points: Vec<_> = flow
            .points
            .iter()
            .filter(|p| p.region == RegionId(1))
            .collect();
        assert_eq!(points.len(), 3);
        assert_eq!(points.iter().filter(|p| p.begin).count(), 1);
        assert_eq!(points.iter().filter(|p| p.complete).count(), 2);
        assert!(points[0].begin && !points[0].complete);
        assert!(points[1..].iter().all(|p| !p.begin && p.complete));
        assert_ne!(points[0].block, points[1].block);
    }

    #[test]
    fn source_evaluation_ordinals_include_resource_free_predecessors() {
        let e = prepare_synthetic(
            hir::resources::fixture_with(
                "fn demo(){let flag=true;let a=acquire();let unused=false;release(a);}",
            ),
            AdmissionLimits::default(),
        )
        .unwrap();
        let f = e.mir.functions().last().unwrap();
        let flow = f.temporaries.as_ref().unwrap();
        assert_eq!(
            flow.regions
                .iter()
                .map(|r| r.evaluation)
                .collect::<Vec<_>>(),
            vec![Some(1), Some(3)]
        );
        assert_eq!(
            flow.points.iter().map(|p| p.statement).collect::<Vec<_>>(),
            vec![1, 3]
        );
    }

    #[test]
    fn each_temporary_authority_allocation_failure_preserves_host_classification() {
        let p = hir::resources::fixture_with(
            "fn take(a:Leaf,b:Leaf){}fn demo(){take(acquire(),acquire());}",
        );
        let mut success = None;
        for cut in 0..512 {
            mir::temporaries::FAIL_ALLOCATION.with(|f| f.set(Some(cut)));
            let result = prepare_synthetic(p.clone(), AdmissionLimits::default());
            mir::temporaries::FAIL_ALLOCATION.with(|f| f.set(None));
            match result {
                Ok(_) => {
                    success = Some(cut);
                    break;
                }
                Err(e) => assert!(matches!(e.kind, ErrorKind::Host(_)), "cut {cut}: {e:?}"),
            }
        }
        assert!(success.unwrap() > 10);
    }

    #[test]
    fn exact_admission_work_and_storage_neighbors_keep_typed_limits() {
        let p = hir::resources::fixture_with(
            "fn take(a:Leaf,b:Leaf){}fn demo(){take(acquire(),acquire());}",
        );
        for work in [false, true] {
            let default = AdmissionLimits::default();
            let mut low = 1u64;
            let mut high = if work {
                default.frontend_work
            } else {
                default.working_storage as u64
            };
            while low < high {
                let mid = low + (high - low) / 2;
                let mut limits = default;
                if work {
                    limits.frontend_work = mid;
                } else {
                    limits.working_storage = mid as usize;
                }
                match prepare_synthetic(p.clone(), limits) {
                    Ok(_) => high = mid,
                    Err(e) => {
                        assert!(matches!(e.kind, ErrorKind::ConfiguredLimit(_)), "{e:?}");
                        low = mid + 1;
                    }
                }
            }
            let mut limits = default;
            if work {
                limits.frontend_work = low;
            } else {
                limits.working_storage = low as usize;
            }
            assert!(prepare_synthetic(p.clone(), limits).is_ok());
            if work {
                limits.frontend_work -= 1;
            } else {
                limits.working_storage -= 1;
            }
            assert!(matches!(
                prepare_synthetic(p.clone(), limits).err().unwrap().kind,
                ErrorKind::ConfiguredLimit(_)
            ));
        }
    }
}

#[cfg(test)]
mod result_authority_tests {
    use super::*;
    const SOURCE: &str = "fn make()->Result<success:bool,failure:bool>{return Ok(true);}fn demo()->bool{let pending=make();match pending{Ok(value)=>{return value;}Err(problem)=>{return problem;}}}";
    #[test]
    fn independent_result_hir_witness_plan_and_origin_mutations_reject_opaque_approval() {
        for mutation in 0..19 {
            let mut e = prepare_execution(SOURCE, AdmissionLimits::default()).unwrap();
            match mutation {
                0 => e.outcomes.clear(),
                1 => {
                    let last = e.outcomes.last().unwrap().clone();
                    e.outcomes.push(last);
                }
                2 => e.outcomes.swap(0, 1),
                3 => e.result_plans.clear(),
                4 => e.result_plans[0].result = hir::ResultTypeId(99),
                5 => {
                    e.result_plans[0].success =
                        Some(hir::results::VariantChild::Result(hir::ResultTypeId(0)))
                }
                6 => e.result_plans[0].resource_bearing = true,
                7 => {
                    e.result_plans[0].effects.insert(crate::effects::Effect::Io);
                }
                8 => e.origins[1][0].span.start += 1,
                9 => {
                    let hir::Function::Ordinary(f) = &mut e.program.functions[0] else {
                        panic!()
                    };
                    let hir::ValueStatement::Return { value, .. } = &mut f.body[0] else {
                        panic!()
                    };
                    let hir::ExprKind::ResultConstruct { tag, .. } = &mut value.kind else {
                        panic!()
                    };
                    *tag = hir::ResultTag::Error;
                }
                10..=17 => {
                    let hir::Function::Ordinary(f) = &mut e.program.functions[1] else {
                        panic!()
                    };
                    let hir::ValueStatement::Match { arms, .. } = &mut f.body[1] else {
                        panic!()
                    };
                    match mutation {
                        10 => {
                            arms.pop();
                        }
                        11 => arms.push(arms[0].clone()),
                        12 => arms.swap(0, 1),
                        13 => arms[0].binding = arms[1].binding,
                        14 => arms[0].binding = hir::LocalId(99),
                        15 => arms[0].pattern_span.start += 1,
                        16 => arms[0].span.end += 1,
                        17 => arms[0].tag = hir::ResultTag::Error,
                        _ => unreachable!(),
                    }
                }
                18 => e.program.locals[1].function = hir::FunctionId(0),
                _ => unreachable!(),
            }
            assert!(e.validate().is_err(), "mutation {mutation}");
            assert!(
                execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).is_err(),
                "mutation {mutation}"
            );
        }
    }
    #[test]
    fn independent_mir_result_dispatch_binding_transfer_scope_and_temporary_mutations_reject() {
        for mutation in 0..19 {
            let mut e = prepare_execution(SOURCE, AdmissionLimits::default()).unwrap();
            mir::mutate_result_mir(&mut e.mir, mutation);
            assert!(e.validate().is_err(), "mutation {mutation}");
            assert!(execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).is_err());
        }
    }
    #[test]
    fn result_outcome_admission_has_exact_storage_and_work_neighbors() {
        for work in [false, true] {
            let mut low = 1u64;
            let mut high = if work {
                AdmissionLimits::default().frontend_work
            } else {
                AdmissionLimits::default().working_storage as u64
            };
            let prepare = |bound| {
                let mut limits = AdmissionLimits::default();
                if work {
                    limits.frontend_work = bound;
                } else {
                    limits.working_storage = bound as usize;
                }
                prepare_execution(SOURCE, limits)
            };
            assert!(prepare(high).is_ok());
            while low < high {
                let mid = low + (high - low) / 2;
                if prepare(mid).is_ok() {
                    high = mid;
                } else {
                    low = mid + 1;
                }
            }
            assert!(prepare(low).is_ok());
            let error = prepare(low - 1).err().unwrap();
            assert!(
                matches!(
                    error.kind,
                    ErrorKind::ConfiguredLimit(
                        "Result outcome storage reservation" | "Result outcome work reservation"
                    )
                ),
                "{work}: {error:?}"
            );
        }
    }
    #[test]
    fn detected_result_witness_and_variant_plan_allocation_failures_publish_no_artifact() {
        for witness in [true, false] {
            let mut succeeded = false;
            for cut in 0..256 {
                if witness {
                    crate::ownership::outcomes::FAIL_ALLOCATION.with(|f| f.set(Some(cut)));
                } else {
                    hir::resources::FAIL_PLAN_ALLOCATION.with(|f| f.set(Some(cut)));
                }
                let outcome = prepare_execution(SOURCE, AdmissionLimits::default());
                crate::ownership::outcomes::FAIL_ALLOCATION.with(|f| f.set(None));
                hir::resources::FAIL_PLAN_ALLOCATION.with(|f| f.set(None));
                match outcome {
                    Ok(_) => {
                        succeeded = true;
                        break;
                    }
                    Err(error) => assert!(
                        matches!(error.kind, ErrorKind::Host(_)),
                        "{witness}/{cut}: {error:?}"
                    ),
                }
            }
            assert!(succeeded);
        }
    }
    #[test]
    fn result_rendering_rejects_forged_flat_payloads_and_enforces_output_limits() {
        let e = prepare_execution(
            "fn demo()->Result<success:bool,failure:bool>{return Err(false);}",
            AdmissionLimits::default(),
        )
        .unwrap();
        let actual = execute(&e, e.entry("demo").unwrap(), ExecutionLimits::default()).unwrap();
        for limit in 1..18 {
            assert!(e.render_result(&actual, limit).is_err());
        }
        assert_eq!(e.render_result(&actual, 18).unwrap(), "Error(Bool(false))");
        assert!(render::with_allocation_failure(|| e.render_result(&actual, 100)).is_err());
        for layers in [
            vec![],
            vec![ResultLayer {
                ty: hir::ResultTypeId(0),
                tag: hir::ResultTag::Error,
                payload: None,
            }],
            vec![ResultLayer {
                ty: hir::ResultTypeId(99),
                tag: hir::ResultTag::Ok,
                payload: Some(ResultPayload::Bool(true)),
            }],
            vec![
                ResultLayer {
                    ty: hir::ResultTypeId(0),
                    tag: hir::ResultTag::Ok,
                    payload: Some(ResultPayload::Bool(true))
                };
                33
            ],
        ] {
            assert!(e
                .render_result(&ExecutionResult::Value(RuntimeValue::Result(layers)), 100)
                .is_err());
        }
    }
}
