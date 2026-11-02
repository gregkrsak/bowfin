use bowfin_compiler::{
    compile_source,
    hir::{self, ExprKind, ValueStatement},
    mir::{self, Statement, Terminator},
    parse_source, FrontendError,
};
const DECL: &str =
    "type Percent = range 0..100; type Other = range 0..100; record Ticket { value: Percent }";
fn good(body: &str) -> hir::Program {
    let p = compile_source(&format!("{DECL} {body}")).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
    p
}
fn bad(body: &str, phase: &str, wording: &str) {
    let source = format!("{DECL} {body}");
    let e = compile_source(&source).unwrap_err();
    let (actual, detail) = match &e {
        FrontendError::Lex(_) => ("Lex", e.to_string()),
        FrontendError::Parse(_) => ("Parse", e.to_string()),
        FrontendError::Resolve(ds) => ("Resolve", ds[0].render("case.bow", &source)),
        FrontendError::Type(ds) => ("Type", ds[0].render("case.bow", &source)),
        FrontendError::Ownership(ds) => ("Ownership", ds[0].render("case.bow", &source)),
        FrontendError::Effects(ds) => ("Effects", ds[0].render("case.bow", &source)),
    };
    assert_eq!(actual, phase, "{body}: {detail}");
    assert!(detail.contains(wording), "{body}: {detail}");
    for id in ["LocalId", "ParameterId", "LoanId", "BasicBlockId"] {
        assert!(!detail.contains(id), "{detail}");
    }
}
macro_rules! accept {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            good($body);
        }
    };
}
macro_rules! reject {
    ($name:ident, $body:expr, $phase:expr, $wording:expr) => {
        #[test]
        fn $name() {
            bad($body, $phase, $wording);
        }
    };
}
accept!(simple_bool_reassignment, "fn f(flag: bool) -> bool { let mut running = flag; while running { running = false; } return running; }");
accept!(
    empty_loop_and_parenthesized_condition,
    "fn f(flag: bool) -> bool { while (flag) {} return flag; }"
);
accept!(range_mutation_copy_calls_and_old_rhs, "fn next(a: Percent) -> Percent { return a; } fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let mutable running = flag; while running { current = next(current); current = current - b; current = current; running = false; } return current; }");
accept!(body_local_copy_declarations, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; let mut running = flag; while running { let temporary = current; let mut inner = true; inner = false; current = temporary; running = inner; } return current; }");
accept!(conditional_value_inside_loop, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let mut running = flag; while running { let chosen = if flag { if flag { a } else { b } } else { b }; current = chosen; running = false; } return current; }");
accept!(statement_if_inside_loop, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let mut running = flag; while running { if flag { current = a; } else { current = b; } running = false; } return current; }");
accept!(loop_inside_statement_if, "fn f(flag: bool) -> bool { let mut running = flag; if flag { while running { running = false; } } else { while running { running = false; } } return running; }");
accept!(nested_loops_independent_locals, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; let mut running = flag; while running { let mut inner = flag; while inner { let copied = current; current = copied; inner = false; } running = false; } return current; }");
accept!(copy_call_condition, "fn predicate(a: Percent) -> bool { return a == a; } fn f(a: Percent) -> Percent { let mut current = a; while predicate(current) { current = current - 0; } return current; }");
accept!(untouched_non_copy_owner, "fn f(ticket: Ticket, flag: bool) -> Ticket { let mut running = flag; while running { running = false; } return ticket; }");
accept!(untouched_reference_handle_outside_loop, "fn f(view: &Percent, flag: bool) -> Percent { let mut running = flag; while running { running = false; } return *view; }");
accept!(untouched_move_state_preserved, "fn f(ticket: Ticket, flag: bool) -> Ticket { let moved = ticket; let mut running = flag; while running { running = false; } return moved; }");
reject!(
    range_condition,
    "fn f(a: Percent) -> Percent { while a {} return a; }",
    "Type",
    "while condition must have type Bool"
);
reject!(
    record_condition,
    "fn f(ticket: Ticket) -> Ticket { while ticket {} return ticket; }",
    "Type",
    "while condition must have type Bool"
);
reject!(
    reference_condition,
    "fn f(view: &bool) -> bool { while view {} return true; }",
    "Type",
    "while condition must have type Bool"
);
reject!(
    integer_condition,
    "fn f() -> bool { while 1 {} return true; }",
    "Type",
    "while condition must have type Bool"
);
reject!(nominal_assignment_mismatch, "fn f(flag: bool, a: Percent, b: Other) -> Percent { let mut current = a; while flag { current = b; } return current; }", "Type", "distinct named ranges");
reject!(literal_assignment_bounds_checked, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; while flag { current = 101; } return current; }", "Type", "literal outside named range");
reject!(immutable_assignment_unchanged, "fn f(flag: bool, a: Percent) -> Percent { let current = a; while flag { current = a; } return current; }", "Type", "immutable local");
reject!(
    condition_unknown_name,
    "fn f() -> bool { while missing {} return true; }",
    "Resolve",
    "unknown value"
);
reject!(
    body_unknown_name,
    "fn f(flag: bool) -> bool { while flag { let temporary = missing; } return true; }",
    "Resolve",
    "unknown value"
);
reject!(
    later_body_name,
    "fn f(flag: bool) -> bool { while flag { let first = later; let later = true; } return true; }",
    "Resolve",
    "unknown value"
);
reject!(
    body_local_does_not_escape,
    "fn f(flag: bool) -> bool { while flag { let inside = true; } return inside; }",
    "Resolve",
    "unknown value"
);
reject!(
    parameter_shadowing,
    "fn f(flag: bool) -> bool { while flag { let flag = true; } return flag; }",
    "Resolve",
    "duplicate"
);
reject!(
    outer_local_shadowing,
    "fn f(flag: bool) -> bool { let outer = flag; while flag { let outer = true; } return outer; }",
    "Resolve",
    "duplicate"
);
reject!(nested_loop_shadowing, "fn f(flag: bool) -> bool { while flag { let inner = flag; while flag { let inner = true; } } return flag; }", "Resolve", "duplicate");
accept!(
    borrow_creation_in_body,
    "fn f(flag: bool, a: Percent) -> Percent { while flag { let view = &a; } return a; }"
);
accept!(mutable_borrow_creation_in_body, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; while flag { let access = &mutable current; } return current; }");
reject!(borrow_creation_in_condition, "fn inspect(view: &Percent) -> bool { return true; } fn f(a: Percent) -> Percent { while inspect(&a) {} return a; }", "Ownership", "reference activity");
accept!(dereference_in_body, "fn f(flag: bool, view: &Percent) -> Percent { while flag { let observed = *view; } return *view; }");
reject!(
    dereference_in_condition,
    "fn f(view: &bool) -> bool { while *view {} return true; }",
    "Ownership",
    "reference activity"
);
accept!(write_through_in_body, "fn f(flag: bool, access: &mut Percent, a: Percent) -> Percent { while flag { *access = a; } return a; }");
accept!(reference_call_argument, "fn inspect(view: &Percent) -> bool { return true; } fn f(flag: bool, view: &Percent) -> bool { while flag { let observed = inspect(view); } return true; }");
reject!(mutable_handle_call_argument, "fn inspect(access: &mut Percent) -> bool { return true; } fn f(flag: bool, access: &mut Percent) -> bool { while flag { let observed = inspect(access); } return true; }", "Ownership", "loop-carried mutable reference");
reject!(condition_reference_call_argument, "fn inspect(view: &Percent) -> bool { return true; } fn f(view: &Percent) -> bool { while inspect(view) {} return true; }", "Ownership", "reference activity");
accept!(direct_borrow_call_argument, "fn inspect(view: &Percent) -> bool { return true; } fn f(flag: bool, a: Percent) -> bool { while flag { let observed = inspect(&a); } return true; }");
accept!(
    reference_valued_body_alias,
    "fn f(flag: bool, view: &Percent) -> bool { while flag { let alias = view; } return true; }"
);
reject!(record_by_value_call, "fn consume(ticket: Ticket) -> bool { return true; } fn f(flag: bool, ticket: Ticket) -> bool { while flag { let consumed = consume(ticket); } return true; }", "Ownership", "pre-existing non-Copy value");
reject!(record_condition_call, "fn consume(ticket: Ticket) -> bool { return true; } fn f(ticket: Ticket) -> bool { while consume(ticket) {} return true; }", "Ownership", "cyclic ownership analysis");
reject!(
    non_copy_body_local,
    "fn f(flag: bool, ticket: Ticket) -> bool { while flag { let moved = ticket; } return true; }",
    "Ownership",
    "pre-existing non-Copy value"
);
accept!(fresh_non_copy_call_result, "fn fresh() -> Ticket { return fresh(); } fn f(flag: bool) -> bool { while flag { let made = fresh(); } return true; }");
reject!(nested_branch_move, "fn consume(ticket: Ticket) -> bool { return true; } fn f(flag: bool, ticket: Ticket) -> bool { while flag { if flag { let done = consume(ticket); } } return true; }", "Ownership", "pre-existing non-Copy value");
reject!(record_replacement_inside_loop, "fn f(flag: bool, first: Ticket, second: Ticket) -> Ticket { let mut current = first; while flag { current = second; } return current; }", "Ownership", "cannot replace pre-existing non-Copy");
accept!(reference_in_conditional_value_leaf, "fn f(flag: bool, view: &Percent, a: Percent) -> Percent { while flag { let chosen = if flag { a } else { *view }; } return a; }");
accept!(
    constant_false_does_not_relax_loop_restrictions,
    "fn f(a: Percent) -> Percent { while false { let view = &a; } return a; }"
);
accept!(shared_loan_dies_before_entry, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; let observed = *view; while flag { current = b; } return current; }");
accept!(exclusive_loan_dies_before_entry, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; let observed = *access; while flag { current = b; } return current; }");
accept!(live_shared_loan_allows_owner_copy_reads, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; let view = &current; while flag { let observed = current; } return *view; }");
reject!(live_shared_loan_blocks_loop_assignment, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; while flag { current = b; } return *view; }", "Ownership", "cannot assign");
reject!(live_exclusive_loan_blocks_loop_reads, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; let access = &mut current; while flag { let observed = current; } return *access; }", "Ownership", "exclusively borrowed");
reject!(live_exclusive_loan_blocks_loop_assignment, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let access = &mut current; while flag { current = b; } return *access; }", "Ownership", "cannot assign");
reject!(future_shared_use_prevents_false_expiry, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; while flag { current = b; } let observed = *view; return current; }", "Ownership", "cannot assign");
reject!(condition_exclusive_loan_conflict, "fn f(flag: bool) -> bool { let mut running = flag; let access = &mut running; while running {} return *access; }", "Ownership", "exclusively borrowed");
accept!(sibling_path_precision_outside_loop, "fn f(flag: bool, a: Percent, b: Percent) -> Percent { let mut current = a; let view = &current; if flag { while flag { current = b; } } else { let observed = *view; } return current; }");
accept!(new_reference_after_loop_remains_supported, "fn f(flag: bool, a: Percent) -> Percent { let mut current = a; while flag { current = current; } let view = &current; return *view; }");
accept!(potentially_diverging_constant_loop, "fn f(flag: bool) -> bool { let mut running = flag; while true { running = true; } return running; }");
#[test]
fn unsupported_loop_grammar_boundaries() {
    for body in [
        "while flag true; return flag;",
        "while flag {}; return flag;",
        "while flag {} else {} return flag;",
        "for flag {} return flag;",
        "loop {} return flag;",
        "label: while flag {} return flag;",
        "let predecessor_result = while flag {}; return flag;",
        "return while flag {};",
        "let predecessor_result = id(while flag {}); return flag;",
        "let mut predecessor_result = flag; predecessor_result = while flag {}; return flag;",
    ] {
        bad(&format!("fn id(value: bool) -> bool {{ return value; }} fn f(flag: bool) -> bool {{ {body} }}"), "Parse", "");
    }
}
#[test]
fn lexical_child_scope_and_static_ids() {
    let text = format!("{DECL} fn f(flag: bool) -> bool {{ while flag {{ let temporary = flag; while temporary {{ let inner = temporary; }} }} while flag {{ let temporary = flag; }} return flag; }}");
    let ast = parse_source(&text).unwrap();
    let bowfin_compiler::ast::FunctionDecl::Ordinary(f) = &ast.functions[0] else {
        panic!()
    };
    let bowfin_compiler::ast::ValueStatement::While { span, .. } = &f.body[0] else {
        panic!()
    };
    assert_eq!(
        &text[span.start..span.end],
        "while flag { let temporary = flag; while temporary { let inner = temporary; } }"
    );
    let p = compile_source(&text).unwrap();
    assert_eq!(p.locals().len(), 3);
    assert_eq!(p.locals()[0].name, p.locals()[2].name);
    assert_ne!(p.locals()[0].id, p.locals()[2].id);
    let f = p.functions()[0].as_ordinary().unwrap();
    let ValueStatement::While {
        condition, body, ..
    } = &f.body[0]
    else {
        panic!()
    };
    assert_eq!(condition.kind, ExprKind::Parameter(f.parameters[0]));
    let ValueStatement::While {
        condition, body, ..
    } = &body[1]
    else {
        panic!()
    };
    assert_eq!(condition.kind, ExprKind::Local(p.locals()[0].id));
    let ValueStatement::Let { initializer, .. } = &body[0] else {
        panic!()
    };
    assert_eq!(initializer.kind, ExprKind::Local(p.locals()[0].id));
}
#[test]
fn mir_simple_loop_has_header_backedge_and_exit() {
    for condition in ["running", "true", "false"] {
        let p = good(&format!("fn f(flag: bool) -> bool {{ let mut running = flag; while {condition} {{ running = false; }} return running; }}"));
        let m = mir::lower(&p);
        assert_eq!(m, mir::lower(&p));
        let f = &m.functions()[0];
        assert_eq!(f.blocks().len(), 4);
        let preheader = f.block(f.entry());
        assert!(matches!(preheader.statements(), [Statement::Let { .. }]));
        let Terminator::Goto { target: header } = preheader.terminator() else {
            panic!()
        };
        let Terminator::Branch {
            then_target: body,
            else_target: exit,
            ..
        } = f.block(*header).terminator()
        else {
            panic!()
        };
        assert_ne!(body, exit);
        let [Statement::Assign { local, .. }] = f.block(*body).statements() else {
            panic!()
        };
        assert_eq!(*local, p.locals()[0].id);
        assert_eq!(
            f.block(*body).terminator(),
            &Terminator::Goto { target: *header }
        );
        assert!(matches!(
            f.block(*exit).terminator(),
            Terminator::Return { .. }
        ));
    }
}
#[test]
fn mir_nested_cycles_compose_with_conditional_initialization() {
    let p = good("fn f(flag: bool, a: Percent) -> Percent { let mut current = a; while flag { let chosen = if flag { a } else { current }; let mut inner = flag; while inner { current = chosen; inner = false; } } return current; }");
    let m = mir::lower(&p);
    assert_eq!(m, mir::lower(&p));
    let f = &m.functions()[0];
    f.validate().unwrap();
    let body_local = p.locals()[1].id;
    assert_eq!(
        f.blocks()
            .flat_map(|(_, b)| b.statements())
            .filter(|s| matches!(s, Statement::Let { local, .. } if *local == body_local))
            .count(),
        2
    );
    // A loop backedge targets its Branch header; conditional joins can also
    // have earlier IDs, so numbering alone does not identify a cycle.
    let order: Vec<_> = f.blocks().map(|(id, _)| id).collect();
    let backedges = f
        .blocks()
        .filter(|(from, b)| match b.terminator() {
            Terminator::Goto { target } => {
                matches!(f.block(*target).terminator(), Terminator::Branch { .. })
                    && order.iter().position(|id| id == target).unwrap()
                        < order.iter().position(|id| id == from).unwrap()
            }
            _ => false,
        })
        .count();
    assert_eq!(backedges, 2);
}
#[test]
fn verified_loop_syntax_remains_unsupported() {
    let text = include_str!("../../examples/battery_ok.bow").replace(
        "battery.charge -= amount;",
        "while true { battery.charge -= amount; }",
    );
    assert!(matches!(
        compile_source(&text),
        Err(FrontendError::Parse(_))
    ));
}
