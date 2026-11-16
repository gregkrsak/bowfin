struct InvokeOnDrop<T: Copy, F: FnMut(T)>(T, F);

fn main() {}
