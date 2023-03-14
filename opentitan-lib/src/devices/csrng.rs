//! Driver code for the opentitan csrng IP
//!
//! TODO:
//!     - make functions on CsrngRegisters unsafe by default
//!     - implementation for CsrngRaw
//!     - implementation for Csrng

use super::addresses;
use opentitan_macros::registers;

#[registers("hw/ip/csrng/data/csrng.hjson")]
pub struct CsrngRegisters;

const CSRNG: *const CsrngRegisters = addresses::CSRNG as *const CsrngRegisters;

/// Returns a pointer to the registers of the csrng IP
///
/// This should only be used if either [`CsrngRaw`] or [`Csrng`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the csrng registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_csrng_registers() -> *const CsrngRegisters {
    CSRNG
}

/// Returns a pointer to the csrng IP
///
/// This should only be used if [`Csrng`] does not meet the
/// requirements (eg. performance or functionality)
///
/// The returned value is an unsafe wrapper for the [`CsrngRegisters`] struct
/// that implements a set of commonly used functionality.
///
/// # Safety
/// Reading and modifying the csrng registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_csrng_raw() -> *const impl CsrngRaw {
    CSRNG
}

pub trait CsrngRaw {}

impl CsrngRaw for CsrngRegisters {}

pub trait Csrng {}
