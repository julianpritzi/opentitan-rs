//! Driver code for the opentitan hmac IP
//!
//! TODO:
//!     - make functions on HmacRegisters unsafe by default
//!     - implementation for HmacRaw
//!     - implementation for Hmac

use super::addresses;
use opentitan_macros::registers;

#[registers("hw/ip/hmac/data/hmac.hjson")]
pub struct HmacRegisters;

const HMAC: *const HmacRegisters = addresses::HMAC as *const HmacRegisters;

/// Returns a pointer to the registers of the hmac IP
///
/// This should only be used if either [`HmacRaw`] or [`Hmac`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the hmac registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_hmac_registers() -> *const HmacRegisters {
    HMAC
}

/// Returns a pointer to the hmac IP
///
/// This should only be used if [`Hmac`] does not meet the
/// requirements (eg. performance or functionality)
///
/// The returned value is an unsafe wrapper for the [`HmacRegisters`] struct
/// that implements a set of commonly used functionality.
///
/// # Safety
/// Reading and modifying the hmac registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_hmac_raw() -> *const impl HmacRaw {
    HMAC
}

pub trait HmacRaw {}

impl HmacRaw for HmacRegisters {}

pub trait Hmac {}
