// Userspace debugger
#[macro_export]
macro_rules! debug_process {
    ($($arg:tt)+) => {
        log::debug!("[Now: {} | P{}] {}", now(), rank(), format_args!($($arg)+));
    }
}
