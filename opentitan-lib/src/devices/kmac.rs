//! Driver code for the opentitan kmac IP
//!
//! TODO:
//!     - make functions on KmacRegisters unsafe by default
//!     - implementation for KmacRaw
//!     - implementation for Kmac

use super::addresses;
use opentitan_macros::registers;

#[registers("hw/ip/kmac/data/kmac.hjson")]
pub struct KmacRegisters;

const KMAC: *const KmacRegisters = addresses::KMAC as *const KmacRegisters;

/// Returns a pointer to the registers of the kmac IP
///
/// This should only be used if either [`RawKmac`] or [`Kmac`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the kmac registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_kmac_registers() -> *const KmacRegisters {
    KMAC
}

/// Returns a pointer to the kmac IP
///
/// This should only be used if [`Kmac`] does not meet the
/// requirements (eg. performance or functionality)
///
/// The returned value is an unsafe wrapper for the [`KmacRegisters`] struct
/// that implements a set of commonly used functionality.
///
/// # Safety
/// Reading and modifying the kmac registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_kmac_raw() -> *const impl KmacRaw {
    KMAC
}

pub trait KmacRaw {}

impl KmacRaw for KmacRegisters {}

pub trait Kmac {}
