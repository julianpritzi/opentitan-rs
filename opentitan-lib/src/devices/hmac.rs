//! Driver code for the opentitan hmac IP
//!
//! TODO:
//!     - make functions on HmacRegisters unsafe by default
//!     - implementation for HmacRaw
//!     - implementation for Hmac

use super::addresses;
use opentitan_macros::registers;
use tock_registers::interfaces::{Readable, Writeable};

#[registers("hw/ip/hmac/data/hmac.hjson")]
pub struct HmacRegisters;

const HMAC: *mut HmacRegisters = addresses::HMAC as *mut HmacRegisters;

/// Returns a pointer to the registers of the hmac IP
///
/// This should only be used if either [`HmacRaw`] or [`Hmac`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the hmac registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_hmac_registers() -> *mut HmacRegisters {
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
pub unsafe fn get_hmac_raw() -> *mut impl HmacRaw {
    HMAC
}

pub trait HmacRaw {
    unsafe fn hash_data(&mut self, data: &[u32], digest: &mut [u32; 8]);
}

impl HmacRaw for HmacRegisters {
    unsafe fn hash_data(&mut self, data: &[u32], digest: &mut [u32; 8]) {
        self.cfg.write(cfg::sha_en::SET);
        self.cmd.write(cmd::hash_start::SET);

        for val in data {
            while self.status.is_set(status::fifo_full) {}

            // TODO: use full msg_fifo size
            self.msg_fifo[0].set(*val);
        }

        self.cmd.write(cmd::hash_process::SET);

        while !self.intr_state.is_set(intr::hmac_done) {}

        for i in 0..8 {
            digest[i] = self.digest[i].get();
        }
    }
}

pub trait Hmac {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case]
    fn basic() {
        unsafe {
            let hmac = get_hmac_raw();

            let data = [32u32; 32];
            let mut digest = [0u32; 8];

            (*hmac).hash_data(&data, &mut digest);

            assert_eq!(
                digest,
                [
                    1226795820, 575703230, 291893938, 2935539018, 2827460678, 30448964, 4171743692,
                    4048342112
                ]
            );
        }
    }
}
