#[macro_export]
macro_rules! check_error {
   ($cond:expr) => { if !($cond) { return Err(std::io::Error::last_os_error()); } };
}

#[macro_export]
macro_rules! abort_if {
   ($cond:expr) => { if ($cond) { std::process::abort(); } };
}