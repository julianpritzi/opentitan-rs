//! Driver code for the opentitan aes IP
//!
//! TODO:
//!     - make functions on AesRegisters unsafe by default
//!     - implementation for AesRaw
//!     - implementation for Aes

use super::addresses;
use opentitan_macros::registers;
use tock_registers::{
    fields::FieldValue,
    interfaces::{Readable, Writeable},
};

#[registers("hw/ip/aes/data/aes.hjson")]
pub struct AesRegisters;

const AES: *mut AesRegisters = addresses::AES as *mut AesRegisters;

/// Returns a pointer to the registers of the aes IP
///
/// This should only be used if either [`AesRaw`] or [`Aes`] do not meet the
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
pub unsafe fn get_aes_raw() -> *mut impl AesRaw {
    AES
}

pub enum Operation {
    Encrypt,
    Decrypt,
}

impl Operation {
    pub fn reg_val(&self) -> FieldValue<u32, ctrl_shadowed::Register> {
        match self {
            Operation::Encrypt => ctrl_shadowed::operation::AES_ENC,
            Operation::Decrypt => ctrl_shadowed::operation::AES_DEC,
        }
    }
}

pub enum Mode {
    ECB,
    /// The iv corresponds to 4 consecutive little endian u32s
    CBC {
        iv: [u32; 4],
    },
    CFB,
    OFB,
    /// The iv corresponds to 4 consecutive little endian u32s
    CTR {
        iv: [u32; 4],
    },
}

impl Mode {
    pub fn reg_val(&self) -> FieldValue<u32, ctrl_shadowed::Register> {
        match self {
            Mode::ECB => ctrl_shadowed::mode::AES_ECB,
            Mode::CBC { .. } => ctrl_shadowed::mode::AES_CBC,
            Mode::CFB => ctrl_shadowed::mode::AES_CFB,
            Mode::OFB => ctrl_shadowed::mode::AES_OFB,
            Mode::CTR { .. } => ctrl_shadowed::mode::AES_CTR,
        }
    }
}

pub enum KeyLength {
    Aes128,
    Aes192,
    Aes256,
}

impl KeyLength {
    fn length(&self) -> FieldValue<u32, ctrl_shadowed::Register> {
        match self {
            KeyLength::Aes128 => ctrl_shadowed::key_len::AES_128,
            KeyLength::Aes192 => ctrl_shadowed::key_len::AES_192,
            KeyLength::Aes256 => ctrl_shadowed::key_len::AES_256,
        }
    }
}

pub trait AesRaw {
    unsafe fn configure(
        &mut self,
        mode: Mode,
        operation: Operation,
        key_length: KeyLength,
        key_share0: &[u32; 8],
        key_share1: &[u32; 8],
    );

    unsafe fn execute(&mut self, input: &[u32], output: &mut [u32]);

    unsafe fn deinitialize(&self);
}

impl AesRaw for AesRegisters {
    unsafe fn configure(
        &mut self,
        mode: Mode,
        operation: Operation,
        key_length: KeyLength,
        key_share0: &[u32; 8],
        key_share1: &[u32; 8],
    ) {
        while !self.status.is_set(status::idle) {}

        let ctrl_value = key_length.length() + operation.reg_val() + mode.reg_val();
        self.ctrl_shadowed.write(ctrl_value);
        self.ctrl_shadowed.write(ctrl_value);

        for i in 0..8 {
            self.key_share0[i].set(key_share0[i]);
            self.key_share1[i].set(key_share1[i]);
        }

        while !self.status.is_set(status::idle) {}

        match mode {
            Mode::CBC { iv } | Mode::CTR { iv } => {
                for i in 0..4 {
                    self.iv[i].set(iv[i]);
                }
            }
            _ => (),
        }
    }

    unsafe fn execute(&mut self, input: &[u32], output: &mut [u32]) {
        let length = input.len();
        assert!(length % 4 == 0);
        let block_num = length / 4;

        for block_idx in 0..(block_num + 2) {
            if block_idx == 1 {
                while !self.status.is_set(status::input_ready) {}
            }

            if block_idx > 1 {
                while !self.status.is_set(status::output_valid) {}

                for i in 0..4 {
                    output[(block_idx - 2) * 4 + i] = self.data_out[i].get();
                }
            }

            if block_idx < block_num {
                for i in 0..4 {
                    self.data_in[i].set(input[block_idx * 4 + i]);
                }
            }
        }
    }

    unsafe fn deinitialize(&self) {
        self.ctrl_shadowed
            .write(ctrl_shadowed::manual_operation::SET);
        self.ctrl_shadowed
            .write(ctrl_shadowed::manual_operation::SET);

        self.trigger
            .write(trigger::key_iv_data_in_clear::SET + trigger::data_out_clear::SET);

        while !self.status.is_set(status::idle) {}
    }
}

pub trait Aes {}
