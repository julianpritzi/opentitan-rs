use core::panic::PanicInfo;

use crate::{devices, print, println, suspend};

/// Signal to the testing suite that the current test is skipped
#[macro_export]
macro_rules! mark_test_as_skipped {
    () => {
        unsafe {
            $crate::tests::TEST_RESULT_SKIPPED = true;
        }
    };
}

/// Used by the `mark_test_as_skipped!` macro to signal that the current test was skipped
pub static mut TEST_RESULT_SKIPPED: bool = false;

/// Helper trait to allow automatically outputting text before and after a test function is run.
pub trait TestFunction {
    fn test_run(&self) -> ();
}

impl<T> TestFunction for T
where
    T: Fn(),
{
    fn test_run(&self) {
        print!("{}... ", core::any::type_name::<T>());

        unsafe { TEST_RESULT_SKIPPED = false };
        self();
        if unsafe { TEST_RESULT_SKIPPED } {
            println!("[skipped]");
        } else {
            println!("[ok]")
        }
    }
}

/// Automatically called to run all the tests
pub fn test_runner(tests: &[&dyn TestFunction]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.test_run();
    }
    println!("All tests passed!");
}

/// Automatically called when the suite tests fail/panic
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    unsafe {
        riscv::interrupt::disable();
        let mut out = devices::uart::get_panic_uart();
        let _ = writeln!(out, "[failed]");
        let _ = writeln!(out, "[Error] {}", info);
    }
    suspend();
}
