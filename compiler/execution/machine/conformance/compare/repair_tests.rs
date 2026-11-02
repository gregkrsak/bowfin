//! Discriminating I01/I02 source witnesses and private malformed-state controls.
use super::*;

fn observation() -> Limits {
    Limits {
        records: 262_144,
        work: 10_000_000,
        storage: 67_108_864,
        lineage: 128,
    }
}
fn comparison() -> ComparisonLimits {
    ComparisonLimits {
        work: 100_000_000,
        ..ComparisonLimits::default()
    }
}
fn session<'a>(contract: &'a contract::Contract<'a>, steps: u64) -> Session<'a> {
    let mut session = Session::new(
        contract,
        ExecutionLimits {
            steps,
            ..ExecutionLimits::default()
        },
        observation(),
    )
    .unwrap();
    let _ = session.advance();
    assert_eq!(session.completeness(), Ok(()));
    session
}
fn contract(e: &CheckedExecutable) -> contract::Contract<'_> {
    contract::Contract::extract(e, e.entry("demo").unwrap(), contract::Limits::default()).unwrap()
}
fn equal_bool(source: &str, expected: bool) {
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(
        a.primary().unwrap(),
        Ok(&ExecutionResult::Value(RuntimeValue::Bool(expected)))
    );
    assert_eq!(b.primary(), a.primary());
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
}
const TYPES: &str = "type Percent=range 0..100;record Battery{charge:Percent}";
const VERIFIED: &str = "verified fn consume(battery:&mut Battery,amount:Percent) requires amount<=battery.charge ensures battery.charge==old(battery.charge)-amount {battery.charge-=amount;}";

#[test]
fn i01_verified_prefix_resolves_ordinary_boundary_and_nonboundary_origins() {
    let source = format!("{TYPES}{VERIFIED}fn demo()->bool{{return true;}}");
    let e = prepare_execution(&source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    assert_eq!(c.entry().0, 1);
    assert!(e.entry("consume").is_err());
    for steps in [1, 2, 3] {
        let a = session(&c, steps);
        let b = session(&c, steps);
        if steps == 3 {
            assert_eq!(
                a.primary().unwrap(),
                Ok(&ExecutionResult::Value(RuntimeValue::Bool(true)))
            );
        } else {
            assert_eq!(
                a.primary().unwrap().unwrap_err().kind,
                ErrorKind::ConfiguredLimit("steps")
            );
        }
        assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
        assert!(a
            .capture()
            .records
            .iter()
            .any(|r| matches!(r, Record::OriginKind(OriginKind::FunctionBoundary))));
        if steps == 3 {
            assert!(a.capture().records.iter().any(
                |r| matches!(r,Record::OriginKind(kind) if *kind!=OriginKind::FunctionBoundary)
            ));
        }
        assert!(a
            .capture()
            .records
            .iter()
            .all(|r| !matches!(r,Record::Frame(_,function,..) if function.0==0)));
    }
}
#[test]
fn i01_verified_between_caller_callee_and_failing_harness_stay_isolated() {
    let source = format!(
        "{TYPES}fn demo()->bool{{return helper();}}{VERIFIED}fn helper()->bool{{return true;}}"
    );
    equal_bool(&source, true);
    let e = prepare_execution(&source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    assert_eq!(c.functions(), &[FunctionId(0), FunctionId(2)]);
    for steps in [1, 3, 4] {
        let a = session(&c, steps);
        let b = session(&c, steps);
        assert_eq!(
            a.primary().unwrap().unwrap_err().kind,
            ErrorKind::ConfiguredLimit("steps")
        );
        assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    }
    // This existing mixed-file fixture has an intentionally failing proof/call.
    // Public ordinary preparation/execution must not evaluate the harness.
    let source = format!(
        "{}\nfn demo()->bool{{return true;}}",
        include_str!("../../../../../examples/battery_fail.bow")
    );
    assert!(
        !crate::verify::verify_report(&crate::compile_source(&source).unwrap())
            .diagnostics
            .is_empty()
    );
    equal_bool(&source, true);
}
#[test]
fn i01_foreign_function_and_origin_metadata_reject_without_indexing_panics() {
    let source = format!("{TYPES}{VERIFIED}fn demo()->bool{{return true;}}");
    let e = prepare_execution(&source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    for mutation in 0..3 {
        let b = session(&c, 1024);
        let mut capture = b.machine().observation.borrow_mut();
        let records = &mut capture.as_mut().unwrap().records;
        if mutation < 2 {
            let function = records
                .iter_mut()
                .find_map(|r| {
                    if let Record::Frame(_, function, ..) = r {
                        Some(function)
                    } else {
                        None
                    }
                })
                .unwrap();
            *function = FunctionId(if mutation == 0 { 0 } else { usize::MAX });
        } else {
            let span = e.program.function(FunctionId(0)).span();
            let origin = records
                .iter_mut()
                .find_map(|r| {
                    if let Record::Cut(_, _, _, origin, _, _) = r {
                        if *origin != START {
                            Some(origin)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap();
            *origin = span;
        }
        drop(capture);
        assert_eq!(
            compare(&a, &b, comparison()),
            Comparison::Inconclusive(Incomplete::InvalidState)
        );
    }
}
const SHARED_THEN_EXCLUSIVE: &str = "fn demo()->bool{let mutable value=true;let shared=&value;let seen=*shared;let exclusive=&mut value;*exclusive=false;return value;}";
const SHARED_THEN_MOVE: &str = "type N=range 0..1;record R{x:N}fn demo()->R{let item=R{x:1};let view=&item;let observed=view.x;return item;}";

#[test]
fn i02_final_shared_use_allows_exclusive_access_with_old_bits_retained() {
    equal_bool(SHARED_THEN_EXCLUSIVE, false);
    let e = prepare_execution(SHARED_THEN_EXCLUSIVE, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let records = &a.capture().records;
    let cuts: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| matches!(r, Record::Cut(..)).then_some(i))
        .collect();
    assert!(cuts.windows(2).any(|cut| {
        let rows = &records[cut[0]..cut[1]];
        rows.iter()
            .any(|r| matches!(r, Record::Reference(BorrowKind::Shared, ..)))
            && rows
                .iter()
                .any(|r| matches!(r, Record::Reference(BorrowKind::Mutable, ..)))
    }));
}
#[test]
fn i02_record_move_after_final_field_read_preserves_stored_reference_identity() {
    let e = prepare_execution(SHARED_THEN_MOVE, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    let ExecutionResult::Value(RuntimeValue::Record { record, fields }) =
        a.primary().unwrap().unwrap()
    else {
        panic!("record result")
    };
    assert_eq!(*record, e.program.records()[0].id);
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].1.value, 1);
    assert_eq!(a.primary(), b.primary());
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    let item = e
        .program
        .locals()
        .iter()
        .find(|local| local.name == "item")
        .unwrap()
        .id;
    let records = &a.capture().records;
    let cuts: Vec<_> = records
        .iter()
        .enumerate()
        .filter_map(|(i, r)| matches!(r, Record::Cut(..)).then_some(i))
        .collect();
    assert!(cuts.windows(2).any(|cut| {
        let rows = &records[cut[0]..cut[1]];
        rows.contains(&Record::Slot(Place::Local(item), 2))
            && rows
                .iter()
                .any(|r| matches!(r, Record::Reference(BorrowKind::Shared, ..)))
    }));
}
#[test]
fn i02_repeated_scopes_expire_shared_handles_before_exclusive_access_and_move() {
    equal_bool("type N=range 0..2;fn seed()->N{return 2;}fn demo()->bool{let mutable item=true;let mutable count=seed();while !(count==0){let first=&mut item;let second=first;*second=false;count=count-1;}return item;}", false);
    equal_bool("type N=range 0..3;fn seed()->N{return 3;}fn demo()->bool{let mutable value=true;let mutable count=seed();while !(count==0){let shared=&value;let seen=*shared;let exclusive=&mut value;*exclusive=false;count=count-1;}return value;}",false);
    equal_bool("type N=range 0..2;record R{x:N}fn seed()->N{return 2;}fn consume(item:R){}fn demo()->bool{let mutable count=seed();while !(count==0){let item=R{x:1};let view=&item;let seen=view.x;consume(item);count=count-1;}return true;}",true);
}
#[test]
fn i02_repeated_activations_and_exclusive_transfers_keep_distinct_provenance() {
    equal_bool("fn round(value:bool)->bool{let mutable item=value;let shared=&item;let seen=*shared;let first=&mut item;let second=first;*second=false;return item;}fn demo()->bool{let first=round(true);return round(first);}",false);
    equal_bool("fn demo()->bool{let mutable item=true;let first=&mut item;*first=false;let second=&mut item;*second=true;return item;}",true);
}
#[test]
fn i02_foreign_wrong_referent_and_permission_bits_remain_invalid() {
    let source="fn demo()->bool{let value=true;let other=false;let first=&value;let alias=first;return *alias;}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let other = e
        .program
        .locals()
        .iter()
        .find(|local| local.name == "other")
        .unwrap()
        .id;
    for mutation in 0..3 {
        let b = session(&c, 1024);
        let mut observer = b.machine().observation.borrow_mut();
        // Corrupt every occurrence, so cross-snapshot agreement cannot certify it.
        for record in &mut observer.as_mut().unwrap().records {
            if let Record::Reference(kind, _, frame, place) = record {
                match mutation {
                    0 => *frame = 9_000_000,
                    1 => *place = Place::Local(other),
                    _ => *kind = BorrowKind::Mutable,
                }
            }
        }
        drop(observer);
        assert_eq!(
            compare(&a, &b, comparison()),
            Comparison::Inconclusive(Incomplete::InvalidState)
        );
    }
}
#[test]
fn i02_duplicate_mutable_transfer_provenance_rejects_even_when_both_views_agree() {
    let source="fn demo()->bool{let mutable item=true;let first=&mut item;let second=first;*second=false;return item;}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    let first = e
        .program
        .locals()
        .iter()
        .find(|local| local.name == "first")
        .unwrap()
        .id;
    let second = e
        .program
        .locals()
        .iter()
        .find(|local| local.name == "second")
        .unwrap()
        .id;
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    for session in [&a, &b] {
        let mut observer = session.machine().observation.borrow_mut();
        let capture = observer.as_mut().unwrap();
        let value = capture
            .records
            .iter()
            .find_map(|r| match r {
                Record::Reference(BorrowKind::Mutable, ..) => Some(*r),
                _ => None,
            })
            .unwrap();
        assert!(capture
            .records
            .iter()
            .any(|r| *r == Record::Slot(Place::Local(second), 1)));
        // Keep the original producing source available throughout the transfer.
        // Its history is never Moved then Available: duplicate production, not
        // resurrection, must be the effective discriminator.
        let mut count = 0;
        let mut at = 0;
        while at < capture.records.len() {
            if capture.records[at] == Record::Slot(Place::Local(first), 2) {
                capture.records[at] = Record::Slot(Place::Local(first), 1);
                capture.records.insert(at + 1, value);
                count += 1;
                at += 1;
            }
            at += 1;
        }
        assert!(count > 1);
        // Private malformed builder only: reference bits fit the same physical
        // Slot. Inventories/capacities/accounting stay intact; the larger private
        // record envelope must be coherent before semantic validation starts.
        capture.sealed_count = Some(capture.records.len());
    }
    assert_protection(&a, &b, Protection::Duplicate);
}
#[test]
fn i02_expired_reference_use_is_not_granted_authority_by_stored_bits() {
    for source in [
        "fn demo()->bool{let mutable item=true;let shared=&item;let seen=*shared;let exclusive=&mut item;*exclusive=false;return *shared;}",
        "type N=range 0..1;record R{x:N}fn consume(item:R){}fn demo()->N{let item=R{x:1};let view=&item;let seen=view.x;consume(item);return view.x;}",
    ] {
        assert!(matches!(crate::compile_source(source), Err(crate::FrontendError::Ownership(_))));
        assert!(prepare_execution(source, AdmissionLimits::default()).is_err());
    }
    // Replace the old unsealed, unmatched Executing insertions with the exact
    // matching queued/reached witness. The owner remains available, so only
    // independent approved-transition binding discriminates this stale access.
    coordinated_stale_case();
}

fn disabled<T>(boundary: Protection, action: impl FnOnce() -> T) -> T {
    struct Restore(u8);
    impl Drop for Restore {
        fn drop(&mut self) {
            DISABLED_PROTECTIONS.with(|slot| slot.set(self.0));
        }
    }
    let restore = Restore(DISABLED_PROTECTIONS.with(|slot| slot.replace(boundary as u8)));
    let result = action();
    drop(restore);
    result
}
fn assert_protection(a: &Session<'_>, b: &Session<'_>, boundary: Protection) {
    REJECTED_PROTECTION.with(|slot| slot.set(None));
    assert_eq!(
        compare(a, b, comparison()),
        Comparison::Inconclusive(Incomplete::InvalidState)
    );
    assert_eq!(REJECTED_PROTECTION.with(|slot| slot.get()), Some(boundary));
    assert_eq!(
        disabled(boundary, || compare(a, b, comparison())),
        Comparison::Equal
    );
    eprintln!("K35 semantic discriminator: {boundary:?} rejects InvalidState; removing only that guard yields Equal");
}
fn return_and_deref(c: &contract::Contract<'_>) -> (usize, usize) {
    let function = c.executable.function(c.entry()).unwrap();
    let value = function
        .blocks()
        .find_map(|(_, b)| match b.terminator() {
            Terminator::Return { value } => Some(value),
            _ => None,
        })
        .unwrap();
    assert!(matches!(value.kind, ExprKind::Local(_)));
    let returned = c
        .expressions()
        .iter()
        .find(|(e, _)| std::ptr::eq(*e, value))
        .unwrap()
        .1;
    let deref = c
        .expressions()
        .iter()
        .find(|(e, _)| matches!(e.kind, ExprKind::Deref { .. }))
        .unwrap()
        .1;
    assert_ne!(returned, deref);
    (returned, deref)
}
fn substitute(session: &Session<'_>, returned: usize, deref: usize, reached_only: bool) {
    let mut observer = session.machine().observation.borrow_mut();
    let capture = observer.as_mut().unwrap();
    let before = capture.records.clone();
    let sealed = capture.sealed_count;
    let capacity = capture.records.capacity();
    let mut executing = false;
    let (mut queued, mut reached) = (0, 0);
    for record in &mut capture.records {
        if matches!(record, Record::Cut(..)) {
            executing = false;
        }
        if *record == Record::Executing {
            executing = true;
        }
        if *record == Record::Expression(returned) && (!reached_only || executing) {
            *record = Record::Expression(deref);
            if executing {
                reached += 1;
            } else {
                queued += 1;
            }
        }
    }
    assert_eq!(reached, 1);
    assert_eq!(queued > 0, !reached_only);
    assert_eq!(capture.records.len(), before.len());
    assert_eq!(capture.sealed_count, sealed);
    assert_eq!(sealed, Some(capture.records.len()));
    assert_eq!(capture.records.capacity(), capacity);
    for (old, new) in before.iter().zip(&capture.records) {
        assert!(
            old == new
                || (*old == Record::Expression(returned) && *new == Record::Expression(deref))
        );
    }
}
fn coordinated_stale_case() {
    let e = prepare_execution(SHARED_THEN_EXCLUSIVE, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    for s in [&a, &b] {
        assert_eq!(
            s.primary().unwrap(),
            Ok(&ExecutionResult::Value(RuntimeValue::Bool(false)))
        );
        assert_eq!(s.completeness(), Ok(()));
    }
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    let (returned, deref) = return_and_deref(&c);
    for s in [&a, &b] {
        substitute(s, returned, deref, false);
    }
    assert_protection(&a, &b, Protection::Operand);
    for s in [&a, &b] {
        assert_eq!(
            s.primary().unwrap(),
            Ok(&ExecutionResult::Value(RuntimeValue::Bool(false)))
        );
    }
}
#[test]
fn i03_coordinated_final_return_substitution_rejects_at_approved_operand_boundary() {
    coordinated_stale_case();
}
#[test]
fn i03_reached_only_substitution_rejects_at_queued_consumption_boundary() {
    let e = prepare_execution(SHARED_THEN_EXCLUSIVE, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    let (returned, deref) = return_and_deref(&c);
    for s in [&a, &b] {
        substitute(s, returned, deref, true);
    }
    assert_protection(&a, &b, Protection::Queue);
}
#[test]
fn i03_same_parent_operand_substitution_cannot_redefine_canonical_order() {
    let source = "fn demo()->bool{let a=true;let b=false;let first=&a;let second=&b;return *first==*second;}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(
        a.primary().unwrap(),
        Ok(&ExecutionResult::Value(RuntimeValue::Bool(false)))
    );
    assert_eq!(a.primary(), b.primary());
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    let children: Vec<_> = c
        .expressions()
        .iter()
        .filter(|(e, _)| matches!(e.kind, ExprKind::Deref { .. }))
        .map(|(_, i)| *i)
        .collect();
    assert_eq!(children.len(), 2);
    let left = c.sites()[children[0]];
    let right = c.sites()[children[1]];
    assert_eq!(
        (left.function, left.block, left.statement),
        (right.function, right.block, right.statement)
    );
    for session in [&a, &b] {
        substitute(session, children[0], children[1], false);
    }
    assert_protection(&a, &b, Protection::Operand);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    for session in [&a, &b] {
        let mut observer = session.machine().observation.borrow_mut();
        let mut changed = 0;
        for record in &mut observer.as_mut().unwrap().records {
            if let Record::Frame(_, _, _, statement, _, _) = record {
                if *statement == left.statement {
                    *statement = 1;
                    changed += 1;
                }
            }
        }
        assert!(changed > 0);
    }
    REJECTED_PROTECTION.with(|slot| slot.set(None));
    assert_eq!(
        compare(&a, &b, comparison()),
        Comparison::Inconclusive(Incomplete::InvalidState)
    );
    assert_eq!(
        REJECTED_PROTECTION.with(|slot| slot.get()),
        Some(Protection::Control)
    );
    // A bad control cursor is rejected independently before its downstream
    // operand/continuation mismatch. Removing Control reaches that second gate.
    REJECTED_PROTECTION.with(|slot| slot.set(None));
    assert_eq!(
        disabled(Protection::Control, || compare(&a, &b, comparison())),
        Comparison::Inconclusive(Incomplete::InvalidState)
    );
    assert_eq!(
        REJECTED_PROTECTION.with(|slot| slot.get()),
        Some(Protection::Operand)
    );
}
#[test]
fn i03_nested_operands_calls_and_short_circuit_remain_authorized() {
    equal_bool("fn read(view:&bool)->bool{return *view;}fn demo()->bool{let value=true;let view=&value;return !(*view && read(view))==false;}", true);
    equal_bool("fn read(first:&bool,second:&bool)->bool{return *first==*second;}fn demo()->bool{let value=true;let view=&value;return read(view,view);}", true);
    equal_bool("fn read(view:&bool)->bool{return *view;}fn demo()->bool{let value=true;let view=&value;return false || read(view);}", true);
}
#[test]
fn i03_approved_transition_binding_preserves_stopped_primary_and_budgets() {
    let e = prepare_execution(SHARED_THEN_EXCLUSIVE, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let mut stopped = 0;
    let mut complete = 0;
    for steps in 1..40 {
        let a = session(&c, steps);
        let b = session(&c, steps);
        assert_eq!(a.primary(), b.primary());
        assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
        match a.primary().unwrap() {
            Ok(_) => complete += 1,
            Err(_) => stopped += 1,
        }
    }
    assert!(stopped > 0 && complete > 0);
}
#[test]
fn i04_resurrected_transfer_rejects_at_reference_history_boundary() {
    let source="fn demo()->bool{let mutable item=true;let first=&mut item;let second=first;*second=false;return item;}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let first = e
        .program
        .locals()
        .iter()
        .find(|l| l.name == "first")
        .unwrap()
        .id;
    let second = e
        .program
        .locals()
        .iter()
        .find(|l| l.name == "second")
        .unwrap()
        .id;
    let steps = (1..40)
        .find(|steps| {
            let s = session(&c, *steps);
            let capture = s.capture();
            let at = capture
                .records
                .iter()
                .rposition(|r| matches!(r, Record::Cut(Cut::Final, ..)))
                .unwrap();
            s.primary().unwrap().is_err()
                && capture.records[at..].contains(&Record::Slot(Place::Local(first), 2))
                && capture.records[at..].contains(&Record::Slot(Place::Local(second), 1))
        })
        .unwrap();
    let a = session(&c, steps);
    let b = session(&c, steps);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    for s in [&a, &b] {
        let mut observer = s.machine().observation.borrow_mut();
        let capture = observer.as_mut().unwrap();
        let length = capture.records.len();
        let cut = capture
            .records
            .iter()
            .rposition(|r| matches!(r, Record::Cut(Cut::Final, ..)))
            .unwrap();
        let at = cut
            + capture.records[cut..]
                .iter()
                .position(|r| *r == Record::Slot(Place::Local(second), 1))
                .unwrap();
        let value = capture.records.remove(at + 1);
        assert!(matches!(value, Record::Reference(BorrowKind::Mutable, ..)));
        capture.records[at] = Record::Slot(Place::Local(second), 2);
        let at = cut
            + capture.records[cut..]
                .iter()
                .position(|r| *r == Record::Slot(Place::Local(first), 2))
                .unwrap();
        capture.records[at] = Record::Slot(Place::Local(first), 1);
        capture.records.insert(at + 1, value);
        assert_eq!(capture.records.len(), length);
        assert_eq!(capture.sealed_count, Some(length));
    }
    assert_protection(&a, &b, Protection::Resurrection);
}
#[test]
fn i04_authorized_reached_access_requires_available_referent() {
    let e = prepare_execution(
        "fn demo()->bool{let value=true;let view=&value;return *view;}",
        AdmissionLimits::default(),
    )
    .unwrap();
    let c = contract(&e);
    let owner = e
        .program
        .locals()
        .iter()
        .find(|l| l.name == "value")
        .unwrap()
        .id;
    let ordinal = c
        .expressions()
        .iter()
        .find(|(e, _)| matches!(e.kind, ExprKind::Deref { .. }))
        .unwrap()
        .1;
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    for s in [&a, &b] {
        let mut observer = s.machine().observation.borrow_mut();
        let capture = observer.as_mut().unwrap();
        let expression = capture
            .records
            .iter()
            .position(|r| *r == Record::Expression(ordinal))
            .unwrap();
        let mut at = capture.records[..expression]
            .iter()
            .rposition(|r| matches!(r, Record::Cut(..)))
            .unwrap();
        let mut changed = 0;
        while at < capture.records.len() {
            if capture.records[at] == Record::Slot(Place::Local(owner), 1) {
                capture.records[at] = Record::Slot(Place::Local(owner), 2);
                assert_eq!(capture.records.remove(at + 1), Record::Bool(true));
                changed += 1;
            }
            at += 1;
        }
        assert!(changed > 1);
        capture.sealed_count = Some(capture.records.len());
    }
    assert_protection(&a, &b, Protection::Access);
}
#[test]
fn i04_capture_length_gate_is_distinct_from_reference_validation() {
    let e = prepare_execution(SHARED_THEN_EXCLUSIVE, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    for s in [&a, &b] {
        s.machine()
            .observation
            .borrow_mut()
            .as_mut()
            .unwrap()
            .records
            .push(Record::Executing);
    }
    REJECTED_PROTECTION.with(|slot| slot.set(None));
    assert_eq!(
        compare(&a, &b, comparison()),
        Comparison::Inconclusive(Incomplete::InvalidState)
    );
    assert_eq!(
        REJECTED_PROTECTION.with(|slot| slot.get()),
        Some(Protection::Seal)
    );
    // Removing only the seal check still leaves an invalid trailing record.
    // This fixture establishes the shape gate, never downstream reference rules.
    assert_eq!(
        disabled(Protection::Seal, || compare(&a, &b, comparison())),
        Comparison::Inconclusive(Incomplete::InvalidState)
    );
}

#[test]
fn repair_canonical_resolution_limits_preserve_primary_and_prevent_truncated_equality() {
    let source = "fn demo()->bool{let item=true;let first=&item;let second=first;return *second;}";
    let e = prepare_execution(source, AdmissionLimits::default()).unwrap();
    let c = contract(&e);
    let a = session(&c, 1024);
    let b = session(&c, 1024);
    assert_eq!(compare(&a, &b, comparison()), Comparison::Equal);
    assert_eq!(
        compare(
            &a,
            &b,
            ComparisonLimits {
                work: 1,
                ..comparison()
            }
        ),
        Comparison::Inconclusive(Incomplete::Work)
    );
    let mut narrow = Session::new(
        &c,
        ExecutionLimits {
            steps: 1024,
            ..ExecutionLimits::default()
        },
        Limits {
            lineage: 1,
            ..observation()
        },
    )
    .unwrap();
    narrow.advance().unwrap();
    assert_eq!(narrow.completeness(), Ok(()));
    assert_eq!(
        compare(&narrow, &narrow, comparison()),
        Comparison::Inconclusive(Incomplete::Lineage)
    );
    assert_eq!(narrow.primary(), a.primary());
    assert_eq!(
        a.primary().unwrap(),
        Ok(&ExecutionResult::Value(RuntimeValue::Bool(true)))
    );
}
