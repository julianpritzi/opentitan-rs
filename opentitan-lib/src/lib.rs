#![no_std]
#![no_main]
#![feature(naked_functions)]

use core::{arch::asm, panic::PanicInfo};
pub use opentitan_macros::entry;

/// Specifies the
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

// Values provided by linker script
extern "C" {
    static __global_pointer: usize;

    /// End address of the stack
    static _estack: usize;

    /// Start address of BSS section
    static _szero: usize;
    /// End address of BSS section (exclusive)
    static _ezero: usize;

    /// Start address of data section
    static _srelocate: usize;
    /// End address of data section (exclusive)
    static _erelocate: usize;

    /// End address of text section
    static _etext: usize;
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

            j main
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

#[panic_handler]
pub fn panic_handler(_: &PanicInfo) -> ! {
    unsafe {
        asm!(
            "
            100: 
            wfi
            j 100b
            ",
            options(noreturn)
        );
    }
}
