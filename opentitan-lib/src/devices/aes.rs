//! Driver code for the opentitan aes IP
//!
//! TODO:
//!     - make functions on AesRegisters unsafe by default
//!     - implementation for AesRaw
//!     - implementation for Aes

use super::addresses;
use opentitan_macros::registers;

#[registers("hw/ip/aes/data/aes.hjson")]
pub struct AesRegisters;

const AES: *const AesRegisters = addresses::AES as *const AesRegisters;

/// Returns a pointer to the registers of the aes IP
///
/// This should only be used if either [`RawAes`] or [`Aes`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the aes registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_aes_registers() -> *const AesRegisters {
    AES
}

/// Returns a pointer to the aes IP
///
/// This should only be used if [`Aes`] does not meet the
/// requirements (eg. performance or functionality)
///
/// The returned value is an unsafe wrapper for the [`AesRegisters`] struct
/// that implements a set of commonly used functionality.
///
/// # Safety
/// Reading and modifying the aes registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_aes_raw() -> *const impl AesRaw {
    AES
}

pub trait AesRaw {}

impl AesRaw for AesRegisters {}

pub trait Aes {}
