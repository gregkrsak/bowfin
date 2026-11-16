struct RequiresStatic<T: 'static>(Option<T>);

fn main() {
    let local = String::from("temporary");
    let mut value = RequiresStatic(None);
    value.0 = Some(&local);
}
