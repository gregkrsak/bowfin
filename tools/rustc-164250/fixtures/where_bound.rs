struct RequiresStatic<T>(Option<T>)
where
    T: 'static;

fn main() {}
