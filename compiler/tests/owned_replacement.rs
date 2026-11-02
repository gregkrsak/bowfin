use bowfin_compiler::{compile_source, mir, FrontendError};

const DECL: &str = "type P=range 0..100;record R{x:P}record S{x:P}fn fresh()->R{return R{x:1};}fn inspect(v:&R)->R{return R{x:v.x};}fn touch(v:&mut R)->R{v.x=2;return R{x:v.x};}fn pass(v:R)->R{return v;}fn pair(a:R,b:R)->R{return a;}";

fn accepted(body: &str) {
    let p = compile_source(&format!("{DECL}{body}")).unwrap();
    for f in mir::lower(&p).functions() {
        f.validate().unwrap();
    }
}
fn rejected(body: &str, message: &str) {
    let error = compile_source(&format!("{DECL}{body}")).unwrap_err();
    assert!(matches!(error, FrontendError::Ownership(_)), "{error}");
    assert!(error.to_string().contains(message), "{error}");
}

#[test]
fn repeated_and_distinct_source_replacement() {
    accepted("fn f()->R{let mut a=fresh();a=fresh();let b=fresh();a=b;a=fresh();return a;}");
}
#[test]
fn final_shared_and_exclusive_borrow_uses() {
    accepted("fn f()->R{let mut a=fresh();let v=&a;a=inspect(v);a=touch(&mut a);return a;}");
}
#[test]
fn later_shared_handle_blocks_commit() {
    rejected(
        "fn f()->R{let mut a=fresh();let v=&a;a=inspect(v);return inspect(v);}",
        "while it is borrowed",
    );
}
#[test]
fn later_exclusive_handle_blocks_commit() {
    rejected(
        "fn f()->R{let mut a=fresh();let v=&mut a;a=R{x:v.x};return touch(v);}",
        "while it is borrowed",
    );
}
#[test]
fn target_consumption_rejects_at_every_rhs_depth() {
    for rhs in ["a", "pass(a)", "pair(fresh(),pass(a))"] {
        rejected(
            &format!("fn f()->R{{let mut a=fresh();a={rhs};return a;}}"),
            "inside its own RHS",
        );
    }
}
#[test]
fn moved_and_conditionally_unavailable_targets_reject() {
    rejected(
        "fn f()->R{let mut a=fresh();let b=a;a=fresh();return b;}",
        "moved",
    );
    rejected(
        "fn f(flag:bool)->R{let mut a=fresh();if flag{let b=a;}a=fresh();return a;}",
        "moved",
    );
}
#[test]
fn iteration_local_repeated_replacement_and_exits() {
    for exit in ["", "break;", "continue;", "return;"] {
        accepted(&format!(
            "fn f(flag:bool){{while flag{{let mut a=fresh();a=fresh();a=fresh();{exit}}}}}"
        ));
    }
}
#[test]
fn header_target_and_outer_iteration_target_reject() {
    rejected(
        "fn f(flag:bool){let mut a=fresh();while flag{a=fresh();break;}}",
        "cannot replace pre-existing",
    );
    rejected(
        "fn f(flag:bool){while flag{let mut a=fresh();while flag{a=fresh();break;}}}",
        "cannot replace pre-existing",
    );
}
#[test]
fn header_source_is_not_authorized_by_following_return() {
    rejected(
        "fn f(flag:bool,item:R)->R{while flag{let mut a=fresh();a=item;return a;}return fresh();}",
        "cannot move pre-existing",
    );
}
#[test]
fn branches_preserve_target_and_conservative_source_joins() {
    accepted("fn f(flag:bool)->R{let mut a=fresh();if flag{a=fresh();}else{a=fresh();}return a;}");
    rejected(
        "fn f(flag:bool)->R{let mut a=fresh();let b=fresh();if flag{a=b;}return b;}",
        "moved",
    );
}
#[test]
fn exact_nominal_type_and_reference_boundaries() {
    for body in [
        "fn f(){let mut a=fresh();a=S{x:1};}",
        "fn f(){let mut a=fresh();let v=&mut a;v=&mut a;}",
        "fn f(){let mut a=fresh();let v=&mut a;*v=fresh();}",
    ] {
        assert!(compile_source(&format!("{DECL}{body}")).is_err());
    }
}

#[test]
fn transferred_exclusive_handle_remains_moved() {
    rejected(
        "fn f()->R{let mut a=fresh();let v=&mut a;a=touch(v);return touch(v);}",
        "use of moved value",
    );
}
