//! B31-MR01: sealed allocation injection, typed limits, public preparation and
//! the production CLI reporting boundary. No OOM or production injection switch.
use super::*;
use crate::bounded::result_admission::{with_failure, Allocation};
use crate::hir::results::{AdmissionError, MAX_RESULT_DEPTH, MAX_RESULT_TYPES};

const SOURCE: &str = "fn demo()->Result<success:bool,failure:bool>{return Ok(true);}";

fn failure(site: Allocation, after: usize, source: &str) -> ExecutionError {
    with_failure(site, after, || {
        prepare_execution(source, AdmissionLimits::MAX)
    })
    .err()
    .expect("an admission failure must publish no executable")
}
fn host(site: Allocation, resource: &'static str) {
    let error = failure(site, 0, SOURCE);
    assert_eq!(error.kind, ErrorKind::Host(resource));
    match site {
        Allocation::Syntax => assert!(error.origin.start > 0),
        Allocation::Scratch | Allocation::Table => {
            assert_eq!((error.origin.start, error.origin.end), (0, SOURCE.len()));
        }
    }
    // The scoped injection restores itself; this is a separate control, not an
    // allocation retry inside preparation.
    assert!(prepare_execution(SOURCE, AdmissionLimits::MAX).is_ok());
}

#[test]
fn parser_node_reservation_failure_is_host_and_publishes_no_executable() {
    host(Allocation::Syntax, "Result type syntax allocation");
}
#[test]
fn resolver_scratch_reservation_failure_is_host_and_publishes_no_executable() {
    host(Allocation::Scratch, "Result resolution scratch allocation");
}
#[test]
fn canonical_table_reservation_failure_is_host_and_publishes_no_executable() {
    host(Allocation::Table, "Result canonical table allocation");
}

#[test]
fn late_admission_allocation_failures_keep_their_host_category() {
    let source = "type A=range 0..1;type B=range 0..1;type C=range 0..1;type D=range 0..1;fn a()->Result<success:bool,failure:bool>{return Err(false);}fn b()->Result<success:A,failure:bool>{return Err(false);}fn c()->Result<success:B,failure:bool>{return Err(false);}fn d()->Result<success:C,failure:bool>{return Err(false);}fn e()->Result<success:D,failure:bool>{return Err(false);}";
    for (site, resource) in [
        (Allocation::Syntax, "Result type syntax allocation"),
        (Allocation::Scratch, "Result resolution scratch allocation"),
        (Allocation::Table, "Result canonical table allocation"),
    ] {
        let error = failure(site, 1, source);
        assert_eq!(error.kind, ErrorKind::Host(resource));
        assert!(error.origin.start > source.find("fn a").unwrap());
    }
}

#[test]
fn cli_reports_each_detected_result_admission_host_failure_as_class_two() {
    for site in [Allocation::Syntax, Allocation::Scratch, Allocation::Table] {
        let error = failure(site, 0, SOURCE);
        let mut output = Vec::new();
        assert_eq!(
            crate::run_reporting::preparation_error(&error, 4096, &mut output),
            Err(2)
        );
        assert!(std::str::from_utf8(&output)
            .unwrap()
            .contains("detected host/tool failure"));
    }
}

fn nested(depth: usize) -> String {
    let mut ty = "bool".to_owned();
    for _ in 0..depth {
        ty = format!("Result<success:{ty},failure:bool>");
    }
    ty
}
#[test]
fn public_result_syntax_depth_boundary_and_excessive_neighbor_keep_limit_category() {
    // The syntax walker counts the owned leaf as a node-depth level. Preserve
    // that supported boundary; this repair does not enlarge the grammar.
    let source = |depth| {
        let ty = nested(depth);
        format!("fn earlier(){{}}fn demo(pending:{ty})->{ty}{{return pending;}}")
    };
    let limits = AdmissionLimits::MAX;
    assert!(prepare_execution(&source(limits.traversal_depth - 1), limits).is_ok());
    let error = prepare_execution(&source(limits.traversal_depth), limits)
        .err()
        .unwrap();
    assert_eq!(
        error.kind,
        ErrorKind::ConfiguredLimit("Result syntax depth")
    );
    assert!(error.origin.start > "fn earlier(){}".len());
}

#[test]
fn exact_canonical_result_count_succeeds_and_next_distinct_pair_is_limit() {
    let mut p = hir::Program::default();
    for i in 0..65 {
        p.ranges.push(hir::RangeType {
            id: hir::RangeTypeId(i),
            name: format!("R{i}"),
            min: 0,
            max: 1,
            origin: crate::hir::DeclarationOrigin::Source(START),
        });
    }
    for index in 0..MAX_RESULT_TYPES {
        let id = p
            .intern_result(
                ValueType::Range(hir::RangeTypeId(index / 65)),
                ValueType::Range(hir::RangeTypeId(index % 65)),
            )
            .unwrap();
        assert_eq!(id.0, index);
    }
    p.validate_result_tables().unwrap();
    // An already interned pair remains usable at the full table boundary.
    assert_eq!(
        p.intern_result(
            ValueType::Range(hir::RangeTypeId(0)),
            ValueType::Range(hir::RangeTypeId(0))
        )
        .unwrap()
        .0,
        0
    );
    assert_eq!(
        p.intern_result(
            ValueType::Range(hir::RangeTypeId(MAX_RESULT_TYPES / 65)),
            ValueType::Range(hir::RangeTypeId(MAX_RESULT_TYPES % 65))
        ),
        Err(AdmissionError::Resource(ResourceFailure::limit(
            "Result instantiation count",
            START
        )))
    );
    assert_eq!(p.result_types().len(), MAX_RESULT_TYPES);
}

fn syntax(depth: usize) -> crate::ast::ValueType {
    use crate::ast::ResultTypeSyntax;
    let mut nodes = vec![ResultTypeSyntax::Owned("bool".into())];
    for i in 0..depth {
        nodes.push(ResultTypeSyntax::Result {
            success: i,
            failure: 0,
        });
    }
    crate::ast::ValueType {
        name: "Result".into(),
        references: Vec::new(),
        result_nodes: nodes,
    }
}
#[test]
fn canonical_depth_limit_survives_resolution_after_successful_interning() {
    // Private AST control reaches canonical interning independently of the
    // earlier source syntax-depth limit, without sealing execution authority.
    for depth in [MAX_RESULT_DEPTH, MAX_RESULT_DEPTH + 1] {
        let mut ast = crate::parse_source("fn demo(pending:bool)->bool{return pending;}").unwrap();
        let crate::ast::FunctionDecl::Ordinary(f) = &mut ast.functions[0] else {
            panic!()
        };
        f.parameters[0].ty = syntax(depth);
        f.return_type = Some(syntax(depth));
        let context = Context::new(AdmissionLimits::MAX);
        let outcome = crate::resolve::resolve_bounded(&ast, &context);
        if depth == MAX_RESULT_DEPTH {
            let resolved = outcome.unwrap();
            assert_eq!(resolved.declarations.result_types().len(), MAX_RESULT_DEPTH);
            assert!(crate::types::check(resolved).is_ok());
            assert!(context.failure.get().is_none());
        } else {
            let error = frontend("resolve", outcome.err().unwrap(), &context);
            assert_eq!(
                error.kind,
                ErrorKind::ConfiguredLimit("Result type nesting")
            );
        }
    }
}

#[test]
fn result_node_budget_failures_remain_limits_in_parser_and_resolver() {
    let mut ty = "bool".to_owned();
    for _ in 0..11 {
        ty = format!("Result<success:{ty},failure:{ty}>");
    }
    let source = |ty: &str| format!("fn demo(pending:{ty}){{}}");
    // Isolate the Result walker from the independent lexer token ceiling.
    let context = Context::new(AdmissionLimits::MAX);
    let tokens = crate::lexer::lex(&source(&ty)).unwrap();
    let ast = crate::parser::parse_bounded(&tokens, &context).unwrap();
    let crate::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    assert_eq!(f.parameters[0].ty.result_nodes.len(), 4095);
    let tokens = crate::lexer::lex(&source(&format!("Result<success:{ty},failure:bool>"))).unwrap();
    let context = Context::new(AdmissionLimits::MAX);
    assert!(crate::parser::parse_bounded(&tokens, &context).is_err());
    assert_eq!(context.failure.get().unwrap().kind, ResourceKind::Limit);
    assert_eq!(
        context.failure.get().unwrap().resource,
        "Result syntax nodes"
    );
    let mut ast = ast;
    let crate::ast::FunctionDecl::Ordinary(f) = &mut ast.functions[0] else {
        panic!()
    };
    f.parameters[0]
        .ty
        .result_nodes
        .resize(4097, crate::ast::ResultTypeSyntax::Owned("bool".into()));
    let context = Context::new(AdmissionLimits::MAX);
    let errors = crate::resolve::resolve_bounded(&ast, &context)
        .err()
        .unwrap();
    assert_eq!(
        frontend("resolve", errors, &context).kind,
        ErrorKind::ConfiguredLimit("Result syntax nodes")
    );
}

#[test]
fn malformed_result_syntax_and_nominal_mismatches_remain_frontend_errors() {
    for (source,expected) in [
        ("fn demo()->Result<failure:bool,success:bool>{return Ok(true);}","parse"),
        ("fn demo()->Result<success:bool,success:bool>{return Ok(true);}","parse"),
        ("fn demo()->Result<success:Unknown,failure:bool>{return Err(false);}","resolve"),
        ("fn demo()->Result<success:&bool,failure:bool>{return Err(false);}","parse"),
        ("type A=range 0..1;type B=range 0..1;fn value()->A{return 0;}fn demo()->Result<success:B,failure:bool>{return Ok(value());}","type"),
    ] {
        let error=prepare_execution(source,AdmissionLimits::MAX).err().unwrap();
        assert!(matches!(error.kind,ErrorKind::Frontend{phase,..} if phase==expected),"{source}: {error:?}");
    }
}
