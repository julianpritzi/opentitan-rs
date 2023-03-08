#![no_std]
#![no_main]

use core::arch::asm;

#[allow(unused_imports)]
use opentitan_lib;
use opentitan_lib::entry;

#[entry]
fn main() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}
