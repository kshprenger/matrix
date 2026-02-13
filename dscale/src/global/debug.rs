// Userspace debugger
#[macro_export]
macro_rules! Debug {
    ($($arg:tt)+) => {
        log::debug!("[Now: {} | P{}] {}", Now(), Rank(), format_args!($($arg)+));
    }
}
