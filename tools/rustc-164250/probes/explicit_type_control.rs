#![allow(dead_code)]

struct RequiresStatic<T: 'static>(Option<T>);

fn explicit_type_restores_the_required_check() {
    let local = String::from("temporary");
    let mut value: RequiresStatic<&'static String> = RequiresStatic(None);
    value.0 = Some(&local);
}
