//! Driver code for the opentitan otbn IP
//!
//! TODO:
//!     - make functions on OtbnRegisters unsafe by default
//!     - implementation for OtbnRaw
//!     - implementation for Otbn

use super::addresses;
use opentitan_macros::registers;

#[registers("hw/ip/otbn/data/otbn.hjson")]
pub struct OtbnRegisters;

const OTBN: *const OtbnRegisters = addresses::OTBN as *const OtbnRegisters;

/// Returns a pointer to the registers of the otbn IP
///
/// This should only be used if either [`RawOtbn`] or [`Otbn`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the otbn registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_otbn_registers() -> *const OtbnRegisters {
    OTBN
}

/// Returns a pointer to the otbn IP
///
/// This should only be used if [`Otbn`] does not meet the
/// requirements (eg. performance or functionality)
///
/// The returned value is an unsafe wrapper for the [`OtbnRegisters`] struct
/// that implements a set of commonly used functionality.
///
/// # Safety
/// Reading and modifying the otbn registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_otbn_raw() -> *const impl OtbnRaw {
    OTBN
}

pub trait OtbnRaw {}

impl OtbnRaw for OtbnRegisters {}

pub trait Otbn {}
