#![no_std]
#![no_main]

extern crate alloc;

#[allow(unused_imports)]
use opentitan_lib::{entry, log, print, println};

#[entry]
fn main() -> ! {
    log!("Test");

    opentitan_lib::suspend();
}
