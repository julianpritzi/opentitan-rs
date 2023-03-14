//! Driver code for the opentitan uart IP
//!
//! TODO:
//!     - make functions on UartRegisters unsafe by default
//!     - macro for const uart generation
//!     - safe wrapper for raw uart

use core::fmt::Write;

use crate::synch::Lock;

use super::{addresses, platform};
use opentitan_macros::registers;
use tock_registers::interfaces::{Readable, Writeable};

#[registers("hw/ip/uart/data/uart.hjson")]
pub struct UartRegisters;

const UART0: *mut UartRegisters = addresses::UART0 as *mut UartRegisters;
static mut UART0_LOCK: Lock = Lock::new();

/// Returns a pointer to the registers of uart0
///
/// This should only be used if either [`UartRaw`] or [`Uart`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the uart registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_uart0_registers() -> *const UartRegisters {
    UART0
}

/// Returns a pointer to the uart0
///
/// This should only be used if [`Uart`] does not meet the
/// requirements (eg. performance or functionality)
///
/// The returned value is an unsafe wrapper for the [`UartRegisters`] struct
/// that implements a set of commonly used functionality.
///
/// # Safety
/// Reading and modifying the uart registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_uart0_raw() -> *mut impl UartRaw {
    UART0
}

pub unsafe fn get_panic_uart() -> Uart {
    unsafe { Uart::new(UART0, &mut UART0_LOCK) }
}

pub fn get_uart0() -> Result<Uart, ()> {
    unsafe {
        if UART0_LOCK.try_lock().is_ok() {
            Ok(Uart::new(UART0, &mut UART0_LOCK))
        } else {
            Err(())
        }
    }
}

pub trait UartRaw {
    unsafe fn configure(&mut self, baudrate: Option<u32>);

    unsafe fn send_blocking(&mut self, data: &[u8]);

    unsafe fn recieve_blocking(&mut self, data: &mut [u8]);

    unsafe fn try_send(&mut self, data: &u8) -> Result<(), ()>;

    unsafe fn try_recieve(&mut self, data: &mut u8) -> Result<(), ()>;

    unsafe fn flush(&mut self);
}

impl UartRaw for UartRegisters {
    unsafe fn configure(&mut self, baudrate: Option<u32>) {
        let nco = ((baudrate.unwrap_or(platform::UART_BAUD_RATE) as u64) << 20u64)
            / platform::PERIPHERAL_FREQ as u64;
        self.ctrl
            .write(ctrl::nco.val((nco & 0xffff) as u32) + ctrl::rx::SET + ctrl::tx::SET);

        self.fifo_ctrl
            .write(fifo_ctrl::txrst::SET + fifo_ctrl::rxrst::SET);
    }

    unsafe fn send_blocking(&mut self, data: &[u8]) {
        for val in data {
            while self.try_send(val).is_err() {}
        }
    }

    unsafe fn recieve_blocking(&mut self, data: &mut [u8]) {
        for val in data {
            while self.try_recieve(val).is_err() {}
        }
    }

    unsafe fn try_send(&mut self, data: &u8) -> Result<(), ()> {
        if !self.status.is_set(status::txfull) {
            self.wdata.write(wdata::data.val(*data as u32));
            Ok(())
        } else {
            Err(())
        }
    }

    unsafe fn try_recieve(&mut self, data: &mut u8) -> Result<(), ()> {
        if !self.status.is_set(status::rxempty) {
            *data = self.rdata.read(rdata::data) as u8;
            Ok(())
        } else {
            Err(())
        }
    }

    unsafe fn flush(&mut self) {
        while !self.status.is_set(status::txidle) {}
    }
}

pub struct Uart {
    regs: *mut UartRegisters,
    lock: *mut Lock,
}

impl Uart {
    unsafe fn new(regs: *mut UartRegisters, lock: *mut Lock) -> Uart {
        let mut uart = Uart { regs, lock };
        uart.reconfigure(None);
        uart
    }

    pub fn reconfigure(&mut self, baudrate: Option<u32>) {
        unsafe {
            (*self.regs).flush();
            (*self.regs).configure(baudrate);
        }
    }

    pub fn send_blocking(&mut self, data: &[u8]) {
        unsafe {
            (*self.regs).send_blocking(data);
        }
    }

    pub fn recieve_blocking(&mut self, data: &mut [u8]) {
        unsafe {
            (*self.regs).recieve_blocking(data);
        }
    }
}

impl Drop for Uart {
    fn drop(&mut self) {
        unsafe { (*self.lock).unlock() }
    }
}

impl Write for Uart {
    fn write_str(&mut self, data: &str) -> core::fmt::Result {
        unsafe {
            for c in data.as_bytes() {
                // Convert \n to \r\n
                if *c == '\n' as u8 {
                    while (*self.regs).try_send(&('\r' as u8)).is_err() {
                        core::hint::spin_loop();
                    }
                }
                while (*self.regs).try_send(c).is_err() {
                    core::hint::spin_loop();
                }
            }
            Ok(())
        }
    }
}
