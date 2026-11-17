use std::mem::{align_of, offset_of, size_of};
#[repr(C)]
#[derive(Default)]
struct Wide {
    lo: u64,
    hi: u64,
}
type Callback = unsafe extern "C" fn(*mut u32, *const Wide, *const u8) -> u32;
extern "C" {
    fn b38_probe(a: i64, b: i64, out: *mut Wide, callback: Callback, context: *mut u32) -> u32;
}
unsafe extern "C" fn callback(context: *mut u32, wide: *const Wide, guard: *const u8) -> u32 {
    if *context != 0 {
        return 7;
    }
    *context = 1 + (((*wide).lo ^ (*wide).hi) & 3) as u32;
    u32::from(std::ptr::read_volatile(guard))
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let a = args[0].parse().unwrap();
    let b = args[1].parse().unwrap();
    let mut context = args[2].parse().unwrap();
    let mut out = Wide::default();
    assert_eq!((size_of::<Wide>(), align_of::<Wide>(), offset_of!(Wide, lo), offset_of!(Wide, hi)), (16, 8, 0, 8), "X01_LAYOUT");
    let out_ptr = std::ptr::addr_of_mut!(out);
    let context_ptr = std::ptr::addr_of_mut!(context);
    let status = unsafe { b38_probe(a, b, out_ptr, callback, context_ptr) };
    println!("{} {} {} {}", out.lo, out.hi, status, context);
}
