#![no_std]
#![no_main]
#![feature(naked_functions)]

pub mod devices;
pub mod print;
pub mod synch;

#[cfg(feature = "alloc")]
mod alloc;

use core::{arch::asm, panic::PanicInfo, ptr};
pub use opentitan_macros::entry;

/// Specifies the stack size
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

    #[cfg(feature = "verbose_logging")]
    log!("Finished library initialization, jumping to entry");

    unsafe {
        asm!("j main", options(noreturn));
    }
}

#[panic_handler]
pub fn _default_panic_handler(info: &PanicInfo) -> ! {
    use core::fmt::Write;
    unsafe {
        riscv::interrupt::disable();
        let mut out = devices::uart::get_panic_uart();
        let _ = writeln!(out, "[Panic] {}", info);

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
