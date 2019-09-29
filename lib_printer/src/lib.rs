#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => (out_impl(format!($($arg)*)))
}

pub fn out_impl(s: String) {
    // println!("{}", s);
}

#[allow(unused)]
fn test_out() {
    out!("{}{}", "hello", "world");
}
