#![allow(dead_code)]

struct RequiresStatic<T: 'static>(Option<T>);

fn affected_compiler_accepts_invalid_local_type() {
    let local = String::from("temporary");
    let mut value = RequiresStatic(None);
    value.0 = Some(&local);
}
