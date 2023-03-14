#![no_std]
#![no_main]

use core::arch::asm;

extern crate alloc;

#[allow(unused_imports)]
use opentitan_lib::{entry, log, print, println};

#[entry]
fn main() -> ! {
    log!("Test");

    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
