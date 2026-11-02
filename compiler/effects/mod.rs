//! Canonical BED-030 audit identities. Membership never grants permission.
use crate::lexer::Span;
use std::fmt;
pub(crate) mod calls;
pub(crate) mod custody;

/// A complete ordinary signature contract. Absent clauses allocate no origins.
/// Verified and harness declarations never contain this representation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Declaration {
    set: EffectSet,
    origins: Option<std::rc::Rc<DeclarationOrigins>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclarationOrigins {
    clause: Span,
    members: [Option<Span>; 8],
}

impl Declaration {
    #[cfg(test)]
    pub(crate) fn synthetic(set: EffectSet, span: Span) -> Self {
        if set == EffectSet::EMPTY {
            return Self::default();
        }
        let mut members = [None; 8];
        for effect in set.iter() {
            members[effect as usize] = Some(span);
        }
        Self {
            set,
            origins: Some(std::rc::Rc::new(DeclarationOrigins {
                clause: span,
                members,
            })),
        }
    }

    pub fn set(&self) -> EffectSet {
        self.set
    }

    pub fn origin(&self, effect: Effect) -> Option<Span> {
        self.origins
            .as_ref()
            .and_then(|o| o.members[effect as usize])
    }

    pub fn clause(&self) -> Option<Span> {
        self.origins.as_ref().map(|o| o.clause)
    }

    pub(crate) fn resolve(
        clause: Option<&crate::ast::EffectClause>,
    ) -> Result<Self, Box<crate::diagnostics::Diagnostic>> {
        let mut result = Self::default();
        let Some(clause) = clause else {
            return Ok(result);
        };
        if clause.names.is_empty() || clause.names.len() > 8 {
            return Err(Box::new(crate::diagnostics::Diagnostic::semantic(
                clause.span,
                "effects clause requires one to eight distinct identities",
                "omit the clause to declare the empty set",
            )));
        }
        let mut origins = DeclarationOrigins {
            clause: clause.span,
            members: [None; 8],
        };
        for name in &clause.names {
            let Some(effect) = Effect::resolve(&name.name) else {
                return Err(Box::new(crate::diagnostics::Diagnostic::semantic(
                    name.span,
                    format!("unknown effect identity `{}`", name.name),
                    "use a canonical built-in effect identity",
                )));
            };
            if !result.set.insert(effect) {
                return Err(Box::new(crate::diagnostics::Diagnostic::semantic(
                    name.span,
                    format!("duplicate effect identity `{}`", name.name),
                    "each effect identity may occur only once in a declaration",
                )));
            }
            origins.members[effect as usize] = Some(name.span);
        }
        // This fixed-size allocation is behind the existing frontend envelope.
        // Clones share immutable diagnostic origins; absent contracts allocate none.
        result.origins = Some(std::rc::Rc::new(origins));
        Ok(result)
    }

    pub(crate) fn validate(&self) -> bool {
        let valid = |span: Span| span.line > 0 && span.column > 0 && span.start <= span.end;
        self.clause().is_some() == (self.set != EffectSet::EMPTY)
            && self.clause().map_or(true, valid)
            && Effect::ALL.into_iter().all(|effect| {
                let member = self.origin(effect);
                member.is_some() == self.set.contains(effect)
                    && member.map_or(true, |span| {
                        valid(span)
                            && self.clause().is_some_and(|clause| {
                                span.start >= clause.start && span.end <= clause.end
                            })
                    })
            })
    }
}

/// Independent built-in identities, in canonical presentation order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Effect {
    Io,
    Heap,
    Blocking,
    Network,
    Filesystem,
    Clock,
    Random,
    Unsafe,
}

impl Effect {
    pub const ALL: [Self; 8] = [
        Self::Io,
        Self::Heap,
        Self::Blocking,
        Self::Network,
        Self::Filesystem,
        Self::Clock,
        Self::Random,
        Self::Unsafe,
    ];

    pub const fn name(self) -> &'static str {
        match self {
            Self::Io => "io",
            Self::Heap => "heap",
            Self::Blocking => "blocking",
            Self::Network => "network",
            Self::Filesystem => "filesystem",
            Self::Clock => "clock",
            Self::Random => "random",
            Self::Unsafe => "unsafe",
        }
    }

    /// Source-name resolution only; runtime operations use canonical IDs.
    pub fn resolve(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|effect| effect.name() == name)
    }
}

/// All 256 byte values are valid sets. There are no spare byte bits.
/// Private storage prevents unchecked wider encodings from being narrowed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EffectSet(u8);

impl EffectSet {
    pub const EMPTY: Self = Self(0);

    pub const fn contains(self, effect: Effect) -> bool {
        self.0 & (1 << effect as u8) != 0
    }

    /// Returns false for a duplicate; source resolution must reject that case.
    pub fn insert(&mut self, effect: Effect) -> bool {
        let fresh = !self.contains(effect);
        self.0 |= 1 << effect as u8;
        fresh
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn is_subset(self, declared: Self) -> bool {
        self.0 & !declared.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = Effect> {
        Effect::ALL.into_iter().filter(move |e| self.contains(*e))
    }
}

impl TryFrom<u16> for EffectSet {
    type Error = &'static str;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .map(Self)
            .map_err(|_| "foreign effect identity bits")
    }
}

impl fmt::Display for EffectSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, effect) in self.iter().enumerate() {
            if index != 0 {
                f.write_str(", ")?;
            }
            f.write_str(effect.name())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declaration_invariants_reject_independent_mutations() {
        let p = crate::compile_source("fn f() effects clock, io {}").unwrap();
        let valid = p.functions()[0].as_ordinary().unwrap().effects.clone();
        assert!(valid.validate());
        let mut missing = valid.clone();
        missing.set = EffectSet::EMPTY;
        assert!(!missing.validate());
        let mut missing = valid.clone();
        missing.origins = None;
        assert!(!missing.validate());
        let mut missing = valid.clone();
        std::rc::Rc::make_mut(missing.origins.as_mut().unwrap()).members[Effect::Clock as usize] =
            None;
        assert!(!missing.validate());
        let mut foreign = valid.clone();
        let origins = std::rc::Rc::make_mut(foreign.origins.as_mut().unwrap());
        origins.members[Effect::Unsafe as usize] = valid.origin(Effect::Clock);
        assert!(!foreign.validate());
        let mut bad_span = valid.clone();
        let origins = std::rc::Rc::make_mut(bad_span.origins.as_mut().unwrap());
        origins.members[Effect::Clock as usize]
            .as_mut()
            .unwrap()
            .line = 0;
        assert!(!bad_span.validate());
        let mut outside = valid.clone();
        let origins = std::rc::Rc::make_mut(outside.origins.as_mut().unwrap());
        origins.members[Effect::Clock as usize]
            .as_mut()
            .unwrap()
            .end = origins.clause.end + 1;
        assert!(!outside.validate());
        assert!(Declaration::default().validate());
    }

    #[test]
    fn identities_are_independent_and_render_in_canonical_order() {
        let mut set = EffectSet::EMPTY;
        for effect in Effect::ALL.into_iter().rev() {
            assert_eq!(Effect::resolve(effect.name()), Some(effect));
            assert!(set.insert(effect));
            assert!(!set.insert(effect));
        }
        assert_eq!(
            set.to_string(),
            "io, heap, blocking, network, filesystem, clock, random, unsafe"
        );
        for name in ["IO", "unknown", "", "trusted", "network_io"] {
            assert_eq!(Effect::resolve(name), None);
        }
        let mut network = EffectSet::EMPTY;
        network.insert(Effect::Network);
        assert!(!network.contains(Effect::Io));
        assert!(!network.contains(Effect::Unsafe));
    }

    #[test]
    fn every_bounded_set_obeys_union_and_subset() {
        for a in 0u16..=255 {
            let left = EffectSet::try_from(a).unwrap();
            assert_eq!(left.iter().count(), a.count_ones() as usize);
            assert!(left.is_subset(left));
            assert!(EffectSet::EMPTY.is_subset(left));
            assert_eq!(left.union(left), left);
            for b in 0u16..=255 {
                let right = EffectSet::try_from(b).unwrap();
                assert_eq!(left.is_subset(right), a & !b == 0);
                assert_eq!(left.union(right), EffectSet::try_from(a | b).unwrap());
            }
        }
    }

    #[test]
    fn wider_foreign_encodings_reject_before_narrowing() {
        for value in 256u16..=u16::MAX {
            assert!(EffectSet::try_from(value).is_err());
        }
        assert_eq!(std::mem::size_of::<EffectSet>(), 1);
        assert_eq!(EffectSet::EMPTY.to_string(), "");
    }
}
