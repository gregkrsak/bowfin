//! Resolved, typed representation of the supported expression/value-flow and one-field integer prototype.
//! IDs are compilation-local indices into the canonical tables owned by Program.
//! Public inspection is read-only; compiler-internal mutation must preserve identity
//! and reference invariants. Names and spans are diagnostic metadata.
use crate::lexer::Span;
pub use crate::trusted::{ResourceTypeId, TrustedOperationId};
pub(crate) mod propagation;
pub(crate) mod resources;
pub(crate) mod results;
mod validate;

macro_rules! id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub(crate) usize);
    };
}
id!(RangeTypeId);
id!(RecordId);
id!(FieldId);
id!(FunctionId);
id!(ParameterId);
id!(BindingId);
id!(LocalId);
id!(ResultTypeId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultType {
    pub id: ResultTypeId,
    pub success: ValueType,
    pub failure: ValueType,
    pub depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Program {
    pub(crate) trusted: Option<crate::trusted::Catalog>,
    pub(crate) results: Vec<ResultType>,
    pub(crate) resources: resources::Catalog,
    pub(crate) ranges: Vec<RangeType>,
    pub(crate) records: Vec<Record>,
    pub(crate) fields: Vec<Field>,
    pub(crate) functions: Vec<Function>,
    pub(crate) locals: Vec<Local>,
    pub(crate) parameters: Vec<Parameter>,
    pub(crate) bindings: Vec<RecordBinding>,
    pub(crate) statements: Vec<Statement>,
}

impl Program {
    pub fn ranges(&self) -> &[RangeType] {
        &self.ranges
    }
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    pub fn fields(&self) -> &[Field] {
        &self.fields
    }
    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
    pub fn locals(&self) -> &[Local] {
        &self.locals
    }
    pub fn local(&self, id: LocalId) -> &Local {
        &self.locals[id.0]
    }
    pub fn parameters(&self) -> &[Parameter] {
        &self.parameters
    }
    pub fn bindings(&self) -> &[RecordBinding] {
        &self.bindings
    }
    pub fn statements(&self) -> &[Statement] {
        &self.statements
    }

    pub fn range(&self, id: RangeTypeId) -> &RangeType {
        &self.ranges[id.0]
    }
    pub fn record(&self, id: RecordId) -> &Record {
        &self.records[id.0]
    }
    pub fn field(&self, id: FieldId) -> &Field {
        &self.fields[id.0]
    }
    pub fn function(&self, id: FunctionId) -> &Function {
        &self.functions[id.0]
    }
    pub fn parameter(&self, id: ParameterId) -> &Parameter {
        &self.parameters[id.0]
    }
    pub fn binding(&self, id: BindingId) -> &RecordBinding {
        &self.bindings[id.0]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RangeType {
    pub id: RangeTypeId,
    pub name: String,
    pub min: i64,
    pub max: i64,
    pub origin: DeclarationOrigin,
}
/// Catalog rows have no source declaration or source location.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeclarationOrigin {
    Source(Span),
    Catalog(crate::trusted::CatalogOrigin),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub id: RecordId,
    pub name: String,
    /// Nonempty canonical declaration order; this specifies no physical layout.
    pub fields: Vec<FieldId>,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub id: FieldId,
    pub record: RecordId,
    pub name: String,
    pub ty: ValueType,
    pub span: Span,
}
impl Field {
    pub fn range_type(&self) -> Option<RangeTypeId> {
        match self.ty {
            ValueType::Range(id) => Some(id),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterType {
    MutableRecord(RecordId),
    Range(RangeTypeId),
    Value(ValueType),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub id: ParameterId,
    pub function: FunctionId,
    pub name: String,
    pub ty: ParameterType,
    pub span: Span,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeParameter {
    pub parameter: ParameterId,
    pub ty: RangeTypeId,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldAccess {
    pub parameter: ParameterId,
    pub field: FieldId,
    pub ty: RangeTypeId,
    pub span: Span,
}
pub use crate::ast::{BinaryOp, BorrowKind, ResultTag, UnaryOp};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprType {
    Result(ResultTypeId),
    Bool,
    Range(RangeTypeId),
    Record(RecordId),
    Resource(ResourceTypeId),
    IntegerLiteral,
    SharedRef(ReferentType),
    MutableRef(ReferentType),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedExpr {
    pub kind: ExprKind,
    pub ty: ExprType,
    pub span: Span,
}
/// Canonical entries in written initializer order, independent of declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFieldInit {
    pub field: FieldId,
    pub value: TypedExpr,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExprKind {
    TrustedCall {
        operation: TrustedOperationId,
        arguments: Vec<TypedExpr>,
    },
    ResultConstruct {
        result: ResultTypeId,
        tag: ResultTag,
        payload: Box<TypedExpr>,
        variant_span: Span,
    },
    RecordConstruct {
        record: RecordId,
        fields: Vec<RecordFieldInit>,
    },
    CopyFieldRead {
        owner: Place,
        record: RecordId,
        field: FieldId,
    },
    /// Copy a field under an existing named reference handle, without reborrowing.
    CopyReferenceFieldRead {
        reference: Place,
        record: RecordId,
        field: FieldId,
    },
    IfValue {
        condition: Box<TypedExpr>,
        then_value: Box<TypedExpr>,
        else_value: Box<TypedExpr>,
    },
    BoolLiteral(bool),
    IntegerLiteral(i64),
    /// Contextually formed Copy value; canonical identity occurs only in `ty`.
    FormedRangeLiteral(i64),
    Parameter(ParameterId),
    Local(LocalId),
    Deref {
        reference: Place,
    },
    Borrow {
        kind: BorrowKind,
        place: Place,
    },
    Call {
        function: FunctionId,
        arguments: Vec<TypedExpr>,
    },
    FieldAccess(FieldAccess),
    OldField(FieldAccess),
    Unary {
        op: UnaryOp,
        operand: Box<TypedExpr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subtract {
    pub target: FieldAccess,
    pub operand: TypedExpr,
    pub span: Span,
}
/// Declared/materialized values deliberately exclude IntegerLiteral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Result(ResultTypeId),
    Bool,
    Range(RangeTypeId),
    Record(RecordId),
    Resource(ResourceTypeId),
    SharedRef(ReferentType),
    MutableRef(ReferentType),
}
/// References have only non-reference referents in this milestone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferentType {
    Bool,
    Range(RangeTypeId),
    Record(RecordId),
    Resource(ResourceTypeId),
}
impl ReferentType {
    pub(crate) fn value_type(self) -> ValueType {
        match self {
            Self::Bool => ValueType::Bool,
            Self::Range(id) => ValueType::Range(id),
            Self::Record(id) => ValueType::Record(id),
            Self::Resource(id) => ValueType::Resource(id),
        }
    }
}
/// Canonical ordinary whole-value place; never a field or temporary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Place {
    Parameter(ParameterId),
    Local(LocalId),
}
impl From<ValueType> for ExprType {
    fn from(ty: ValueType) -> Self {
        match ty {
            ValueType::Result(id) => Self::Result(id),
            ValueType::Bool => Self::Bool,
            ValueType::Range(id) => Self::Range(id),
            ValueType::Record(id) => Self::Record(id),
            ValueType::Resource(id) => Self::Resource(id),
            ValueType::SharedRef(ty) => Self::SharedRef(ty),
            ValueType::MutableRef(ty) => Self::MutableRef(ty),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Local {
    pub mutable: bool,
    pub id: LocalId,
    pub function: FunctionId,
    pub name: String,
    pub ty: ValueType,
    pub span: Span,
}
/// One canonical FunctionId space in source declaration order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Function {
    Ordinary(ValueFunction),
    Verified(VerifiedFunction),
}
impl Function {
    pub fn span(&self) -> Span {
        match self {
            Self::Ordinary(f) => f.span,
            Self::Verified(f) => f.span,
        }
    }
    pub fn id(&self) -> FunctionId {
        match self {
            Self::Ordinary(f) => f.id,
            Self::Verified(f) => f.id,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            Self::Ordinary(f) => &f.name,
            Self::Verified(f) => &f.name,
        }
    }
    pub fn as_verified(&self) -> Option<&VerifiedFunction> {
        match self {
            Self::Verified(f) => Some(f),
            Self::Ordinary(_) => None,
        }
    }
    pub fn as_ordinary(&self) -> Option<&ValueFunction> {
        match self {
            Self::Ordinary(f) => Some(f),
            Self::Verified(_) => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueFunction {
    pub id: FunctionId,
    pub name: String,
    pub parameters: Vec<ParameterId>,
    /// None is explicitly NoValue; Some(T) is Value(T), never result inference.
    pub return_type: Option<ValueType>,
    pub effects: crate::effects::Declaration,
    pub body: Vec<ValueStatement>,
    pub span: Span,
}
/// Canonical local-initializer propagation. Scope identity is derived from this
/// statement's actual lexical placement by the existing structured MIR builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Propagation {
    pub span: Span,
    pub inner: ResultTypeId,
    pub outer: ResultTypeId,
    pub failure: ValueType,
    pub syntax: crate::ast::PropagationSyntax,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueStatement {
    Match {
        scrutinee: TypedExpr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    ReturnNoValue {
        span: Span,
    },
    CallNoValue {
        function: FunctionId,
        arguments: Vec<TypedExpr>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Continue {
        span: Span,
    },
    While {
        condition: TypedExpr,
        body: Vec<ValueStatement>,
        span: Span,
    },
    CopyReferenceFieldAssign {
        reference: Place,
        record: RecordId,
        field: FieldId,
        value: TypedExpr,
        reference_span: Span,
        field_span: Span,
        span: Span,
    },
    CopyFieldAssign {
        owner: LocalId,
        record: RecordId,
        field: FieldId,
        owner_span: Span,
        field_span: Span,
        value: TypedExpr,
        span: Span,
    },
    Assign {
        local: LocalId,
        value: TypedExpr,
        span: Span,
    },
    If {
        condition: TypedExpr,
        then_body: Vec<ValueStatement>,
        else_body: Vec<ValueStatement>,
        span: Span,
    },
    DerefAssign {
        reference: Place,
        value: TypedExpr,
        span: Span,
    },
    Let {
        local: LocalId,
        initializer: TypedExpr,
        propagation: Option<Propagation>,
        span: Span,
    },
    Return {
        value: TypedExpr,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchArm {
    pub tag: ResultTag,
    pub binding: LocalId,
    pub pattern_span: Span,
    pub body: Vec<ValueStatement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedFunction {
    pub id: FunctionId,
    pub name: String,
    pub span: Span,
    pub state_param: ParameterId,
    pub state_type: RecordId,
    pub amount_param: RangeParameter,
    pub requires: TypedExpr,
    pub ensures: TypedExpr,
    pub body: Vec<Subtract>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordBinding {
    pub id: BindingId,
    pub name: String,
    pub mutable: bool,
    pub record_type: RecordId,
    pub field: FieldId,
    pub value: i64,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub function: FunctionId,
    pub binding: BindingId,
    pub amount: i64,
    pub span: Span,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Binding(BindingId),
    Call(Call),
}

#[cfg(test)]
mod tests {
    use crate::{compile_source, verify::verify_report};

    const SOURCE: &str = "\
let mut first = Battery { charge: 80 };
consume(&mut first, 50);
let mut second = Other { charge: 70 };
drain(&mut second, 20);
record Battery { charge: Percent }
record Other { charge: Percent }
type Unused = range 0..100;
type Percent = range 0..100;
verified fn consume(battery: &mut Battery, amount: Percent)
 requires amount <= battery.charge
 ensures battery.charge == old(battery.charge) - amount
 { battery.charge -= amount; }
verified function drain(battery: &mutable Other, amount: Percent)
 requires amount <= battery.charge
 ensures battery.charge == old(battery.charge) - amount
 { battery.charge -= amount; }
";

    #[test]
    fn verifier_uses_identities_even_when_display_names_collide() {
        let mut program = compile_source(SOURCE).unwrap();
        // Metadata changes cannot change which entities the references denote.
        for range in &mut program.ranges {
            range.name = "display".into();
        }
        for record in &mut program.records {
            record.name = "display".into();
        }
        for field in &mut program.fields {
            field.name = "display".into();
        }
        for function in &mut program.functions {
            match function {
                super::Function::Verified(f) => f.name = "display".into(),
                super::Function::Ordinary(f) => f.name = "display".into(),
            }
        }
        for parameter in &mut program.parameters {
            parameter.name = "display".into();
        }
        program.bindings[0].name = "renamed_first".into();
        program.bindings[1].name = "renamed_second".into();
        let report = verify_report(&program);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.functions_proven, 2);
        assert_eq!(report.calls_checked, 2);
        assert_eq!(report.final_values["renamed_first"], 30);
        assert_eq!(report.final_values["renamed_second"], 50);

        let wrong_record = SOURCE.replace("drain(&mut second, 20)", "drain(&mut first, 20)");
        let mut program = compile_source(&wrong_record).unwrap();
        for record in &mut program.records {
            record.name = "display".into();
        }
        let report = verify_report(&program);
        assert_eq!(report.calls_checked, 1);
        assert_eq!(
            report.diagnostics[0].message,
            "record argument type mismatch"
        );
    }
    #[test]
    fn ordinary_metadata_changes_preserve_canonical_local_and_call_references() {
        let mut program = compile_source(include_str!("../../examples/value_flow.bow")).unwrap();
        let original = program.clone();
        for function in &mut program.functions {
            if let super::Function::Ordinary(f) = function {
                f.name = "display".into();
            }
        }
        for local in &mut program.locals {
            local.name = "display".into();
        }
        for parameter in &mut program.parameters {
            parameter.name = "display".into();
        }
        for (before, after) in original.functions().iter().zip(program.functions()) {
            let before = before.as_ordinary().unwrap();
            let after = after.as_ordinary().unwrap();
            assert_eq!(before.id, after.id);
            assert_eq!(before.parameters, after.parameters);
            assert_eq!(before.body, after.body);
        }
        for local in program.locals() {
            assert_eq!(
                program.local(local.id).function,
                original.local(local.id).function
            );
            assert_eq!(program.local(local.id).ty, original.local(local.id).ty);
        }
        let report = verify_report(&program);
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.functions_proven, 0);
        assert_eq!(report.calls_checked, 0);
    }
}
