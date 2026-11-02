//! Separately admitted, non-owning conformance evidence. This module does not
//! execute source or grant an operation. The ordinary driver remains unchanged.
use super::*;
use std::cell::Ref;
#[cfg(test)]
thread_local! {
    static SWAP_PARAMETERS:std::cell::Cell<Option<(FunctionId,Place,Place)>>=const {std::cell::Cell::new(None)};
}
mod authority;
mod compare;
pub(super) mod replay;
#[cfg(test)]
mod sentinel_tests;
pub use compare::{compare, Comparison, ComparisonLimits};

/// All limits belong to observation, never to ordinary fuel or storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub records: usize,
    pub work: usize,
    pub storage: usize,
    pub lineage: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            records: 32_768,
            work: 1_000_000,
            storage: 16_777_216,
            lineage: 128,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Incomplete {
    Admission,
    Allocation,
    Records,
    Work,
    UncoveredState,
    InvalidState,
    Pending,
    Lineage,
    Authority,
    UnmatchedPhysicalFailure,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cut {
    Start,
    Fuel,
    Storage(usize),
    Depth,
    ExpressionDepth(usize),
    Transition,
    Final,
}

// A flat vocabulary bounds traversal, comparison and destruction. No record
// contains a live payload, heap-owning recursive value or authority handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Record {
    Cut(Cut, u64, usize, Span, Option<Span>, bool),
    Accounting(ExecutionLimits, Option<usize>),
    Frame(u64, FunctionId, BasicBlockId, usize, Option<u64>, Span),
    FrameStorage(usize),
    Slot(Place, u8),
    Operand,
    Bool(bool),
    Range(hir::RangeTypeId, i64),
    Raw(i64),
    Reference(BorrowKind, ReferentType, u64, Place),
    NoValue,
    End,
    Delivered,
    Stop(u8, Span, Option<Span>),
    Trap(hir::RangeTypeId, i64, i64, i128),
    Limit(&'static str),
    Meta(Part, [u64; 8]),
    Origin(Span),
    Payload,
    Absent,
    TemporaryBegin(u64, u64, crate::mir::temporaries::RegionId, Option<u64>),
    TemporaryRegister(u64, usize, crate::mir::temporaries::SiteId),
    TemporaryTransfer(u64, usize, crate::mir::temporaries::Destination),
    TemporaryCleanup(u64, usize),
    TemporaryRetire(u64),
    ResourceEvent(u8, u64),
    TrustedEvent(trusted::Event),
    Continuation(u64),
    CallSite(Span),
    Expression(usize),
    ExprType(ExprType),
    Binary(BinaryOp),
    Destination(u64, Place, Option<(hir::RecordId, hir::FieldId)>),
    OriginKind(OriginKind),
    Inventory(usize, usize, usize, usize, u8),
    Executing,
    Publication(usize, bool),
    Transit(usize, bool),
    CleanupLeaf(u64, bool),
    LogicalActivation(usize, u64, FunctionId, Option<usize>, Span, u64),
    Control(replay::Address, u8),
    ControlledResponse(replay::Address, replay::Response),
    Releasing(replay::Address, u8, u64),
}
/// Fixed-shape relational rows. Positions are defined by the named row schema,
/// not by a Rust struct layout, address or renderer representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Part {
    Record,
    Field,
    ResultLayer,
    ResultChain,
    ResultHandle,
    TemporaryHandle,
    SensorHandle,
    Leaf,
    ResultCell,
    ResultMatch,
    ResultMatchFrame,
    InertRecord,
    TemporaryActivation,
    TemporaryOwner,
    State,
    ResourceFrame,
    Scope,
    Generation,
    GenerationSlot,
    PendingRecord,
    PendingField,
    Replacement,
    ReplacementState,
    Holder,
    Buffers,
    TrustedState,
    Token,
    Receipt,
    PendingReceipt,
    Production,
    Completion,
    CompletionOutcome,
    ResultTicket,
    TrustedRetained,
    Task,
    Initializer,
}
pub(super) const ABSENT: u64 = u64::MAX;
pub(super) struct Capture {
    pub(super) records: Vec<Record>,
    limits: Limits,
    work: usize,
    pub(super) incomplete: Option<Incomplete>,
    finished: bool,
    expressions: Vec<(usize, usize)>,
    snapshots: usize,
    sealed_count: Option<usize>,
    controlled: bool,
}
pub(super) fn compatibility_shapes() -> [usize; 2] {
    [
        size_of::<Record>(),
        size_of::<std::cell::RefCell<Option<Capture>>>(),
    ]
}
pub(super) fn comparison_shapes() -> [usize; 8] {
    compare::compatibility_shapes()
}
impl Capture {
    fn new(limits: Limits, contract: &contract::Contract<'_>) -> Self {
        let mut capture = Self {
            records: Vec::new(),
            limits,
            work: 0,
            incomplete: None,
            finished: false,
            expressions: Vec::new(),
            snapshots: 0,
            sealed_count: None,
            controlled: false,
        };
        let units = contract.schedule();
        let bytes = limits
            .records
            .checked_mul(units.observation[0])
            .and_then(|n| n.checked_add(units.observation[1]));
        let bytes = bytes.and_then(|n| {
            n.checked_add(
                contract
                    .expressions()
                    .len()
                    .checked_mul(units.extraction[2])?,
            )
        });
        if limits.records == 0
            || limits.records > 1_048_576
            || limits.work == 0
            || limits.work > 100_000_000
            || limits.storage > 268_435_456
            || limits.lineage == 0
            || limits.lineage > 256
            || bytes.map_or(true, |n| n > limits.storage)
        {
            capture.incomplete = Some(Incomplete::Admission);
        } else if capture.records.try_reserve_exact(limits.records).is_err()
            || capture
                .expressions
                .try_reserve_exact(contract.expressions().len())
                .is_err()
        {
            capture.incomplete = Some(Incomplete::Allocation);
        } else if contract.expressions().len() > limits.work {
            capture.incomplete = Some(Incomplete::Work);
        } else {
            capture.work = contract.expressions().len();
            capture.expressions.extend(
                contract
                    .expressions()
                    .iter()
                    .map(|(value, ordinal)| (*value as *const TypedExpr as usize, *ordinal)),
            );
        }
        capture
    }
    pub(super) fn fail(&mut self, reason: Incomplete) {
        if self.incomplete.is_none() {
            self.incomplete = Some(reason);
        }
    }
    pub(super) fn push(&mut self, record: Record) {
        if self.incomplete.is_some() {
            return;
        }
        if !self.spend() {
            return;
        }
        if self.records.len() == self.limits.records {
            self.fail(Incomplete::Records);
        } else {
            // Capacity was reserved before execution. This cannot allocate.
            self.records.push(record);
        }
    }
    pub(super) fn spend(&mut self) -> bool {
        if self.incomplete.is_some() {
            return false;
        }
        if self.work == self.limits.work {
            self.fail(Incomplete::Work);
            return false;
        }
        self.work += 1;
        true
    }
    fn spend_many(&mut self, amount: usize) -> bool {
        if self.incomplete.is_some() {
            return false;
        }
        let Some(work) = self
            .work
            .checked_add(amount)
            .filter(|n| *n <= self.limits.work)
        else {
            self.fail(Incomplete::Work);
            return false;
        };
        self.work = work;
        true
    }
    pub(super) fn row(&mut self, part: Part, values: &[u64]) {
        if values.len() > 8 {
            self.fail(Incomplete::InvalidState);
            return;
        }
        let mut data = [ABSENT; 8];
        data[..values.len()].copy_from_slice(values);
        self.push(Record::Meta(part, data));
    }
    fn expression(&mut self, value: &TypedExpr) {
        for index in 0..self.expressions.len() {
            if !self.spend() {
                return;
            }
            let (address, ordinal) = self.expressions[index];
            if address == value as *const TypedExpr as usize {
                self.push(Record::Expression(ordinal));
                return;
            }
        }
        self.fail(Incomplete::InvalidState);
    }
    fn fields(&mut self, fields: &[hir::RecordFieldInit]) {
        for field in fields {
            if !self.spend() {
                return;
            }
            self.row(Part::Initializer, &[field.field.0 as u64]);
            self.expression(&field.value);
        }
        self.push(Record::End);
    }
    fn task(&mut self, task: &Task<'_>) {
        // Copied scheduler origins are private until a logical action reaches
        // its approved gate. Reached cuts carry the actual canonical origin.
        self.push(Record::Continuation(task.frame));
        match &task.action {
            Action::Drive => self.row(Part::Task, &[0]),
            Action::Eval(e, d) => {
                self.row(Part::Task, &[1, *d as u64]);
                self.expression(e);
            }
            Action::SiteEval(e, d, a, s) => {
                self.row(Part::Task, &[2, *d as u64, *a, s.0 as u64]);
                self.expression(e);
            }
            Action::RegisterTemporary(a, s) => self.row(Part::Task, &[3, *a, s.0 as u64]),
            Action::FinishTemporary(a) => self.row(Part::Task, &[4, *a]),
            Action::SiteShortCircuit(op, e, d, a, s) => {
                self.row(Part::Task, &[5, *d as u64, *a, s.0 as u64]);
                self.push(Record::Binary(*op));
                self.expression(e);
            }
            Action::Negate => self.row(Part::Task, &[6]),
            Action::Binary(op, ty) => {
                self.row(Part::Task, &[7]);
                self.push(Record::Binary(*op));
                self.push(Record::ExprType(*ty));
            }
            Action::ShortCircuit(op, e, d) => {
                self.row(Part::Task, &[8, *d as u64]);
                self.push(Record::Binary(*op));
                self.expression(e);
            }
            Action::Record(r, fields) => {
                self.row(Part::Task, &[9, r.0 as u64]);
                self.fields(fields);
            }
            Action::Invoke(f, count) => {
                self.row(Part::Task, &[10, f.0 as u64, *count as u64]);
                self.push(Record::CallSite(
                    task.expression.unwrap_or(task.origin.span),
                ));
            }
            Action::Commit(destination) => {
                self.row(Part::Task, &[11]);
                match destination {
                    Destination::Slot(at) => {
                        self.push(Record::Destination(at.frame, at.place, None))
                    }
                    Destination::Field(at, r, f) => {
                        self.push(Record::Destination(at.frame, at.place, Some((*r, *f))))
                    }
                }
            }
            Action::ReplaceCommit(id) => self.row(Part::Task, &[12, *id]),
            Action::Select(yes, no) => self.row(Part::Task, &[13, yes.0 as u64, no.0 as u64]),
            Action::Return => self.row(Part::Task, &[14]),
            Action::DropNoValue => self.row(Part::Task, &[15]),
            Action::PendingNext(id, fields, index, depth) => {
                self.row(Part::Task, &[16, *id, *index as u64, *depth as u64]);
                self.fields(fields);
            }
            Action::PendingField(id, field) => self.row(Part::Task, &[17, *id, field.0 as u64]),
            Action::ResultRecordNext(id, fields, depth) => {
                self.row(Part::Task, &[18, *id as u64, *depth as u64]);
                self.fields(fields);
            }
            Action::ResultRecordField(id, fields, depth) => {
                self.row(Part::Task, &[19, *id as u64, *depth as u64]);
                self.fields(fields);
            }
            Action::ResultComplete(id) => self.row(Part::Task, &[20, id.0 as u64]),
            Action::ResultSelect(ty, yes, no) => {
                self.row(Part::Task, &[21, ty.0 as u64, yes.0 as u64, no.0 as u64])
            }
            Action::TrustedInvoke(plan, count, activation) => {
                self.row(Part::Task, &[22, *plan as u64, *count as u64, *activation])
            }
            Action::CompletionAttempt => self.row(Part::Task, &[23]),
            Action::CompletionRelease => self.row(Part::Task, &[24]),
            Action::CompletionDeliver => self.row(Part::Task, &[25]),
        }
        self.push(Record::End);
    }
    fn public(&mut self, value: &RuntimeValue) {
        match value {
            RuntimeValue::Bool(v) => self.push(Record::Bool(*v)),
            RuntimeValue::Range(v) => self.push(Record::Range(v.ty, v.value)),
            RuntimeValue::Record { record, fields } => {
                self.range_record(*record, fields, fields.capacity());
            }
            RuntimeValue::Result(layers) => {
                self.row(Part::ResultChain, &[layers.capacity() as u64]);
                for layer in layers {
                    if !self.spend() {
                        return;
                    }
                    self.row(
                        Part::ResultLayer,
                        &[
                            layer.ty.0 as u64,
                            match layer.tag {
                                hir::ResultTag::Ok => 0,
                                hir::ResultTag::Error => 1,
                            },
                        ],
                    );
                    match &layer.payload {
                        None => self.push(Record::Absent),
                        Some(ResultPayload::Bool(value)) => self.push(Record::Bool(*value)),
                        Some(ResultPayload::Range(value)) => {
                            self.push(Record::Range(value.ty, value.value))
                        }
                        Some(ResultPayload::Record { record, fields }) => {
                            self.range_record(*record, fields, fields.capacity());
                        }
                    }
                }
                self.push(Record::End);
            }
        }
    }
    fn ordered_index<T>(
        &mut self,
        fields: &[(hir::FieldId, T)],
        previous: Option<usize>,
    ) -> Option<usize> {
        let mut selected = None;
        for (index, (field, _)) in fields.iter().enumerate() {
            if !self.spend() {
                return None;
            }
            if previous.is_some_and(|p| field.0 <= p) {
                continue;
            }
            match selected {
                None => selected = Some(index),
                Some(old) if fields[old].0 == *field => {
                    self.fail(Incomplete::InvalidState);
                    return None;
                }
                Some(old) if field.0 < fields[old].0 .0 => selected = Some(index),
                _ => (),
            }
        }
        if selected.is_none() {
            self.fail(Incomplete::InvalidState);
        }
        selected
    }
    fn range_record(
        &mut self,
        record: hir::RecordId,
        fields: &[(hir::FieldId, RangeValue)],
        capacity: usize,
    ) {
        self.row(Part::Record, &[record.0 as u64, capacity as u64, 0]);
        let mut previous = None;
        for _ in 0..fields.len() {
            let Some(index) = self.ordered_index(fields, previous) else {
                return;
            };
            let (field, value) = &fields[index];
            previous = Some(field.0);
            self.row(Part::Field, &[field.0 as u64]);
            self.push(Record::Range(value.ty, value.value));
        }
        self.push(Record::End);
    }
    pub(super) fn value(&mut self, value: &Value) {
        self.value_at(value, 0);
    }
    fn value_at(&mut self, value: &Value, depth: usize) {
        if !self.spend() {
            return;
        }
        if depth == self.limits.lineage {
            self.fail(Incomplete::Lineage);
            return;
        }
        match value {
            Value::Public(v) => self.public(v),
            Value::Raw(v) => self.push(Record::Raw(*v)),
            Value::Reference {
                kind,
                referent,
                location,
            } => self.push(Record::Reference(
                *kind,
                *referent,
                location.frame,
                location.place,
            )),
            Value::Completion => self.push(Record::NoValue),
            Value::Result(id) => self.row(Part::ResultHandle, &[id.0 as u64]),
            Value::Temporary(id) => {
                self.row(Part::TemporaryHandle, &[id.activation, id.ordinal as u64])
            }
            Value::Resource { ty, identity } => {
                self.row(Part::SensorHandle, &[ty.0 as u64, *identity])
            }
            Value::Leaf {
                resource,
                record,
                identity,
            } => self.row(Part::Leaf, &[resource.0 as u64, record.0 as u64, *identity]),
            Value::Aggregate { record, fields } => {
                self.row(
                    Part::Record,
                    &[record.0 as u64, fields.capacity() as u64, 1],
                );
                let mut previous = None;
                for _ in 0..fields.len() {
                    let Some(index) = self.ordered_index(fields, previous) else {
                        return;
                    };
                    let (field, value) = &fields[index];
                    previous = Some(field.0);
                    self.row(Part::Field, &[field.0 as u64]);
                    self.value_at(value, depth + 1);
                }
                self.push(Record::End);
            }
        }
    }
    pub(super) fn outcome(&mut self, outcome: Result<&ExecutionResult, &ExecutionError>) {
        match outcome {
            Ok(v) => {
                self.push(Record::Delivered);
                match v {
                    ExecutionResult::NoValue => self.push(Record::NoValue),
                    ExecutionResult::Value(v) => self.public(v),
                }
            }
            Err(e) => {
                let class = match e.kind {
                    ErrorKind::RangeTrap { .. } => 1,
                    ErrorKind::ConfiguredLimit(_) => 2,
                    ErrorKind::Integrity(_) => 3,
                    ErrorKind::Host(_) => 4,
                    _ => {
                        self.fail(Incomplete::UncoveredState);
                        return;
                    }
                };
                self.push(Record::Stop(class, e.origin, e.expression));
                match e.kind {
                    ErrorKind::RangeTrap {
                        ty,
                        min,
                        max,
                        computed,
                    } => self.push(Record::Trap(ty, min, max, computed)),
                    ErrorKind::ConfiguredLimit(name) => self.push(Record::Limit(name)),
                    _ => (),
                }
                // Diagnostic trace wording/capacity are presentation. Logical
                // activations have already been captured, without taking the
                // session's reserved diagnostic backing.
            }
        }
        self.push(Record::End);
        self.finished = true;
        self.sealed_count = Some(self.records.len());
    }
}

impl Machine<'_> {
    #[cfg(test)]
    pub(super) fn conformance_parameter_control(&mut self, function: FunctionId) {
        if self.observation.borrow().is_none() {
            return;
        }
        let control = SWAP_PARAMETERS.with(|slot| slot.get());
        let Some((target, first, second)) = control else {
            return;
        };
        if target != function {
            return;
        }
        SWAP_PARAMETERS.with(|slot| slot.set(None));
        let slots = &mut self.frames.last_mut().unwrap().slots;
        let a = slots.iter().position(|(place, _)| *place == first).unwrap();
        let b = slots
            .iter()
            .position(|(place, _)| *place == second)
            .unwrap();
        let (a, b) = if a < b { (a, b) } else { (b, a) };
        let (left, right) = slots.split_at_mut(b);
        // A deliberately wrong, but locally typed and conserving, engine
        // transfer. Actual payloads move; no token or Result is cloned.
        std::mem::swap(&mut left[a].1, &mut right[0].1);
    }
    pub(super) fn observe_cleanup_leaf(&self, value: &Value, finished: bool) {
        let Value::Leaf { identity, .. } = value else {
            return;
        };
        if let Some(capture) = self.observation.borrow_mut().as_mut() {
            capture.push(Record::CleanupLeaf(*identity, finished));
        }
    }
    pub(super) fn observe_transit(
        &self,
        id: results::OwnerId,
        finished: bool,
        destination: Option<Destination>,
    ) {
        if let Some(c) = self.observation.borrow_mut().as_mut() {
            c.push(Record::Transit(id.0, finished));
            if !finished {
                c.push(match destination {
                    None => Record::Operand,
                    Some(Destination::Slot(at)) => Record::Destination(at.frame, at.place, None),
                    Some(Destination::Field(at, record, field)) => {
                        Record::Destination(at.frame, at.place, Some((record, field)))
                    }
                });
            }
        }
    }
    pub(super) fn observe_publication(&self, root: results::OwnerId, delivered: bool) {
        if let Some(c) = self.observation.borrow_mut().as_mut() {
            c.push(Record::Publication(root.0, delivered));
        }
    }
    pub(super) fn observe_action(&self, task: &Task<'_>) {
        if let Some(c) = self.observation.borrow_mut().as_mut() {
            c.push(Record::Executing);
            c.task(task);
        }
    }
    pub(super) fn observe(&self, cut: Cut) {
        let mut observer = self.observation.borrow_mut();
        let Some(c) = observer.as_mut() else {
            return;
        };
        if c.incomplete.is_some() {
            return;
        }
        if self.frames.len() > c.limits.lineage {
            c.fail(Incomplete::Lineage);
            return;
        }
        c.push(Record::Cut(
            cut,
            self.steps,
            self.frames.len(),
            self.origin.span,
            self.expression,
            self.terminal,
        ));
        c.push(Record::OriginKind(self.origin.kind));
        let mut slots = 0usize;
        for frame in &self.frames {
            if !c.spend() {
                return;
            }
            slots = slots.saturating_add(frame.slots.len());
        }
        c.push(Record::Inventory(
            c.snapshots,
            slots,
            self.values.len(),
            self.tasks.len(),
            u8::from(self.resources.is_some())
                | (u8::from(self.results.is_some()) << 1)
                | (u8::from(self.temporaries.is_some()) << 2)
                | (u8::from(self.trusted.is_some()) << 3),
        ));
        c.snapshots += 1;
        let start = c.records.len();
        let mut parent = None;
        for f in &self.frames {
            if c.incomplete.is_some() {
                return;
            }
            let Ok(definition) = self.executable.function(f.function) else {
                c.fail(Incomplete::InvalidState);
                return;
            };
            let mut found = false;
            for (id, block) in definition.blocks() {
                if c.incomplete.is_some() {
                    return;
                }
                if !c.spend() {
                    return;
                }
                if id == f.block {
                    found = f.statement <= block.statements().len();
                    break;
                }
            }
            if !found {
                c.fail(Incomplete::InvalidState);
                return;
            }
            c.push(Record::Frame(
                f.id,
                f.function,
                f.block,
                f.statement,
                parent,
                f.call_site,
            ));
            c.push(Record::FrameStorage(f.slots.capacity()));
            parent = Some(f.id);
            for (place, slot) in &f.slots {
                if c.incomplete.is_some() {
                    return;
                }
                let state = match slot {
                    Slot::Uninitialized => 0,
                    Slot::Available(_) => 1,
                    Slot::Vacant => 2,
                    Slot::Transferred => 3,
                    Slot::Cleaned => 4,
                    Slot::Replacing => 5,
                };
                c.push(Record::Slot(*place, state));
                if let Slot::Available(value) = slot {
                    c.value(value);
                }
            }
            c.push(Record::End);
        }
        for value in &self.values {
            if c.incomplete.is_some() {
                return;
            }
            c.push(Record::Operand);
            c.value(value);
        }
        for task in &self.tasks {
            if !c.spend() {
                return;
            }
            c.task(task);
        }
        if let Some(s) = &self.resources {
            s.observe(c);
        }
        if let Some(s) = &self.results {
            s.observe(c);
        }
        if let Some(s) = &self.temporaries {
            s.observe(c);
        }
        if let Some(s) = &self.trusted {
            s.observe(c);
        }
        // Each storage traversal element has a row above. Admit a conservative
        // multiple before the second walk, including nested payloads.
        let work = c
            .records
            .len()
            .saturating_sub(start)
            .saturating_mul(4)
            .saturating_add(1);
        if !c.spend_many(work) {
            return;
        }
        c.push(Record::Accounting(self.limits, self.storage()));
        c.row(
            Part::Buffers,
            &[
                self.frames.capacity() as u64,
                self.tasks.capacity() as u64,
                self.values.capacity() as u64,
                self.staged_return.unwrap_or(ABSENT),
                self.site.map_or(ABSENT, |s| s.0),
                self.site.map_or(ABSENT, |s| s.1 .0 as u64),
            ],
        );
        c.push(Record::End);
    }
}

enum Engine<'a> {
    Ordinary {
        machine: Machine<'a>,
        outcome: Option<Result<ExecutionResult, ExecutionError>>,
    },
    Selected(trusted::ExecutionSession<'a>),
}
/// Retains the actual interpreter session; evidence owns no resource token.
/// Public conformance inputs are the already approved zero-parameter root and
/// existing limits. No source/public resource input authority is added.
pub struct Session<'a> {
    contract: &'a contract::Contract<'a>,
    engine: Engine<'a>,
    entry: FunctionId,
    authority: Option<authority::View<'a>>,
}
impl<'a> Session<'a> {
    pub fn new(
        contract: &'a contract::Contract<'a>,
        execution: ExecutionLimits,
        observation: Limits,
    ) -> Result<Self, ExecutionError> {
        Self::new_inner(contract, execution, observation, None)
    }
    fn new_inner(
        contract: &'a contract::Contract<'a>,
        execution: ExecutionLimits,
        observation: Limits,
        input: Option<replay::Input<'_>>,
    ) -> Result<Self, ExecutionError> {
        contract.validate()?;
        checked_entry(contract.executable, contract.entry(), execution)?;
        let controller = input
            .map(|input| replay::Controller::admit(contract, execution, input))
            .transpose()?;
        let authority = authority::View::extract(contract, authority::Limits::default());
        let engine = if contract.executable.trusted.is_some() {
            Engine::Selected(trusted::ExecutionSession::new(
                contract.executable,
                contract.entry(),
                execution,
            )?)
        } else {
            Engine::Ordinary {
                machine: Machine::new(contract.executable, execution),
                outcome: None,
            }
        };
        let mut session = Self {
            contract,
            engine,
            entry: contract.entry(),
            authority: None,
        };
        *session.machine_mut().control.get_mut() = controller;
        let mut capture = Capture::new(observation, contract);
        match authority {
            Ok(view) => session.authority = Some(view),
            Err(reason) => capture.fail(reason),
        };
        capture.controlled = session.machine().control.borrow().is_some();
        *session.machine_mut().observation.get_mut() = Some(capture);
        session.machine().observe(Cut::Start);
        Ok(session)
    }
    fn machine(&self) -> &Machine<'a> {
        match &self.engine {
            Engine::Ordinary { machine, .. } => machine,
            Engine::Selected(s) => &s.machine,
        }
    }
    fn machine_mut(&mut self) -> &mut Machine<'a> {
        match &mut self.engine {
            Engine::Ordinary { machine, .. } => machine,
            Engine::Selected(s) => &mut s.machine,
        }
    }
    /// Human presentation is separate from complete structured authority.
    /// Rendering failure does not advance or modify the retained execution.
    pub fn render_authority_summary(
        &self,
        allowance: usize,
        verbose: bool,
    ) -> Result<String, ExecutionError> {
        self.authority
            .as_ref()
            .ok_or_else(|| ExecutionError::integrity("structured authority evidence incomplete"))?
            .render_summary(allowance, verbose)
    }
    pub fn advance(&mut self) -> Result<&ExecutionResult, &ExecutionError> {
        match &mut self.engine {
            Engine::Ordinary { machine, outcome } => {
                if outcome.is_none() {
                    *outcome = Some(machine.start(self.entry).and_then(|_| machine.resume()));
                    if outcome.as_ref().unwrap().is_err() {
                        machine.terminal = true;
                    }
                    machine.observe(Cut::Final);
                    machine
                        .observation
                        .borrow_mut()
                        .as_mut()
                        .unwrap()
                        .outcome(outcome.as_ref().unwrap().as_ref());
                }
                outcome.as_ref().unwrap().as_ref()
            }
            Engine::Selected(s) => s.advance(),
        }
    }
    fn capture(&self) -> Ref<'_, Capture> {
        Ref::map(self.machine().observation.borrow(), |o| o.as_ref().unwrap())
    }
    fn primary(&self) -> Option<Result<&ExecutionResult, &ExecutionError>> {
        match &self.engine {
            Engine::Ordinary { outcome, .. } => outcome.as_ref().map(Result::as_ref),
            Engine::Selected(session) => session.recorded(),
        }
    }
    /// Capture completion and capture budgets only. Success does not establish
    /// observation validity or conformance; compare validates both full views.
    pub fn completeness(&self) -> Result<(), Incomplete> {
        let c = self.capture();
        if let Some(reason) = c.incomplete {
            Err(reason)
        } else if !c.finished {
            Err(Incomplete::Pending)
        } else {
            Ok(())
        }
    }
    pub fn record_count(&self) -> usize {
        self.capture().records.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn generous() -> Limits {
        Limits {
            records: 262_144,
            work: 10_000_000,
            storage: 67_108_864,
            lineage: 128,
        }
    }
    #[test]
    fn selected_completion_census_preserves_real_delivery_and_repeat() {
        use crate::trusted::{Grants, Scenario, Selection};
        for scenario in [
            Scenario::Sample,
            Scenario::FinishIoFailure,
            Scenario::FinishPartialIoFailure,
        ] {
            let selection = Selection::sensor_demo(scenario, Grants::ALL, &["sample"]).unwrap();
            let e = prepare_execution_with_profile(
                include_str!("../../../examples/sensor_finish.bow"),
                AdmissionLimits::default(),
                &selection,
            )
            .unwrap();
            let c = contract::Contract::extract(
                &e,
                e.entry("sample").unwrap(),
                contract::Limits::default(),
            )
            .unwrap();
            let mut session = Session::new(&c, ExecutionLimits::default(), generous()).unwrap();
            let mut reference =
                trusted::ExecutionSession::new(&e, c.entry(), ExecutionLimits::default()).unwrap();
            assert_eq!(session.advance().unwrap(), reference.advance().unwrap());
            assert_eq!(session.completeness(), Ok(()));
            let count = session.record_count();
            session.advance().unwrap();
            assert_eq!(session.record_count(), count);
            let Engine::Selected(actual) = &session.engine else {
                unreachable!()
            };
            assert_eq!(actual.events(), reference.events());
            assert_eq!(actual.live_sensors(), 0);
        }
    }
    #[test]
    fn selected_observation_exhaustion_never_reclassifies_a_fuel_stop() {
        use crate::trusted::{Grants, Scenario, Selection};
        let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["sample"]).unwrap();
        let e = prepare_execution_with_profile(
            include_str!("../../../examples/sensor_finish.bow"),
            AdmissionLimits::default(),
            &selection,
        )
        .unwrap();
        let c = contract::Contract::extract(
            &e,
            e.entry("sample").unwrap(),
            contract::Limits::default(),
        )
        .unwrap();
        for steps in [1, 3, 20, 40, 80, 120] {
            let limits = ExecutionLimits {
                steps,
                ..ExecutionLimits::default()
            };
            let mut reference = trusted::ExecutionSession::new(&e, c.entry(), limits).unwrap();
            let mut session = Session::new(
                &c,
                limits,
                Limits {
                    records: 1,
                    ..Limits::default()
                },
            )
            .unwrap();
            assert_eq!(session.advance(), reference.advance());
            assert!(session.completeness().is_err());
            let Engine::Selected(actual) = &session.engine else {
                unreachable!()
            };
            assert_eq!(actual.events(), reference.events());
            assert_eq!(actual.live_sensors(), reference.live_sensors());
        }
    }
    #[test]
    fn ordinary_capture_is_admitted_separately_and_repeat_is_read_only() {
        let e =
            prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        let mut s = Session::new(&c, ExecutionLimits::default(), Limits::default()).unwrap();
        assert_eq!(s.completeness(), Err(Incomplete::Pending));
        assert_eq!(
            s.advance().unwrap(),
            &ExecutionResult::Value(RuntimeValue::Bool(true))
        );
        assert_eq!(s.completeness(), Ok(()));
        let count = s.record_count();
        s.advance().unwrap();
        assert_eq!(s.record_count(), count);
    }
    #[test]
    fn exhausted_observation_preserves_primary_execution() {
        let e =
            prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        for limits in [
            Limits {
                records: 1,
                ..Limits::default()
            },
            Limits {
                work: 1,
                ..Limits::default()
            },
            Limits {
                storage: 1,
                ..Limits::default()
            },
        ] {
            let mut s = Session::new(&c, ExecutionLimits::default(), limits).unwrap();
            assert_eq!(
                s.advance().unwrap(),
                &execute(&e, c.entry(), ExecutionLimits::default()).unwrap()
            );
            assert!(s.completeness().is_err());
        }
    }
}

#[cfg(test)]
mod replay_tests;
