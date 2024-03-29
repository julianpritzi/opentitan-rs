#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::tests::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod devices;
pub mod interrupt;
pub mod print;
pub mod synch;

pub mod tests;

#[cfg(feature = "alloc")]
mod alloc;
mod atomic;

use core::{arch::asm, ptr};
pub use opentitan_macros::entry;
use riscv::register::mtvec;

/// Specifies the stack size
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x4000] = [0; 0x4000];

// Values provided by linker script
extern "C" {
    static __global_pointer: usize;

    /// End address of the stack
    static _estack: usize;

    /// Start address of BSS section
    static _szero: usize;
    /// End address of BSS section (exclusive)
    static _ezero: usize;

    /// Start address of heap
    static _sheap: usize;
    /// End address of heap
    static _eheap: usize;

    /// Start address of data section
    static _srelocate: usize;
    /// End address of data section (exclusive)
    static _erelocate: usize;

    /// End address of text section
    static _etext: usize;

    /// Start address of trap vector
    static _trap_vector: usize;

    pub fn main();
}

#[link_section = ".start"]
#[export_name = "_start"]
#[naked]
pub extern "C" fn _start() {
    unsafe {
        asm!(
            "
            // Set global pointer register
            lui  gp, %hi({gp}$)
            addi gp, gp, %lo({gp}$)

            // Set stack pointer
            lui  sp, %hi({estack})
            addi sp, sp, %lo({estack})

            // Set frame pointer
            add s0, sp, zero

            // --- Clear BSS ---
            la a0, {sbss}
            la a1, {ebss}

            100: // clear bss loop
            beq  a0, a1, 101f      
            sw   zero, 0(a0)          
            addi a0, a0, 4
            j    100b
            101: // clear bss loop end

            // --- Initialize data ---
            la a0, {sdata}
            la a1, {edata}
            la a2, {etext}

            200: // init data loop
            beq  a0, a1, 201f
            lw   a3, 0(a2)
            sw   a3, 0(a0)
            addi a0, a0, 4
            addi a2, a2, 4
            j    200b
            201: // init data loop end

            j _init
            ",
            gp = sym __global_pointer,
            estack = sym _estack,
            sbss = sym _szero,
            ebss = sym _ezero,
            sdata = sym _srelocate,
            edata = sym _erelocate,
            etext = sym _etext,
            options(noreturn)
        );
    }
}

#[export_name = "_init"]
pub unsafe extern "C" fn _init() -> ! {
    print::redirect_stdout(devices::uart::get_uart0().expect("Could not acquire uart for stdout"));
    #[cfg(feature = "alloc")]
    {
        let heap_size = (ptr::addr_of!(_eheap) as usize) - (ptr::addr_of!(_sheap) as usize);
        crate::alloc::ALLOCATOR.init(ptr::addr_of!(_sheap) as *mut u8, heap_size);
        #[cfg(feature = "verbose_logging")]
        {
            log!("Heap set up with size {:#x}", heap_size);
        }
    }

    mtvec::write(
        ptr::addr_of!(_trap_vector) as usize,
        riscv::register::utvec::TrapMode::Vectored,
    );

    #[cfg(feature = "verbose_logging")]
    log!("Finished library initialization, jumping to entry");

    #[cfg(test)]
    {
        test_main();
        suspend();
    }
    #[cfg(not(test))]
    unsafe {
        asm!("j main", options(noreturn));
    }
}

#[inline]
pub fn suspend() -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

mod panic {
    use crate::{devices, suspend, tests};
    use core::panic::PanicInfo;

    #[panic_handler]
    pub fn _default_panic_handler(info: &PanicInfo) -> ! {
        if unsafe { tests::_USE_TEST_PANIC_HANDLER } {
            tests::panic(info)
        } else {
            use core::fmt::Write;
            unsafe {
                riscv::interrupt::disable();
                let mut out = devices::uart::get_panic_uart();
                let _ = writeln!(out, "[Panic] {}", info);

                suspend();
            }
        }
    }
}
