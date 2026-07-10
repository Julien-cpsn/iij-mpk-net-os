use alloc::string::ToString;
use core::fmt;
use crate::apps::user::syscalls::print;

#[macro_export]
macro_rules! println {
    () => ($crate::apps::user::macros::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::apps::user::macros::_print(format_args!("{}\n", format_args!($($arg)*))));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::apps::user::macros::_print(format_args!($($arg)*)));
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