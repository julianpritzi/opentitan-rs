//! Driver code for the opentitan csrng IP
//!
//! TODO:
//!     - make functions on CsrngRegisters unsafe by default
//!     - implementation for CsrngRaw
//!     - implementation for Csrng

use super::addresses;
use opentitan_macros::registers;
use tock_registers::interfaces::{Readable, Writeable};

#[registers("hw/ip/csrng/data/csrng.hjson")]
pub struct CsrngRegisters;

const CSRNG: *mut CsrngRegisters = addresses::CSRNG as *mut CsrngRegisters;

/// Returns a pointer to the registers of the csrng IP
///
/// This should only be used if either [`CsrngRaw`] or [`Csrng`] do not meet the
/// requirements (eg. performance or functionality)
///
/// # Safety
/// Reading and modifying the csrng registers may have potential side effects.
/// Usage of the returned pointer is therefore inherently unsafe.
pub unsafe fn get_csrng_registers() -> *mut CsrngRegisters {
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
pub unsafe fn get_csrng_raw() -> *mut impl CsrngRaw {
    CSRNG
}

pub trait CsrngRaw {
    unsafe fn configure(&mut self, seed: Option<&[u32]>);

    unsafe fn generate(&mut self, data: &mut [u32; 4]);
}

impl CsrngRegisters {
    unsafe fn _send_cmd_data(&mut self, data: u32) {
        while !self.sw_cmd_sts.is_set(sw_cmd_sts::cmd_rdy) {}
        self.cmd_req.set(data);
    }
}

impl CsrngRaw for CsrngRegisters {
    unsafe fn configure(&mut self, seed: Option<&[u32]>) {
        unsafe {
            self.ctrl.write(ctrl::enable::SET);
            self.hw_exc_sts.set(0);

            let header = generate_header(CsrngCMD::Uninstantiate, 0, 0, 0);
            self._send_cmd_data(header);

            if let Some(seed) = seed {
                let seed_len = seed.len();
                let seed_len = if seed_len < 12 { seed_len } else { 12 };

                let header = generate_header(CsrngCMD::Instantiate, seed_len as u32, 1 << 8, 0);
                self._send_cmd_data(header);

                for value in &seed[0..seed_len] {
                    self._send_cmd_data(*value);
                }
            } else {
                let header = generate_header(CsrngCMD::Instantiate, 0, 0, 0);
                self._send_cmd_data(header);
            }
        }
    }

    unsafe fn generate(&mut self, data: &mut [u32; 4]) {
        unsafe {
            let header = generate_header(CsrngCMD::Generate, 0, 0, 1);
            self._send_cmd_data(header);

            while !self.genbits_vld.is_set(genbits_vld::genbits_vld) {}

            for val in data {
                *val = self.genbits.get();
            }
        }
    }
}

pub trait Csrng {}

#[derive(Copy, Clone)]
#[allow(dead_code)]
enum CsrngCMD {
    Instantiate = 0x1,
    Reseed = 0x2,
    Generate = 0x3,
    Update = 0x4,
    Uninstantiate = 0x5,
}

/// Generates an application command header according to the documentation
///
/// # Arguments
///
/// * `acmd` - The application command to execute
/// * `clen` - The command length, has to be between 0 and 12
/// * `flags` - Valid CsrngCMDHeader flags
/// * `glen` - The generate length, has to be between 0 and 4096
///
/// # Safety:
///  - argument restrictions have to be upheld
unsafe fn generate_header(acmd: CsrngCMD, clen: u32, flags: u32, glen: u32) -> u32 {
    acmd as u32 | (clen & 0b1111) << 4 | flags | (glen & 0b1111_1111_1111) << 12
}

#[cfg(test)]
mod tests {
    use crate::mark_test_as_skipped;

    use super::*;

    #[test_case]
    fn basic() {
        // TODO: CSRNG currently not working

        mark_test_as_skipped!()
    }
}
