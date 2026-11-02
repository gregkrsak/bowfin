//! Observation validation precedes correspondence. Neither stage executes,
//! repairs or acquires authority over the retained interpreter state.
use super::*;
mod references;
mod transitions;

// Private discriminators can remove one guard at a time. Production always
// enforces every guard; no public option or capture-sealing bypass exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Protection {
    Seal = 1,
    Queue = 2,
    Operand = 4,
    Resurrection = 8,
    Duplicate = 16,
    Access = 32,
    Control = 64,
}
#[cfg(test)]
thread_local! {
    static DISABLED_PROTECTIONS: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
    static REJECTED_PROTECTION: std::cell::Cell<Option<Protection>> = const { std::cell::Cell::new(None) };
}
fn protection(boundary: Protection, condition: bool) -> Valid {
    #[cfg(test)]
    if DISABLED_PROTECTIONS.with(|mask| mask.get() & boundary as u8 != 0) {
        return Ok(());
    }
    #[cfg(test)]
    if !condition {
        REJECTED_PROTECTION.with(|slot| slot.set(Some(boundary)));
    }
    let _ = boundary;
    require(condition)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Comparison {
    Equal,
    Different { record: usize },
    Inconclusive(Incomplete),
}
#[derive(Clone, Copy, Debug)]
pub struct ComparisonLimits {
    pub records: usize,
    pub nodes: usize,
    pub work: usize,
    pub storage: usize,
}
impl Default for ComparisonLimits {
    fn default() -> Self {
        Self {
            records: 262_144,
            nodes: 65_536,
            work: 20_000_000,
            storage: 67_108_864,
        }
    }
}
type Valid<T = ()> = Result<T, Incomplete>;
#[cfg_attr(test, track_caller)]
fn require(condition: bool) -> Valid {
    if condition {
        Ok(())
    } else {
        #[cfg(test)]
        eprintln!("invalid observation at {}", std::panic::Location::caller());
        Err(Incomplete::InvalidState)
    }
}
struct Work(usize);
impl Work {
    fn spend(&mut self) -> Valid {
        self.0 = self.0.checked_sub(1).ok_or(Incomplete::Work)?;
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Frame,
    Logical,
    Result,
    Temporary,
    Owner,
    Scope,
    Generation,
    Pending,
    Replacement,
    Staging,
    Realm,
    Sensor,
    Invocation,
    Production,
    Leaf,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Key(Kind, u64, u64);
fn key(kind: Kind, id: u64) -> Key {
    Key(kind, id, 0)
}
#[derive(Clone, Copy)]
struct Node {
    key: Key,
    a: u64,
    b: u64,
    parent: Option<Key>,
    ty: Option<ValueType>,
    stamp: usize,
    status: u8,
    born: bool,
    edge: Option<Key>,
    block: usize,
    statement: usize,
    declared: usize,
    payload: Option<(usize, usize)>,
    child: Option<Key>,
    tag: u8,
    origin: Option<Span>,
}
#[derive(Clone, Copy)]
enum Link {
    Reference(Key),
    Borrow(Key, Place, BorrowKind),
    Expected(Key, ValueType),
    Owner(Key, u8),
    Generation(u64, hir::LocalId, u64),
    PayloadParent(Key, Key),
    Completion(Key, Key, usize),
    ProductionCell(Key, Key),
}
#[derive(Clone, Copy)]
struct SlotRow {
    frame: u64,
    place: Place,
    state: u8,
    reference: Option<references::ReferenceValue>,
    reference_root: Option<(u64, usize, usize)>,
}
#[derive(Clone, Copy)]
struct Pair {
    left: Key,
    right: Key,
    seen: usize,
}

/// One reservation covers validation scratch and both directions of the map.
/// The final comparison result has a fixed scalar representation and no writer.
struct Scratch {
    nodes: Vec<Node>,
    links: Vec<Link>,
    slots: Vec<SlotRow>,
    references: Vec<references::History>,
    pairs: Vec<Pair>,
    trusted: Vec<trusted::Event>,
    leaves: Vec<(u8, u64)>,
    temporaries: Vec<Record>,
    controls: Vec<Record>,
    cap: usize,
}
pub(super) fn compatibility_shapes() -> [usize; 8] {
    [
        size_of::<Node>(),
        size_of::<Link>(),
        size_of::<SlotRow>() + size_of::<references::History>(),
        size_of::<Pair>(),
        size_of::<trusted::Event>(),
        size_of::<(u8, u64)>(),
        size_of::<Scratch>(),
        size_of::<Comparison>(),
    ]
}
impl Scratch {
    fn new(limits: ComparisonLimits, schedule: contract::Schedule) -> Valid<Self> {
        let units: usize =
            schedule.comparison[..6].iter().sum::<usize>() + 2 * schedule.observation[0];
        let bytes = limits
            .nodes
            .checked_mul(units)
            .and_then(|n| n.checked_add(schedule.comparison[6]))
            .and_then(|n| n.checked_add(schedule.comparison[7]));
        if limits.records == 0
            || limits.records > 1_048_576
            || limits.nodes == 0
            || limits.nodes > 1_048_576
            || limits.work == 0
            || limits.work > 100_000_000
            || limits.storage > 268_435_456
            || bytes.map_or(true, |n| n > limits.storage)
        {
            return Err(Incomplete::Admission);
        }
        let mut s = Self {
            nodes: Vec::new(),
            links: Vec::new(),
            slots: Vec::new(),
            references: Vec::new(),
            pairs: Vec::new(),
            trusted: Vec::new(),
            leaves: Vec::new(),
            temporaries: Vec::new(),
            controls: Vec::new(),
            cap: limits.nodes,
        };
        s.nodes
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.links
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.slots
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.references
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.pairs
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.trusted
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.leaves
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.temporaries
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        s.controls
            .try_reserve_exact(limits.nodes)
            .map_err(|_| Incomplete::Allocation)?;
        Ok(s)
    }
    fn clear(&mut self) {
        self.nodes.clear();
        self.links.clear();
        self.slots.clear();
        self.references.clear();
        self.trusted.clear();
        self.leaves.clear();
        self.temporaries.clear();
        self.controls.clear();
    }
    fn find(&self, k: Key, work: &mut Work) -> Valid<Option<usize>> {
        for (index, n) in self.nodes.iter().enumerate() {
            work.spend()?;
            if n.key == k {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn node(&mut self, n: Node, declaration: bool, work: &mut Work) -> Valid<usize> {
        require(n.key.1 != ABSENT)?;
        if let Some(index) = self.find(n.key, work)? {
            let old = self.nodes[index];
            // Staging slots may retire and later be reused. Their active
            // lifetimes, rather than their container numbers, are identities.
            if n.key.0 == Kind::Staging && old.stamp.saturating_add(1) < n.stamp {
                self.nodes[index] = n;
                return Ok(index);
            }
            require(old.a == n.a && old.b == n.b && old.parent == n.parent)?;
            if let (Some(a), Some(b)) = (old.ty, n.ty) {
                require(a == b)?;
            }
            if declaration {
                if n.key.0 == Kind::Frame {
                    require(old.status == 0)?;
                }
                require(old.declared != n.stamp)?;
                self.nodes[index].declared = n.stamp;
            }
            self.nodes[index].stamp = n.stamp;
            self.nodes[index].ty = old.ty.or(n.ty);
            Ok(index)
        } else {
            if self.nodes.len() == self.cap {
                return Err(Incomplete::Records);
            }
            self.nodes.push(n);
            Ok(self.nodes.len() - 1)
        }
    }
    fn link(&mut self, l: Link) -> Valid {
        if self.links.len() == self.cap {
            return Err(Incomplete::Records);
        }
        self.links.push(l);
        Ok(())
    }
    fn pair(&mut self, a: Key, b: Key, stamp: usize, work: &mut Work) -> Valid<bool> {
        require(a.0 == b.0)?;
        for pair in &mut self.pairs {
            work.spend()?;
            if a.0 == Kind::Staging && pair.seen.saturating_add(1) < stamp {
                continue;
            }
            if pair.left == a || pair.right == b {
                if pair.left != a || pair.right != b {
                    return Ok(false);
                }
                pair.seen = stamp;
                return Ok(true);
            }
        }
        if self.pairs.len() == self.cap {
            return Err(Incomplete::Records);
        }
        self.pairs.push(Pair {
            left: a,
            right: b,
            seen: stamp,
        });
        Ok(true)
    }
}

#[derive(Clone, Copy)]
struct ValueInfo {
    ty: Option<ValueType>,
    handle: Option<Key>,
    extra: usize,
    reference: Option<references::ReferenceValue>,
}
struct Validator<'a, 'b> {
    contract: &'a contract::Contract<'a>,
    records: &'b [Record],
    at: usize,
    scratch: &'b mut Scratch,
    work: &'b mut Work,
    stamp: usize,
    realm: Option<u64>,
    last_fuel: u64,
    snapshots: usize,
    trusted_count: usize,
    leaf_count: usize,
    lineage: usize,
    mask: u8,
    storage: usize,
    limits: ExecutionLimits,
    delivered: bool,
    action_frame: Option<u64>,
    action_plan: Option<usize>,
    action_callee: Option<FunctionId>,
    action_callsite: Option<Span>,
    leaf_birth: Option<u64>,
    cleanup_leaf: Option<u64>,
    publication: Option<u64>,
    delivered_root: Option<u64>,
    transit: Option<u64>,
    transit_type: Option<ValueType>,
    last_depth: usize,
    last_terminal: bool,
    held: usize,
    pending_states: usize,
    control: Option<&'b replay::Controller>,
    logical_count: usize,
    model_host: Option<replay::Address>,
    last_reached: Option<replay::Address>,
    pending_task: Option<(usize, usize)>,
    queue: Option<(usize, usize)>,
    producer: Option<transitions::Producer<'a>>,
    top_bool: Option<bool>,
    top_result: Option<u64>,
}
impl Validator<'_, '_> {
    fn peek(&self) -> Option<Record> {
        self.records.get(self.at).copied()
    }
    fn take(&mut self) -> Valid<Record> {
        self.work.spend()?;
        let r = self.peek().ok_or(Incomplete::InvalidState)?;
        self.at += 1;
        Ok(r)
    }
    fn end(&mut self) -> Valid {
        require(self.take()? == Record::End)
    }
    fn meta(&mut self, part: Part) -> Valid<[u64; 8]> {
        match self.take()? {
            Record::Meta(p, d) if p == part => Ok(d),
            _ => Err(Incomplete::InvalidState),
        }
    }
    fn is_meta(&self, part: Part) -> bool {
        matches!(self.peek(), Some(Record::Meta(p, _)) if p == part)
    }
    fn shape(&self, d: &[u64; 8], count: usize) -> Valid {
        require(d[count..].iter().all(|n| *n == ABSENT))
    }
    fn span(&mut self, span: Span) -> Valid {
        require(span.line > 0 && span.column > 0 && span.start <= span.end)?;
        if span == START {
            return Ok(());
        }
        for site in self.contract.sites() {
            self.work.spend()?;
            if site.origin == span {
                return Ok(());
            }
        }
        for id in self.contract.functions() {
            self.work.spend()?;
            let function = self.contract.executable.program.function(*id);
            if function.span() == span {
                return Ok(());
            }
            let definition = self
                .contract
                .executable
                .function(*id)
                .map_err(|_| Incomplete::InvalidState)?;
            for (_, block) in definition.blocks() {
                self.work.spend()?;
                for statement in block.statements() {
                    self.work.spend()?;
                    if matches!(statement, Statement::BindResult {pattern_origin, ..} if *pattern_origin == span)
                    {
                        return Ok(());
                    }
                }
                if let Terminator::Propagate { operation, .. }
                | Terminator::ForwardError { operation } = block.terminator()
                {
                    if [
                        operation.span,
                        operation.syntax.phrase,
                        operation.syntax.binding,
                    ]
                    .contains(&span)
                    {
                        return Ok(());
                    }
                }
            }
            if let Some(flow) = &definition.temporaries {
                for region in &flow.regions {
                    self.work.spend()?;
                    if [
                        region.origin.span,
                        region.retirement.span,
                        region.publication.span,
                    ]
                    .contains(&span)
                    {
                        return Ok(());
                    }
                }
                for site in &flow.sites {
                    self.work.spend()?;
                    if [site.origin.span, site.transfer.span, site.cleanup.span].contains(&span) {
                        return Ok(());
                    }
                }
            }
            if let Some(flow) = &definition.cleanup {
                for scope in &flow.scopes {
                    self.work.spend()?;
                    if scope.span == span {
                        return Ok(());
                    }
                }
                for block in &flow.blocks {
                    self.work.spend()?;
                    if block.enter.is_some_and(|(_, origin)| origin.span == span)
                        || block.exit_origin.is_some_and(|origin| origin.span == span)
                    {
                        return Ok(());
                    }
                    for initialized in &block.initialize {
                        self.work.spend()?;
                        if initialized.origin.span == span {
                            return Ok(());
                        }
                    }
                }
            }
        }
        #[cfg(test)]
        eprintln!("unbound observation origin: {span:?}");
        Err(Incomplete::InvalidState)
    }
    fn origin_pair(&mut self, origin: Origin) -> Valid {
        if origin.span == START {
            return require(origin.kind == OriginKind::FunctionBoundary);
        }
        for function in self.contract.functions() {
            self.work.spend()?;
            if origin.kind == OriginKind::FunctionBoundary
                && self.contract.executable.program.function(*function).span() == origin.span
            {
                return Ok(());
            }
            // FunctionId is source-global; origin rows follow ordinary MIR order.
            // Resolve that canonical relationship, including verified-ID gaps.
            for (position, ordinary) in self.contract.executable.mir.functions().iter().enumerate()
            {
                self.work.spend()?;
                if ordinary.function() == *function {
                    let row = self
                        .contract
                        .executable
                        .origins
                        .get(position)
                        .ok_or(Incomplete::InvalidState)?;
                    for canonical in row {
                        self.work.spend()?;
                        if *canonical == origin {
                            return Ok(());
                        }
                    }
                    break;
                }
            }
            let definition = self
                .contract
                .executable
                .function(*function)
                .map_err(|_| Incomplete::InvalidState)?;
            for (_, block) in definition.blocks() {
                for statement in block.statements() {
                    self.work.spend()?;
                    if origin.kind == OriginKind::Statement
                        && (contract::statement_span(statement) == origin.span
                            || matches!(statement,Statement::BindResult{pattern_origin,..} if *pattern_origin==origin.span))
                    {
                        return Ok(());
                    }
                    if matches!(statement,Statement::ReplaceOwned{origin:canonical,..} if *canonical==origin)
                    {
                        return Ok(());
                    }
                }
            }
            if let Some(flow) = &definition.temporaries {
                for region in &flow.regions {
                    self.work.spend()?;
                    if [region.origin, region.retirement, region.publication].contains(&origin) {
                        return Ok(());
                    }
                }
                for site in &flow.sites {
                    self.work.spend()?;
                    if [site.origin, site.transfer, site.cleanup].contains(&origin) {
                        return Ok(());
                    }
                }
            }
            if let Some(flow) = &definition.cleanup {
                for block in &flow.blocks {
                    self.work.spend()?;
                    if block
                        .enter
                        .is_some_and(|(_, canonical)| canonical == origin)
                        || block.exit_origin == Some(origin)
                    {
                        return Ok(());
                    }
                    for initialization in &block.initialize {
                        self.work.spend()?;
                        if initialization.origin == origin {
                            return Ok(());
                        }
                    }
                }
            }
        }
        Err(Incomplete::InvalidState)
    }
    fn origin(&mut self) -> Valid<Span> {
        match self.take()? {
            Record::Origin(s) => {
                self.span(s)?;
                Ok(s)
            }
            _ => Err(Incomplete::InvalidState),
        }
    }
    fn declaration(
        &mut self,
        k: Key,
        a: u64,
        b: u64,
        parent: Option<Key>,
        ty: Option<ValueType>,
        unique: bool,
    ) -> Valid<usize> {
        if let Some(p) = parent {
            self.scratch.link(Link::Reference(p))?;
        }
        self.scratch.node(
            Node {
                key: k,
                a,
                b,
                parent,
                ty,
                stamp: self.stamp,
                status: 0,
                born: false,
                edge: None,
                block: 0,
                statement: 0,
                declared: if unique { self.stamp } else { usize::MAX },
                payload: None,
                child: None,
                tag: 0,
                origin: None,
            },
            unique,
            self.work,
        )
    }
    fn reference(&mut self, k: Key) -> Valid {
        self.scratch.link(Link::Reference(k))
    }
    fn owner(&mut self, k: Key, role: u8) -> Valid {
        self.scratch.link(Link::Owner(k, role))
    }
    fn node(&mut self, k: Key) -> Valid<Node> {
        let index = self
            .scratch
            .find(k, self.work)?
            .ok_or(Incomplete::InvalidState)?;
        Ok(self.scratch.nodes[index])
    }
    fn function(&mut self, frame: u64) -> Valid<FunctionId> {
        Ok(FunctionId(self.node(key(Kind::Frame, frame))?.a as usize))
    }
    fn place(&mut self, frame: u64, place: Place) -> Valid<ValueType> {
        let f = self.function(frame)?;
        let p = &self.contract.executable.program;
        match place {
            Place::Local(id) => {
                let l = p.locals().get(id.0).ok_or(Incomplete::InvalidState)?;
                require(l.function == f && l.id == id)?;
                Ok(l.ty)
            }
            Place::Parameter(id) => {
                let parameter = p.parameters().get(id.0).ok_or(Incomplete::InvalidState)?;
                require(parameter.function == f && parameter.id == id)?;
                Ok(match parameter.ty {
                    hir::ParameterType::Value(t) => t,
                    hir::ParameterType::Range(t) => ValueType::Range(t),
                    hir::ParameterType::MutableRecord(t) => {
                        ValueType::MutableRef(ReferentType::Record(t))
                    }
                })
            }
        }
    }
    fn expected(&mut self, value: ValueInfo, ty: ValueType) -> Valid {
        if let Some(actual) = value.ty {
            require(actual == ty)
        } else if let Some(k) = value.handle {
            self.scratch.link(Link::Expected(k, ty))
        } else {
            Err(Incomplete::InvalidState)
        }
    }
    fn optional_value(&mut self, cleanup: bool) -> Valid {
        match self.take()? {
            Record::Absent => Ok(()),
            Record::Payload => {
                self.held_value(cleanup)?;
                Ok(())
            }
            _ => Err(Incomplete::InvalidState),
        }
    }
    fn add_storage(&mut self, bytes: usize) -> Valid {
        self.storage = self
            .storage
            .checked_add(bytes)
            .ok_or(Incomplete::InvalidState)?;
        Ok(())
    }
    fn capacity(&mut self, count: u64, unit: usize) -> Valid {
        let bytes = usize::try_from(count)
            .ok()
            .and_then(|n| n.checked_mul(unit))
            .ok_or(Incomplete::InvalidState)?;
        self.add_storage(bytes)
    }
    fn held_value(&mut self, cleanup: bool) -> Valid<ValueInfo> {
        self.held = self.held.checked_add(1).ok_or(Incomplete::InvalidState)?;
        let value = self.value(0, cleanup)?;
        if let Some(reference) = value.reference {
            self.scratch.link(Link::Borrow(
                key(Kind::Frame, reference.frame),
                reference.place,
                reference.kind,
            ))?;
        }
        self.add_storage(value.extra)?;
        Ok(value)
    }
    fn stored_value(&mut self) -> Valid<ValueInfo> {
        self.held = self.held.checked_add(1).ok_or(Incomplete::InvalidState)?;
        let value = self.value(0, false)?;
        self.add_storage(value.extra)?;
        Ok(value)
    }
    fn value(&mut self, depth: usize, cleanup: bool) -> Valid<ValueInfo> {
        if depth >= self.lineage {
            return Err(Incomplete::Lineage);
        }
        let mut info = ValueInfo {
            ty: None,
            handle: None,
            extra: 0,
            reference: None,
        };
        match self.take()? {
            Record::Bool(_) => info.ty = Some(ValueType::Bool),
            Record::Range(id, value) => {
                let range = self
                    .contract
                    .executable
                    .program
                    .ranges()
                    .get(id.0)
                    .ok_or(Incomplete::InvalidState)?;
                require(range.id == id && range.min <= value && value <= range.max)?;
                info.ty = Some(ValueType::Range(id));
            }
            Record::Raw(_) | Record::NoValue => (),
            Record::Reference(kind, referent, frame, place) => {
                info.reference = Some(references::ReferenceValue {
                    kind,
                    referent,
                    frame,
                    place,
                });
                let target = self.place(frame, place)?;
                require(target == referent.value_type())?;
                info.ty = Some(match kind {
                    BorrowKind::Shared => ValueType::SharedRef(referent),
                    BorrowKind::Mutable => ValueType::MutableRef(referent),
                });
            }
            Record::Meta(Part::Record, d) => {
                self.shape(&d, 3)?;
                require(d[2] <= 1)?;
                let id = hir::RecordId(d[0] as usize);
                let record = self
                    .contract
                    .executable
                    .program
                    .records()
                    .get(id.0)
                    .ok_or(Incomplete::InvalidState)?;
                let unit = self.contract.schedule().machine[if d[2] == 0 { 5 } else { 4 }];
                info.extra = usize::try_from(d[1])
                    .ok()
                    .and_then(|n| n.checked_mul(unit))
                    .ok_or(Incomplete::InvalidState)?;
                let mut count = 0;
                let mut previous = None;
                while self.is_meta(Part::Field) {
                    let field = self.meta(Part::Field)?;
                    self.shape(&field, 1)?;
                    require(previous.map_or(true, |p| field[0] > p))?;
                    previous = Some(field[0]);
                    let row = self
                        .contract
                        .executable
                        .program
                        .fields()
                        .get(field[0] as usize)
                        .ok_or(Incomplete::InvalidState)?;
                    require(row.record == id)?;
                    let expected = row.ty;
                    let value = self.value(depth + 1, cleanup)?;
                    if d[2] == 0 {
                        require(matches!(value.ty, Some(ValueType::Range(_))))?;
                    }
                    info.extra = info
                        .extra
                        .checked_add(value.extra)
                        .ok_or(Incomplete::InvalidState)?;
                    self.expected(value, expected)?;
                    count += 1;
                }
                require(count as u64 <= d[1])?;
                if !cleanup {
                    require(count == record.fields.len())?;
                }
                self.end()?;
                info.ty = Some(ValueType::Record(id));
            }
            Record::Meta(Part::ResultChain, head) => {
                self.shape(&head, 1)?;
                require(self.delivered)?;
                let d = self.meta(Part::ResultLayer)?;
                self.shape(&d, 2)?;
                let outer = hir::ResultTypeId(d[0] as usize);
                let mut layer = d;
                let mut layers = 0;
                loop {
                    self.work.spend()?;
                    layers += 1;
                    require(layers <= 32)?;
                    let definition = self
                        .contract
                        .executable
                        .program
                        .result_type(hir::ResultTypeId(layer[0] as usize))
                        .ok_or(Incomplete::InvalidState)?;
                    require(layer[1] <= 1)?;
                    let ty = if layer[1] == 0 {
                        definition.success
                    } else {
                        definition.failure
                    };
                    if let ValueType::Result(child) = ty {
                        require(self.take()? == Record::Absent)?;
                        layer = self.meta(Part::ResultLayer)?;
                        self.shape(&layer, 2)?;
                        require(layer[0] == child.0 as u64)?;
                    } else {
                        let value = self.value(depth + 1, false)?;
                        self.expected(value, ty)?;
                        break;
                    }
                }
                require(layers as u64 <= head[0])?;
                self.end()?;
                info.ty = Some(ValueType::Result(outer));
            }
            Record::Meta(part, d) => {
                match part {
                    Part::ResultHandle => {
                        self.shape(&d, 1)?;
                        info.handle = Some(key(Kind::Result, d[0]));
                    }
                    Part::TemporaryHandle => {
                        self.shape(&d, 2)?;
                        info.handle = Some(Key(Kind::Owner, d[0], d[1]));
                    }
                    Part::SensorHandle => {
                        self.shape(&d, 2)?;
                        let catalog = self
                            .contract
                            .executable
                            .trusted
                            .as_ref()
                            .ok_or(Incomplete::InvalidState)?;
                        require(d[0] == catalog.catalog.sensor.0 as u64)?;
                        info.ty = Some(ValueType::Resource(catalog.catalog.sensor));
                        info.handle = Some(key(Kind::Sensor, d[1]));
                    }
                    Part::Leaf => {
                        self.shape(&d, 3)?;
                        let leaf = self
                            .contract
                            .executable
                            .program
                            .resources
                            .leaves
                            .get(d[0] as usize)
                            .ok_or(Incomplete::InvalidState)?;
                        require(leaf.record.0 as u64 == d[1])?;
                        info.ty = Some(ValueType::Record(leaf.record));
                        let identity = key(Kind::Leaf, d[2]);
                        if self.scratch.find(identity, self.work)?.is_none() {
                            // The actual leaf can reach its value destination
                            // before its acquisition audit row is committed.
                            // Only the sealed private primitive licenses this
                            // pending birth; the observer creates no resource.
                            require(self.leaf_birth.is_none())?;
                            let mut permitted = false;
                            for operation in &self.contract.executable.program.resources.operations
                            {
                                self.work.spend()?;
                                if Some(operation.function) == self.action_callee
                                    && matches!(operation.kind, hir::resources::OperationKind::Acquire(resource) if resource.0 as u64 == d[0])
                                {
                                    permitted = true;
                                }
                            }
                            require(permitted)?;
                            let at = self.declaration(identity, 0, 0, None, None, false)?;
                            self.scratch.nodes[at].born = true;
                            self.leaf_birth = Some(d[2]);
                        }
                        info.handle = Some(identity);
                    }
                    _ => return Err(Incomplete::InvalidState),
                }
                if let Some(k) = info.handle {
                    self.reference(k)?;
                    self.owner(k, u8::from(cleanup))?;
                }
            }
            _ => return Err(Incomplete::InvalidState),
        }
        Ok(info)
    }
    fn expression(&mut self, frame: u64) -> Valid<usize> {
        let Record::Expression(index) = self.take()? else {
            return Err(Incomplete::InvalidState);
        };
        let site = self
            .contract
            .sites()
            .get(index)
            .ok_or(Incomplete::InvalidState)?;
        require(site.role == contract::Role::Expression && site.function == self.function(frame)?)?;
        Ok(index)
    }
    fn public_record(
        &mut self,
        id: hir::RecordId,
        fields: &[(hir::FieldId, RangeValue)],
        capacity: usize,
    ) -> Valid {
        let d = self.meta(Part::Record)?;
        self.shape(&d, 3)?;
        require(d[0] == id.0 as u64 && d[1] == capacity as u64 && d[2] == 0)?;
        let definition = self
            .contract
            .executable
            .program
            .records()
            .get(id.0)
            .ok_or(Incomplete::InvalidState)?;
        require(definition.fields.len() == fields.len())?;
        for field in &definition.fields {
            let mut actual = None;
            for (candidate, value) in fields {
                self.work.spend()?;
                if candidate == field {
                    require(actual.is_none())?;
                    actual = Some(value);
                }
            }
            let value = actual.ok_or(Incomplete::InvalidState)?;
            let row = self.meta(Part::Field)?;
            self.shape(&row, 1)?;
            require(row[0] == field.0 as u64)?;
            require(self.take()? == Record::Range(value.ty, value.value))?;
        }
        self.end()
    }
    fn public_value(&mut self, value: &RuntimeValue) -> Valid {
        match value {
            RuntimeValue::Bool(value) => require(self.take()? == Record::Bool(*value)),
            RuntimeValue::Range(value) => {
                require(self.take()? == Record::Range(value.ty, value.value))
            }
            RuntimeValue::Record { record, fields } => {
                self.public_record(*record, fields, fields.capacity())
            }
            RuntimeValue::Result(layers) => {
                let head = self.meta(Part::ResultChain)?;
                self.shape(&head, 1)?;
                require(head[0] == layers.capacity() as u64)?;
                for layer in layers {
                    self.work.spend()?;
                    let d = self.meta(Part::ResultLayer)?;
                    self.shape(&d, 2)?;
                    require(
                        d[0] == layer.ty.0 as u64
                            && d[1] == u64::from(layer.tag == hir::ResultTag::Error),
                    )?;
                    match &layer.payload {
                        None => require(self.take()? == Record::Absent)?,
                        Some(ResultPayload::Bool(value)) => {
                            require(self.take()? == Record::Bool(*value))?
                        }
                        Some(ResultPayload::Range(value)) => {
                            require(self.take()? == Record::Range(value.ty, value.value))?
                        }
                        Some(ResultPayload::Record { record, fields }) => {
                            self.public_record(*record, fields, fields.capacity())?
                        }
                    }
                }
                self.end()
            }
        }
    }
    fn fields(&mut self, frame: u64) -> Valid {
        while self.is_meta(Part::Initializer) {
            let d = self.meta(Part::Initializer)?;
            self.shape(&d, 1)?;
            require((d[0] as usize) < self.contract.executable.program.fields().len())?;
            self.expression(frame)?;
        }
        self.end()
    }
    fn task(&mut self, executing: bool) -> Valid {
        let Record::Continuation(frame) = self.take()? else {
            return Err(Incomplete::InvalidState);
        };
        self.reference(key(Kind::Frame, frame))?;
        require(self.node(key(Kind::Frame, frame))?.stamp == self.stamp)?;
        let d = self.meta(Part::Task)?;
        if executing {
            self.action_frame = Some(frame);
            self.action_plan = if d[0] == 22 {
                Some(d[1] as usize)
            } else {
                None
            };
            self.leaf_birth = None;
            self.action_callee = None;
            self.action_callsite = None;
        }
        let count = match d[0] {
            0 | 6 | 7 | 11 | 14 | 15 | 23 | 24 | 25 => 1,
            1 | 4 | 8 | 9 | 12 | 20 => 2,
            3 | 10 | 13 | 17 | 18 | 19 => 3,
            2 | 5 | 16 | 21 | 22 => 4,
            _ => return Err(Incomplete::InvalidState),
        };
        self.shape(&d, count)?;
        let function = self.function(frame)?;
        let definition = self
            .contract
            .executable
            .function(function)
            .map_err(|_| Incomplete::InvalidState)?;
        match d[0] {
            1 | 2 | 5 | 8 => {
                require(d[1] > 0 && d[1] <= ExecutionLimits::MAX.expression_depth as u64 + 1)?
            }
            9 => require((d[1] as usize) < self.contract.executable.program.records().len())?,
            10 => {
                let callee = self
                    .contract
                    .executable
                    .program
                    .functions()
                    .get(d[1] as usize)
                    .and_then(|f| f.as_ordinary())
                    .ok_or(Incomplete::InvalidState)?;
                require(callee.parameters.len() as u64 == d[2])?;
                let Record::CallSite(callsite) = self.take()? else {
                    return Err(Incomplete::InvalidState);
                };
                let mut found = false;
                for (expression, site) in self.contract.expressions() {
                    self.work.spend()?;
                    if self.contract.sites()[*site].function == function
                        && expression.span == callsite
                        && matches!(&expression.kind,ExprKind::Call{function:target,..} if target.0 as u64==d[1])
                    {
                        found = true;
                        break;
                    }
                }
                if !found {
                    for (_, body) in definition.blocks() {
                        for statement in body.statements() {
                            self.work.spend()?;
                            if matches!(statement, Statement::CallNoValue {function: target, span, ..} if target.0 as u64 == d[1] && *span == callsite)
                            {
                                found = true;
                            }
                        }
                    }
                }
                require(found)?;
                if executing {
                    self.action_callee = Some(callee.id);
                    self.action_callsite = Some(callsite);
                }
            }
            13 => {
                let control = self.node(key(Kind::Frame, frame))?;
                let body = definition
                    .blocks()
                    .nth(control.block)
                    .map(|(_, b)| b)
                    .ok_or(Incomplete::InvalidState)?;
                let Terminator::Branch {
                    then_target,
                    else_target,
                    ..
                } = body.terminator()
                else {
                    return Err(Incomplete::InvalidState);
                };
                require(
                    then_target.0 as u64 == d[1]
                        && else_target.0 as u64 == d[2]
                        && control.statement == body.statements().len(),
                )?;
            }
            21 => {
                require((d[1] as usize) < self.contract.executable.program.result_types().len())?;
                let control = self.node(key(Kind::Frame, frame))?;
                let body = definition
                    .blocks()
                    .nth(control.block)
                    .map(|(_, b)| b)
                    .ok_or(Incomplete::InvalidState)?;
                let (result, yes, no) = match body.terminator() {
                    Terminator::Match {
                        result,
                        ok_target,
                        error_target,
                        ..
                    } => (*result, *ok_target, *error_target),
                    Terminator::Propagate {
                        operation,
                        ok_target,
                        error_target,
                        ..
                    } => (operation.inner, *ok_target, *error_target),
                    _ => return Err(Incomplete::InvalidState),
                };
                require(result.0 as u64 == d[1] && yes.0 as u64 == d[2] && no.0 as u64 == d[3])?;
            }
            22 => {
                let approved = self
                    .contract
                    .executable
                    .trusted
                    .as_ref()
                    .ok_or(Incomplete::InvalidState)?;
                let plan = approved
                    .invocations
                    .get(d[1] as usize)
                    .ok_or(Incomplete::InvalidState)?;
                require(plan.function == function)?;
            }
            23..=25 => require(self.contract.executable.trusted.is_some())?,
            _ => (),
        }
        match d[0] {
            1 | 2 | 5 | 8 => {
                if d[0] == 5 || d[0] == 8 {
                    require(matches!(
                        self.take()?,
                        Record::Binary(BinaryOp::And | BinaryOp::Or)
                    ))?;
                }
                let index = self.expression(frame)?;
                if executing && matches!(d[0], 1 | 2) {
                    self.reference_expression_use(frame, index)?;
                }
            }
            7 => {
                require(matches!(self.take()?, Record::Binary(_)))?;
                require(matches!(self.take()?, Record::ExprType(_)))?;
            }
            9 | 16 | 18 | 19 => self.fields(frame)?,
            11 => {
                let Record::Destination(at, place, field) = self.take()? else {
                    return Err(Incomplete::InvalidState);
                };
                self.place(at, place)?;
                if let Some((record, id)) = field {
                    require(
                        self.contract
                            .executable
                            .program
                            .fields()
                            .get(id.0)
                            .is_some_and(|f| f.record == record),
                    )?;
                }
            }
            _ => (),
        }
        if executing && d[0] == 0 {
            self.reference_write_use(frame)?;
        }
        for k in task_keys(d) {
            self.reference(k)?;
        }
        self.end()
    }
    fn invocation(&mut self, id: u64, plan: usize, frame: Option<u64>) -> Valid<usize> {
        let approved = self
            .contract
            .executable
            .trusted
            .as_ref()
            .ok_or(Incomplete::InvalidState)?;
        let p = approved
            .invocations
            .get(plan)
            .ok_or(Incomplete::InvalidState)?;
        let parent = if let Some(index) = self.scratch.find(key(Kind::Invocation, id), self.work)? {
            self.scratch.nodes[index].parent
        } else {
            require(self.action_plan == Some(plan))?;
            let f = frame
                .or(self.action_frame)
                .ok_or(Incomplete::InvalidState)?;
            require(self.function(f)? == p.function)?;
            Some(key(Kind::Frame, f))
        };
        if let Some(frame) = frame {
            require(parent == Some(key(Kind::Frame, frame)))?;
        }
        self.declaration(
            key(Kind::Invocation, id),
            p.operation.0 as u64,
            plan as u64,
            parent,
            None,
            false,
        )
    }
    fn trusted_state(&mut self) -> Valid {
        let d = self.meta(Part::TrustedState)?;
        self.shape(&d, 7)?;
        require(d[0] != 0 && d[1] == self.contract.entry().0 as u64 && d[2] <= d[3])?;
        if let Some(realm) = self.realm {
            require(realm == d[0])?;
        } else {
            self.realm = Some(d[0]);
        }
        self.declaration(key(Kind::Realm, d[0]), 0, 0, None, None, false)?;
        let units = self.contract.schedule();
        self.add_storage(units.trusted[0])?;
        self.capacity(d[3], units.trusted[1])?;
        self.capacity(d[4], units.trusted[2])?;
        self.capacity(d[5], units.trusted[3])?;
        let retained = self.meta(Part::TrustedRetained)?;
        self.shape(&retained, 4)?;
        require(retained[0] <= 2 && retained[1] <= 3)?;
        require(
            d[6] as usize
                == self
                    .limits
                    .trace_depth
                    .checked_mul(units.machine[6])
                    .ok_or(Incomplete::InvalidState)?,
        )?;
        let unit = match retained[1] {
            0 => {
                require(retained[2] == 0)?;
                0
            }
            1 => units.machine[6],
            2 => units.machine[7],
            3 => units.machine[5],
            _ => unreachable!(),
        };
        let extra = (retained[2] as usize)
            .checked_mul(unit)
            .and_then(|n| {
                (retained[3] as usize)
                    .checked_mul(units.machine[6])
                    .and_then(|d| n.checked_add(d))
            })
            .ok_or(Incomplete::InvalidState)?;
        self.add_storage((d[6] as usize).max(extra))?;
        let approved = self
            .contract
            .executable
            .trusted
            .as_ref()
            .ok_or(Incomplete::InvalidState)?;
        let mut productions = 0;
        while self.is_meta(Part::Production) {
            let p = self.meta(Part::Production)?;
            self.shape(&p, 7)?;
            require(p[1] == d[0] && p[5] <= 1)?;
            let plan = approved
                .invocations
                .get(p[3] as usize)
                .ok_or(Incomplete::InvalidState)?;
            self.invocation(p[4], p[3] as usize, None)?;
            let index = self.declaration(
                key(Kind::Production, p[0]),
                p[3],
                p[2],
                Some(key(Kind::Invocation, p[4])),
                Some(ValueType::Result(plan.result)),
                true,
            )?;
            self.scratch.nodes[index].born = true;
            self.scratch.nodes[index].status = p[5] as u8;
            self.scratch.nodes[index].edge = (p[6] != ABSENT).then_some(key(Kind::Sensor, p[6]));
            self.reference(key(Kind::Result, p[2]))?;
            let cell = self.node(key(Kind::Result, p[2]))?;
            let (payload, end) = cell.payload.ok_or(Incomplete::InvalidState)?;
            if let Some(Record::Range(ty, value)) = self.peek() {
                require(p[6] == ABSENT)?;
                self.response_value(p[4], p[3] as usize, p[5], ty, value)?;
                require(end == payload + 1 && self.records[payload] == Record::Range(ty, value))?;
                let expected = self
                    .contract
                    .executable
                    .program
                    .result_payload(
                        plan.result,
                        if p[5] == 0 {
                            hir::ResultTag::Ok
                        } else {
                            hir::ResultTag::Error
                        },
                    )
                    .ok_or(Incomplete::InvalidState)?;
                let v = self.value(0, false)?;
                self.expected(v, expected)?;
                if p[5] == 1 {
                    let operation = approved
                        .catalog
                        .operation(plan.operation)
                        .ok_or(Incomplete::InvalidState)?;
                    require((0..8).contains(&value) && operation.errors & (1u8 << value) != 0)?;
                }
                if plan.completion.is_some() {
                    require(
                        (p[5] == 0 && ty == approved.catalog.finish_receipt && value == 1)
                            || (p[5] == 1 && ty == approved.catalog.error && value == 3),
                    )?;
                }
            } else {
                require(self.take()? == Record::Absent && p[6] != ABSENT)?;
                if let Some(response) = self.recorded_response(p[4], p[3] as usize)? {
                    require(
                        response == replay::Response::Open(approved.catalog.sensor) && p[5] == 0,
                    )?;
                }
                let Record::Meta(Part::SensorHandle, active) = self.records[payload] else {
                    return Err(Incomplete::InvalidState);
                };
                require(
                    end == payload + 1
                        && active[0] == approved.catalog.sensor.0 as u64
                        && active[1] == p[6],
                )?;
                self.reference(key(Kind::Sensor, p[6]))?;
            }
            if self.is_meta(Part::Token) {
                let t = self.meta(Part::Token)?;
                self.shape(&t, 2)?;
                require(t[0] == d[0] && t[1] == p[6])?;
                let n = self.declaration(
                    key(Kind::Sensor, t[1]),
                    0,
                    0,
                    Some(key(Kind::Realm, t[0])),
                    Some(ValueType::Resource(approved.catalog.sensor)),
                    true,
                )?;
                self.scratch.nodes[n].status = 1;
                self.scratch.nodes[n].born = true;
            }
            self.end()?;
            productions += 1;
        }
        require(productions <= d[4])?;
        if self.is_meta(Part::PendingReceipt) {
            self.pending_states += 1;
            let receipt = self.meta(Part::PendingReceipt)?;
            self.shape(&receipt, 3)?;
            require(receipt[2] == 255)?;
            let address = self.model_host.ok_or(Incomplete::InvalidState)?;
            require(
                address.site == replay::Site::Invocation(receipt[1] as usize)
                    && matches!(
                        address.phase,
                        replay::Phase::HandoffAccepted
                            | replay::Phase::BeforeAttempt
                            | replay::Phase::Attempt
                    ),
            )?;
            let control = self.control.ok_or(Incomplete::InvalidState)?;
            require(
                control.raw_for_birth(replay::Birth {
                    plan: receipt[1] as usize,
                    activation: address.activation,
                    occurrence: address.occurrence,
                }) == Some(receipt[0]),
            )?;
            self.invocation(receipt[0], receipt[1] as usize, None)?;
            require(self.take()? == Record::Absent)?;
            self.end()?;
        }
        if self.is_meta(Part::Receipt) {
            self.pending_states += 1;
            let r = self.meta(Part::Receipt)?;
            self.shape(&r, 3)?;
            require(r[2] <= 1)?;
            let plan = approved
                .invocations
                .get(r[1] as usize)
                .ok_or(Incomplete::InvalidState)?;
            self.invocation(r[0], r[1] as usize, None)?;
            let expected = self
                .contract
                .executable
                .program
                .result_payload(
                    plan.result,
                    if r[2] == 0 {
                        hir::ResultTag::Ok
                    } else {
                        hir::ResultTag::Error
                    },
                )
                .ok_or(Incomplete::InvalidState)?;
            if self.is_meta(Part::Token) {
                if let Some(response) = self.recorded_response(r[0], r[1] as usize)? {
                    require(
                        response == replay::Response::Open(approved.catalog.sensor) && r[2] == 0,
                    )?;
                }
                let t = self.meta(Part::Token)?;
                self.shape(&t, 2)?;
                require(t[0] == d[0] && expected == ValueType::Resource(approved.catalog.sensor))?;
                let n = self.declaration(
                    key(Kind::Sensor, t[1]),
                    0,
                    0,
                    Some(key(Kind::Realm, t[0])),
                    Some(expected),
                    true,
                )?;
                self.scratch.nodes[n].status = 2;
                self.scratch.nodes[n].born = true;
            } else {
                let Some(Record::Range(ty, number)) = self.peek() else {
                    return Err(Incomplete::InvalidState);
                };
                self.response_value(r[0], r[1] as usize, r[2], ty, number)?;
                if r[2] == 1 {
                    let Some(Record::Range(_, code)) = self.peek() else {
                        return Err(Incomplete::InvalidState);
                    };
                    let operation = approved
                        .catalog
                        .operation(plan.operation)
                        .ok_or(Incomplete::InvalidState)?;
                    require((0..8).contains(&code) && operation.errors & (1u8 << code) != 0)?;
                }
                let v = self.value(0, false)?;
                self.expected(v, expected)?;
            }
            self.end()?;
        }
        if self.is_meta(Part::Completion) {
            self.pending_states += 1;
            let p = self.meta(Part::Completion)?;
            self.shape(&p, 7)?;
            require(p[6] <= 7)?;
            let plan = approved
                .invocations
                .get(p[2] as usize)
                .ok_or(Incomplete::InvalidState)?;
            require(plan.completion.is_some())?;
            self.invocation(p[1], p[2] as usize, Some(p[0]))?;
            self.reference(key(Kind::Frame, p[0]))?;
            self.reference(Key(Kind::Owner, p[3], p[4]))?;
            self.reference(key(Kind::Sensor, p[5]))?;
            if self.is_meta(Part::ResultTicket) {
                let t = self.meta(Part::ResultTicket)?;
                self.shape(&t, 3)?;
                require(t[1] == 2 && p[6] <= 2)?;
                self.declaration(
                    key(Kind::Result, t[0]),
                    plan.result.0 as u64,
                    plan.function.0 as u64,
                    None,
                    Some(ValueType::Result(plan.result)),
                    false,
                )?;
                self.declaration(
                    key(Kind::Production, t[2]),
                    p[2],
                    t[0],
                    Some(key(Kind::Invocation, p[1])),
                    Some(ValueType::Result(plan.result)),
                    false,
                )?;
            } else {
                require(self.take()? == Record::Absent)?;
            }
            let result = self.meta(Part::CompletionOutcome)?;
            self.shape(&result, 1)?;
            if result[0] != ABSENT {
                require(p[6] >= 3)?;
                self.owner(key(Kind::Result, result[0]), 3)?;
                self.scratch.link(Link::Completion(
                    key(Kind::Result, result[0]),
                    key(Kind::Invocation, p[1]),
                    p[2] as usize,
                ))?;
            }
            self.optional_value(false)?;
            self.end()?;
        }
        for node in &mut self.scratch.nodes {
            self.work.spend()?;
            if node.key.0 == Kind::Invocation {
                node.status = 0;
                node.tag = 255;
                node.statement = 0;
            }
            if node.key.0 == Kind::Sensor {
                node.statement = 0;
            }
        }
        self.trusted_count = 0;
        while let Some(Record::TrustedEvent(event)) = self.peek() {
            self.take()?;
            if self.trusted_count < self.scratch.trusted.len() {
                require(self.scratch.trusted[self.trusted_count] == event)?;
            } else {
                if self.scratch.trusted.len() == self.scratch.cap {
                    return Err(Incomplete::Records);
                }
                self.scratch.trusted.push(event);
            }
            self.trusted_count += 1;
            match event {
                trusted::Event::Attempt(id, operation) => {
                    let at = self
                        .scratch
                        .find(key(Kind::Invocation, id), self.work)?
                        .ok_or(Incomplete::InvalidState)?;
                    let node = &mut self.scratch.nodes[at];
                    require(node.a == operation.0 as u64 && node.status == 0)?;
                    node.born = true;
                    node.status = 1;
                }
                trusted::Event::Received(id, tag) => {
                    let at = self
                        .scratch
                        .find(key(Kind::Invocation, id), self.work)?
                        .ok_or(Incomplete::InvalidState)?;
                    let node = &mut self.scratch.nodes[at];
                    require(tag <= 1 && node.status == 1)?;
                    node.status = 2;
                    node.tag = tag;
                }
                trusted::Event::Published(id, tag) | trusted::Event::FinishProtected(id, tag) => {
                    let at = self
                        .scratch
                        .find(key(Kind::Invocation, id), self.work)?
                        .ok_or(Incomplete::InvalidState)?;
                    let node = self.scratch.nodes[at];
                    let completion = approved
                        .invocations
                        .get(node.b as usize)
                        .ok_or(Incomplete::InvalidState)?
                        .completion
                        .is_some();
                    let protected = matches!(event, trusted::Event::FinishProtected(..));
                    require(node.tag == u8::from(tag == hir::ResultTag::Error))?;
                    require(if protected {
                        completion && node.status == 2
                    } else if completion {
                        node.status == 3
                    } else {
                        node.status == 2
                    })?;
                    self.scratch.nodes[at].status = if protected { 3 } else { 4 };
                }
                trusted::Event::FinishPrefix(id, prefix) => {
                    let at = self
                        .scratch
                        .find(key(Kind::Invocation, id), self.work)?
                        .ok_or(Incomplete::InvalidState)?;
                    let node = self.scratch.nodes[at];
                    require(
                        approved
                            .invocations
                            .get(node.b as usize)
                            .ok_or(Incomplete::InvalidState)?
                            .completion
                            .is_some()
                            && node.status == 1
                            && node.statement == 0
                            && prefix == 1,
                    )?;
                    self.scratch.nodes[at].statement = 1;
                }
                trusted::Event::ReleaseAttempt(id, operation) => {
                    require(operation == approved.catalog.resource.release)?;
                    let at = self
                        .scratch
                        .find(key(Kind::Sensor, id), self.work)?
                        .ok_or(Incomplete::InvalidState)?;
                    require(self.scratch.nodes[at].statement == 0)?;
                    self.scratch.nodes[at].statement = 1;
                }
                trusted::Event::ReleaseComplete(id) | trusted::Event::ReleaseFailure(id) => {
                    let at = self
                        .scratch
                        .find(key(Kind::Sensor, id), self.work)?
                        .ok_or(Incomplete::InvalidState)?;
                    require(self.scratch.nodes[at].statement == 1)?;
                    self.scratch.nodes[at].statement =
                        if matches!(event, trusted::Event::ReleaseComplete(_)) {
                            2
                        } else {
                            3
                        };
                }
            }
        }
        require(
            self.trusted_count == self.scratch.trusted.len() && self.trusted_count as u64 <= d[5],
        )?;
        Ok(())
    }
    fn state(&mut self, which: u64) -> Valid {
        let d = self.meta(Part::State)?;
        require(d[0] == which)?;
        let units = self.contract.schedule();
        match which {
            0 => {
                self.shape(&d, 7)?;
                require(d[6] <= 1)?;
                self.add_storage(units.cleanup[0])?;
                for (i, u) in [
                    (1, units.cleanup[1]),
                    (2, units.cleanup[2]),
                    (3, units.cleanup[5]),
                    (4, units.machine[2]),
                    (5, units.cleanup[8]),
                ] {
                    self.capacity(d[i], u)?;
                }
                while self.is_meta(Part::ResourceFrame) {
                    self.pending_states += 1;
                    let f = self.meta(Part::ResourceFrame)?;
                    self.shape(&f, 4)?;
                    require(f[1] <= 1)?;
                    self.reference(key(Kind::Frame, f[0]))?;
                    self.capacity(f[2], units.cleanup[3])?;
                    self.capacity(f[3], units.cleanup[6])?;
                    let function = self.function(f[0])?;
                    let flow = self
                        .contract
                        .executable
                        .function(function)
                        .map_err(|_| Incomplete::InvalidState)?
                        .cleanup
                        .as_ref()
                        .ok_or(Incomplete::InvalidState)?;
                    let mut count = 0;
                    let mut parent = None;
                    while self.is_meta(Part::Scope) {
                        let s = self.meta(Part::Scope)?;
                        self.shape(&s, 4)?;
                        require(s[0] == f[0])?;
                        let scope = flow
                            .scopes
                            .get(s[1] as usize)
                            .ok_or(Incomplete::InvalidState)?;
                        require(scope.parent.map(|id| id.0 as u64) == parent)?;
                        parent = Some(s[1]);
                        self.capacity(s[3], units.cleanup[4])?;
                        self.declaration(
                            key(Kind::Scope, s[2]),
                            s[1],
                            0,
                            Some(key(Kind::Frame, f[0])),
                            None,
                            true,
                        )?;
                        let mut initialized = 0;
                        while self.is_meta(Part::Generation) {
                            let g = self.meta(Part::Generation)?;
                            self.shape(&g, 4)?;
                            require(g[0] == s[2] && g[3] <= 1)?;
                            let local = hir::LocalId(g[1] as usize);
                            let mut member = false;
                            for owner in &scope.owners {
                                self.work.spend()?;
                                if *owner == local {
                                    member = true;
                                }
                            }
                            require(member)?;
                            let index = self.declaration(
                                key(Kind::Generation, g[2]),
                                g[1],
                                0,
                                Some(key(Kind::Scope, s[2])),
                                None,
                                true,
                            )?;
                            require(self.scratch.nodes[index].status <= g[3] as u8)?;
                            self.scratch.nodes[index].status = g[3] as u8;
                            initialized += 1;
                        }
                        require(initialized <= s[3])?;
                        count += 1;
                    }
                    require(count <= f[2])?;
                    count = 0;
                    while self.is_meta(Part::GenerationSlot) {
                        let g = self.meta(Part::GenerationSlot)?;
                        self.shape(&g, 3)?;
                        require(g[0] == f[0])?;
                        self.scratch.link(Link::Generation(
                            f[0],
                            hir::LocalId(g[1] as usize),
                            g[2],
                        ))?;
                        count += 1;
                    }
                    require(count <= f[3])?;
                }
                for holder in 0..3 {
                    let h = self.meta(Part::Holder)?;
                    self.shape(&h, 1)?;
                    require(h[0] == holder)?;
                    self.optional_value(holder != 0)?;
                    self.end()?;
                }
                let mut index = 0;
                while self.is_meta(Part::Holder) {
                    let h = self.meta(Part::Holder)?;
                    self.shape(&h, 2)?;
                    require(h[0] == 3 && h[1] == index)?;
                    self.held_value(true)?;
                    self.end()?;
                    index += 1;
                }
                require(index <= d[4])?;
                while self.is_meta(Part::PendingRecord) {
                    self.pending_states += 1;
                    let p = self.meta(Part::PendingRecord)?;
                    self.shape(&p, 8)?;
                    require(p[3] <= 1)?;
                    require((p[2] as usize) < self.contract.executable.program.records().len())?;
                    self.declaration(
                        key(Kind::Pending, p[0]),
                        p[2],
                        0,
                        Some(key(Kind::Frame, p[1])),
                        None,
                        true,
                    )?;
                    if p[4] != ABSENT {
                        self.reference(key(Kind::Temporary, p[4]))?;
                    }
                    if p[6] != ABSENT {
                        self.reference(key(Kind::Temporary, p[6]))?;
                    }
                    self.capacity(p[7], units.cleanup[7])?;
                    let mut count = 0;
                    while self.is_meta(Part::PendingField) {
                        let field = self.meta(Part::PendingField)?;
                        self.shape(&field, 1)?;
                        let row = self
                            .contract
                            .executable
                            .program
                            .fields()
                            .get(field[0] as usize)
                            .ok_or(Incomplete::InvalidState)?;
                        require(row.record.0 as u64 == p[2])?;
                        let ty = row.ty;
                        let value = self.held_value(false)?;
                        self.expected(value, ty)?;
                        count += 1;
                    }
                    require(count <= p[7])?;
                    self.end()?;
                }
                while self.is_meta(Part::Replacement) {
                    self.pending_states += 1;
                    let r = self.meta(Part::Replacement)?;
                    require(r[2] <= 1)?;
                    let place = if r[2] == 0 {
                        Place::Local(hir::LocalId(r[3] as usize))
                    } else {
                        Place::Parameter(hir::ParameterId(r[3] as usize))
                    };
                    require(
                        self.place(r[1], place)? == ValueType::Record(hir::RecordId(r[4] as usize)),
                    )?;
                    self.declaration(
                        key(Kind::Replacement, r[0]),
                        r[4],
                        r[3],
                        Some(key(Kind::Frame, r[1])),
                        None,
                        true,
                    )?;
                    self.reference(key(Kind::Scope, r[6]))?;
                    if r[7] != ABSENT {
                        self.reference(key(Kind::Generation, r[7]))?;
                    }
                    let phase = self.meta(Part::ReplacementState)?;
                    self.shape(&phase, 1)?;
                    require(phase[0] <= 4)?;
                    self.optional_value(false)?;
                    self.optional_value(true)?;
                    self.end()?;
                }
                self.leaf_count = 0;
                while let Some(Record::ResourceEvent(kind, id)) = self.peek() {
                    self.take()?;
                    require(kind <= 4)?;
                    let at = self.leaf_count;
                    if at < self.scratch.leaves.len() {
                        require(self.scratch.leaves[at] == (kind, id))?;
                    } else {
                        if self.scratch.leaves.len() == self.scratch.cap {
                            return Err(Incomplete::Records);
                        }
                        self.scratch.leaves.push((kind, id));
                    }
                    self.leaf_count += 1;
                    if kind == 4 {
                        self.reference(key(Kind::Frame, id))?;
                    } else {
                        let n = if kind == 0 {
                            self.declaration(key(Kind::Leaf, id), 0, 0, None, None, false)?
                        } else {
                            self.scratch
                                .find(key(Kind::Leaf, id), self.work)?
                                .ok_or(Incomplete::InvalidState)?
                        };
                        if kind == 0 {
                            self.scratch.nodes[n].born = true;
                            self.scratch.nodes[n].status = 0;
                        } else if kind == 2 {
                            self.scratch.nodes[n].status = 2;
                        }
                    }
                }
                require(
                    self.leaf_count == self.scratch.leaves.len() && self.leaf_count as u64 <= d[1],
                )?;
            }
            1 => {
                self.shape(&d, 4)?;
                self.add_storage(units.results[0])?;
                self.capacity(d[1], units.results[1])?;
                self.capacity(d[2], units.results[2])?;
                self.capacity(d[3], units.results[3])?;
                let mut count = 0;
                while self.is_meta(Part::ResultCell) {
                    let cell = self.meta(Part::ResultCell)?;
                    require(cell[2] <= 2 && cell[5] <= 1 && cell[7] <= 5)?;
                    let function = FunctionId(cell[1] as usize);
                    let ty = hir::ResultTypeId(cell[4] as usize);
                    let result = self
                        .contract
                        .executable
                        .program
                        .result_type(ty)
                        .ok_or(Incomplete::InvalidState)?;
                    let expected = if cell[5] == 0 {
                        result.success
                    } else {
                        result.failure
                    };
                    let index = self.declaration(
                        key(Kind::Result, cell[0]),
                        cell[4],
                        cell[1],
                        None,
                        Some(ValueType::Result(ty)),
                        true,
                    )?;
                    if self.scratch.nodes[index].born && self.scratch.nodes[index].status >= 4 {
                        require(self.scratch.nodes[index].status == cell[7] as u8)?;
                    }
                    self.scratch.nodes[index].status = cell[7] as u8;
                    self.scratch.nodes[index].block = cell[3] as usize;
                    self.scratch.nodes[index].statement = cell[2] as usize;
                    self.scratch.nodes[index].tag = cell[5] as u8;
                    self.scratch.nodes[index].born = true;
                    self.scratch.nodes[index].edge = if cell[6] == ABSENT {
                        None
                    } else {
                        Some(key(Kind::Result, cell[6]))
                    };
                    if cell[6] != ABSENT {
                        self.reference(key(Kind::Result, cell[6]))?;
                    }
                    let origin = self.origin()?;
                    self.scratch.nodes[index].origin = Some(origin);
                    if cell[2] == 0 {
                        let flow = self
                            .contract
                            .executable
                            .function(function)
                            .map_err(|_| Incomplete::InvalidState)?
                            .temporaries
                            .as_ref()
                            .ok_or(Incomplete::InvalidState)?;
                        let site = flow
                            .sites
                            .get(cell[3] as usize)
                            .ok_or(Incomplete::InvalidState)?;
                        require(site.ty == ExprType::Result(ty) && site.origin.span == origin)?;
                        require(site.children.len() == 1)?;
                        let child = flow
                            .sites
                            .get(site.children[0].0)
                            .ok_or(Incomplete::InvalidState)?;
                        require(
                            child.destination
                                == crate::mir::temporaries::Destination::ResultPayload(
                                    ty,
                                    if cell[5] == 0 {
                                        hir::ResultTag::Ok
                                    } else {
                                        hir::ResultTag::Error
                                    },
                                ),
                        )?;
                    } else if cell[2] == 1 {
                        let block = self
                            .contract
                            .executable
                            .function(function)
                            .map_err(|_| Incomplete::InvalidState)?
                            .blocks()
                            .nth(usize::try_from(cell[3]).map_err(|_| Incomplete::InvalidState)?)
                            .map(|(_, b)| b)
                            .ok_or(Incomplete::InvalidState)?;
                        let Terminator::ForwardError { operation } = block.terminator() else {
                            return Err(Incomplete::InvalidState);
                        };
                        require(
                            operation.outer == ty
                                && cell[5] == 1
                                && operation.syntax.phrase == origin,
                        )?;
                    } else {
                        self.reference(key(Kind::Production, cell[3]))?;
                        self.scratch.link(Link::ProductionCell(
                            key(Kind::Result, cell[0]),
                            key(Kind::Production, cell[3]),
                        ))?;
                    }
                    if cell[7] == 2 {
                        let f = self.meta(Part::ResultMatchFrame)?;
                        self.shape(&f, 1)?;
                        self.reference(key(Kind::Frame, f[0]))?;
                        self.origin()?;
                    }
                    match self.take()? {
                        Record::Absent => require(matches!(cell[7], 0 | 4 | 5))?,
                        Record::Payload => {
                            require(matches!(cell[7], 1..=3))?;
                            let begin = self.at;
                            let value = self.held_value(false)?;
                            if let Some((previous, end)) = self.scratch.nodes[index].payload {
                                require(end - previous == self.at - begin)?;
                                for offset in 0..end - previous {
                                    self.work.spend()?;
                                    require(
                                        self.records[previous + offset]
                                            == self.records[begin + offset],
                                    )?;
                                }
                            }
                            self.scratch.nodes[index].payload = Some((begin, self.at));
                            self.scratch.nodes[index].child =
                                value.handle.filter(|k| k.0 == Kind::Result);
                            if let Some(k) = value.handle {
                                require(k.0 != Kind::Owner)?;
                                if k.0 == Kind::Result {
                                    self.scratch
                                        .link(Link::PayloadParent(k, key(Kind::Result, cell[0])))?;
                                }
                            }
                            self.expected(value, expected)?;
                        }
                        _ => return Err(Incomplete::InvalidState),
                    }
                    self.end()?;
                    count += 1;
                }
                require(count <= d[1])?;
                let mut matches = 0;
                while self.is_meta(Part::ResultMatch) {
                    matches += 1;
                    let r = self.meta(Part::ResultMatch)?;
                    self.shape(&r, 2)?;
                    self.reference(key(Kind::Frame, r[0]))?;
                    self.owner(key(Kind::Result, r[1]), 2)?;
                }
                require(matches <= d[2])?;
                let mut inert = 0;
                while self.is_meta(Part::InertRecord) {
                    inert += 1;
                    self.pending_states += 1;
                    let r = self.meta(Part::InertRecord)?;
                    let index = self.declaration(
                        key(Kind::Staging, r[0]),
                        r[2],
                        0,
                        Some(key(Kind::Frame, r[1])),
                        None,
                        true,
                    )?;
                    self.staging_cursor(index, r[4] as usize)?;
                    require((r[2] as usize) < self.contract.executable.program.records().len())?;
                    if r[5] != ABSENT {
                        self.reference(key(Kind::Temporary, r[5]))?;
                    }
                    if r[7] != ABSENT {
                        self.reference(key(Kind::Temporary, r[7]))?;
                    }
                }
                require(inert <= d[3])?;
            }
            2 => {
                self.shape(&d, 3)?;
                self.add_storage(units.temporaries[0])?;
                self.capacity(d[1], units.temporaries[1])?;
                self.capacity(d[2], units.temporaries[3])?;
                let mut activation_count = 0;
                let mut previous_activation = None;
                while self.is_meta(Part::TemporaryActivation) {
                    self.pending_states += 1;
                    let t = self.meta(Part::TemporaryActivation)?;
                    require(t[5] <= 2 && (t[4] != ABSENT).then_some(t[4]) == previous_activation)?;
                    activation_count += 1;
                    require(activation_count <= self.lineage)?;
                    if let Some(parent) = previous_activation {
                        require(self.node(key(Kind::Temporary, parent))?.status == 0)?;
                    }
                    previous_activation = Some(t[0]);
                    require(self.function(t[1])?.0 as u64 == t[2])?;
                    let flow = self
                        .contract
                        .executable
                        .function(FunctionId(t[2] as usize))
                        .map_err(|_| Incomplete::InvalidState)?
                        .temporaries
                        .as_ref()
                        .ok_or(Incomplete::InvalidState)?;
                    require((t[3] as usize) < flow.regions.len())?;
                    let index = self.declaration(
                        key(Kind::Temporary, t[0]),
                        t[2],
                        t[3],
                        if t[4] == ABSENT {
                            None
                        } else {
                            Some(key(Kind::Temporary, t[4]))
                        },
                        None,
                        true,
                    )?;
                    let frame = key(Kind::Frame, t[1]);
                    if self.scratch.nodes[index].born {
                        require(self.scratch.nodes[index].edge == Some(frame))?;
                    }
                    self.scratch.nodes[index].born = true;
                    self.scratch.nodes[index].edge = Some(frame);
                    let description = &flow.regions[t[3] as usize];
                    if let Some(parent) = self.scratch.nodes[index].parent {
                        let parent = self.node(parent)?;
                        require(if let Some(expected) = description.parent {
                            parent.edge == Some(frame)
                                && parent.a == t[2]
                                && parent.b == expected.0 as u64
                        } else {
                            parent.edge != Some(frame)
                        })?;
                    } else {
                        require(description.parent.is_none())?;
                    }
                    self.scratch.nodes[index].status = t[5] as u8;
                    self.capacity(t[7], units.temporaries[2])?;
                    let mut count = 0;
                    while self.is_meta(Part::TemporaryOwner) {
                        let o = self.meta(Part::TemporaryOwner)?;
                        self.shape(&o, 4)?;
                        require(o[0] == t[0] && o[3] <= 3)?;
                        let site = flow
                            .sites
                            .get(o[2] as usize)
                            .ok_or(Incomplete::InvalidState)?;
                        require(site.region.0 as u64 == t[3])?;
                        let ty = value_type(site.ty).ok_or(Incomplete::InvalidState)?;
                        let n = self.declaration(
                            Key(Kind::Owner, o[0], o[1]),
                            o[2],
                            0,
                            Some(key(Kind::Temporary, t[0])),
                            Some(ty),
                            true,
                        )?;
                        self.scratch.nodes[n].status = o[3] as u8;
                        match self.take()? {
                            Record::Absent => require(o[3] != 0)?,
                            Record::Payload => {
                                require(o[3] == 0)?;
                                let v = self.held_value(false)?;
                                self.expected(v, ty)?;
                            }
                            _ => return Err(Incomplete::InvalidState),
                        }
                        self.end()?;
                        count += 1;
                    }
                    require(count <= t[7] && t[6] <= count)?;
                    self.optional_value(false)?;
                    self.end()?;
                }
                require(activation_count as u64 <= d[1])?;
                let mut count = 0;
                loop {
                    if let Some(
                        event @ (Record::TemporaryBegin(..)
                        | Record::TemporaryRegister(..)
                        | Record::TemporaryTransfer(..)
                        | Record::TemporaryCleanup(..)
                        | Record::TemporaryRetire(..)),
                    ) = self.peek()
                    {
                        if count < self.scratch.temporaries.len() {
                            require(self.scratch.temporaries[count] == event)?;
                        } else {
                            if self.scratch.temporaries.len() == self.scratch.cap {
                                return Err(Incomplete::Records);
                            }
                            self.scratch.temporaries.push(event);
                        }
                        count += 1;
                    }
                    match self.peek() {
                        Some(Record::TemporaryBegin(id, frame, region, parent)) => {
                            self.take()?;
                            let function = self.function(frame)?;
                            self.declaration(
                                key(Kind::Temporary, id),
                                function.0 as u64,
                                region.0 as u64,
                                parent.map(|p| key(Kind::Temporary, p)),
                                None,
                                false,
                            )?;
                        }
                        Some(Record::TemporaryRegister(id, ordinal, site)) => {
                            self.take()?;
                            let activation = self.node(key(Kind::Temporary, id))?;
                            let flow = self
                                .contract
                                .executable
                                .function(FunctionId(activation.a as usize))
                                .map_err(|_| Incomplete::InvalidState)?
                                .temporaries
                                .as_ref()
                                .ok_or(Incomplete::InvalidState)?;
                            let s = flow.sites.get(site.0).ok_or(Incomplete::InvalidState)?;
                            self.declaration(
                                Key(Kind::Owner, id, ordinal as u64),
                                site.0 as u64,
                                0,
                                Some(key(Kind::Temporary, id)),
                                value_type(s.ty),
                                false,
                            )?;
                        }
                        Some(Record::TemporaryTransfer(id, ordinal, _))
                        | Some(Record::TemporaryCleanup(id, ordinal)) => {
                            self.take()?;
                            self.reference(Key(Kind::Owner, id, ordinal as u64))?;
                        }
                        Some(Record::TemporaryRetire(id)) => {
                            self.take()?;
                            let at = self
                                .scratch
                                .find(key(Kind::Temporary, id), self.work)?
                                .ok_or(Incomplete::InvalidState)?;
                            self.scratch.nodes[at].status = 3;
                        }
                        _ => break,
                    }
                }
                require(count == self.scratch.temporaries.len() && count as u64 <= d[2])?;
            }
            _ => return Err(Incomplete::InvalidState),
        }
        Ok(())
    }
    fn control_address(&mut self, address: replay::Address) -> Valid {
        let control = self.control.ok_or(Incomplete::InvalidState)?;
        self.work.0 = self
            .work
            .0
            .checked_sub(control.site_work() + 1)
            .ok_or(Incomplete::Work)?;
        require(address.occurrence > 0 && control.has_site(address.site))?;
        let (function, _, _, _) = control
            .activation(address.activation)
            .ok_or(Incomplete::InvalidState)?;
        if address.activation == 0 {
            require(function == self.contract.entry())?;
        } else {
            require(
                self.node(key(Kind::Logical, address.activation as u64))?.a == function.0 as u64,
            )?;
        }
        match address.site {
            replay::Site::Capacity(actual, span) => {
                require(actual == function && address.phase == replay::Phase::Reservation)?;
                self.span(span)?;
            }
            replay::Site::Release(actual, span) => {
                require(
                    actual == function
                        && matches!(
                            address.phase,
                            replay::Phase::BeforeAttempt
                                | replay::Phase::ReleaseAttempt
                                | replay::Phase::ReleaseComplete
                                | replay::Phase::ReleaseFailure
                        ),
                )?;
                self.span(span)?;
            }
            replay::Site::Invocation(plan) => {
                let invocation = self
                    .contract
                    .executable
                    .trusted
                    .as_ref()
                    .ok_or(Incomplete::InvalidState)?
                    .invocations
                    .get(plan)
                    .ok_or(Incomplete::InvalidState)?;
                require(invocation.function == function)?;
                require(match address.phase {
                    replay::Phase::BeforeHandoff
                    | replay::Phase::HandoffAccepted
                    | replay::Phase::BeforeAttempt
                    | replay::Phase::Attempt
                    | replay::Phase::Response
                    | replay::Phase::Validation
                    | replay::Phase::Delivery => true,
                    replay::Phase::Protection => invocation.completion.is_some(),
                    _ => false,
                })?;
            }
            replay::Site::Call(..) => return Err(Incomplete::InvalidState),
        }
        Ok(())
    }
    fn recorded_response(&mut self, raw: u64, plan: usize) -> Valid<Option<replay::Response>> {
        let Some(control) = self.control else {
            return Ok(None);
        };
        self.work.0 = self
            .work
            .0
            .checked_sub(control.history_work() + 1)
            .ok_or(Incomplete::Work)?;
        let birth = control.birth_for_raw(raw).ok_or(Incomplete::InvalidState)?;
        require(birth.plan == plan)?;
        let address = replay::Address {
            site: replay::Site::Invocation(plan),
            activation: birth.activation,
            occurrence: birth.occurrence,
            phase: replay::Phase::Response,
        };
        for record in &self.scratch.controls {
            self.work.spend()?;
            if let Record::ControlledResponse(actual, response) = record {
                if *actual == address {
                    return Ok(Some(*response));
                }
            }
        }
        Err(Incomplete::InvalidState)
    }
    fn response_value(
        &mut self,
        raw: u64,
        plan: usize,
        tag: u64,
        ty: hir::RangeTypeId,
        value: i64,
    ) -> Valid {
        if let Some(response) = self.recorded_response(raw, plan)? {
            require(match response {
                replay::Response::Value(expected, number, _) => {
                    tag == 0 && expected == ty && number == value
                }
                replay::Response::Error(expected, number, _) => {
                    tag == 1 && expected == ty && number == value
                }
                _ => false,
            })?;
        }
        Ok(())
    }
    fn control_history(&mut self, record: Record) -> Valid {
        if self.scratch.controls.len() == self.scratch.cap {
            return Err(Incomplete::Records);
        }
        for previous in &self.scratch.controls {
            self.work.spend()?;
            let duplicate = match (*previous, record) {
                (Record::Control(a, 1), Record::Control(b, 1)) => a == b,
                (Record::Control(a, 0), Record::Control(b, 0)) => a == b,
                (Record::ControlledResponse(a, _), Record::ControlledResponse(b, _)) => a == b,
                (Record::Releasing(a, _, _), Record::Releasing(b, _, _)) => a == b,
                _ => false,
            };
            require(!duplicate)?;
        }
        self.scratch.controls.push(record);
        Ok(())
    }
    fn finish_controls(&mut self) -> Valid {
        let Some(control) = self.control else {
            return Ok(());
        };
        self.work.0 = self
            .work
            .0
            .checked_sub(control.input_work())
            .ok_or(Incomplete::Work)?;
        require(
            self.logical_count == control.activation_count()
                && self.model_host == control.stopped(),
        )?;
        let mut consumed = 0;
        for index in 0..self.scratch.controls.len() {
            self.work.spend()?;
            let record = self.scratch.controls[index];
            if matches!(record, Record::Control(_, 1)) {
                consumed += 1;
            }
            if let Record::ControlledResponse(address, response) = record {
                match address.site {
                    replay::Site::Invocation(plan) => {
                        self.work.0 = self
                            .work
                            .0
                            .checked_sub(control.history_work() + 1)
                            .ok_or(Incomplete::Work)?;
                        let raw = control
                            .raw_for_birth(replay::Birth {
                                plan,
                                activation: address.activation,
                                occurrence: address.occurrence,
                            })
                            .ok_or(Incomplete::InvalidState)?;
                        let invocation = self.node(key(Kind::Invocation, raw))?;
                        let (tag, prefix) = match response {
                            replay::Response::Open(_) => (0, 0),
                            replay::Response::Value(_, _, prefix) => (0, prefix),
                            replay::Response::Error(_, _, prefix) => (1, prefix),
                            _ => return Err(Incomplete::InvalidState),
                        };
                        require(
                            invocation.status >= 2
                                && invocation.tag == tag
                                && invocation.statement == prefix as usize,
                        )?;
                    }
                    replay::Site::Release(..) => {
                        let replay::Response::Release(birth, success) = response else {
                            return Err(Incomplete::InvalidState);
                        };
                        let mut binding = address;
                        binding.phase = replay::Phase::BeforeAttempt;
                        let mut sensor = None;
                        for other in &self.scratch.controls {
                            self.work.spend()?;
                            if let Record::Releasing(actual, 1, raw) = other {
                                if *actual == binding {
                                    sensor = Some(*raw);
                                }
                            }
                        }
                        let sensor = sensor.ok_or(Incomplete::InvalidState)?;
                        let state = self.node(key(Kind::Sensor, sensor))?;
                        require(state.statement == if success { 2 } else { 3 })?;
                        self.work.0 = self
                            .work
                            .0
                            .checked_sub(control.history_work() + 1)
                            .ok_or(Incomplete::Work)?;
                        let invocation = control
                            .raw_for_birth(birth)
                            .ok_or(Incomplete::InvalidState)?;
                        let mut associated = false;
                        for production in &self.scratch.nodes {
                            self.work.spend()?;
                            if production.key.0 == Kind::Production
                                && production.edge == Some(key(Kind::Sensor, sensor))
                                && production.parent == Some(key(Kind::Invocation, invocation))
                                && production.a == birth.plan as u64
                            {
                                associated = true;
                            }
                        }
                        require(associated)?;
                    }
                    _ => return Err(Incomplete::InvalidState),
                }
                let mut reached = false;
                for other in &self.scratch.controls {
                    self.work.spend()?;
                    if *other == Record::Control(address, 0) {
                        reached = true;
                    }
                }
                require(reached)?;
            }
        }
        require(consumed == control.used_responses())?;
        for index in 0..self.scratch.nodes.len() {
            self.work.spend()?;
            let node = self.scratch.nodes[index];
            if node.key.0 == Kind::Invocation && node.status >= 2 {
                self.recorded_response(node.key.1, node.b as usize)?;
            }
        }
        Ok(())
    }
    fn finish_graph(&mut self) -> Valid {
        self.stored_references()?;
        if let Some(root) = self.transit {
            let node = self.node(key(Kind::Result, root))?;
            require(node.status == 1 && node.edge.is_none())?;
            if let Some(expected) = self.transit_type {
                require(node.ty == Some(expected))?;
            }
        }
        for index in 0..self.scratch.links.len() {
            self.work.spend()?;
            let link = self.scratch.links[index];
            let k = match link {
                Link::Reference(k) | Link::Expected(k, _) | Link::Owner(k, _) => k,
                Link::Borrow(frame, place, kind) => {
                    let target = self.node(frame)?;
                    require(target.declared == self.stamp && target.status == 0)?;
                    let mut available = false;
                    for slot in &self.scratch.slots {
                        self.work.spend()?;
                        if slot.frame == frame.1 && slot.place == place && slot.state == 1 {
                            available = true;
                        }
                    }
                    protection(Protection::Access, available)?;
                    for prior in &self.scratch.links[..index] {
                        self.work.spend()?;
                        if let Link::Borrow(other, location, access) = prior {
                            if *other == frame && *location == place {
                                require(
                                    kind == BorrowKind::Shared && *access == BorrowKind::Shared,
                                )?;
                            }
                        }
                    }
                    continue;
                }
                Link::Generation(frame, local, id) => {
                    let generation = self.node(key(Kind::Generation, id))?;
                    require(generation.a == local.0 as u64)?;
                    let scope = self.node(generation.parent.ok_or(Incomplete::InvalidState)?)?;
                    require(scope.parent == Some(key(Kind::Frame, frame)))?;
                    for slot in &self.scratch.slots {
                        self.work.spend()?;
                        if slot.frame == frame
                            && slot.place == Place::Local(local)
                            && slot.state == 1
                        {
                            require(
                                generation.status == 0
                                    && generation.declared == self.stamp
                                    && scope.declared == self.stamp,
                            )?;
                        }
                    }
                    continue;
                }
                Link::PayloadParent(child, parent) => {
                    let node = self.node(child)?;
                    require(node.edge == Some(parent))?;
                    continue;
                }
                Link::ProductionCell(cell, production) => {
                    let cell = self.node(cell)?;
                    let row = self.node(production)?;
                    let approved = self
                        .contract
                        .executable
                        .trusted
                        .as_ref()
                        .ok_or(Incomplete::InvalidState)?;
                    let plan = approved
                        .invocations
                        .get(row.a as usize)
                        .ok_or(Incomplete::InvalidState)?;
                    let invocation = self.node(row.parent.ok_or(Incomplete::InvalidState)?)?;
                    require(invocation.status >= 2 && invocation.tag == row.status)?;
                    require(
                        row.b == cell.key.1
                            && row.born
                            && row.status == cell.tag
                            && cell.ty == Some(ValueType::Result(plan.result))
                            && cell.b == plan.function.0 as u64
                            && cell.origin == Some(plan.source),
                    )?;
                    continue;
                }
                Link::Completion(cell, invocation, plan) => {
                    let cell = self.node(cell)?;
                    require(cell.status == 3 && cell.statement == 2)?;
                    let production = self.node(key(Kind::Production, cell.block as u64))?;
                    require(production.parent == Some(invocation) && production.a == plan as u64)?;
                    continue;
                }
            };
            let node = self.node(k)?;
            if matches!(k.0, Kind::Staging | Kind::Pending | Kind::Replacement) {
                require(node.declared == self.stamp)?;
            }
            match link {
                Link::Expected(_, ty) => require(node.ty == Some(ty))?,
                Link::Owner(_, role) => {
                    for prior in &self.scratch.links[..index] {
                        self.work.spend()?;
                        if let Link::Owner(other, _) = prior {
                            require(*other != k)?;
                        }
                    }
                    match k.0 {
                        Kind::Result => require(match role {
                            0 => node.status == 1,
                            1 => matches!(node.status, 1 | 4),
                            2 => node.status == 2,
                            3 => node.status == 3,
                            _ => false,
                        })?,
                        Kind::Owner => {
                            require(role == 0 && node.status == 0)?;
                            let activation =
                                self.node(node.parent.ok_or(Incomplete::InvalidState)?)?;
                            require(activation.status < 2)?;
                        }
                        Kind::Sensor => require(node.status == 1)?,
                        Kind::Leaf => require(node.status != 2)?,
                        _ => return Err(Incomplete::InvalidState),
                    }
                }
                _ => (),
            }
        }
        for index in 0..self.scratch.nodes.len() {
            self.work.spend()?;
            let node = self.scratch.nodes[index];
            if node.key.0 == Kind::Logical {
                let frame = self.node(node.edge.ok_or(Incomplete::InvalidState)?)?;
                require(frame.a == node.a && frame.origin == node.origin)?;
                let parent = if let Some(parent) = node.parent {
                    self.node(parent)?.edge
                } else {
                    None
                };
                require(frame.parent == parent)?;
            }
            if node.key.0 == Kind::Frame && node.declared != self.stamp {
                self.scratch.nodes[index].status = 1;
            }
            if node.key.0 == Kind::Result && node.born {
                let mut owners = 0;
                let mut parents = 0;
                for link in &self.scratch.links {
                    self.work.spend()?;
                    match link {
                        Link::Owner(k, role)
                            if *k == node.key && !(node.status == 4 && *role == 1) =>
                        {
                            owners += 1
                        }
                        Link::PayloadParent(k, _) if *k == node.key => parents += 1,
                        _ => (),
                    }
                }
                require(parents == usize::from(node.edge.is_some()))?;
                if node.status == 1
                    && (self.publication == Some(node.key.1) || self.transit == Some(node.key.1))
                {
                    require(owners <= 1)?;
                } else {
                    require(owners == usize::from(matches!(node.status, 1..=3)))?;
                }
                let mut edge = node.edge;
                let mut depth = 0;
                while let Some(parent) = edge {
                    self.work.spend()?;
                    depth += 1;
                    require(depth <= 32)?;
                    edge = self.node(parent)?.edge;
                }
            }
            if matches!(node.key.0, Kind::Sensor | Kind::Leaf) {
                let mut owners = 0;
                for link in &self.scratch.links {
                    self.work.spend()?;
                    if let Link::Owner(k, _) = link {
                        if *k == node.key {
                            owners += 1;
                        }
                    }
                }
                if node.key.0 == Kind::Sensor {
                    require(owners == usize::from(node.status == 1))?;
                } else if node.born {
                    if self.cleanup_leaf == Some(node.key.1) {
                        require(node.status != 2 && owners <= 1)?;
                    } else {
                        require(owners == usize::from(node.status != 2))?;
                    }
                }
            }
        }
        Ok(())
    }
    fn snapshot(&mut self) -> Valid<Cut> {
        let Record::Cut(cut, fuel, depth, origin, expression, terminal) = self.take()? else {
            return Err(Incomplete::InvalidState);
        };
        require(
            fuel >= self.last_fuel && fuel <= self.limits.steps && depth <= self.limits.call_depth,
        )?;
        if self.model_host.is_some() {
            require(matches!(cut, Cut::Transition | Cut::Final))?;
        }
        self.last_fuel = fuel;
        require(!self.last_terminal || terminal)?;
        self.last_depth = depth;
        self.last_terminal = terminal;
        self.span(origin)?;
        if let Some(s) = expression {
            self.span(s)?;
        }
        let Record::OriginKind(kind) = self.take()? else {
            return Err(Incomplete::InvalidState);
        };
        self.origin_pair(Origin { span: origin, kind })?;
        let Record::Inventory(stamp, slots, operands, tasks, mask) = self.take()? else {
            return Err(Incomplete::InvalidState);
        };
        require(stamp == self.snapshots && mask <= 15)?;
        require(mask & self.mask == self.mask)?;
        self.mask = mask;
        self.stamp = stamp;
        self.snapshots += 1;
        self.scratch.links.clear();
        self.scratch.slots.clear();
        self.storage = 0;
        self.held = 0;
        self.pending_states = 0;
        for node in &mut self.scratch.nodes {
            self.work.spend()?;
            if node.key.0 == Kind::Sensor {
                node.status = 0;
            }
        }
        let mut parent = None;
        let mut frame_count = 0;
        let mut slot_count = 0;
        while let Some(Record::Frame(id, function, block, statement, actual_parent, call_site)) =
            self.peek()
        {
            self.take()?;
            require(parent == actual_parent)?;
            self.span(call_site)?;
            let mut permitted = false;
            for f in self.contract.functions() {
                self.work.spend()?;
                if *f == function {
                    permitted = true;
                    break;
                }
            }
            require(permitted)?;
            let definition = self
                .contract
                .executable
                .function(function)
                .map_err(|_| Incomplete::InvalidState)?;
            let mut found = false;
            for (bid, body) in definition.blocks() {
                self.work.spend()?;
                if bid == block {
                    found = statement <= body.statements().len();
                    break;
                }
            }
            require(found)?;
            if self
                .scratch
                .find(key(Kind::Frame, id), self.work)?
                .is_none()
            {
                if let Some(parent) = parent {
                    require(
                        self.action_frame == Some(parent)
                            && self.action_callee == Some(function)
                            && self.action_callsite == Some(call_site),
                    )?;
                } else {
                    require(
                        function == self.contract.entry()
                            && call_site
                                == self.contract.executable.program.function(function).span(),
                    )?;
                }
            }
            let index = self.declaration(
                key(Kind::Frame, id),
                function.0 as u64,
                0,
                parent.map(|p| key(Kind::Frame, p)),
                None,
                true,
            )?;
            if let Some(previous) = self.scratch.nodes[index].origin {
                require(previous == call_site)?;
            }
            self.control_cursor(index, block, statement, definition.entry())?;
            self.scratch.nodes[index].origin = Some(call_site);
            self.scratch.nodes[index].block = block.0;
            self.scratch.nodes[index].statement = statement;
            parent = Some(id);
            frame_count += 1;
            let Record::FrameStorage(capacity) = self.take()? else {
                return Err(Incomplete::InvalidState);
            };
            self.capacity(capacity as u64, self.contract.schedule().machine[3])?;
            let mut local_count = 0;
            while let Some(Record::Slot(place, state)) = self.peek() {
                self.take()?;
                require(state <= 5)?;
                let ty = self.place(id, place)?;
                for prior in &self.scratch.slots {
                    self.work.spend()?;
                    require(prior.frame != id || prior.place != place)?;
                }
                if self.scratch.slots.len() == self.scratch.cap {
                    return Err(Incomplete::Records);
                }
                self.scratch.slots.push(SlotRow {
                    frame: id,
                    place,
                    state,
                    reference: None,
                    reference_root: None,
                });
                if state == 1 {
                    let value = self.stored_value()?;
                    if let Some(reference) = value.reference {
                        let (expected, root) = self.reference_origin(id, place)?;
                        require(reference == expected)?;
                        let root = self.reference_binding(id, place, reference, root)?;
                        let slot = self
                            .scratch
                            .slots
                            .last_mut()
                            .ok_or(Incomplete::InvalidState)?;
                        slot.reference = Some(reference);
                        slot.reference_root = Some(root);
                    }
                    self.expected(value, ty)?;
                }
                if matches!(ty, ValueType::SharedRef(_) | ValueType::MutableRef(_)) {
                    self.reference_history(id, place, state)?;
                }
                local_count += 1;
                slot_count += 1;
            }
            require(local_count <= capacity)?;
            self.end()?;
        }
        require(frame_count == depth && slot_count == slots)?;
        self.top_bool = None;
        self.top_result = None;
        for _ in 0..operands {
            require(self.take()? == Record::Operand)?;
            self.top_bool = match self.peek() {
                Some(Record::Bool(value)) => Some(value),
                _ => None,
            };
            self.top_result = self
                .held_value(false)?
                .handle
                .filter(|k| k.0 == Kind::Result)
                .map(|k| k.1);
        }
        let queue_start = self.at;
        let (mut previous, previous_end) = self.queue.unwrap_or((0, 0));
        self.pending_task = None;
        for _ in 0..tasks {
            let start = self.at;
            self.task(false)?;
            if previous < previous_end {
                let end = previous
                    .checked_add(self.at - start)
                    .ok_or(Incomplete::InvalidState)?;
                protection(Protection::Queue, end <= previous_end)?;
                self.same_task_records((previous, end), (start, self.at))?;
                previous = end;
            } else {
                self.bind_queued_task(start, self.at)?;
            }
            self.pending_task = Some((start, self.at));
        }
        protection(Protection::Queue, previous == previous_end)?;
        self.queue = Some((queue_start, self.at));
        for which in 0..3 {
            if mask & (1 << which) != 0 {
                self.state(which)?;
            }
        }
        require((mask & 8 != 0) == self.contract.executable.trusted.is_some())?;
        if mask & 8 != 0 {
            self.trusted_state()?;
        }
        let Record::Accounting(limits, storage) = self.take()? else {
            return Err(Incomplete::InvalidState);
        };
        require(limits == self.limits)?;
        let b = self.meta(Part::Buffers)?;
        self.shape(&b, 6)?;
        self.capacity(b[0], self.contract.schedule().machine[0])?;
        self.capacity(b[1], self.contract.schedule().machine[1])?;
        self.capacity(b[2], self.contract.schedule().machine[2])?;
        require(depth as u64 <= b[0] && tasks as u64 <= b[1] && operands as u64 <= b[2])?;
        if b[3] != ABSENT {
            self.reference(key(Kind::Frame, b[3]))?;
        }
        if b[4] != ABSENT {
            self.reference(key(Kind::Temporary, b[4]))?;
        }
        require(storage == Some(self.storage))?;
        self.end()?;
        self.finish_graph()?;
        Ok(cut)
    }
}

fn value_type(ty: ExprType) -> Option<ValueType> {
    Some(match ty {
        ExprType::Result(t) => ValueType::Result(t),
        ExprType::Bool => ValueType::Bool,
        ExprType::Range(t) => ValueType::Range(t),
        ExprType::Record(t) => ValueType::Record(t),
        ExprType::Resource(t) => ValueType::Resource(t),
        ExprType::SharedRef(t) => ValueType::SharedRef(t),
        ExprType::MutableRef(t) => ValueType::MutableRef(t),
        ExprType::IntegerLiteral => return None,
    })
}

fn validate<'a>(
    contract: &'a contract::Contract<'a>,
    capture: &Capture,
    scratch: &mut Scratch,
    work: &mut Work,
    limits: ExecutionLimits,
    outcome: Option<Result<&ExecutionResult, &ExecutionError>>,
    control: Option<&replay::Controller>,
) -> Valid {
    scratch.clear();
    let mut v = Validator {
        contract,
        records: &capture.records,
        at: 0,
        scratch,
        work,
        stamp: 0,
        realm: None,
        last_fuel: 0,
        snapshots: 0,
        trusted_count: 0,
        leaf_count: 0,
        lineage: capture.limits.lineage,
        mask: 0,
        storage: 0,
        limits,
        delivered: false,
        action_frame: None,
        action_plan: None,
        action_callee: None,
        action_callsite: None,
        leaf_birth: None,
        cleanup_leaf: None,
        publication: None,
        delivered_root: None,
        transit: None,
        transit_type: None,
        last_depth: 0,
        last_terminal: false,
        held: 0,
        pending_states: 0,
        control,
        logical_count: 0,
        model_host: None,
        last_reached: None,
        pending_task: None,
        queue: None,
        producer: None,
        top_bool: None,
        top_result: None,
    };
    let result = (|| {
        require(capture.controlled == control.is_some())?;
        if let Some(control) = control {
            require(control.bound(contract.executable, contract.entry(), limits))?;
            if control.rejected() {
                return Err(Incomplete::Admission);
            }
        }
        require(v.snapshot()? == Cut::Start)?;
        let mut last = Cut::Start;
        loop {
            if last == Cut::Final {
                break;
            }
            match v.peek() {
                Some(Record::Cut(..)) => last = v.snapshot()?,
                Some(Record::LogicalActivation(
                    id,
                    raw,
                    function,
                    parent,
                    call_site,
                    occurrence,
                )) => {
                    v.take()?;
                    require(v.model_host.is_none() && id == v.logical_count)?;
                    let control = v.control.ok_or(Incomplete::InvalidState)?;
                    require(
                        control.activation(id) == Some((function, parent, call_site, occurrence)),
                    )?;
                    if id == 0 {
                        require(
                            function == contract.entry() && parent.is_none() && occurrence == 0,
                        )?;
                    } else {
                        require(parent.is_some_and(|parent| parent < id) && occurrence > 0)?;
                    }
                    let index = v.declaration(
                        key(Kind::Logical, id as u64),
                        function.0 as u64,
                        occurrence,
                        parent.map(|id| key(Kind::Logical, id as u64)),
                        None,
                        true,
                    )?;
                    v.scratch.nodes[index].edge = Some(key(Kind::Frame, raw));
                    v.scratch.nodes[index].origin = Some(call_site);
                    v.logical_count += 1;
                }
                Some(Record::Releasing(address, kind, raw)) => {
                    v.take()?;
                    require(
                        v.model_host.is_none()
                            && kind <= 1
                            && address.phase == replay::Phase::BeforeAttempt,
                    )?;
                    v.control_address(address)?;
                    let resource = key(if kind == 0 { Kind::Leaf } else { Kind::Sensor }, raw);
                    let node = v.node(resource)?;
                    require(if kind == 0 {
                        node.born && node.status != 2
                    } else {
                        node.status == 1
                    })?;
                    let mut held = false;
                    for link in &v.scratch.links {
                        v.work.spend()?;
                        if matches!(link,Link::Owner(k,_) if *k==resource) {
                            held = true;
                        }
                    }
                    require(held)?;
                    v.control_history(Record::Releasing(address, kind, raw))?;
                }
                Some(Record::ControlledResponse(address, response)) => {
                    v.take()?;
                    require(v.model_host.is_none())?;
                    v.control_address(address)?;
                    let control = v.control.ok_or(Incomplete::InvalidState)?;
                    v.work.0 = v
                        .work
                        .0
                        .checked_sub(control.site_work() + 1)
                        .ok_or(Incomplete::Work)?;
                    control
                        .response_valid(contract, replay::Item { address, response })
                        .map_err(|_| Incomplete::InvalidState)?;
                    v.control_history(Record::ControlledResponse(address, response))?;
                }
                Some(Record::Control(address, status)) => {
                    v.take()?;
                    v.control_address(address)?;
                    require(status <= 2)?;
                    let control = v.control.ok_or(Incomplete::InvalidState)?;
                    if v.model_host.is_some() {
                        let mut failure = v.model_host.unwrap();
                        failure.phase = replay::Phase::ReleaseFailure;
                        require(status == 0 && address == failure)?;
                    }
                    if status == 0 {
                        if address.phase == replay::Phase::Reservation {
                            let Cut::Storage(bytes) = last else {
                                return Err(Incomplete::InvalidState);
                            };
                            require(
                                bytes > 0
                                    && v.storage
                                        .checked_add(bytes)
                                        .is_some_and(|n| n <= limits.working_storage),
                            )?;
                        }
                        if address.phase == replay::Phase::BeforeHandoff {
                            let replay::Site::Invocation(plan) = address.site else {
                                return Err(Incomplete::InvalidState);
                            };
                            require(v.action_plan == Some(plan))?;
                        }
                        v.last_reached = Some(address);
                        v.control_history(Record::Control(address, 0))?;
                    } else if status == 1 {
                        v.work.0 = v
                            .work
                            .0
                            .checked_sub(control.input_work())
                            .ok_or(Incomplete::Work)?;
                        require(control.consumed(address))?;
                        v.control_history(Record::Control(address, 1))?;
                    } else {
                        v.work.0 = v
                            .work
                            .0
                            .checked_sub(control.input_work())
                            .ok_or(Incomplete::Work)?;
                        require(v.last_reached == Some(address) && control.modeled(address))?;
                        v.model_host = Some(address);
                    }
                }
                Some(Record::Executing) => {
                    require(v.model_host.is_none())?;
                    v.transition_complete()?;
                    v.take()?;
                    let Some(Record::Continuation(frame)) = v.peek() else {
                        return Err(Incomplete::InvalidState);
                    };
                    v.action_frame = Some(frame);
                    let expected = v.pending_task.take();
                    let start = v.at;
                    v.task(true)?;
                    if let Some(expected) = expected {
                        v.same_task_records(expected, (start, v.at))?;
                        let (from, _) = v.queue.ok_or(Incomplete::InvalidState)?;
                        v.queue = Some((from, expected.0));
                    } else {
                        // The initial Drive can be pushed and popped between
                        // the last admission snapshot and its first action.
                        let node = v.node(key(Kind::Frame, frame))?;
                        protection(
                            Protection::Queue,
                            v.producer.is_none()
                                && v.records.get(start + 1).is_some_and(
                                    |r| matches!(r, Record::Meta(Part::Task, d) if d[0] == 0),
                                )
                                && node.a == contract.entry().0 as u64
                                && node.block == 0
                                && node.statement == 0,
                        )?;
                    }
                    v.transition_producer(start, v.at)?;
                    v.finish_graph()?;
                }
                Some(Record::Publication(id, delivered)) => {
                    v.take()?;
                    let root = id as u64;
                    if delivered {
                        require(v.publication == Some(root))?;
                        v.publication = None;
                        v.delivered_root = Some(root);
                    } else {
                        require(
                            v.publication.is_none()
                                && v.delivered_root.is_none()
                                && v.last_depth == 0,
                        )?;
                        let node = v.node(key(Kind::Result, root))?;
                        let declared = contract
                            .executable
                            .program
                            .function(contract.entry())
                            .as_ordinary()
                            .ok_or(Incomplete::InvalidState)?
                            .return_type;
                        require(node.status == 1 && node.edge.is_none() && node.ty == declared)?;
                        v.publication = Some(root);
                    }
                }
                Some(Record::CleanupLeaf(id, finished)) => {
                    v.take()?;
                    if finished {
                        require(v.cleanup_leaf == Some(id))?;
                        v.cleanup_leaf = None;
                    } else {
                        require(v.cleanup_leaf.is_none())?;
                        let node = v.node(key(Kind::Leaf, id))?;
                        require(node.born && node.status != 2)?;
                        v.cleanup_leaf = Some(id);
                    }
                }
                Some(Record::Transit(id, finished)) => {
                    v.take()?;
                    let root = id as u64;
                    if finished {
                        require(v.transit == Some(root))?;
                        v.finish_graph()?;
                        v.transit = None;
                        v.transit_type = None;
                    } else {
                        require(v.transit.is_none())?;
                        match v.take()? {
                            Record::Operand => v.transit_type = None,
                            Record::Destination(frame, place, None) => {
                                v.transit_type = Some(v.place(frame, place)?)
                            }
                            _ => return Err(Incomplete::InvalidState),
                        }
                        v.transit = Some(root);
                    }
                }
                _ => break,
            }
        }
        require(last == Cut::Final && v.snapshots == capture.snapshots)?;
        v.delivered = true;
        match v.take()? {
            Record::Delivered => {
                require(
                    v.last_depth == 0
                        && !v.last_terminal
                        && v.held == 0
                        && v.pending_states == 0
                        && v.publication.is_none()
                        && v.transit.is_none()
                        && v.cleanup_leaf.is_none(),
                )?;
                for node in &v.scratch.nodes {
                    v.work.spend()?;
                    if node.key.0 == Kind::Result && node.born {
                        require(node.status >= 4)?;
                    }
                    if node.key.0 == Kind::Temporary {
                        require(node.status == 3)?;
                    }
                }
                let actual = outcome
                    .ok_or(Incomplete::Pending)?
                    .map_err(|_| Incomplete::InvalidState)?;
                let start = v.at;
                let definition = contract
                    .executable
                    .program
                    .function(contract.entry())
                    .as_ordinary()
                    .ok_or(Incomplete::InvalidState)?;
                if let Some(ValueType::Result(ty)) = definition.return_type {
                    let root = v.delivered_root.ok_or(Incomplete::InvalidState)?;
                    let node = v.node(key(Kind::Result, root))?;
                    require(node.status == 5 && node.ty == Some(ValueType::Result(ty)))?;
                } else {
                    require(v.delivered_root.is_none())?;
                }
                match definition.return_type {
                    Some(ty) => {
                        let value = v.value(0, false)?;
                        v.expected(value, ty)?;
                    }
                    None => require(v.take()? == Record::NoValue)?,
                }
                let end = v.at;
                v.at = start;
                match actual {
                    ExecutionResult::NoValue => require(v.take()? == Record::NoValue)?,
                    ExecutionResult::Value(value) => v.public_value(value)?,
                }
                require(v.at == end)?;
            }
            Record::Stop(class, origin, expression) => {
                require(v.delivered_root.is_none())?;
                require((1..=4).contains(&class))?;
                v.span(origin)?;
                if let Some(s) = expression {
                    v.span(s)?;
                }
                let actual = outcome
                    .ok_or(Incomplete::Pending)?
                    .err()
                    .ok_or(Incomplete::InvalidState)?;
                require(origin == actual.origin && expression == actual.expression)?;
                let actual_class = match actual.kind {
                    ErrorKind::RangeTrap { .. } => 1,
                    ErrorKind::ConfiguredLimit(_) => 2,
                    ErrorKind::Integrity(_) => 3,
                    ErrorKind::Host(_) => 4,
                    _ => 0,
                };
                require(class == actual_class)?;
                if class == 4 {
                    let modeled = v
                        .model_host
                        .is_some_and(|address| control.is_some_and(|c| c.modeled(address)));
                    if !modeled || actual.kind != ErrorKind::Host("modeled abstract Host") {
                        return Err(Incomplete::UnmatchedPhysicalFailure);
                    }
                } else {
                    require(v.model_host.is_none())?;
                }
                if class == 1 {
                    let Record::Trap(ty, min, max, computed) = v.take()? else {
                        return Err(Incomplete::InvalidState);
                    };
                    let r = contract
                        .executable
                        .program
                        .ranges()
                        .get(ty.0)
                        .ok_or(Incomplete::InvalidState)?;
                    require(
                        r.min == min
                            && r.max == max
                            && (computed < i128::from(min) || computed > i128::from(max)),
                    )?;
                    require(
                        actual.kind
                            == ErrorKind::RangeTrap {
                                ty,
                                min,
                                max,
                                computed,
                            },
                    )?;
                } else if class == 2 {
                    let Record::Limit(name) = v.take()? else {
                        return Err(Incomplete::InvalidState);
                    };
                    require(!name.is_empty() && name.len() <= 128)?;
                    require(actual.kind == ErrorKind::ConfiguredLimit(name))?;
                }
            }
            _ => return Err(Incomplete::InvalidState),
        }
        v.end()?;
        v.finish_controls()?;
        require(v.at == v.records.len())
    })();
    #[cfg(test)]
    if let Err(reason) = result {
        eprintln!(
            "K35 independent validation at record {}: {:?}",
            v.at, reason
        );
    }
    result
}

fn task_keys(d: [u64; 8]) -> impl Iterator<Item = Key> {
    let mut keys = [None; 2];
    keys[0] = match d[0] {
        2 | 5 => Some(key(Kind::Temporary, d[2])),
        3 | 4 => Some(key(Kind::Temporary, d[1])),
        12 => Some(key(Kind::Replacement, d[1])),
        16 | 17 => Some(key(Kind::Pending, d[1])),
        18 | 19 => Some(key(Kind::Staging, d[1])),
        20 => Some(key(Kind::Result, d[1])),
        22 => Some(key(Kind::Temporary, d[3])),
        _ => None,
    };
    keys.into_iter().flatten()
}

// M5 validation and correspondence are kept separate from capture. No capture
// success flag is accepted as a substitute for parsing and validating evidence.
pub fn compare(a: &Session<'_>, b: &Session<'_>, limits: ComparisonLimits) -> Comparison {
    match compare_inner(a, b, limits) {
        Ok(result) => result,
        Err(reason) => Comparison::Inconclusive(reason),
    }
}
fn compare_inner(a: &Session<'_>, b: &Session<'_>, limits: ComparisonLimits) -> Valid<Comparison> {
    if !a.contract.same_approval(b.contract) || a.entry != b.entry {
        return Err(Incomplete::Authority);
    }
    let mut common_a = a.machine().limits;
    let mut common_b = b.machine().limits;
    common_a.rendered_output = 1;
    common_b.rendered_output = 1;
    if common_a != common_b {
        return Err(Incomplete::Admission);
    }
    let mut scratch = Scratch::new(limits, a.contract.schedule())?;
    let mut work = Work(limits.work);
    let control_a = a.machine().control.borrow();
    let control_b = b.machine().control.borrow();
    match (control_a.as_ref(), control_b.as_ref()) {
        (None, None) => (),
        (Some(left), Some(right)) => {
            let amount = left
                .input_work()
                .checked_add(right.input_work())
                .ok_or(Incomplete::Work)?;
            work.0 = work.0.checked_sub(amount).ok_or(Incomplete::Work)?;
            if !left.same_input(right) {
                return Err(Incomplete::Admission);
            }
        }
        _ => return Err(Incomplete::Admission),
    }

    let (Some(authority_a), Some(authority_b)) = (&a.authority, &b.authority) else {
        return Err(Incomplete::UncoveredState);
    };
    let amount = authority_a
        .comparison_work()
        .checked_add(authority_b.comparison_work())
        .ok_or(Incomplete::Work)?;
    work.0 = work.0.checked_sub(amount).ok_or(Incomplete::Work)?;
    if !authority_a.equivalent(authority_b)? {
        return Err(Incomplete::InvalidState);
    }
    let left = a.capture();
    let right = b.capture();
    for capture in [&*left, &*right] {
        if let Some(reason) = capture.incomplete {
            return Err(reason);
        }
        if !capture.finished {
            return Err(Incomplete::Pending);
        }
        protection(
            Protection::Seal,
            capture.sealed_count == Some(capture.records.len()),
        )?;
        if capture.records.len() > limits.records {
            return Err(Incomplete::Records);
        }
    }
    validate(
        a.contract,
        &left,
        &mut scratch,
        &mut work,
        a.machine().limits,
        a.primary(),
        control_a.as_ref(),
    )?;
    validate(
        b.contract,
        &right,
        &mut scratch,
        &mut work,
        b.machine().limits,
        b.primary(),
        control_b.as_ref(),
    )?;
    correspondence(&left.records, &right.records, &mut scratch, &mut work)
}

fn meta_kind(part: Part, d: [u64; 8], index: usize) -> Option<Kind> {
    use Kind::*;
    match (part, index) {
        (Part::ResultHandle, 0)
        | (Part::ResultCell, 0)
        | (Part::ResultCell, 6)
        | (Part::ResultMatch, 1)
        | (Part::CompletionOutcome, 0)
        | (Part::ResultTicket, 0) => Some(Result),
        (Part::ResultCell, 3) if d[2] == 2 => Some(Production),
        (Part::ResultTicket, 2) if d[1] == 2 => Some(Production),
        (Part::TemporaryHandle, 0)
        | (Part::TemporaryActivation, 0)
        | (Part::TemporaryActivation, 4)
        | (Part::TemporaryOwner, 0)
        | (Part::PendingRecord, 4)
        | (Part::PendingRecord, 6)
        | (Part::InertRecord, 5)
        | (Part::InertRecord, 7)
        | (Part::Buffers, 4)
        | (Part::Completion, 3) => Some(Temporary),
        (Part::SensorHandle, 1)
        | (Part::Token, 1)
        | (Part::Production, 6)
        | (Part::Completion, 5) => Some(Sensor),
        (Part::Leaf, 2) => Some(Leaf),
        (Part::ResultMatch, 0)
        | (Part::ResultMatchFrame, 0)
        | (Part::TemporaryActivation, 1)
        | (Part::ResourceFrame, 0)
        | (Part::Scope, 0)
        | (Part::GenerationSlot, 0)
        | (Part::PendingRecord, 1)
        | (Part::Replacement, 1)
        | (Part::InertRecord, 1)
        | (Part::Buffers, 3)
        | (Part::Completion, 0) => Some(Frame),
        (Part::Scope, 2) | (Part::Generation, 0) | (Part::Replacement, 6) => Some(Scope),
        (Part::Generation, 2) | (Part::GenerationSlot, 2) | (Part::Replacement, 7) => {
            Some(Generation)
        }
        (Part::PendingRecord, 0) => Some(Pending),
        (Part::Replacement, 0) => Some(Replacement),
        (Part::InertRecord, 0) => Some(Staging),
        (Part::TrustedState, 0) | (Part::Token, 0) | (Part::Production, 1) => Some(Realm),
        (Part::Receipt | Part::PendingReceipt, 0)
        | (Part::Production, 4)
        | (Part::Completion, 1) => Some(Invocation),
        (Part::Production, 0) => Some(Production),
        (Part::Production, 2) => Some(Result),
        (Part::Task, 1) => match d[0] {
            3 | 4 => Some(Temporary),
            12 => Some(Replacement),
            16 | 17 => Some(Pending),
            18 | 19 => Some(Staging),
            20 => Some(Result),
            _ => None,
        },
        (Part::Task, 2) if matches!(d[0], 2 | 5) => Some(Temporary),
        (Part::Task, 3) if d[0] == 22 => Some(Temporary),
        _ => None,
    }
}
fn pair_optional(
    s: &mut Scratch,
    w: &mut Work,
    kind: Kind,
    a: u64,
    b: u64,
    stamp: usize,
) -> Valid<bool> {
    if a == ABSENT || b == ABSENT {
        Ok(a == b)
    } else {
        s.pair(key(kind, a), key(kind, b), stamp, w)
    }
}
fn equal_record(a: Record, b: Record, s: &mut Scratch, w: &mut Work, stamp: usize) -> Valid<bool> {
    match (a, b) {
        (Record::Frame(x, f, block, at, p, origin), Record::Frame(y, g, other, to, q, source)) => {
            if f != g || block != other || at != to || origin != source {
                return Ok(false);
            }
            Ok(s.pair(key(Kind::Frame, x), key(Kind::Frame, y), stamp, w)?
                && pair_optional(
                    s,
                    w,
                    Kind::Frame,
                    p.unwrap_or(ABSENT),
                    q.unwrap_or(ABSENT),
                    stamp,
                )?)
        }
        (Record::Reference(k, t, x, p), Record::Reference(l, u, y, q)) => {
            if k != l || t != u || p != q {
                return Ok(false);
            }
            s.pair(key(Kind::Frame, x), key(Kind::Frame, y), stamp, w)
        }
        (Record::Continuation(x), Record::Continuation(y)) => {
            s.pair(key(Kind::Frame, x), key(Kind::Frame, y), stamp, w)
        }
        (Record::Publication(x, a), Record::Publication(y, b))
        | (Record::Transit(x, a), Record::Transit(y, b)) => {
            if a != b {
                return Ok(false);
            }
            s.pair(
                key(Kind::Result, x as u64),
                key(Kind::Result, y as u64),
                stamp,
                w,
            )
        }
        (Record::Destination(x, p, f), Record::Destination(y, q, g)) => {
            if p != q || f != g {
                return Ok(false);
            }
            s.pair(key(Kind::Frame, x), key(Kind::Frame, y), stamp, w)
        }
        (Record::TemporaryBegin(x, f, r, p), Record::TemporaryBegin(y, g, t, q)) => {
            if r != t {
                return Ok(false);
            }
            Ok(
                s.pair(key(Kind::Temporary, x), key(Kind::Temporary, y), stamp, w)?
                    && s.pair(key(Kind::Frame, f), key(Kind::Frame, g), stamp, w)?
                    && pair_optional(
                        s,
                        w,
                        Kind::Temporary,
                        p.unwrap_or(ABSENT),
                        q.unwrap_or(ABSENT),
                        stamp,
                    )?,
            )
        }
        (Record::TemporaryRegister(x, i, site), Record::TemporaryRegister(y, j, other)) => {
            if site != other {
                return Ok(false);
            }
            Ok(
                s.pair(key(Kind::Temporary, x), key(Kind::Temporary, y), stamp, w)?
                    && s.pair(
                        Key(Kind::Owner, x, i as u64),
                        Key(Kind::Owner, y, j as u64),
                        stamp,
                        w,
                    )?,
            )
        }
        (Record::TemporaryTransfer(x, i, at), Record::TemporaryTransfer(y, j, to)) => {
            if at != to {
                return Ok(false);
            }
            Ok(
                s.pair(key(Kind::Temporary, x), key(Kind::Temporary, y), stamp, w)?
                    && s.pair(
                        Key(Kind::Owner, x, i as u64),
                        Key(Kind::Owner, y, j as u64),
                        stamp,
                        w,
                    )?,
            )
        }
        (Record::TemporaryCleanup(x, i), Record::TemporaryCleanup(y, j)) => {
            Ok(
                s.pair(key(Kind::Temporary, x), key(Kind::Temporary, y), stamp, w)?
                    && s.pair(
                        Key(Kind::Owner, x, i as u64),
                        Key(Kind::Owner, y, j as u64),
                        stamp,
                        w,
                    )?,
            )
        }
        (Record::TemporaryRetire(x), Record::TemporaryRetire(y)) => {
            s.pair(key(Kind::Temporary, x), key(Kind::Temporary, y), stamp, w)
        }
        (
            Record::LogicalActivation(id, frame, function, parent, span, occurrence),
            Record::LogicalActivation(other, raw, target, ancestor, origin, count),
        ) => Ok(id == other
            && function == target
            && parent == ancestor
            && span == origin
            && occurrence == count
            && s.pair(key(Kind::Frame, frame), key(Kind::Frame, raw), stamp, w)?),
        (Record::Releasing(address, kind, raw), Record::Releasing(other, tag, value)) => {
            let k = if kind == 0 { Kind::Leaf } else { Kind::Sensor };
            Ok(address == other && kind == tag && s.pair(key(k, raw), key(k, value), stamp, w)?)
        }
        (Record::CleanupLeaf(x, finished), Record::CleanupLeaf(y, other)) => {
            Ok(finished == other && s.pair(key(Kind::Leaf, x), key(Kind::Leaf, y), stamp, w)?)
        }
        (Record::ResourceEvent(kind, x), Record::ResourceEvent(other, y)) => {
            if kind != other {
                return Ok(false);
            }
            let k = if kind == 4 { Kind::Frame } else { Kind::Leaf };
            s.pair(key(k, x), key(k, y), stamp, w)
        }
        (Record::TrustedEvent(x), Record::TrustedEvent(y)) => {
            use trusted::Event::*;
            let (kind, a, b, static_equal) = match (x, y) {
                (Attempt(a, op), Attempt(b, other)) => (Kind::Invocation, a, b, op == other),
                (Received(a, tag), Received(b, other)) => (Kind::Invocation, a, b, tag == other),
                (Published(a, tag), Published(b, other))
                | (FinishProtected(a, tag), FinishProtected(b, other)) => {
                    (Kind::Invocation, a, b, tag == other)
                }
                (FinishPrefix(a, prefix), FinishPrefix(b, other)) => {
                    (Kind::Invocation, a, b, prefix == other)
                }
                (ReleaseAttempt(a, op), ReleaseAttempt(b, other)) => {
                    (Kind::Sensor, a, b, op == other)
                }
                (ReleaseComplete(a), ReleaseComplete(b))
                | (ReleaseFailure(a), ReleaseFailure(b)) => (Kind::Sensor, a, b, true),
                _ => return Ok(false),
            };
            if !static_equal {
                return Ok(false);
            }
            s.pair(key(kind, a), key(kind, b), stamp, w)
        }
        (Record::Meta(part, x), Record::Meta(other, y)) => {
            if part != other {
                return Ok(false);
            }
            for index in 0..8 {
                w.spend()?;
                let owner = match (part, index) {
                    (Part::TemporaryHandle | Part::TemporaryOwner, 1) => {
                        Some((x[0], x[1], y[0], y[1]))
                    }
                    (Part::Completion, 4) => Some((x[3], x[4], y[3], y[4])),
                    _ => None,
                };
                if let Some((a, i, b, j)) = owner {
                    if !s.pair(Key(Kind::Owner, a, i), Key(Kind::Owner, b, j), stamp, w)? {
                        return Ok(false);
                    }
                } else if let Some(kind) = meta_kind(part, x, index) {
                    if meta_kind(part, y, index) != Some(kind)
                        || !pair_optional(s, w, kind, x[index], y[index], stamp)?
                    {
                        return Ok(false);
                    }
                } else if x[index] != y[index] {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        (Record::Accounting(mut x, storage), Record::Accounting(mut y, other)) => {
            // Presentation allowance is not a common logical execution counter.
            x.rendered_output = 1;
            y.rendered_output = 1;
            Ok(x == y && storage == other)
        }
        _ => Ok(a == b),
    }
}
fn correspondence(
    a: &[Record],
    b: &[Record],
    scratch: &mut Scratch,
    work: &mut Work,
) -> Valid<Comparison> {
    scratch.pairs.clear();
    let mut stamp = 0;
    for (index, (x, y)) in a.iter().copied().zip(b.iter().copied()).enumerate() {
        work.spend()?;
        if let Record::Inventory(ordinal, ..) = x {
            stamp = ordinal;
        }
        if !equal_record(x, y, scratch, work, stamp)? {
            return Ok(Comparison::Different { record: index });
        }
    }
    if a.len() != b.len() {
        Ok(Comparison::Different {
            record: a.len().min(b.len()),
        })
    } else {
        Ok(Comparison::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn run<'a>(contract: &'a contract::Contract<'a>) -> Session<'a> {
        let mut session = Session::new(
            contract,
            ExecutionLimits::default(),
            Limits {
                records: 262_144,
                work: 10_000_000,
                storage: 67_108_864,
                lineage: 128,
            },
        )
        .unwrap();
        session.advance().unwrap();
        session
    }
    #[test]
    fn ordinary_complete_evidence_compares_equal() {
        let e =
            prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        let a = run(&c);
        let b = run(&c);
        assert_eq!(
            compare(&a, &b, ComparisonLimits::default()),
            Comparison::Equal
        );
    }
    #[test]
    fn valid_wrong_delivered_bool_is_different_from_the_independent_true_oracle() {
        let e =
            prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        let a = run(&c);
        let mut b = run(&c);
        // A locally well-typed engine defect, with its actual primary outcome
        // changed too. It is not an observer/engine-state inconsistency.
        let Engine::Ordinary { outcome, .. } = &mut b.engine else {
            unreachable!()
        };
        *outcome = Some(Ok(ExecutionResult::Value(RuntimeValue::Bool(false))));
        let mut observer = b.machine().observation.borrow_mut();
        let record = observer
            .as_mut()
            .unwrap()
            .records
            .iter_mut()
            .rev()
            .find(|r| matches!(r, Record::Bool(_)))
            .unwrap();
        *record = Record::Bool(false);
        drop(observer);
        assert_eq!(
            a.primary().unwrap().unwrap(),
            &ExecutionResult::Value(RuntimeValue::Bool(true))
        );
        assert!(matches!(
            compare(&a, &b, ComparisonLimits::default()),
            Comparison::Different { .. }
        ));
    }
    #[test]
    fn a_forged_projection_cannot_contradict_the_actual_primary_outcome() {
        let e =
            prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        let a = run(&c);
        let b = run(&c);
        let mut observer = b.machine().observation.borrow_mut();
        *observer
            .as_mut()
            .unwrap()
            .records
            .iter_mut()
            .rev()
            .find(|r| matches!(r, Record::Bool(_)))
            .unwrap() = Record::Bool(false);
        drop(observer);
        assert_eq!(
            compare(&a, &b, ComparisonLimits::default()),
            Comparison::Inconclusive(Incomplete::InvalidState)
        );
    }
    #[test]
    fn fresh_selected_realms_preserve_one_identity_correspondence() {
        use crate::trusted::{Grants, Scenario, Selection};
        let selection = Selection::sensor_demo(Scenario::Sample, Grants::ALL, &["sample"]).unwrap();
        let e = prepare_execution_with_profile(
            include_str!("../../../../examples/sensor_finish.bow"),
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
        let a = run(&c);
        let b = run(&c);
        let realm = |s: &Session<'_>| {
            s.capture()
                .records
                .iter()
                .find_map(|r| {
                    if let Record::Meta(Part::TrustedState, d) = r {
                        Some(d[0])
                    } else {
                        None
                    }
                })
                .unwrap()
        };
        assert_ne!(realm(&a), realm(&b));
        assert_eq!(
            compare(&a, &b, ComparisonLimits::default()),
            Comparison::Equal
        );
    }
    #[test]
    fn two_truncated_views_and_a_comparison_budget_never_establish_equal() {
        let e =
            prepare_execution("fn demo()->bool{return true;}", AdmissionLimits::default()).unwrap();
        let c =
            contract::Contract::extract(&e, e.entry("demo").unwrap(), contract::Limits::default())
                .unwrap();
        let mut a = Session::new(
            &c,
            ExecutionLimits::default(),
            Limits {
                records: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        let mut b = Session::new(
            &c,
            ExecutionLimits::default(),
            Limits {
                records: 1,
                ..Limits::default()
            },
        )
        .unwrap();
        a.advance().unwrap();
        b.advance().unwrap();
        assert!(matches!(
            compare(&a, &b, ComparisonLimits::default()),
            Comparison::Inconclusive(_)
        ));
        let a = run(&c);
        let b = run(&c);
        assert_eq!(
            compare(
                &a,
                &b,
                ComparisonLimits {
                    work: 1,
                    ..ComparisonLimits::default()
                }
            ),
            Comparison::Inconclusive(Incomplete::Work)
        );
        assert_eq!(
            a.primary().unwrap().unwrap(),
            &ExecutionResult::Value(RuntimeValue::Bool(true))
        );
    }
}

#[cfg(test)]
mod oracle_tests;

#[cfg(test)]
mod identity_tests;

#[cfg(test)]
mod repair_tests;
