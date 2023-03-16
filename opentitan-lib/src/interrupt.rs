use core::arch::{asm, global_asm};

use riscv::register::{
    mcause::{self, Exception, Trap},
    mepc,
};

#[repr(C)]
pub struct ExceptionFrame {
    pub registers: [usize; 32],
    pub pc: usize,
}

global_asm!(
    "
    .section .trap_vectored, \"ax\"
    .option push
    .option norvc
    .option norelax

    .balign 256
    .global _trap_vector
    .type _trap_vector, @function
_trap_vector:
    // Exception and User Software Interrupt Handler.
    j _trap_exception
    // Supervisor Software Interrupt Handler.
    unimp
    // Reserved.
    unimp
    // Machine Software Interrupt Handler.
    // TODO: implement
    unimp

    // User Timer Interrupt Handler.
    unimp
    // Supervisor Timer Interrupt Handler.
    unimp
    // Reserved.
    unimp
    // Machine Timer Interrupt Handler.
    // TODO: implement
    unimp

    // User External Interrupt Handler.
    unimp
    // Supervisor External Interrupt Handler.
    unimp
    // Reserved.
    unimp
    // Machine External Interrupt Handler.
    // TODO: implement
    unimp

    // Reserved.
    unimp
    unimp
    unimp
    unimp

    // Vendor Interrupt Handlers:

    // On Ibex, interrupt IDs 16-30 are for 'fast' interrupts.
    .rept 15
    unimp
    .endr

    // On Ibex, interrupt ID 31 is for non-maskable interrupts
    // TODO: do we need this?
    unimp

    // Set size
    .size _ottf_interrupt_vector, .-_ottf_interrupt_vector

    .option pop
"
);

#[link_section = ".trap"]
#[export_name = "_trap_exception"]
#[naked]
pub extern "C" fn _trap_exception() {
    unsafe {
        asm!(
            "
            // Save registers in frame
            addi sp, sp, -33*4

            sw zero, 0*4(sp)
            sw ra, 1*4(sp)
            sw gp, 3*4(sp)
            sw tp, 4*4(sp)
            sw t0, 5*4(sp)
            sw t1, 6*4(sp)
            sw t2, 7*4(sp)
            sw fp, 8*4(sp)
            sw s1, 9*4(sp)
            sw a0, 10*4(sp)
            sw a1, 11*4(sp)
            sw a2, 12*4(sp)
            sw a3, 13*4(sp)
            sw a4, 14*4(sp)
            sw a5, 15*4(sp)
            sw a6, 16*4(sp)
            sw a7, 17*4(sp)
            sw s2, 18*4(sp)
            sw s3, 19*4(sp)
            sw s4, 20*4(sp)
            sw s5, 21*4(sp)
            sw s6, 22*4(sp)
            sw s7, 23*4(sp)
            sw s8, 24*4(sp)
            sw s9, 25*4(sp)
            sw s10, 26*4(sp)
            sw s11, 27*4(sp)
            sw t3, 28*4(sp)
            sw t4, 29*4(sp)
            sw t5, 30*4(sp)
            sw t6, 31*4(sp)
            // store original stack pointer x2
            addi t0, sp, 33*4
            sw t0, 2*4(sp)
            // store exception pc
            csrr t0, mepc
            sw t0, 32*4(sp)

            // call trap_handler
            add a0, sp, zero
            j _trap_exception_rust

            // restore registers
            lw zero, 0*4(sp)
            lw ra, 1*4(sp)
            lw gp, 3*4(sp)
            lw tp, 4*4(sp)
            lw t0, 5*4(sp)
            lw t1, 6*4(sp)
            lw t2, 7*4(sp)
            lw fp, 8*4(sp)
            lw s1, 9*4(sp)
            lw a0, 10*4(sp)
            lw a1, 11*4(sp)
            lw a2, 12*4(sp)
            lw a3, 13*4(sp)
            lw a4, 14*4(sp)
            lw a5, 15*4(sp)
            lw a6, 16*4(sp)
            lw a7, 17*4(sp)
            lw s2, 18*4(sp)
            lw s3, 19*4(sp)
            lw s4, 20*4(sp)
            lw s5, 21*4(sp)
            lw s6, 22*4(sp)
            lw s7, 23*4(sp)
            lw s8, 24*4(sp)
            lw s9, 25*4(sp)
            lw s10, 26*4(sp)
            lw s11, 27*4(sp)
            lw t3, 28*4(sp)
            lw t4, 29*4(sp)
            lw t5, 30*4(sp)
            lw t6, 31*4(sp)
            // restore stack pointer
            lw sp, 2*4(sp)

            mret
            ",
            options(noreturn)
        );
    }
}

#[link_section = ".trap"]
#[export_name = "_trap_exception_rust"]
pub extern "C" fn _trap_exception_rust(trap_frame: *mut ExceptionFrame) {
    unsafe {
        let cause = mcause::read().cause();

        #[cfg(feature = "atomic_emulation")]
        {
            if let Trap::Exception(Exception::IllegalInstruction) = cause {
                use riscv_atomic_emulation_trap::atomic_emulation;
                if atomic_emulation((*trap_frame).pc, &mut (*trap_frame).registers) {
                    // Emulation was successfull, increment PC and finish exception handling
                    mepc::write((*trap_frame).pc + 1);
                    return;
                }
            }
        }

        EXCEPTION_HANDLER(cause, trap_frame);
    }
}

pub static mut EXCEPTION_HANDLER: fn(Trap, *mut ExceptionFrame) = _default_exception_handler;

pub fn _default_exception_handler(cause: Trap, trap_frame: *mut ExceptionFrame) {
    panic!("Unhandled Trap {:?} at PC {:#x}", cause, unsafe {
        (*trap_frame).pc
    })
}
