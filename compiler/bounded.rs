//! Resource envelopes for the existing semantic pipeline. No source semantics.
use std::cell::Cell;
pub(crate) mod result_admission;

use crate::{ast, lexer::Span};

pub const START: Span = Span {
    line: 1,
    column: 1,
    start: 0,
    end: 0,
};

/// Version-one admission limits. Reservations are conservative peak envelopes,
/// not Bowfin execution fuel. Existing check/verify do not use this contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionLimits {
    pub source_bytes: usize,
    pub tokens: usize,
    pub identifier_bytes: usize,
    pub frontend_work: u64,
    pub traversal_depth: usize,
    pub alias_depth: usize,
    pub alias_work: u64,
    pub working_storage: usize,
    pub diagnostic_storage: usize,
    pub rendered_output: usize,
}
impl AdmissionLimits {
    pub const MAX: Self = Self {
        source_bytes: 1_048_576,
        tokens: 4096,
        identifier_bytes: 256,
        frontend_work: 1_000_000_000_000_000_000,
        traversal_depth: 32,
        alias_depth: 64,
        alias_work: 10_000_000,
        working_storage: 1_073_741_824,
        diagnostic_storage: 4_194_304,
        rendered_output: 1_048_576,
    };
    pub fn validate(self) -> Result<(), ResourceFailure> {
        let m = Self::MAX;
        for (value, max, name) in [
            (self.source_bytes, m.source_bytes, "source bytes"),
            (self.tokens, m.tokens, "tokens"),
            (
                self.identifier_bytes,
                m.identifier_bytes,
                "identifier bytes",
            ),
            (self.traversal_depth, m.traversal_depth, "traversal depth"),
            (self.alias_depth, m.alias_depth, "alias depth"),
            (self.working_storage, m.working_storage, "admission storage"),
            (
                self.diagnostic_storage,
                m.diagnostic_storage,
                "diagnostic storage",
            ),
            (self.rendered_output, m.rendered_output, "rendered output"),
        ] {
            if value == 0 || value > max {
                return Err(ResourceFailure::limit(name, START));
            }
        }
        if self.frontend_work == 0 || self.frontend_work > m.frontend_work {
            return Err(ResourceFailure::limit("frontend work", START));
        }
        if self.alias_work == 0 || self.alias_work > m.alias_work {
            return Err(ResourceFailure::limit("alias work", START));
        }
        Ok(())
    }
}
impl Default for AdmissionLimits {
    fn default() -> Self {
        Self {
            source_bytes: 65_536,
            tokens: 1024,
            identifier_bytes: 128,
            frontend_work: 100_000_000_000_000,
            traversal_depth: 24,
            alias_depth: 48,
            alias_work: 1_000_000,
            working_storage: 134_217_728,
            diagnostic_storage: 1_048_576,
            rendered_output: 65_536,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceKind {
    Limit,
    Host,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceFailure {
    pub kind: ResourceKind,
    pub resource: &'static str,
    pub span: Span,
}
impl ResourceFailure {
    pub fn limit(resource: &'static str, span: Span) -> Self {
        Self {
            kind: ResourceKind::Limit,
            resource,
            span,
        }
    }
    pub fn host(resource: &'static str, span: Span) -> Self {
        Self {
            kind: ResourceKind::Host,
            resource,
            span,
        }
    }
}

/// Shared only within one preparation. Checker snapshots share counters, so
/// analysis rollback cannot refund tool work or hide a resource failure.
pub(crate) struct Context {
    pub(crate) resource_nodes: Cell<usize>,
    pub limits: AdmissionLimits,
    pub failure: Cell<Option<ResourceFailure>>,
    depth: Cell<usize>,
    alias_work: Cell<u64>,
    envelope_nodes: Cell<usize>,
    envelope_source: Cell<usize>,
    outcome_extra: Cell<usize>,
    cleanup_extra: Cell<usize>,
    catalog_storage: Cell<usize>,
    catalog_work: Cell<u64>,
    catalog_diagnostics: Cell<usize>,
    pub(crate) outcome_limit: Cell<usize>,
}
impl Context {
    pub fn new(limits: AdmissionLimits) -> Self {
        Self {
            resource_nodes: Cell::new(0),
            limits,
            failure: Cell::new(None),
            depth: Cell::new(0),
            alias_work: Cell::new(0),
            envelope_nodes: Cell::new(0),
            envelope_source: Cell::new(0),
            outcome_extra: Cell::new(0),
            cleanup_extra: Cell::new(0),
            catalog_storage: Cell::new(0),
            catalog_work: Cell::new(0),
            catalog_diagnostics: Cell::new(0),
            outcome_limit: Cell::new(0),
        }
    }
    /// Source-independent catalog/selection/approval costs. This reservation is
    /// retained in every later envelope, including an empty source program.
    pub(crate) fn reserve_catalog(
        &self,
        roots: usize,
        names: usize,
    ) -> Result<(), ResourceFailure> {
        let storage = roots
            .checked_mul(4096)
            .and_then(|n| n.checked_add(names.checked_mul(16)?))
            .and_then(|n| n.checked_add(65_536))
            .ok_or_else(|| ResourceFailure::limit("trusted catalog storage", START))?;
        let work = (roots as u64)
            .checked_mul(4096)
            .and_then(|n| n.checked_add(65_536))
            .ok_or_else(|| ResourceFailure::limit("trusted catalog work", START))?;
        let diagnostics = names
            .checked_mul(16)
            .and_then(|n| n.checked_add(4096))
            .ok_or_else(|| ResourceFailure::limit("trusted catalog diagnostics", START))?;
        self.catalog_storage.set(storage);
        self.catalog_work.set(work);
        self.catalog_diagnostics.set(diagnostics);
        self.reserve_frontend(self.envelope_nodes.get(), self.envelope_source.get())
    }
    /// Additional retained cleanup witnesses are bounded before checker snapshots.
    /// BED-034 adds explicit descriptor, sealed protocol copies, traversal and
    /// diagnostic terms on top of the unchanged predecessor catalog envelope.
    pub(crate) fn reserve_completion_catalog(
        &self,
        roots: usize,
        names: usize,
    ) -> Result<(), ResourceFailure> {
        self.reserve_catalog(roots, names)?;
        let per_site = std::mem::size_of::<crate::trusted::approval::CompletionPlan>()
            .checked_mul(3)
            .and_then(|n| {
                n.checked_add(std::mem::size_of::<crate::trusted::approval::Invocation>())
            })
            .ok_or_else(|| ResourceFailure::limit("completion approval storage", START))?;
        let storage = self
            .limits
            .tokens
            .checked_mul(per_site)
            .and_then(|n| n.checked_add(8192))
            .and_then(|n| n.checked_add(self.catalog_storage.get()))
            .ok_or_else(|| ResourceFailure::limit("completion catalog storage", START))?;
        let work = (self.limits.tokens as u64)
            .checked_mul(32)
            .and_then(|n| n.checked_add(128))
            .and_then(|n| n.checked_add(self.catalog_work.get()))
            .ok_or_else(|| ResourceFailure::limit("completion catalog work", START))?;
        let diagnostics = self
            .catalog_diagnostics
            .get()
            .checked_add(4096)
            .ok_or_else(|| ResourceFailure::limit("completion catalog diagnostics", START))?;
        self.catalog_storage.set(storage);
        self.catalog_work.set(work);
        self.catalog_diagnostics.set(diagnostics);
        self.reserve_frontend(self.envelope_nodes.get(), self.envelope_source.get())
    }

    /// Additional retained cleanup witnesses are bounded before checker snapshots.
    /// Inactive/all-empty cleanup effects retain the predecessor envelope exactly.
    pub(crate) fn reserve_cleanup_effects(
        &self,
        nodes: usize,
        holders: usize,
    ) -> Result<(), ResourceFailure> {
        self.reserve_cleanup_effects_extra(nodes, holders, 0)
    }
    pub(crate) fn reserve_cleanup_effects_extra(
        &self,
        nodes: usize,
        holders: usize,
        retained: usize,
    ) -> Result<(), ResourceFailure> {
        let n = nodes.max(self.envelope_nodes.get()).max(1);
        let entries = n.checked_mul(holders);
        let factor = self.limits.traversal_depth.checked_mul(16);
        let extra = entries
            .and_then(|e| {
                factor
                    .and_then(|d| d.checked_add(16))
                    .and_then(|d| e.checked_mul(d))
            })
            .and_then(|e| e.checked_mul(256))
            .and_then(|e| e.checked_add(retained));
        let base = factor
            .and_then(|d| d.checked_add(32))
            .and_then(|d| d.checked_mul(n))
            .and_then(|e| e.checked_mul(self.limits.identifier_bytes + 512))
            .and_then(|e| {
                self.envelope_source
                    .get()
                    .checked_mul(8)
                    .and_then(|s| e.checked_add(s))
            });
        if base
            .and_then(|b| {
                extra
                    .and_then(|e| b.checked_add(e))
                    .and_then(|b| b.checked_add(self.outcome_extra.get()))
                    .and_then(|b| b.checked_add(self.catalog_storage.get()))
            })
            .map_or(true, |e| e > self.limits.working_storage)
        {
            return Err(ResourceFailure::limit(
                "cleanup effect witness storage",
                START,
            ));
        }
        let work = (n as u64)
            .checked_pow(4)
            .and_then(|e| {
                e.checked_mul(if self.outcome_extra.get() == 0 {
                    640
                } else {
                    896
                })
            })
            .and_then(|e| e.checked_add(self.envelope_source.get() as u64))
            .and_then(|e| e.checked_add(self.catalog_work.get()));
        if work.map_or(true, |w| w > self.limits.frontend_work) {
            return Err(ResourceFailure::limit("cleanup effect witness work", START));
        }
        self.cleanup_extra
            .set(self.cleanup_extra.get().max(extra.unwrap_or(0)));
        Ok(())
    }

    // Reserve the additional finite Result DAG, checker snapshots, and retained
    // event products before ownership analysis. The existing envelope covers the
    // syntax and HIR; this reservation explicitly covers outcome-only machinery.
    pub(crate) fn reserve_outcomes(&self, holders: usize) -> Result<(), ResourceFailure> {
        let n = self.envelope_nodes.get().max(holders).max(1);
        let storage = n
            .checked_mul(n)
            .and_then(|v| v.checked_mul(512))
            .and_then(|v| {
                self.limits
                    .traversal_depth
                    .checked_mul(holders)
                    .and_then(|h| h.checked_mul(128))
                    .and_then(|h| v.checked_add(h))
            });
        let base = (self.limits.traversal_depth * 16 + 32)
            .checked_mul(n)
            .and_then(|v| v.checked_mul(self.limits.identifier_bytes + 512))
            .and_then(|v| v.checked_add(self.envelope_source.get() * 8));
        if base
            .and_then(|b| {
                storage
                    .and_then(|v| b.checked_add(v))
                    .and_then(|v| v.checked_add(self.cleanup_extra.get()))
                    .and_then(|v| v.checked_add(self.catalog_storage.get()))
            })
            .map_or(true, |v| v > self.limits.working_storage)
        {
            return Err(ResourceFailure::limit(
                "Result outcome storage reservation",
                START,
            ));
        }
        if (n as u64)
            .checked_pow(4)
            .and_then(|v| {
                v.checked_mul(if self.cleanup_extra.get() == 0 {
                    256
                } else {
                    896
                })
            })
            .and_then(|v| v.checked_add(self.envelope_source.get() as u64))
            .and_then(|v| v.checked_add(self.catalog_work.get()))
            .map_or(true, |v| v > self.limits.frontend_work)
        {
            return Err(ResourceFailure::limit(
                "Result outcome work reservation",
                START,
            ));
        }
        self.outcome_extra.set(storage.unwrap());
        self.outcome_limit.set(n.checked_mul(n).unwrap());
        Ok(())
    }
    pub(crate) fn resource_node(&self, span: Span) -> Result<(), ResourceFailure> {
        let next = self
            .resource_nodes
            .get()
            .checked_add(1)
            .ok_or_else(|| ResourceFailure::limit("resource representation nodes", span))?;
        if next > self.limits.tokens {
            return Err(ResourceFailure::limit(
                "resource representation nodes",
                span,
            ));
        }
        self.resource_nodes.set(next);
        Ok(())
    }
    pub fn fail(&self, failure: ResourceFailure) {
        if self.failure.get().is_none() {
            self.failure.set(Some(failure));
        }
    }
    pub fn enter(&self, span: Span) -> Result<Depth<'_>, ()> {
        let Some(next) = self.depth.get().checked_add(1) else {
            self.fail(ResourceFailure::limit("parser traversal depth", span));
            return Err(());
        };
        if next > self.limits.traversal_depth {
            self.fail(ResourceFailure::limit("parser traversal depth", span));
            return Err(());
        }
        self.depth.set(next);
        Ok(Depth(self))
    }
    pub fn lineage(&self, depth: usize, span: Span) -> bool {
        if depth > self.limits.alias_depth {
            self.fail(ResourceFailure::limit("alias lineage depth", span));
            return false;
        }
        let Some(work) = self.alias_work.get().checked_add(1) else {
            self.fail(ResourceFailure::limit("alias lineage work", span));
            return false;
        };
        if work > self.limits.alias_work {
            self.fail(ResourceFailure::limit("alias lineage work", span));
            return false;
        }
        self.alias_work.set(work);
        true
    }
    /// Reserve before entering unchanged resolver/typer/checker/lowerer code.
    /// n bounds all source nodes, tables and CFG blocks up to a constant factor.
    /// Recursive visitors are protected by parser construction depth; lineage
    /// has separate actual counters. The quartic work reservation covers repeated
    /// suffix walks, table scans, tree comparisons and nested checker snapshots.
    /// Peak snapshots/scopes are bounded by depth, each with O(n) entries/names.
    pub fn reserve_frontend(&self, n: usize, source: usize) -> Result<(), ResourceFailure> {
        let work = (n as u64)
            .checked_pow(4)
            .and_then(|v| v.checked_mul(128))
            .and_then(|v| v.checked_add(source as u64))
            .and_then(|v| v.checked_add(self.catalog_work.get()));
        if work.map_or(true, |v| v > self.limits.frontend_work) {
            return Err(ResourceFailure::limit("frontend work reservation", START));
        }
        let storage = self
            .limits
            .traversal_depth
            .checked_mul(16)
            .and_then(|v| v.checked_add(32))
            .and_then(|v| v.checked_mul(n))
            .and_then(|v| v.checked_mul(self.limits.identifier_bytes + 512))
            .and_then(|v| source.checked_mul(8).and_then(|s| v.checked_add(s)))
            .and_then(|v| v.checked_add(self.catalog_storage.get()));
        if storage.map_or(true, |v| v > self.limits.working_storage) {
            return Err(ResourceFailure::limit(
                "frontend storage reservation",
                START,
            ));
        }
        let diagnostics = n
            .checked_mul(self.limits.identifier_bytes + 128)
            .and_then(|v| v.checked_mul(4))
            .and_then(|v| v.checked_add(512))
            .and_then(|v| v.checked_add(self.catalog_diagnostics.get()));
        if diagnostics.map_or(true, |v| v > self.limits.diagnostic_storage) {
            return Err(ResourceFailure::limit(
                "diagnostic storage reservation",
                START,
            ));
        }
        self.envelope_nodes.set(self.envelope_nodes.get().max(n));
        self.envelope_source
            .set(self.envelope_source.get().max(source));
        Ok(())
    }
    pub fn expression(&self, expression: &ast::Expr) -> Result<(), ()> {
        fn visit(e: &ast::Expr, d: usize, c: &Context) -> Result<(), ()> {
            if d > c.limits.traversal_depth {
                c.fail(ResourceFailure::limit(
                    "expression construction depth",
                    e.span,
                ));
                return Err(());
            }
            match &e.kind {
                ast::ExprKind::Binary { left, right, .. } => {
                    visit(left, d + 1, c)?;
                    visit(right, d + 1, c)?;
                }
                ast::ExprKind::Unary { operand, .. } => visit(operand, d + 1, c)?,
                ast::ExprKind::ResultConstruct { payload, .. } => visit(payload, d + 1, c)?,
                ast::ExprKind::Call { arguments, .. } => {
                    for e in arguments {
                        visit(e, d + 1, c)?;
                    }
                }
                ast::ExprKind::RecordConstruct { fields, .. } => {
                    for f in fields {
                        visit(&f.value, d + 1, c)?;
                    }
                }
                ast::ExprKind::IfValue {
                    condition,
                    then_value,
                    else_value,
                } => {
                    visit(condition, d + 1, c)?;
                    visit(then_value, d + 1, c)?;
                    visit(else_value, d + 1, c)?;
                }
                _ => {}
            }
            Ok(())
        }
        visit(expression, 1, self)
    }
}
pub(crate) struct Depth<'a>(&'a Context);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.depth.set(self.0.depth.get() - 1);
    }
}
