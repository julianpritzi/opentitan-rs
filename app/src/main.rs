#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(opentitan_lib::tests::test_runner)]
#![reexport_test_harness_main = "__ot_lib_test_main"]

extern crate alloc;

#[allow(unused_imports)]
use opentitan_lib::{entry, log, print, println};

#[entry]
fn main() -> ! {
    log!("Test");

    opentitan_lib::suspend();
}

#[cfg(test)]
mod tests {
    #[test_case]
    pub fn test() {
        assert!(true);
    }
}
