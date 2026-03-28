// Simple color-coded logging macros for Cangkang.
// Using ANSI escape codes for zero-dependency coloring.

pub const RESET: &str = "\x1b[0m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const BOLD: &str = "\x1b[1m";

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        println!("{}{}[INFO]{} {}", $crate::logger::BOLD, $crate::logger::BLUE, $crate::logger::RESET, format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_success {
    ($($arg:tt)*) => {
        println!("{}{}[ OK ]{} {}", $crate::logger::BOLD, $crate::logger::GREEN, $crate::logger::RESET, format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        eprintln!("{}{}[WARN]{} {}", $crate::logger::BOLD, $crate::logger::YELLOW, $crate::logger::RESET, format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("{}{}[ERR ]{} {}", $crate::logger::BOLD, $crate::logger::RED, $crate::logger::RESET, format_args!($($arg)*));
    }
}
