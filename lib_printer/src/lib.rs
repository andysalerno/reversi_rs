#[macro_export]
macro_rules! out {
    ($($arg:tt)*) => (out_impl(format!($($arg)*)))
}

#[cfg(not(test))]
pub fn out_impl(s: String) {
    println!("{}", s);
}

#[cfg(test)]
pub fn out_impl(_s: String) {}

#[allow(unused)]
fn test_out() {
    out!("{}{}", "hello", "world");
}
