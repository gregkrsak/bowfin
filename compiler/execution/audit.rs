//! Read-only account of sealed authority. Graph reachability is structural, not
//! an effects filter or observed fixture trace. It creates no execution authority.
use super::*;
use crate::effects::calls::Callee;
use std::fmt::Write;

#[cfg(test)]
thread_local! {static FAIL_SCRATCH: std::cell::Cell<bool> = const {std::cell::Cell::new(false)};}

struct Work(u64);
impl Work {
    fn charge(&mut self, n: usize) -> Result<(), ExecutionError> {
        self.0 = self.0.checked_sub(n as u64).ok_or_else(|| {
            ExecutionError::new(ErrorKind::ConfiguredLimit("authority audit work"), START)
        })?;
        Ok(())
    }
}
pub(super) fn render(
    e: &CheckedExecutable,
    limits: ExecutionLimits,
    retained: usize,
) -> Result<String, ExecutionError> {
    limits.validate()?;
    e.validate()?;
    let t = e.trusted.as_ref().ok_or_else(|| {
        ExecutionError::integrity("authority audit requires an explicitly selected profile")
    })?;
    let count = e.program.functions().len();
    // Output is conservatively reserved in full alongside the graph and any
    // stopped session. Every output byte and bounded metadata scan is charged.
    let storage = retained
        .checked_add(count)
        .and_then(|n| n.checked_add(std::mem::size_of::<render::Output>()))
        .and_then(|n| n.checked_add(limits.rendered_output));
    if storage.map_or(true, |n| n > limits.working_storage) {
        return Err(ExecutionError::new(
            ErrorKind::ConfiguredLimit("authority audit storage"),
            START,
        ));
    }
    let mut work = Work(limits.steps);
    work.charge(limits.rendered_output)?;
    work.charge(count)?;
    let mut reachable = Vec::new();
    #[cfg(test)]
    if count > 0 && FAIL_SCRATCH.with(std::cell::Cell::get) {
        return Err(ExecutionError::new(
            ErrorKind::Host("authority reachability allocation"),
            START,
        ));
    }
    reachable.try_reserve_exact(count).map_err(|_| {
        ExecutionError::new(ErrorKind::Host("authority reachability allocation"), START)
    })?;
    reachable.resize(count, 0u8);
    for root in &t.roots {
        work.charge(1)?;
        reachable[root.0] = 1;
    }
    // At most count passes; recursion only visits canonical IDs. All ordinary
    // edges, empty contracts and alternatives are already sealed in this table.
    for _ in 0..count {
        let mut changed = false;
        for edge in &t.edges {
            work.charge(1)?;
            if reachable[edge.caller.0] != 0 {
                if let Callee::Source(id) = edge.callee {
                    if reachable[id.0] == 0 {
                        reachable[id.0] = 1;
                        changed = true;
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    let mut used = [false; 4];
    for edge in &t.edges {
        work.charge(1)?;
        if reachable[edge.caller.0] != 0 {
            if let Callee::Trusted(id) = edge.callee {
                for (index, op) in t.catalog.operations.iter().enumerate() {
                    work.charge(1)?;
                    used[index] |= op.id == id;
                }
            }
        }
    }
    let mut out = render::Output::new(limits.rendered_output)?;
    let _ = writeln!(
        out,
        "authority account v1\ntarget={}\ncompiler={}\ncatalog={}\nprofile={}\nassumptions={}",
        t.target, t.compiler, t.catalog_identity, t.profile_identity, t.assumptions
    );
    let _ = writeln!(
        out,
        "configuration: scenario={:?}; live-sensor-limit={}; selection=fixed deterministic fixture",
        t.catalog.selection.scenario(),
        t.catalog.selection.sensor_limit()
    );
    let _=writeln!(out,"contracts: Reading=nominal range 0..4095; SensorError=nominal range 1..3; open Error={{1:no selected sensor,3:recoverable fixture I/O}}; read Error={{2:no sample,3:recoverable fixture I/O}}; read Error preserves owner");
    let _=writeln!(out,"completion contract: sensor_finish consumes exact owned Sensor; FinishReceipt=nominal range 1..1, Copy status data, source-formable and not proof or authority; only Ok(FinishReceipt(1)) or Error(SensorError(3)); Host/Limit/Integrity never become Error; partial-prefix Error remains Error");
    let _=writeln!(out,"completion protocol is static possible behavior, never an observed runtime trace: completion attempt/success or recoverable Error, protected outcome, fixed release attempt/complete/failure, then inner delivery only after release succeeds; any terminal stop retains prefix, protected outcome and remaining duty without retry/resume/rollback/compensation");
    let _=writeln!(out,"assumptions: fixed in-process demonstration binding, not a physical device; shared observation cannot mutate Bowfin-visible Sensor state; release has no source Result, flush, commit, durability or completion promise");
    for root in &t.roots {
        work.charge(1)?;
        let _ = writeln!(
            out,
            "root: function={} name={}",
            root.0,
            e.program.functions()[root.0].name()
        );
    }
    for (index, op) in t.catalog.operations.iter().enumerate() {
        work.charge(1)?;
        let _=writeln!(out,"operation: id={:?}; source={:?}; class={:?}; binding={:?}; granted={}; structurally-used={}; parameter={:?}; success={:?}; failure={:?}; Result={:?}; effects={}; catalog-origin={:?}",op.id,op.source_name,op.class,op.binding,t.catalog.granted(op.id),used[index],op.parameter,op.success,op.failure,op.result,op.effects,op.origin);
    }
    let _=writeln!(out,"opaque Resource: id={:?}; name=Sensor; acquisition={:?}; cleanup-only-release={:?}; release-granted={}; release-binding={:?}; payload-bound={}; representation=unavailable",t.catalog.sensor,t.catalog.operations[0].id,t.catalog.resource.release,t.catalog.granted(t.catalog.resource.release),t.catalog.operations[2].binding,t.catalog.resource.payload_bytes);
    let _=writeln!(out,"pairing: every admitted acquisition requires this exact fixed release grant and binding; source cannot call release");
    for edge in &t.edges {
        work.charge(1)?;
        if reachable[edge.caller.0] != 0 {
            let _ = writeln!(
                out,
                "reachable edge: caller={} callee={:?} source={:?}",
                edge.caller.0, edge.callee, edge.origin
            );
        }
    }
    for call in &t.invocations {
        work.charge(1)?;
        if reachable[call.function.0] != 0 {
            let _=writeln!(out,"trusted site: operation={:?}; function={}; block={:?}; statement={}; site={:?}; Result={:?}; destination={:?}; source={:?}; catalog={:?}",call.operation,call.function.0,call.block,call.statement,call.site,call.result,call.destination,call.source,call.catalog);
            if let Some(plan) = &call.completion {
                work.charge(20)?;
                let _=writeln!(out,"possible finish protocol: function={}; input={:?}; completion-holder={:?}; protected-result={:?}; parameter={:?}; ordered-Result={:?}; errors={}; exact-fixed-release={:?}; direct-finish-effects={}; cleanup-derived-release-effects={}; phases={:?}; event-identities={:?}; transfer={:?}; protection={:?}; cleanup={:?}; delivery={:?}; destination={:?}",call.function.0,plan.input,plan.holder,plan.protected,plan.parameter,plan.result,plan.errors,plan.release,plan.direct_effects,plan.release_effects,plan.phases,plan.events,plan.transfer,plan.protection,plan.cleanup,plan.delivery,plan.destination);
            }
        }
    }
    for f in e.mir.functions() {
        work.charge(1)?;
        if reachable[f.function().0] == 0 {
            continue;
        }
        for site in &f.cleanup_effect_sites {
            work.charge(1)?;
            let _=writeln!(out,"possible cleanup: function={} holder={:?} plan={:?} kind={:?} effects={} source={:?}",site.function.0,site.holder,site.plan,site.kind,site.effects,site.origin);
            if site.kind == crate::effects::custody::Kind::CompletionRelease {
                let _=writeln!(out,"possible completion cleanup: both protected Ok and Error require exact fixed release before inner delivery; release failure is terminal non-delivery; cleanup-only authority is not source-callable");
            } else if matches!(site.plan, hir::resources::CleanupType::Resource(_)) {
                let _=writeln!(out,"possible release-only cleanup: no completion event and no automatically produced FinishReceipt; independently formed status data proves no completion");
            }
        }
        if let Some(flow) = &f.temporaries {
            for site in &flow.sites {
                work.charge(1)?;
                if site.plan.is_some() || site.result_plan.is_some() {
                    let _=writeln!(out,"possible temporary cleanup: function={} site={:?} plan={:?} Result={:?} destination={:?} origin={:?}",f.function().0,site.id,site.plan,site.result_plan,site.destination,site.cleanup);
                }
            }
        }
    }
    let _=writeln!(out,"outcomes: only enumerated binding outcomes become source Ok/Error; denial, Limit, Integrity, Host, range traps and cleanup failure are terminal; protected Results at a terminal stop are retained, never delivered; repeat advancement does not retry, resume, unwind or release; teardown is not successful Bowfin cleanup");
    out.finish()
}

#[cfg(test)]
mod completion_tests;
#[cfg(test)]
mod migration_tests;
#[cfg(test)]
mod tests;
