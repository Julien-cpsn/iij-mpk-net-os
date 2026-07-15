use alloc::string::ToString;
use core::fmt;
use crate::user::api::user_syscalls::{log, print};

#[macro_export]
macro_rules! println {
    () => ($crate::user::api::macros::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::user::api::macros::_print(format_args!("{}\n", format_args!($($arg)*))));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::user::api::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! dbg {
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::println!("{} = {:#?}", stringify!($val), &tmp as &dyn core::fmt::Debug);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($($crate::dbg!($val)),+,)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ({
        $crate::user::api::macros::_log(TARGET, 0, format_args!($($arg)*))
    });
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ({
        $crate::user::api::macros::_log(TARGET, 1, format_args!($($arg)*))
    });
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ({
        $crate::user::api::macros::_log(TARGET, 2, format_args!($($arg)*))
    });
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ({
        $crate::user::api::macros::_log(TARGET, 3, format_args!($($arg)*))
    });
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => ({
        $crate::user::api::macros::_log(TARGET, 4, format_args!($($arg)*))
    });
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Some(text) = args.as_str() {
        print(text.as_ptr(), text.len());
    }
    else {
        let text = args.to_string();
        print(text.as_ptr(), text.len());
    };
}

#[doc(hidden)]
pub fn _log(target: &str, level: usize, args: fmt::Arguments) {
    if let Some(text) = args.as_str() {
        log(target, level, text.as_ptr(), text.len());
    }
    else {
        let text = args.to_string();
        log(target, level, text.as_ptr(), text.len());
    };
}