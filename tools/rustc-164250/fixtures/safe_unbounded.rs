struct Plain<T>(Option<T>);
struct Array<T, const N: usize>([T; N]);

fn main() {
    let _text = "struct Hidden<T: 'static>(T);";
    // struct Commented<T: 'static>(T);
    let _raw = r#"enum Raw<T: Copy> { Value(T) }"#;
}
