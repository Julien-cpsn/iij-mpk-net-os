use crate::drivers::serial::SERIAL;
use core::fmt;

#[macro_export]
macro_rules! kprintln {
    () => ($crate::utils::macros::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::utils::macros::_print(format_args!("{}\n", format_args!($($arg)*))));
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => ($crate::utils::macros::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! kdbg {
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                $crate::kprintln!("{} = {:#?}", stringify!($val), &tmp as &dyn core::fmt::Debug);
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
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL.lock().write_fmt(args).unwrap();
    });
}