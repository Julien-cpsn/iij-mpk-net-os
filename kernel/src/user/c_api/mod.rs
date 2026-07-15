mod rust_code;

unsafe extern "C" {
    pub fn c_add_numbers(a: i32, b: i32) -> i32;
}