//! The one compiler-supplied demonstration catalog. Selection is explicit build
//! configuration, never authority supplied by source or discovered at runtime.
use std::sync::atomic::{AtomicUsize, Ordering};
pub(crate) mod approval;

use crate::{
    bounded::{Context, ResourceFailure, START},
    effects::{Effect, EffectSet},
    hir::{self, RangeTypeId},
};

// Current machine/evidence spellings, not aliases or authority constructors.
// Historical audit text is not imported into an opaque approval. Keep shape
// validation exact: a spelling change must never substitute for the retained
// approval, grants, catalog origins, custody, or replay identity.
pub const CATALOG: &str = "bowfin.sensor-demo.catalog.v1";
pub const PROFILE: &str = "bowfin.sensor-demo.profile.v1";
pub const TARGET: &str = "bowfin.ordinary-mir.v1";
pub const COMPILER: &str = concat!("bowfin-compiler/", env!("CARGO_PKG_VERSION"));
pub const ASSUMPTIONS: &str = "bowfin.sensor-demo.assumptions.v1";

/// Indices from distinct collections never authenticate one another. A checked,
/// process-local serial is packed with the small row index without enlarging
/// the predecessor scalar-ID representation. It is not a host resource handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceTypeId(pub(crate) usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TrustedOperationId(pub(crate) usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogOrigin {
    pub(crate) compilation: usize,
    pub(crate) row: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    Sample,
    NoSensor,
    OpenIoFailure,
    NoSample,
    ReadIoFailure,
    FinishIoFailure,
    FinishPartialIoFailure,
    Warning,
    Critical,
}

/// Operation-specific grants. Merely declaring `effects io` changes none of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Grants {
    pub open: bool,
    pub read: bool,
    pub release: bool,
    pub finish: bool,
}
impl Grants {
    pub const ALL: Self = Self {
        open: true,
        read: true,
        release: true,
        finish: true,
    };
    pub const NONE: Self = Self {
        open: false,
        read: false,
        release: false,
        finish: false,
    };
}

/// One immutable selection. Roots are explicit even when the source is empty;
/// their presence here grants neither execution nor source-callable release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    scenario: Scenario,
    grants: Grants,
    roots: Vec<String>,
    sensor_limit: usize,
}
impl Selection {
    pub fn sensor_demo(
        scenario: Scenario,
        grants: Grants,
        roots: &[&str],
    ) -> Result<Self, ResourceFailure> {
        if roots.is_empty() || roots.len() > 64 {
            return Err(ResourceFailure::limit("trusted entry roots", START));
        }
        let mut names = Vec::new();
        names
            .try_reserve_exact(roots.len())
            .map_err(|_| ResourceFailure::host("trusted entry roots", START))?;
        for root in roots {
            if root.is_empty()
                || root.len() > 128
                || !root.bytes().enumerate().all(|(i, b)| {
                    b == b'_' || b.is_ascii_alphabetic() || (i > 0 && b.is_ascii_digit())
                })
                || names.iter().any(|name| name == root)
            {
                return Err(ResourceFailure::limit(
                    "trusted entry root configuration",
                    START,
                ));
            }
            let mut name = String::new();
            name.try_reserve_exact(root.len())
                .map_err(|_| ResourceFailure::host("trusted entry root name", START))?;
            name.push_str(root);
            names.push(name);
        }
        Ok(Self {
            scenario,
            grants,
            roots: names,
            sensor_limit: 16,
        })
    }
    pub fn scenario(&self) -> Scenario {
        self.scenario
    }
    pub fn grants(&self) -> Grants {
        self.grants
    }
    pub fn roots(&self) -> &[String] {
        &self.roots
    }
    pub fn with_sensor_limit(mut self, limit: usize) -> Result<Self, ResourceFailure> {
        if limit == 0 || limit > 1024 {
            return Err(ResourceFailure::limit("live Sensor configuration", START));
        }
        self.sensor_limit = limit;
        Ok(self)
    }
    pub fn sensor_limit(&self) -> usize {
        self.sensor_limit
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationClass {
    Acquisition,
    Observation,
    Completion,
    Release,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Binding {
    SensorOpenV1,
    SensorReadV1,
    SensorFinishV1,
    SensorReleaseV1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Payload {
    Sensor(ResourceTypeId),
    Range(RangeTypeId),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Passing {
    Shared(ResourceTypeId),
    Owned(ResourceTypeId),
}
impl Passing {
    pub(crate) fn value_type(self) -> hir::ValueType {
        match self {
            Self::Shared(id) => hir::ValueType::SharedRef(hir::ReferentType::Resource(id)),
            Self::Owned(id) => hir::ValueType::Resource(id),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Resource {
    pub id: ResourceTypeId,
    pub name: &'static str,
    pub release: TrustedOperationId,
    pub effects: EffectSet,
    pub payload_bytes: usize,
    pub origin: CatalogOrigin,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Operation {
    pub id: TrustedOperationId,
    pub source_name: Option<&'static str>,
    pub parameter: Option<Passing>,
    pub success: Option<Payload>,
    pub failure: Option<RangeTypeId>,
    pub errors: u8,
    pub result: Option<hir::ResultTypeId>,
    pub effects: EffectSet,
    pub class: OperationClass,
    pub binding: Binding,
    pub resource: ResourceTypeId,
    pub origin: CatalogOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Catalog {
    pub selection: Selection,
    pub compilation: usize,
    pub sensor: ResourceTypeId,
    pub reading: RangeTypeId,
    pub error: RangeTypeId,
    pub finish_receipt: RangeTypeId,
    pub resource: Resource,
    pub operations: [Operation; 4],
}

impl Catalog {
    pub(crate) fn collect(
        selection: &Selection,
        context: &Context,
    ) -> Result<Self, ResourceFailure> {
        // These terms cover every row, immutable selection copy, approval copies,
        // bounded root/name scans and diagnostics independently of source size.
        context.reserve_completion_catalog(
            selection.roots.len(),
            selection.roots.iter().map(String::len).sum(),
        )?;
        static NEXT: AtomicUsize = AtomicUsize::new(1);
        let compilation = NEXT
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| {
                (id < usize::MAX / 4).then_some(id + 1)
            })
            .map_err(|_| ResourceFailure::limit("trusted compilation identities", START))?;
        let sensor = ResourceTypeId(compilation * 4);
        let reading = RangeTypeId(0);
        let error = RangeTypeId(1);
        let mut effects = EffectSet::EMPTY;
        effects.insert(Effect::Io);
        let row = |index, source_name, parameter, success, failure, class, binding| Operation {
            id: TrustedOperationId(compilation * 4 + index),
            source_name,
            parameter,
            success,
            failure,
            result: match index {
                0 | 1 => Some(hir::ResultTypeId(index)),
                3 => Some(hir::ResultTypeId(2)),
                _ => None,
            },
            errors: match index {
                0 => 0b1010,
                1 => 0b1100,
                3 => 0b1000,
                _ => 0,
            },
            effects,
            class,
            binding,
            resource: sensor,
            origin: CatalogOrigin {
                compilation,
                row: if index == 3 { 7 } else { index + 3 },
            },
        };
        Ok(Self {
            selection: selection.clone(),
            compilation,
            sensor,
            reading,
            error,
            finish_receipt: RangeTypeId(2),
            resource: Resource {
                id: sensor,
                name: "Sensor",
                release: TrustedOperationId(compilation * 4 + 2),
                effects,
                payload_bytes: 64,
                origin: CatalogOrigin {
                    compilation,
                    row: 2,
                },
            },
            operations: [
                row(
                    0,
                    Some("sensor_open"),
                    None,
                    Some(Payload::Sensor(sensor)),
                    Some(error),
                    OperationClass::Acquisition,
                    Binding::SensorOpenV1,
                ),
                row(
                    1,
                    Some("sensor_read"),
                    Some(Passing::Shared(sensor)),
                    Some(Payload::Range(reading)),
                    Some(error),
                    OperationClass::Observation,
                    Binding::SensorReadV1,
                ),
                row(
                    2,
                    None,
                    Some(Passing::Owned(sensor)),
                    None,
                    None,
                    OperationClass::Release,
                    Binding::SensorReleaseV1,
                ),
                row(
                    3,
                    Some("sensor_finish"),
                    Some(Passing::Owned(sensor)),
                    Some(Payload::Range(RangeTypeId(2))),
                    Some(error),
                    OperationClass::Completion,
                    Binding::SensorFinishV1,
                ),
            ],
        })
    }

    pub(crate) fn origin(&self, row: usize) -> CatalogOrigin {
        CatalogOrigin {
            compilation: self.compilation,
            row,
        }
    }
    pub(crate) fn callable(&self, name: &str) -> Option<&Operation> {
        self.operations
            .iter()
            .find(|op| op.source_name == Some(name))
    }
    pub(crate) fn operation(&self, id: TrustedOperationId) -> Option<&Operation> {
        self.operations.iter().find(|op| op.id == id)
    }
    pub(crate) fn granted(&self, id: TrustedOperationId) -> bool {
        self.operations
            .iter()
            .position(|op| op.id == id)
            .is_some_and(|index| match index {
                0 => self.selection.grants.open,
                1 => self.selection.grants.read,
                2 => self.selection.grants.release,
                3 => self.selection.grants.finish,
                _ => false,
            })
    }

    pub(crate) fn validate(&self, program: &hir::Program) -> Result<(), &'static str> {
        if self.compilation == 0
            || self.compilation >= usize::MAX / 4
            || self.sensor != ResourceTypeId(self.compilation * 4)
            || self.reading != RangeTypeId(0)
            || self.error != RangeTypeId(1)
            || self.finish_receipt != RangeTypeId(2)
        {
            return Err("invalid trusted catalog identity");
        }
        for (index, (name, min, max)) in [
            ("Reading", 0, 4095),
            ("SensorError", 1, 3),
            ("FinishReceipt", 1, 1),
        ]
        .into_iter()
        .enumerate()
        {
            let Some(range) = program.ranges.get(index) else {
                return Err("missing imported range");
            };
            if range.id != RangeTypeId(index)
                || range.name != name
                || range.min != min
                || range.max != max
                || range.origin
                    != hir::DeclarationOrigin::Catalog(self.origin(if index == 2 {
                        6
                    } else {
                        index
                    }))
            {
                return Err("altered imported range descriptor");
            }
        }
        let mut effects = EffectSet::EMPTY;
        effects.insert(Effect::Io);
        if self.resource
            != (Resource {
                id: self.sensor,
                name: "Sensor",
                release: self.operations[2].id,
                effects,
                payload_bytes: 64,
                origin: self.origin(2),
            })
        {
            return Err("altered opaque resource descriptor");
        }
        for (index, op) in self.operations.iter().enumerate() {
            let (name, parameter, success, failure, class, binding) = match index {
                0 => (
                    Some("sensor_open"),
                    None,
                    Some(Payload::Sensor(self.sensor)),
                    Some(self.error),
                    OperationClass::Acquisition,
                    Binding::SensorOpenV1,
                ),
                1 => (
                    Some("sensor_read"),
                    Some(Passing::Shared(self.sensor)),
                    Some(Payload::Range(self.reading)),
                    Some(self.error),
                    OperationClass::Observation,
                    Binding::SensorReadV1,
                ),
                2 => (
                    None,
                    Some(Passing::Owned(self.sensor)),
                    None,
                    None,
                    OperationClass::Release,
                    Binding::SensorReleaseV1,
                ),
                3 => (
                    Some("sensor_finish"),
                    Some(Passing::Owned(self.sensor)),
                    Some(Payload::Range(self.finish_receipt)),
                    Some(self.error),
                    OperationClass::Completion,
                    Binding::SensorFinishV1,
                ),
                _ => return Err("extra trusted operation"),
            };
            if op.id != TrustedOperationId(self.compilation * 4 + index)
                || op.source_name != name
                || op.parameter != parameter
                || op.success != success
                || op.failure != failure
                || op.result
                    != match index {
                        0 | 1 => Some(hir::ResultTypeId(index)),
                        3 => Some(hir::ResultTypeId(2)),
                        _ => None,
                    }
                || op.errors
                    != match index {
                        0 => 0b1010,
                        1 => 0b1100,
                        3 => 0b1000,
                        _ => 0,
                    }
                || op.class != class
                || op.binding != binding
                || op.resource != self.sensor
                || op.effects != effects
                || op.origin != self.origin(if index == 3 { 7 } else { index + 3 })
            {
                return Err("altered trusted operation descriptor");
            }
            if let Some(result) = op.result {
                let success = match op.success {
                    Some(Payload::Sensor(id)) => hir::ValueType::Resource(id),
                    Some(Payload::Range(id)) => hir::ValueType::Range(id),
                    None => return Err("trusted Result without success contract"),
                };
                if program.result_type(result).map(|r| (r.success, r.failure))
                    != Some((success, hir::ValueType::Range(self.error)))
                {
                    return Err("altered trusted Result contract");
                }
            }
        }
        // No other catalog is selected; a foreign origin cannot hide in an
        // otherwise unused ordinary range row.
        if program
            .ranges
            .iter()
            .skip(3)
            .any(|r| !matches!(r.origin, hir::DeclarationOrigin::Source(_)))
        {
            return Err("foreign catalog range origin");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod completion_tests;
