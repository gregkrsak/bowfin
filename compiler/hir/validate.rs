//! Table-aware invariants for ordinary record expressions at typed-HIR publication.
//! This checks canonical relationships and placement, not inference or ownership.
use super::*;
use crate::diagnostics::Diagnostic;
type Result = std::result::Result<(), Box<Diagnostic>>;

impl Program {
    pub(crate) fn validate_record_expressions(&self) -> Result {
        self.resources.validate(self).map_err(|message| {
            Box::new(Diagnostic::semantic(
                crate::bounded::START,
                message.message(),
                "canonical trusted resource schema must remain valid",
            ))
        })?;
        self.validate_record_expressions_without_resources()
    }
    // Bounded admission has already checked the catalog with typed failure propagation.
    pub(crate) fn validate_record_expressions_without_resources(&self) -> Result {
        let valid_resource_type = |ty| match ty {
            ValueType::Resource(id) | ValueType::SharedRef(ReferentType::Resource(id)) => {
                self.trusted.as_ref().is_some_and(|c| c.sensor == id)
            }
            ValueType::MutableRef(ReferentType::Resource(_)) => false,
            _ => true,
        };
        if self
            .parameters
            .iter()
            .any(|p| matches!(p.ty, ParameterType::Value(ty) if !valid_resource_type(ty)))
            || self.locals.iter().any(|l| !valid_resource_type(l.ty))
            || self
                .functions
                .iter()
                .filter_map(Function::as_ordinary)
                .any(|f| f.return_type.is_some_and(|ty| !valid_resource_type(ty)))
            || self
                .results
                .iter()
                .any(|r| !valid_resource_type(r.success) || !valid_resource_type(r.failure))
        {
            return Err(Box::new(Diagnostic::semantic(crate::bounded::START, "invalid opaque Resource interface", "Resource identity must belong to this exact catalog and mutable Resource references are excluded")));
        }
        if let Some(catalog) = &self.trusted {
            catalog.validate(self).map_err(|message| {
                Box::new(Diagnostic::semantic(
                    crate::bounded::START,
                    message,
                    "every selected catalog row must preserve its exact contract and honest origin",
                ))
            })?;
        } else if self
            .ranges
            .iter()
            .any(|r| matches!(r.origin, DeclarationOrigin::Catalog(_)))
        {
            return Err(Box::new(Diagnostic::semantic(
                crate::bounded::START,
                "catalog range without selection",
                "catalog origins require their exact compilation",
            )));
        }
        self.validate_result_tables().map_err(|message| {
            Box::new(Diagnostic::semantic(
                crate::bounded::START,
                message,
                "Result tables must be bounded, canonical, acyclic, and exact",
            ))
        })?;
        self.validate_record_tables()?;
        for function in &self.functions {
            match function {
                Function::Ordinary(f) => {
                    if !f.effects.validate() {
                        return Err(Box::new(Diagnostic::semantic(
                            f.span,
                            "invalid canonical ordinary effect declaration",
                            "effect membership and bounded declaration origins must agree",
                        )));
                    }
                    self.record_statements(f.id, &f.body)?;
                }
                Function::Verified(f) => {
                    if self
                        .records
                        .get(f.state_type.0)
                        .filter(|r| r.id == f.state_type)
                        .map_or(true, |r| {
                            r.fields.len() != 1
                                || self
                                    .fields
                                    .get(r.fields[0].0)
                                    .and_then(Field::range_type)
                                    .is_none()
                        })
                    {
                        return Err(Box::new(Diagnostic::semantic(
                            f.span,
                            "unsupported state in verified prototype",
                            "verified state requires exactly one named-range field",
                        )));
                    }
                    for expression in [&f.requires, &f.ensures]
                        .into_iter()
                        .chain(f.body.iter().map(|s| &s.operand))
                    {
                        self.record_expression(f.id, expression, false, false)?;
                    }
                }
            }
        }
        Ok(())
    }
    fn record_statements(&self, function: FunctionId, body: &[ValueStatement]) -> Result {
        for statement in body {
            match statement {
                ValueStatement::Match {
                    scrutinee,
                    arms,
                    span,
                } => {
                    let invalid = |message| {
                        Box::new(Diagnostic::semantic(*span, message, "Result matching requires exact canonical tags, payload bindings, and lexical origins"))
                    };
                    let ExprType::Result(result) = scrutinee.ty else {
                        return Err(invalid("match scrutinee is not a Result"));
                    };
                    if arms.len() != 2
                        || arms[0].tag == arms[1].tag
                        || arms[0].binding == arms[1].binding
                    {
                        return Err(invalid("invalid exhaustive Result arm mapping"));
                    }
                    self.record_expression(function, scrutinee, true, false)?;
                    for arm in arms {
                        let payload = self
                            .result_payload(result, arm.tag)
                            .ok_or_else(|| invalid("invalid Result payload identity"))?;
                        let local = self
                            .locals
                            .get(arm.binding.0)
                            .filter(|l| {
                                l.id == arm.binding
                                    && l.function == function
                                    && !l.mutable
                                    && l.ty == payload
                                    && l.span == arm.pattern_span
                            })
                            .ok_or_else(|| invalid("invalid or foreign Result payload binding"))?;
                        if arm.pattern_span.start < span.start
                            || arm.span.end > span.end
                            || arm.pattern_span.end > arm.span.end
                            || local.name.is_empty()
                        {
                            return Err(invalid("invalid Result pattern or scope origin"));
                        }
                        self.record_statements(function, &arm.body)?;
                    }
                }
                ValueStatement::CopyReferenceFieldAssign {
                    reference,
                    record,
                    field,
                    value,
                    span,
                    ..
                } => {
                    let invalid = |message| {
                        Box::new(Diagnostic::semantic(*span, message,
                        "canonical exclusive reference, referent record membership and exact Copy-field RHS type must be preserved"))
                    };
                    let ty = match reference {
                        Place::Parameter(id) => self
                            .parameters
                            .get(id.0)
                            .filter(|p| {
                                p.id == *id
                                    && p.function == function
                                    && self
                                        .functions
                                        .get(function.0)
                                        .and_then(Function::as_ordinary)
                                        .is_some_and(|f| {
                                            f.id == function && f.parameters.contains(id)
                                        })
                            })
                            .and_then(|p| match p.ty {
                                ParameterType::Value(ty) => Some(ty),
                                _ => None,
                            }),
                        Place::Local(id) => self
                            .locals
                            .get(id.0)
                            .filter(|l| l.id == *id && l.function == function)
                            .map(|l| l.ty),
                    };
                    if ty != Some(ValueType::MutableRef(ReferentType::Record(*record))) {
                        return Err(invalid("reference Copy-field assignment requires a canonical exclusive reference to the stated record in this function"));
                    }
                    let range = self.record_field(*record, *field).ok_or_else(||
                        invalid("reference Copy-field assignment field does not belong to the canonical referent record"))?;
                    if self.range(range).min > self.range(range).max
                        || value.ty != ExprType::Range(range)
                    {
                        return Err(invalid("reference Copy-field assignment requires valid range bounds and the exact declared named-range RHS type"));
                    }
                    self.record_expression(function, value, true, false)?;
                }
                ValueStatement::CopyFieldAssign {
                    owner,
                    record,
                    field,
                    value,
                    span,
                    ..
                } => {
                    let invalid = |message| {
                        Box::new(Diagnostic::semantic(*span, message, "canonical mutable local, record membership and exact Copy-field RHS type must be preserved"))
                    };
                    let local = self
                        .locals
                        .get(owner.0)
                        .filter(|l| l.id == *owner && l.function == function)
                        .ok_or_else(|| {
                            invalid("Copy-field assignment has an invalid or foreign local owner")
                        })?;
                    if !local.mutable || local.ty != ValueType::Record(*record) {
                        return Err(invalid("Copy-field assignment requires a mutable owned local of the stated record type"));
                    }
                    let range = self.record_field(*record, *field)
                        .ok_or_else(|| invalid("Copy-field assignment field does not belong to the canonical owner record"))?;
                    let declaration = self.range(range); // record_field checked canonical indexing.
                    if declaration.min > declaration.max {
                        return Err(invalid(
                            "Copy-field assignment has invalid canonical range bounds",
                        ));
                    }
                    if value.ty != ExprType::Range(range) {
                        return Err(invalid("Copy-field assignment RHS must have the exact declared named-range Copy type"));
                    }
                    self.record_expression(function, value, true, false)?;
                }
                ValueStatement::Let {
                    local,
                    initializer,
                    propagation,
                    span,
                } => {
                    if let Some(mode) = propagation {
                        mode.validate(self, function, *local, initializer, *span)
                            .map_err(|message| {
                                Box::new(Diagnostic::semantic(
                                    *span,
                                    message,
                                    "canonical propagation statement required",
                                ))
                            })?;
                    }
                    self.record_expression(function, initializer, true, true)?
                }
                ValueStatement::Assign { local, value, span } => {
                    self.record_expression(function, value, true, false)?;
                    let invalid = || {
                        Box::new(Diagnostic::semantic(
                            *span,
                            "assignment requires a canonical mutable owned local and exact RHS type",
                            "Copy assignment and owned Move replacement preserve canonical target identity",
                        ))
                    };
                    let target = self
                        .locals
                        .get(local.0)
                        .filter(|l| l.id == *local && l.function == function && l.mutable)
                        .ok_or_else(invalid)?;
                    if !matches!(
                        target.ty,
                        ValueType::Bool | ValueType::Range(_) | ValueType::Record(_)
                    ) || value.ty != ExprType::from(target.ty)
                    {
                        return Err(invalid());
                    }
                    self.record_expression(function, value, true, false)?;
                }
                ValueStatement::Return { value, .. }
                | ValueStatement::DerefAssign { value, .. } => {
                    self.record_expression(function, value, true, false)?
                }
                ValueStatement::CallNoValue { arguments, .. } => {
                    for value in arguments {
                        self.record_expression(function, value, true, false)?;
                    }
                }
                ValueStatement::If {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    self.record_expression(function, condition, true, false)?;
                    self.record_statements(function, then_body)?;
                    self.record_statements(function, else_body)?;
                }
                ValueStatement::While {
                    condition, body, ..
                } => {
                    self.record_expression(function, condition, true, false)?;
                    self.record_statements(function, body)?;
                }
                ValueStatement::ReturnNoValue { .. }
                | ValueStatement::Break { .. }
                | ValueStatement::Continue { .. } => {}
            }
        }
        Ok(())
    }
    fn record_expression(
        &self,
        function: FunctionId,
        expression: &TypedExpr,
        ordinary: bool,
        conditional: bool,
    ) -> Result {
        let invalid = |message| {
            Box::new(Diagnostic::semantic(expression.span, message, "canonical ordinary record relationships and existing expression positions must be preserved at the typed HIR boundary"))
        };
        match &expression.kind {
            ExprKind::ResultConstruct {
                result,
                tag,
                payload,
                variant_span,
            } => {
                let invalid = |message| {
                    Box::new(Diagnostic::semantic(expression.span, message, "Result construction requires one canonical active payload and exact origins"))
                };
                if !ordinary
                    || expression.ty != ExprType::Result(*result)
                    || self.result_payload(*result, *tag).map(ExprType::from) != Some(payload.ty)
                    || variant_span.start != expression.span.start
                    || variant_span.end > expression.span.end
                {
                    return Err(invalid("invalid canonical Result constructor"));
                }
                self.record_expression(function, payload, true, false)?;
            }
            ExprKind::IntegerLiteral(_) => {
                if expression.ty != ExprType::IntegerLiteral {
                    return Err(invalid(
                        "raw integer literal must retain its expression-only type",
                    ));
                }
            }
            ExprKind::FormedRangeLiteral(value) => {
                if !ordinary {
                    return Err(invalid(
                        "formed range literals are ineligible in verified expressions",
                    ));
                }
                let ExprType::Range(id) = expression.ty else {
                    return Err(invalid(
                        "formed literal requires an exact canonical range type",
                    ));
                };
                let range = self
                    .ranges
                    .get(id.0)
                    .filter(|range| range.id == id)
                    .ok_or_else(|| {
                        invalid("formed literal has an invalid canonical range identity")
                    })?;
                if *value < range.min || *value > range.max {
                    return Err(invalid(
                        "formed literal is outside canonical inclusive range bounds",
                    ));
                }
            }
            ExprKind::RecordConstruct { record, fields } => {
                if !ordinary {
                    return Err(invalid(
                        "ordinary record construction is ineligible in verified expressions",
                    ));
                }
                let declaration = self
                    .records
                    .get(record.0)
                    .filter(|r| r.id == *record)
                    .ok_or_else(|| invalid("construction has an invalid canonical record"))?;
                if fields.is_empty() && !self.resources.operations.iter().any(|op| op.function == function && matches!(op.kind, resources::OperationKind::Acquire(id) if self.resources.leaves.get(id.0).is_some_and(|leaf| leaf.record == *record))) {
                    return Err(invalid(
                        "construction initializer sequence must be nonempty",
                    ));
                }
                let mut seen = std::collections::BTreeSet::new();
                for entry in fields {
                    let field_type = self.field_type(*record, entry.field).ok_or_else(|| {
                        invalid("construction field does not belong to the canonical record")
                    })?;
                    if !seen.insert(entry.field) {
                        return Err(invalid("construction has a duplicate canonical field"));
                    }
                    if entry.value.ty != ExprType::from(field_type) {
                        return Err(invalid("construction result or initializer type is inconsistent with its canonical record field"));
                    }
                    self.record_expression(function, &entry.value, ordinary, false)?;
                }
                if seen != declaration.fields.iter().copied().collect() {
                    return Err(invalid(
                        "construction field set does not equal its complete declaration",
                    ));
                }
                if expression.ty != ExprType::Record(*record) {
                    return Err(invalid("construction result or initializer type is inconsistent with its canonical record field"));
                }
            }
            ExprKind::CopyFieldRead {
                owner,
                record,
                field,
            } => {
                if !ordinary {
                    return Err(invalid(
                        "ordinary Copy-field read is ineligible in verified expressions",
                    ));
                }
                let owner_type = match owner {
                    Place::Parameter(id) => self
                        .parameters
                        .get(id.0)
                        .filter(|p| p.id == *id && p.function == function)
                        .and_then(|p| match p.ty {
                            ParameterType::Value(ty) => Some(ty),
                            _ => None,
                        }),
                    Place::Local(id) => self
                        .locals
                        .get(id.0)
                        .filter(|l| l.id == *id && l.function == function)
                        .map(|l| l.ty),
                };
                if owner_type != Some(ValueType::Record(*record)) {
                    return Err(invalid("Copy-field read requires a canonical named owned root of the stated record type"));
                }
                let range = self.record_field(*record, *field).ok_or_else(|| {
                    invalid("Copy-field read field does not belong to the canonical owner record")
                })?;
                if expression.ty != ExprType::Range(range) {
                    return Err(invalid(
                        "Copy-field read result must be the exact declared named-range Copy type",
                    ));
                }
            }
            ExprKind::CopyReferenceFieldRead {
                reference,
                record,
                field,
            } => {
                if !ordinary {
                    return Err(invalid(
                        "ordinary reference Copy-field read is ineligible in verified expressions",
                    ));
                }
                let root_type = match reference {
                    Place::Parameter(id) => self
                        .parameters
                        .get(id.0)
                        .filter(|p| p.id == *id && p.function == function)
                        .and_then(|p| match p.ty {
                            ParameterType::Value(ty) => Some(ty),
                            _ => None,
                        }),
                    Place::Local(id) => self
                        .locals
                        .get(id.0)
                        .filter(|l| l.id == *id && l.function == function)
                        .map(|l| l.ty),
                };
                if !matches!(root_type, Some(ValueType::SharedRef(ReferentType::Record(id)) | ValueType::MutableRef(ReferentType::Record(id))) if id == *record)
                {
                    return Err(invalid("reference Copy-field read requires a canonical named reference to the stated record in this function"));
                }
                let range = self.record_field(*record, *field).ok_or_else(|| invalid("reference Copy-field read field does not belong to the canonical referent record"))?;
                if expression.ty != ExprType::Range(range) {
                    return Err(invalid("reference Copy-field read result must be the exact declared named-range Copy type"));
                }
            }
            ExprKind::IfValue {
                condition,
                then_value,
                else_value,
            } => {
                if !conditional || matches!(expression.ty, ExprType::Resource(_)) {
                    return Err(invalid("conditional values are permitted only as complete local initializers or their complete branch results"));
                }
                self.record_expression(function, condition, ordinary, false)?;
                self.record_expression(function, then_value, ordinary, true)?;
                self.record_expression(function, else_value, ordinary, true)?;
            }
            ExprKind::TrustedCall {
                operation,
                arguments,
            } => {
                let catalog = self
                    .trusted
                    .as_ref()
                    .ok_or_else(|| invalid("trusted call without selected catalog"))?;
                let op = catalog
                    .operation(*operation)
                    .filter(|op| op.source_name.is_some() && catalog.granted(op.id))
                    .ok_or_else(|| invalid("foreign, denied or cleanup-only trusted target"))?;
                if !ordinary
                    || op.result.map(ExprType::Result) != Some(expression.ty)
                    || arguments.len() != usize::from(op.parameter.is_some())
                    || (op.class == crate::trusted::OperationClass::Acquisition
                        && !catalog.granted(catalog.resource.release))
                {
                    return Err(invalid(
                        "trusted call signature or release authority mismatch",
                    ));
                }
                for argument in arguments {
                    if Some(argument.ty) != op.parameter.map(|p| ExprType::from(p.value_type())) {
                        return Err(invalid("trusted argument type or passing mode mismatch"));
                    }
                    self.record_expression(function, argument, true, false)?;
                }
            }
            ExprKind::Call { arguments, .. } => {
                for argument in arguments {
                    self.record_expression(function, argument, ordinary, false)?;
                }
            }
            ExprKind::Unary { operand, .. } => {
                self.record_expression(function, operand, ordinary, false)?
            }
            ExprKind::Binary { left, right, .. } => {
                self.record_expression(function, left, ordinary, false)?;
                self.record_expression(function, right, ordinary, false)?;
            }
            ExprKind::FieldAccess(_) | ExprKind::OldField(_) if ordinary => {
                return Err(invalid(
                    "verified-state field representation is ineligible in ordinary expressions",
                ))
            }
            ExprKind::FieldAccess(_)
            | ExprKind::OldField(_)
            | ExprKind::Parameter(_)
            | ExprKind::Local(_)
            | ExprKind::Deref { .. }
            | ExprKind::Borrow { .. }
            | ExprKind::BoolLiteral(_) => {}
        }
        Ok(())
    }
    /// Validate all declarations, including unused ones, before inspecting expressions.
    /// Checked indexing rejects malformed internal tables without repairing them.
    fn validate_record_tables(&self) -> Result {
        let invalid = |span, message| {
            Box::new(Diagnostic::semantic(
                span,
                message,
                "canonical record and field tables must have complete reciprocal membership",
            ))
        };
        let mut listed = std::collections::BTreeSet::new();
        for (index, record) in self.records.iter().enumerate() {
            if record.id != RecordId(index) {
                return Err(invalid(record.span, "record table identity is invalid"));
            }
            if record.fields.is_empty() && !self.resources.is_leaf(record.id) {
                return Err(invalid(
                    record.span,
                    "record declaration field sequence must be nonempty",
                ));
            }
            let mut names = std::collections::BTreeSet::new();
            for id in &record.fields {
                let field = self
                    .fields
                    .get(id.0)
                    .filter(|f| f.id == *id)
                    .ok_or_else(|| {
                        invalid(
                            record.span,
                            "record declaration has an invalid canonical field",
                        )
                    })?;
                if !listed.insert(*id) {
                    return Err(invalid(
                        field.span,
                        "canonical field is listed more than once",
                    ));
                }
                if !names.insert(&field.name) {
                    return Err(invalid(
                        field.span,
                        "record declaration has duplicate field names",
                    ));
                }
                if field.record != record.id {
                    return Err(invalid(
                        field.span,
                        "record and field membership is reciprocally inconsistent",
                    ));
                }
                if !self.resources.field_type_valid(self, field.ty) {
                    return Err(invalid(
                        field.span,
                        "record field has an invalid named-range type",
                    ));
                }
            }
        }
        for (index, field) in self.fields.iter().enumerate() {
            if field.id != FieldId(index) {
                return Err(invalid(field.span, "field table identity is invalid"));
            }
            let record = self
                .records
                .get(field.record.0)
                .filter(|r| r.id == field.record)
                .ok_or_else(|| invalid(field.span, "field has an invalid owning record"))?;
            if !listed.contains(&field.id) || !record.fields.contains(&field.id) {
                return Err(invalid(
                    field.span,
                    "orphan field is absent from its owning record",
                ));
            }
        }
        Ok(())
    }
    fn field_type(&self, record: RecordId, field: FieldId) -> Option<ValueType> {
        self.record(record).fields.contains(&field).then_some(())?;
        let f = self
            .fields
            .get(field.0)
            .filter(|f| f.id == field && f.record == record)?;
        self.resources.field_type_valid(self, f.ty).then_some(f.ty)
    }
    fn record_field(&self, record: RecordId, field: FieldId) -> Option<RangeTypeId> {
        let r = self
            .records
            .get(record.0)
            .filter(|r| r.id == record && r.fields.contains(&field))?;
        let f = self
            .fields
            .get(field.0)
            .filter(|f| f.id == field && f.record == r.id)?;
        let id = f.range_type()?;
        self.ranges.get(id.0).filter(|r| r.id == id)?;
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> Program {
        crate::compile_source("type Percent = range 0..100; type OtherPercent = range 0..100; record Battery { charge: Percent } record Capacitor { charge: Percent } fn f(b: Battery,p: Percent) -> Battery { let local = b; let seen = local.charge; return Battery { charge: p }; } fn g(b: Capacitor) -> Percent { return b.charge; }").unwrap()
    }
    fn read(p: &Program) -> TypedExpr {
        let ValueStatement::Let { initializer, .. } =
            &p.functions[0].as_ordinary().unwrap().body[1]
        else {
            panic!()
        };
        initializer.clone()
    }
    fn check_read(p: &Program, expression: &TypedExpr) -> Result {
        p.record_expression(p.functions[0].id(), expression, true, false)
    }
    #[test]
    fn invariant_boundary_accepts_canonical_nodes_and_read_only_tables() {
        let p = fixture();
        p.validate_record_expressions().unwrap();
        assert!(p.bindings.is_empty());
        assert_eq!(p.locals.len(), 2);
        assert!(matches!(read(&p).kind, ExprKind::CopyFieldRead { .. }));
    }
    #[test]
    fn invariant_rejects_field_of_different_record_even_with_identical_spelling() {
        let p = fixture();
        let mut e = read(&p);
        let ExprKind::CopyFieldRead { field, .. } = &mut e.kind else {
            panic!()
        };
        *field = p.records[1].fields[0];
        assert!(check_read(&p, &e)
            .unwrap_err()
            .message
            .contains("does not belong"));
    }
    #[test]
    fn invariant_rejects_stated_record_inconsistent_with_owner_type() {
        let p = fixture();
        let mut e = read(&p);
        let ExprKind::CopyFieldRead { record, field, .. } = &mut e.kind else {
            panic!()
        };
        *record = p.records[1].id;
        *field = p.records[1].fields[0];
        assert!(check_read(&p, &e)
            .unwrap_err()
            .message
            .contains("owned root"));
    }
    #[test]
    fn invariant_rejects_invalid_parameter_and_local_id_without_panics() {
        let p = fixture();
        for owner in [
            Place::Parameter(ParameterId(999)),
            Place::Local(LocalId(999)),
        ] {
            let mut e = read(&p);
            let ExprKind::CopyFieldRead { owner: target, .. } = &mut e.kind else {
                panic!()
            };
            *target = owner;
            assert!(check_read(&p, &e).is_err());
        }
    }
    #[test]
    fn invariant_rejects_foreign_function_owner() {
        let p = fixture();
        let mut e = read(&p);
        let ExprKind::CopyFieldRead { owner, .. } = &mut e.kind else {
            panic!()
        };
        *owner = Place::Parameter(p.functions[1].as_ordinary().unwrap().parameters[0]);
        assert!(check_read(&p, &e).is_err());
    }
    #[test]
    fn invariant_rejects_reference_root_without_auto_deref() {
        let mut p = fixture();
        p.locals[0].ty = ValueType::SharedRef(ReferentType::Record(p.records[0].id));
        assert!(check_read(&p, &read(&p)).is_err());
        p.locals[0].ty = ValueType::MutableRef(ReferentType::Record(p.records[0].id));
        assert!(check_read(&p, &read(&p)).is_err());
    }
    #[test]
    fn invariant_rejects_wrong_nominal_range_and_move_valued_read_result() {
        let p = fixture();
        for ty in [
            ExprType::Range(p.ranges[1].id),
            ExprType::Record(p.records[0].id),
            ExprType::MutableRef(ReferentType::Record(p.records[0].id)),
            ExprType::Bool,
            ExprType::IntegerLiteral,
        ] {
            let mut e = read(&p);
            e.ty = ty;
            assert!(check_read(&p, &e)
                .unwrap_err()
                .message
                .contains("exact declared named-range Copy"));
        }
    }
    #[test]
    fn invariant_rejects_inconsistent_canonical_field_tables_and_invalid_record() {
        let mut p = fixture();
        p.fields[0].record = p.records[1].id;
        assert!(check_read(&p, &read(&p)).is_err());
        let p = fixture();
        let mut e = read(&p);
        let ExprKind::CopyFieldRead { record, .. } = &mut e.kind else {
            panic!()
        };
        *record = RecordId(999);
        assert!(check_read(&p, &e).is_err());
    }
    #[test]
    fn invariant_rejects_verified_parameter_and_state_projection_in_ordinary_code() {
        let mut p = fixture();
        let e = read(&p);
        p.locals[0].ty = ValueType::Bool;
        assert!(check_read(&p, &e).is_err());
        let p = fixture();
        let mut e = read(&p);
        let id = p.functions[0].as_ordinary().unwrap().parameters[0];
        for old in [false, true] {
            let access = FieldAccess {
                parameter: id,
                field: p.fields[0].id,
                ty: p.ranges[0].id,
                span: e.span,
            };
            e.kind = if old {
                ExprKind::OldField(access)
            } else {
                ExprKind::FieldAccess(access)
            };
            assert!(check_read(&p, &e)
                .unwrap_err()
                .message
                .contains("verified-state"));
        }
        let mut p = fixture();
        p.parameters[0].ty = ParameterType::MutableRecord(p.records[0].id);
        let mut e = read(&p);
        let ExprKind::CopyFieldRead { owner, .. } = &mut e.kind else {
            panic!()
        };
        *owner = Place::Parameter(p.parameters[0].id);
        assert!(check_read(&p, &e).is_err());
    }
    #[test]
    fn invariant_rejects_new_ordinary_nodes_at_verified_boundary() {
        let p = fixture();
        assert!(p
            .record_expression(p.functions[0].id(), &read(&p), false, false)
            .is_err());
        let ValueStatement::Return { value, .. } = &p.functions[0].as_ordinary().unwrap().body[2]
        else {
            panic!()
        };
        assert!(p
            .record_expression(p.functions[0].id(), value, false, false)
            .is_err());
    }
    #[test]
    fn invariant_rejects_construction_field_result_and_initializer_mismatches() {
        let p = fixture();
        let ValueStatement::Return { value, .. } = &p.functions[0].as_ordinary().unwrap().body[2]
        else {
            panic!()
        };
        for variant in 0..4 {
            let mut e = value.clone();
            let ExprKind::RecordConstruct { record, fields } = &mut e.kind else {
                panic!()
            };
            match variant {
                0 => fields[0].field = p.records[1].fields[0],
                1 => *record = RecordId(999),
                2 => e.ty = ExprType::Bool,
                3 => fields[0].value.ty = ExprType::Range(p.ranges[1].id),
                _ => unreachable!(),
            };
            assert!(check_read(&p, &e).is_err());
        }
    }
    fn construct(p: &Program, child: TypedExpr) -> TypedExpr {
        TypedExpr {
            ty: ExprType::Record(p.records[0].id),
            span: child.span,
            kind: ExprKind::RecordConstruct {
                record: p.records[0].id,
                fields: vec![RecordFieldInit {
                    field: p.records[0].fields[0],
                    span: child.span,
                    value: child,
                }],
            },
        }
    }
    #[test]
    fn invariant_follows_construction_child_and_rejects_hidden_conditionals_at_depth() {
        let p = fixture();
        let leaf = read(&p);
        let hidden = TypedExpr {
            kind: ExprKind::IfValue {
                condition: Box::new(TypedExpr {
                    kind: ExprKind::BoolLiteral(true),
                    ty: ExprType::Bool,
                    span: leaf.span,
                }),
                then_value: Box::new(leaf.clone()),
                else_value: Box::new(leaf.clone()),
            },
            ..leaf.clone()
        };
        for depth in 0..4 {
            let mut child = hidden.clone();
            for _ in 0..depth {
                child = TypedExpr {
                    kind: ExprKind::Call {
                        function: p.functions[0].id(),
                        arguments: vec![construct(&p, child)],
                    },
                    ..leaf.clone()
                };
            }
            assert!(check_read(&p, &construct(&p, child))
                .unwrap_err()
                .message
                .contains("complete local initializers"));
        }
    }
    #[test]
    fn invariant_visits_every_expression_and_statement_container() {
        let p = fixture();
        let function = p.functions[0].as_ordinary().unwrap();
        let mut bad = read(&p);
        bad.ty = ExprType::Record(p.records[0].id);
        let leaf = read(&p);
        let child = TypedExpr {
            kind: ExprKind::Call {
                function: function.id,
                arguments: vec![bad.clone()],
            },
            ..leaf.clone()
        };
        let constructed = construct(&p, child);
        let wrappers = [
            constructed.clone(),
            TypedExpr {
                kind: ExprKind::Call {
                    function: function.id,
                    arguments: vec![constructed.clone()],
                },
                ..leaf.clone()
            },
            TypedExpr {
                kind: ExprKind::Unary {
                    op: UnaryOp::Not,
                    operand: Box::new(constructed.clone()),
                },
                ..leaf.clone()
            },
            TypedExpr {
                kind: ExprKind::Binary {
                    op: BinaryOp::Equal,
                    left: Box::new(leaf.clone()),
                    right: Box::new(constructed.clone()),
                },
                ..leaf.clone()
            },
        ];
        for expression in wrappers {
            assert!(check_read(&p, &expression)
                .unwrap_err()
                .message
                .contains("exact declared named-range Copy"));
        }
        let local = p.locals[0].id;
        let span = leaf.span;
        let statements = [
            ValueStatement::Let {
                local,
                initializer: constructed.clone(),
                propagation: None,
                span,
            },
            ValueStatement::Assign {
                local,
                value: constructed.clone(),
                span,
            },
            ValueStatement::DerefAssign {
                reference: Place::Local(local),
                value: constructed.clone(),
                span,
            },
            ValueStatement::Return {
                value: constructed.clone(),
                span,
            },
            ValueStatement::CallNoValue {
                function: function.id,
                arguments: vec![constructed.clone()],
                span,
            },
            ValueStatement::While {
                condition: constructed.clone(),
                body: vec![],
                span,
            },
            ValueStatement::If {
                condition: constructed.clone(),
                then_body: vec![],
                else_body: vec![],
                span,
            },
        ];
        for statement in statements {
            assert!(p
                .record_statements(function.id, &[statement])
                .unwrap_err()
                .message
                .contains("exact declared named-range Copy"));
        }
        let mut p = fixture();
        let Function::Ordinary(f) = &mut p.functions[0] else {
            panic!()
        };
        f.body[1] = ValueStatement::Return { value: bad, span };
        assert!(p.validate_record_expressions().is_err());
    }
}

#[cfg(test)]
mod multiple_fields {
    use super::*;
    fn fixture() -> Program {
        crate::compile_source("type P = range 0..100; type Q = range 0..100; record Triple { a: P, b: P, c: P } record Other { a: P, b: P, c: P } fn f(p: P) -> Triple { return Triple { c: p, a: p, b: p }; }").unwrap()
    }
    fn construct(p: &Program) -> TypedExpr {
        let ValueStatement::Return { value, .. } = &p.functions[0].as_ordinary().unwrap().body[0]
        else {
            panic!()
        };
        value.clone()
    }
    fn check(p: &Program, e: &TypedExpr) -> Result {
        p.record_expression(p.functions[0].id(), e, true, false)
    }
    #[test]
    fn declaration_tables_validate_unused_records_and_reciprocal_membership_without_panics() {
        for variant in 0..12 {
            let mut p = fixture();
            // Record 1 is deliberately unused by all expressions.
            match variant {
                0 => p.records[1].fields.clear(),
                1 => p.records[1].id = RecordId(999),
                2 => p.records[1].fields[1] = FieldId(999),
                3 => p.records[1].fields[1] = p.records[1].fields[0],
                4 => p.fields[4].name = p.fields[3].name.clone(),
                5 => p.fields[4].record = p.records[0].id,
                6 => {
                    p.records[1].fields.pop();
                }
                7 => p.fields[4].ty = ValueType::Range(RangeTypeId(999)),
                8 => p.fields[4].id = p.fields[3].id,
                9 => {
                    let mut field = p.fields[4].clone();
                    field.id = FieldId(p.fields.len());
                    p.fields.push(field);
                }
                10 => p.fields[4].record = RecordId(999),
                11 => p.records[1].fields[1] = p.records[0].fields[0],
                _ => unreachable!(),
            }
            let result = std::panic::catch_unwind(|| p.validate_record_expressions());
            assert!(result.is_ok(), "variant {variant} panicked");
            assert!(result.unwrap().is_err(), "variant {variant} was accepted");
        }
    }
    #[test]
    fn malformed_unused_declaration_is_rejected_at_typed_publication() {
        let a = crate::parse_source("type P = range 0..100; record Unused { a: P, b: P }").unwrap();
        let mut r = crate::resolve::resolve(&a).unwrap();
        r.declarations.fields[1].name = r.declarations.fields[0].name.clone();
        let e = crate::types::check(r).unwrap_err();
        assert!(e[0].message.contains("duplicate field names"));
    }
    #[test]
    fn empty_declaration_and_construction_pair_never_pass_vacuously() {
        let mut p = fixture();
        p.records[0].fields.clear();
        let Function::Ordinary(f) = &mut p.functions[0] else {
            panic!()
        };
        let ValueStatement::Return { value, .. } = &mut f.body[0] else {
            panic!()
        };
        let ExprKind::RecordConstruct { fields, .. } = &mut value.kind else {
            panic!()
        };
        fields.clear();
        assert!(p
            .validate_record_expressions()
            .unwrap_err()
            .message
            .contains("declaration field sequence must be nonempty"));
    }
    #[test]
    fn construction_vectors_reject_empty_duplicate_missing_foreign_and_invalid_ids() {
        let p = fixture();
        for variant in 0..7 {
            let mut e = construct(&p);
            let ExprKind::RecordConstruct { record, fields } = &mut e.kind else {
                panic!()
            };
            match variant {
                0 => fields.clear(),
                1 => fields[1].field = fields[0].field,
                2 => {
                    fields.pop();
                }
                3 => fields[1].field = p.records[1].fields[1],
                4 => fields[1].field = FieldId(999),
                5 => *record = RecordId(999),
                6 => e.ty = ExprType::Record(p.records[1].id),
                _ => unreachable!(),
            }
            assert!(std::panic::catch_unwind(|| check(&p, &e)).unwrap().is_err());
        }
    }
    #[test]
    fn initializer_exact_types_are_checked_in_all_three_positions() {
        let p = fixture();
        for position in 0..3 {
            let mut e = construct(&p);
            let ExprKind::RecordConstruct { fields, .. } = &mut e.kind else {
                panic!()
            };
            fields[position].value.ty = ExprType::Range(p.ranges[1].id);
            assert!(check(&p, &e)
                .unwrap_err()
                .message
                .contains("initializer type"));
        }
    }
    #[test]
    fn every_initializer_is_traversed_at_depth_and_in_every_statement_container() {
        let p = fixture();
        let f = p.functions[0].as_ordinary().unwrap();
        for position in 0..3 {
            for depth in 0..3 {
                let mut e = construct(&p);
                let ExprKind::RecordConstruct { fields, .. } = &mut e.kind else {
                    panic!()
                };
                let leaf = fields[position].value.clone();
                let condition = TypedExpr {
                    kind: ExprKind::BoolLiteral(true),
                    ty: ExprType::Bool,
                    span: leaf.span,
                };
                let mut hidden = TypedExpr {
                    kind: ExprKind::IfValue {
                        condition: Box::new(condition),
                        then_value: Box::new(leaf.clone()),
                        else_value: Box::new(leaf.clone()),
                    },
                    ..leaf.clone()
                };
                for _ in 0..depth {
                    hidden = TypedExpr {
                        kind: ExprKind::Call {
                            function: f.id,
                            arguments: vec![hidden],
                        },
                        ..leaf.clone()
                    };
                }
                fields[position].value = hidden;
                let span = e.span;
                let statements = vec![
                    ValueStatement::Let {
                        local: LocalId(999),
                        initializer: e.clone(),
                        propagation: None,
                        span,
                    },
                    ValueStatement::Assign {
                        local: LocalId(999),
                        value: e.clone(),
                        span,
                    },
                    ValueStatement::DerefAssign {
                        reference: Place::Local(LocalId(999)),
                        value: e.clone(),
                        span,
                    },
                    ValueStatement::Return {
                        value: e.clone(),
                        span,
                    },
                    ValueStatement::CallNoValue {
                        function: f.id,
                        arguments: vec![e.clone()],
                        span,
                    },
                    ValueStatement::If {
                        condition: e.clone(),
                        then_body: vec![],
                        else_body: vec![],
                        span,
                    },
                    ValueStatement::While {
                        condition: e,
                        body: vec![],
                        span,
                    },
                ];
                for s in statements {
                    assert!(p
                        .record_statements(f.id, &[s])
                        .unwrap_err()
                        .message
                        .contains("complete local initializers"));
                }
            }
        }
    }
    #[test]
    fn source_orders_are_independent_canonical_vectors() {
        let p = fixture();
        let e = construct(&p);
        let ExprKind::RecordConstruct { record, fields } = &e.kind else {
            panic!()
        };
        assert_eq!(
            p.record(*record).fields,
            [FieldId(0), FieldId(1), FieldId(2)]
        );
        assert_eq!(
            fields.iter().map(|e| e.field).collect::<Vec<_>>(),
            [FieldId(2), FieldId(0), FieldId(1)]
        );
        p.validate_record_expressions().unwrap();
    }
    #[test]
    fn formed_literal_integrity_rejects_forged_leaves_without_panicking() {
        let mut p = fixture();
        let DeclarationOrigin::Source(span) = p.ranges[0].origin else {
            panic!("expected source range")
        };
        let range = p.ranges[0].id;
        let formed = TypedExpr {
            kind: ExprKind::FormedRangeLiteral(80),
            ty: ExprType::Range(range),
            span,
        };
        p.record_expression(p.functions[0].id(), &formed, true, false)
            .unwrap();
        for ty in [
            ExprType::Bool,
            ExprType::IntegerLiteral,
            ExprType::Record(p.records[0].id),
            ExprType::SharedRef(ReferentType::Range(range)),
            ExprType::MutableRef(ReferentType::Range(range)),
            ExprType::Range(RangeTypeId(usize::MAX)),
        ] {
            let mut e = formed.clone();
            e.ty = ty;
            assert!(p
                .record_expression(p.functions[0].id(), &e, true, false)
                .is_err());
        }
        for value in [-1, 101] {
            let mut e = formed.clone();
            e.kind = ExprKind::FormedRangeLiteral(value);
            assert!(p
                .record_expression(p.functions[0].id(), &e, true, false)
                .is_err());
        }
        let mut raw = formed.clone();
        raw.kind = ExprKind::IntegerLiteral(80);
        assert!(p
            .record_expression(p.functions[0].id(), &raw, true, false)
            .is_err());
        raw.ty = ExprType::IntegerLiteral;
        p.record_expression(p.functions[0].id(), &raw, true, false)
            .unwrap();
        assert!(p
            .record_expression(p.functions[0].id(), &formed, false, false)
            .is_err());
        p.ranges[0].id = RangeTypeId(999);
        assert!(p
            .record_expression(p.functions[0].id(), &formed, true, false)
            .is_err());
    }
    #[test]
    fn formed_bounds_validation_reaches_deep_first_middle_final_children() {
        let p=crate::compile_source("type P = range 0..100; record R {a: P,b: P,c: P} fn id(p: P) -> P {return p;} fn f() -> R {return R {c: id(id(80)),a: id(id(80)),b: id(id(80))};}").unwrap();
        let f = p.functions.last().unwrap().as_ordinary().unwrap();
        let ValueStatement::Return { value, .. } = &f.body[0] else {
            panic!()
        };
        for position in 0..3 {
            let mut e = value.clone();
            let ExprKind::RecordConstruct { fields, .. } = &mut e.kind else {
                panic!()
            };
            let ExprKind::Call { arguments, .. } = &mut fields[position].value.kind else {
                panic!()
            };
            let ExprKind::Call { arguments, .. } = &mut arguments[0].kind else {
                panic!()
            };
            arguments[0].kind = ExprKind::FormedRangeLiteral(101);
            let error = p.record_expression(f.id, &e, true, false).unwrap_err();
            assert!(error.message.contains("inclusive range bounds"));
        }
    }

    #[test]
    fn formed_leaf_validation_visits_every_container_statement_and_verified_operand() {
        let p=crate::compile_source("type P = range 0..100; record R {a: P} fn id(p: P) -> P {return p;} fn f(p: P,r: &mut P) -> P {let mut x=p;return x;}").unwrap();
        let f = p.functions[1].as_ordinary().unwrap();
        let span = f.span;
        let leaf = TypedExpr {
            kind: ExprKind::FormedRangeLiteral(101),
            ty: ExprType::Range(p.ranges[0].id),
            span,
        };
        let raw = TypedExpr {
            kind: ExprKind::IntegerLiteral(0),
            ty: ExprType::IntegerLiteral,
            span,
        };
        let boolean = TypedExpr {
            kind: ExprKind::BoolLiteral(true),
            ty: ExprType::Bool,
            span,
        };
        let comparison = TypedExpr {
            kind: ExprKind::Binary {
                op: BinaryOp::LessEqual,
                left: Box::new(leaf.clone()),
                right: Box::new(raw),
            },
            ty: ExprType::Bool,
            span,
        };
        let call = TypedExpr {
            kind: ExprKind::Call {
                function: p.functions[0].id(),
                arguments: vec![leaf.clone()],
            },
            ty: leaf.ty,
            span,
        };
        let unary = TypedExpr {
            kind: ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(comparison.clone()),
            },
            ty: ExprType::Bool,
            span,
        };
        let conditional = TypedExpr {
            kind: ExprKind::IfValue {
                condition: Box::new(boolean.clone()),
                then_value: Box::new(call.clone()),
                else_value: Box::new(leaf.clone()),
            },
            ty: leaf.ty,
            span,
        };
        let record = TypedExpr {
            kind: ExprKind::RecordConstruct {
                record: p.records[0].id,
                fields: vec![RecordFieldInit {
                    field: p.fields[0].id,
                    value: call.clone(),
                    span,
                }],
            },
            ty: ExprType::Record(p.records[0].id),
            span,
        };
        for e in [
            leaf.clone(),
            call,
            comparison.clone(),
            unary,
            conditional,
            record,
        ] {
            assert!(p
                .record_expression(f.id, &e, true, true)
                .unwrap_err()
                .message
                .contains("inclusive range bounds"));
        }
        let statements = vec![
            ValueStatement::Let {
                local: p.locals[0].id,
                initializer: leaf.clone(),
                propagation: None,
                span,
            },
            ValueStatement::Assign {
                local: p.locals[0].id,
                value: leaf.clone(),
                span,
            },
            ValueStatement::DerefAssign {
                reference: Place::Parameter(f.parameters[1]),
                value: leaf.clone(),
                span,
            },
            ValueStatement::Return {
                value: leaf.clone(),
                span,
            },
            ValueStatement::CallNoValue {
                function: f.id,
                arguments: vec![leaf.clone()],
                span,
            },
            ValueStatement::If {
                condition: comparison.clone(),
                then_body: vec![],
                else_body: vec![],
                span,
            },
            ValueStatement::While {
                condition: comparison,
                body: vec![],
                span,
            },
            ValueStatement::If {
                condition: boolean.clone(),
                then_body: vec![ValueStatement::Return {
                    value: leaf.clone(),
                    span,
                }],
                else_body: vec![],
                span,
            },
            ValueStatement::While {
                condition: boolean,
                body: vec![ValueStatement::Return { value: leaf, span }],
                span,
            },
        ];
        for statement in statements {
            assert!(p
                .record_statements(f.id, &[statement])
                .unwrap_err()
                .message
                .contains("inclusive range bounds"));
        }
        let mut verified =
            crate::compile_source(include_str!("../../examples/battery_ok.bow")).unwrap();
        let Function::Verified(v) = &mut verified.functions[0] else {
            panic!()
        };
        let formed = TypedExpr {
            kind: ExprKind::FormedRangeLiteral(80),
            ty: ExprType::Range(verified.ranges[0].id),
            span: v.span,
        };
        v.body[0].operand = formed;
        assert!(verified
            .validate_record_expressions()
            .unwrap_err()
            .message
            .contains("ineligible in verified"));
    }
    #[test]
    fn formed_bounds_validation_reaches_first_middle_final_call_arguments() {
        let p=crate::compile_source("type P = range 0..100; fn tri(a: P,b: P,c: P) -> P {return a;} fn f() -> P {return tri(80,80,80);}").unwrap();
        let f = p.functions[1].as_ordinary().unwrap();
        let ValueStatement::Return { value, .. } = &f.body[0] else {
            panic!()
        };
        for position in 0..3 {
            let mut e = value.clone();
            let ExprKind::Call { arguments, .. } = &mut e.kind else {
                panic!()
            };
            arguments[position].kind = ExprKind::FormedRangeLiteral(101);
            assert!(p
                .record_expression(f.id, &e, true, false)
                .unwrap_err()
                .message
                .contains("inclusive range bounds"));
        }
    }
    fn field_fixture() -> Program {
        crate::compile_source("type Percent = range 0..100; type Other = range 0..100; record Battery { charge: Percent,health: Percent } record Capacitor { charge: Percent } fn id(p: Percent) -> Percent {return p;} fn f(b: Battery) {let mut owned = b; owned.charge = 80;} fn g(b: Battery) {let mut other = b;}").unwrap()
    }
    fn field_statement(p: &mut Program) -> &mut ValueStatement {
        let Function::Ordinary(f) = &mut p.functions[1] else {
            panic!()
        };
        &mut f.body[1]
    }
    #[test]
    fn field_assignment_publication_rejects_foreign_same_spelled_same_typed_field() {
        let mut p = field_fixture();
        let foreign = p.records[1].fields[0];
        let ValueStatement::CopyFieldAssign { field, .. } = field_statement(&mut p) else {
            panic!()
        };
        *field = foreign;
        assert!(p
            .validate_record_expressions()
            .unwrap_err()
            .message
            .contains("canonical owner record"));
    }
    #[test]
    fn field_assignment_publication_rejects_invalid_ids_wrong_membership_and_root_metadata_without_panic(
    ) {
        for fault in 0..13 {
            let mut p = field_fixture();
            match fault {
                0 => {
                    let ValueStatement::CopyFieldAssign { owner, .. } = field_statement(&mut p)
                    else {
                        panic!()
                    };
                    *owner = LocalId(usize::MAX);
                }
                1 => {
                    let ValueStatement::CopyFieldAssign { record, .. } = field_statement(&mut p)
                    else {
                        panic!()
                    };
                    *record = RecordId(usize::MAX);
                }
                2 => {
                    let ValueStatement::CopyFieldAssign { field, .. } = field_statement(&mut p)
                    else {
                        panic!()
                    };
                    *field = FieldId(usize::MAX);
                }
                3 => p.locals[0].mutable = false,
                4 => p.locals[0].function = FunctionId(2),
                5 => p.locals[0].id = LocalId(usize::MAX),
                6 => p.locals[0].ty = ValueType::SharedRef(ReferentType::Record(RecordId(0))),
                7 => p.fields[0].ty = ValueType::Range(RangeTypeId(usize::MAX)),
                8 => p.ranges[0].id = RangeTypeId(usize::MAX),
                9 => {
                    let ValueStatement::CopyFieldAssign { record, .. } = field_statement(&mut p)
                    else {
                        panic!()
                    };
                    *record = RecordId(1);
                }
                10 => p.fields[0].record = RecordId(1),
                11 => {
                    let ValueStatement::CopyFieldAssign { owner, .. } = field_statement(&mut p)
                    else {
                        panic!()
                    };
                    *owner = LocalId(1);
                }
                12 => p.ranges[0].min = 101,
                _ => unreachable!(),
            }
            assert!(p.validate_record_expressions().is_err(), "fault {fault}");
        }
    }
    #[test]
    fn field_assignment_publication_rejects_wrong_exact_rhs_and_formed_metadata() {
        for fault in 0..4 {
            let mut p = field_fixture();
            let ValueStatement::CopyFieldAssign { value, .. } = field_statement(&mut p) else {
                panic!()
            };
            match fault {
                0 => value.ty = ExprType::Range(RangeTypeId(1)),
                1 => value.ty = ExprType::Bool,
                2 => value.kind = ExprKind::FormedRangeLiteral(101),
                3 => value.ty = ExprType::Range(RangeTypeId(usize::MAX)),
                _ => unreachable!(),
            }
            assert!(p.validate_record_expressions().is_err(), "fault {fault}");
        }
    }
    #[test]
    fn field_assignment_hir_walker_validates_deep_rhs_in_every_statement_container() {
        for position in 0..3 {
            for container in 0..3 {
                let mut p = field_fixture();
                let statement = field_statement(&mut p).clone();
                let ValueStatement::CopyFieldAssign { mut value, .. } = statement.clone() else {
                    panic!()
                };
                let mut children = vec![value.clone(); 3];
                children[position].kind = ExprKind::FormedRangeLiteral(101);
                value.kind = ExprKind::Call {
                    function: FunctionId(0),
                    arguments: vec![TypedExpr {
                        kind: ExprKind::RecordConstruct {
                            record: RecordId(0),
                            fields: vec![
                                RecordFieldInit {
                                    field: FieldId(0),
                                    value: TypedExpr {
                                        kind: ExprKind::Call {
                                            function: FunctionId(0),
                                            arguments: children,
                                        },
                                        ..value.clone()
                                    },
                                    span: value.span,
                                },
                                RecordFieldInit {
                                    field: FieldId(1),
                                    value: value.clone(),
                                    span: value.span,
                                },
                            ],
                        },
                        ty: ExprType::Record(RecordId(0)),
                        span: value.span,
                    }],
                };
                let mut statement = statement;
                let ValueStatement::CopyFieldAssign { value: slot, .. } = &mut statement else {
                    panic!()
                };
                *slot = value;
                let condition = TypedExpr {
                    kind: ExprKind::BoolLiteral(true),
                    ty: ExprType::Bool,
                    span: p.locals[0].span,
                };
                let Function::Ordinary(f) = &mut p.functions[1] else {
                    panic!()
                };
                f.body[1] = match container {
                    0 => statement,
                    1 => ValueStatement::If {
                        condition,
                        then_body: vec![],
                        else_body: vec![statement],
                        span: f.span,
                    },
                    2 => ValueStatement::While {
                        condition,
                        body: vec![statement],
                        span: f.span,
                    },
                    _ => unreachable!(),
                };
                assert!(p
                    .validate_record_expressions()
                    .unwrap_err()
                    .message
                    .contains("outside"));
            }
        }
    }
    fn reference_fixture() -> Program {
        crate::compile_source("type Percent = range 0..100; type Other = range 0..100; record Battery { charge: Percent, health: Percent } record Capacitor { charge: Percent } fn f(v: &Battery) -> Percent { return v.charge; } fn g(v: &Battery) -> Percent { let alias = v; return alias.health; }").unwrap()
    }
    fn reference_read_mut(p: &mut Program) -> &mut TypedExpr {
        let Function::Ordinary(f) = &mut p.functions[0] else {
            panic!()
        };
        let ValueStatement::Return { value, .. } = &mut f.body[0] else {
            panic!()
        };
        value
    }
    #[test]
    fn reference_read_publication_rejects_foreign_and_invalid_canonical_identities_without_panic() {
        for defect in 0..13 {
            let mut p = reference_fixture();
            let other_field = p.records[1].fields[0];
            let other_record = p.records[1].id;
            let foreign_parameter = p.functions[1].as_ordinary().unwrap().parameters[0];
            let local = p.locals[0].id;
            let parameter = p.functions[0].as_ordinary().unwrap().parameters[0];
            let read = reference_read_mut(&mut p);
            let ExprKind::CopyReferenceFieldRead {
                reference,
                record,
                field,
            } = &mut read.kind
            else {
                panic!()
            };
            match defect {
                0 => *record = RecordId(usize::MAX),
                1 => *field = FieldId(usize::MAX),
                2 => *field = other_field,
                3 => *reference = Place::Parameter(foreign_parameter),
                4 => *reference = Place::Parameter(ParameterId(usize::MAX)),
                5 => *reference = Place::Local(LocalId(usize::MAX)),
                6 => *reference = Place::Local(local), // Real local, wrong function.
                7 => *record = other_record,
                8 => read.ty = ExprType::Range(RangeTypeId(1)),
                9 => read.ty = ExprType::SharedRef(ReferentType::Record(*record)),
                10 => {
                    p.parameters[parameter.0].ty =
                        ParameterType::Value(ValueType::Record(RecordId(0)))
                }
                11 => {
                    p.parameters[parameter.0].ty =
                        ParameterType::Value(ValueType::SharedRef(ReferentType::Bool))
                }
                12 => p.parameters[parameter.0].id = ParameterId(999),
                _ => unreachable!(),
            }
            assert!(p.validate_record_expressions().is_err(), "defect {defect}");
        }
    }
    #[test]
    fn reference_read_publication_rejects_malformed_declaration_tables_without_panic() {
        for defect in 0..7 {
            let mut p = reference_fixture();
            match defect {
                0 => p.records.clear(),
                1 => p.fields.clear(),
                2 => p.ranges.clear(),
                3 => p.records[0].fields.clear(),
                4 => p.fields[0].record = RecordId(1),
                5 => p.fields[0].ty = ValueType::Range(RangeTypeId(usize::MAX)),
                6 => p.records[0].id = RecordId(usize::MAX),
                _ => unreachable!(),
            }
            assert!(p.validate_record_expressions().is_err(), "defect {defect}");
        }
    }
    #[test]
    fn reference_read_is_ineligible_in_verified_expression_publication() {
        let mut p = reference_fixture();
        let id = p.functions[0].id();
        let read = reference_read_mut(&mut p).clone();
        assert!(p
            .record_expression(id, &read, false, false)
            .unwrap_err()
            .message
            .contains("ineligible"));
    }
    fn write_fixture() -> Program {
        crate::compile_source("type Percent=range 0..100; type Other=range 0..100; record Battery {charge: Percent,health: Percent} record Capacitor {charge: Percent} fn f(v: &mut Battery) {v.charge=80;} fn g(v: &mut Battery) {let next=v; next.health=80;}").unwrap()
    }
    fn write_mut(p: &mut Program) -> &mut ValueStatement {
        let Function::Ordinary(f) = &mut p.functions[0] else {
            panic!()
        };
        &mut f.body[0]
    }
    #[test]
    fn reference_write_hir_rejects_malformed_roots_fields_types_and_canonical_tables() {
        for defect in 0..24 {
            let mut p = write_fixture();
            let foreign = p.functions[1].as_ordinary().unwrap().parameters[0];
            let local = p.locals[0].id;
            let ValueStatement::CopyReferenceFieldAssign {
                reference,
                record,
                field,
                value,
                ..
            } = write_mut(&mut p)
            else {
                panic!()
            };
            match defect {
                0 => *reference = Place::Parameter(ParameterId(usize::MAX)),
                1 => *reference = Place::Local(LocalId(usize::MAX)),
                2 => *reference = Place::Parameter(foreign),
                3 => *reference = Place::Local(local),
                4 => *record = RecordId(usize::MAX),
                5 => *field = FieldId(usize::MAX),
                6 => *field = FieldId(2),
                7 => *record = RecordId(1),
                8 => value.ty = ExprType::Range(RangeTypeId(1)),
                9 => value.ty = ExprType::Range(RangeTypeId(usize::MAX)),
                10 => value.ty = ExprType::Bool,
                11 => value.kind = ExprKind::FormedRangeLiteral(101),
                12 => {
                    p.parameters[0].ty = ParameterType::Value(ValueType::SharedRef(
                        ReferentType::Record(RecordId(0)),
                    ))
                }
                13 => {
                    p.parameters[0].ty =
                        ParameterType::Value(ValueType::MutableRef(ReferentType::Bool))
                }
                14 => p.parameters[0].id = ParameterId(999),
                15 => p.records.clear(),
                16 => p.fields.clear(),
                17 => p.ranges.clear(),
                18 => p.records[0].fields.push(FieldId(0)),
                19 => p.records[0].fields.pop().map(|_| ()).unwrap(),
                20 => p.fields[0].record = RecordId(1),
                21 => p.fields[0].ty = ValueType::Range(RangeTypeId(usize::MAX)),
                22 => p.ranges[0].min = 101,
                23 => {
                    let Function::Ordinary(f) = &mut p.functions[0] else {
                        panic!()
                    };
                    f.parameters.clear();
                }
                _ => unreachable!(),
            }
            assert!(p.validate_record_expressions().is_err(), "defect {defect}");
        }
    }
    #[test]
    fn reference_write_hir_traverses_every_nested_rhs_child_and_statement_container() {
        for construction in [false, true] {
            for pos in 0..3 {
                for container in 0..3 {
                    let source="type Percent=range 0..100; record Battery {charge: Percent} record Triple {first: Percent,middle: Percent,last: Percent} fn three(a: Percent,b: Percent,c: Percent) -> Percent {return a;} fn triple(t: Triple) -> Percent {return t.first;} fn f(v: &mut Battery) {v.charge=three(80,80,80);}";
                    let mut p = crate::compile_source(source).unwrap();
                    let s = write_mut_at_last(&mut p);
                    let ValueStatement::CopyReferenceFieldAssign { value, .. } = s else {
                        panic!()
                    };
                    let mut children = vec![value.clone(); 3]; // Replace each child with a valid formed leaf.
                    for child in &mut children {
                        child.kind = ExprKind::FormedRangeLiteral(80);
                    }
                    children[pos].kind = ExprKind::FormedRangeLiteral(101);
                    if construction {
                        value.kind = ExprKind::Call {
                            function: FunctionId(1),
                            arguments: vec![TypedExpr {
                                kind: ExprKind::RecordConstruct {
                                    record: RecordId(1),
                                    fields: children
                                        .into_iter()
                                        .enumerate()
                                        .map(|(i, value)| RecordFieldInit {
                                            field: FieldId(i + 1),
                                            span: value.span,
                                            value,
                                        })
                                        .collect(),
                                },
                                ty: ExprType::Record(RecordId(1)),
                                span: value.span,
                            }],
                        };
                    } else {
                        value.kind = ExprKind::Call {
                            function: FunctionId(0),
                            arguments: children,
                        };
                    }
                    let Function::Ordinary(f) = p.functions.last_mut().unwrap() else {
                        panic!()
                    };
                    let s = f.body.pop().unwrap();
                    let condition = TypedExpr {
                        kind: ExprKind::BoolLiteral(true),
                        ty: ExprType::Bool,
                        span: f.span,
                    };
                    f.body.push(match container {
                        0 => s,
                        1 => ValueStatement::If {
                            condition,
                            then_body: vec![],
                            else_body: vec![s],
                            span: f.span,
                        },
                        2 => ValueStatement::While {
                            condition,
                            body: vec![s],
                            span: f.span,
                        },
                        _ => unreachable!(),
                    });
                    assert!(p.validate_record_expressions().is_err());
                }
            }
        }
    }
    fn write_mut_at_last(p: &mut Program) -> &mut ValueStatement {
        let Function::Ordinary(f) = p.functions.last_mut().unwrap() else {
            panic!()
        };
        &mut f.body[0]
    }
    #[test]
    fn reference_write_hir_recurses_binary_and_unary_rhs_descendants() {
        let mut p = write_fixture();
        let ValueStatement::CopyReferenceFieldAssign { value, .. } = write_mut(&mut p) else {
            panic!()
        };
        let mut child = value.clone();
        child.kind = ExprKind::FormedRangeLiteral(101);
        for side in 0..3 {
            let mut broken = p.clone();
            let ValueStatement::CopyReferenceFieldAssign { value, .. } = write_mut(&mut broken)
            else {
                panic!()
            };
            value.kind = if side == 0 {
                ExprKind::Unary {
                    op: crate::ast::UnaryOp::Not,
                    operand: Box::new(child.clone()),
                }
            } else {
                let mut valid = child.clone();
                valid.kind = ExprKind::FormedRangeLiteral(80);
                let (left, right) = if side == 1 {
                    (child.clone(), valid)
                } else {
                    (valid, child.clone())
                };
                ExprKind::Binary {
                    op: crate::ast::BinaryOp::Subtract,
                    left: Box::new(left),
                    right: Box::new(right),
                }
            };
            assert!(broken.validate_record_expressions().is_err());
        }
    }
}
