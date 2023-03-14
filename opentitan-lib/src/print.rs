//! Adds support for the print macro
use crate::devices::uart::Uart;

pub static mut STD_OUT: Option<Uart> = None;

pub unsafe fn redirect_stdout(uart: Uart) {
    STD_OUT = Some(uart)
}

/// Print macro that can be used like the print macro from rust's standard library.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        #[allow(unused_unsafe)]
        unsafe {
            use core::fmt::Write;
            if let Some(out) = &mut $crate::print::STD_OUT {
                write!(out, $($arg)*).unwrap();
            }
        }
    );
}

/// Println macro that can be used like the println macro from rust's standard library.
#[macro_export]
macro_rules! println {
    () => (
        #[allow(unused_unsafe)]
        unsafe {
            use core::fmt::Write;
            if let Some(out) = &mut $crate::print::STD_OUT {
                writeln!(out).unwrap();
            }
        }
    );
    ($($arg:tt)*) => (
        #[allow(unused_unsafe)]
        unsafe {
            use core::fmt::Write;
            if let Some(out) = &mut $crate::print::STD_OUT {
                writeln!(out, $($arg)*).unwrap();
            }
        }
    );
}

/// Print macro that can be used like the print macro from rust's standard library.
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => (
        {
            print!("[{}:{}] ", file!(), line!());
            println!($($arg)*);
        }
    );
}
