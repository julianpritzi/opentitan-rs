//! Driver code for the opentitan uart
//!
//! TODO:
//!     - make functions on UartRegisters unsafe by default
//!     - macro for const uart generation
//!     - safe wrapper for raw uart

use super::addresses;
use opentitan_macros::registers;

#[registers("hw/ip/uart/data/uart.hjson")]
pub struct UartRegisters;

const UART0: *const UartRegisters = addresses::UART0 as *const UartRegisters;

/// Returns a pointer to the registers of uart0
///
/// This should only be used if either [`RawUart`] or [`Uart`] do not meet the
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
pub unsafe fn get_uart0_raw() -> *const impl UartRaw {
    UART0
}

pub enum UartMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub trait UartRaw {
    unsafe fn configure(&mut self, mode: UartMode, baudrate: Option<u32>);

    unsafe fn send_blocking(&mut self, data: &[u8]);

    unsafe fn recieve_blocking(&mut self, data: &mut [u8]);
}

impl UartRaw for UartRegisters {
    unsafe fn configure(&mut self, mode: UartMode, baudrate: Option<u32>) {
        todo!()
    }

    unsafe fn send_blocking(&mut self, data: &[u8]) {
        todo!()
    }

    unsafe fn recieve_blocking(&mut self, data: &mut [u8]) {
        todo!()
    }
}

pub trait Uart {
    fn reconfigure(&mut self, mode: UartMode, baudrate: Option<u32>);

    fn send_blocking(&mut self, data: &[u8]);

    fn recieve_blocking(&mut self, data: &mut [u8]);
}
