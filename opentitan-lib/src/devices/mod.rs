pub mod aes;
pub mod csrng;
pub mod hmac;
pub mod kmac;
pub mod otbn;
pub mod uart;

use opentitan_macros::addresses;

addresses!("hw/top_earlgrey/data/top_earlgrey.hjson");

/// Verilator platform constants
///
/// TODO: allow target platform selection using features
pub mod platform {
    pub const CPU_FREQ: usize = 500_000;
    pub const HIGH_SPEED_PERIPHERAL_FREQ: usize = 500_000;
    pub const PERIPHERAL_FREQ: usize = 125_000;
    pub const USB_FREQ: usize = 500_000;
    pub const AON_FREQ: usize = 125_000;

    pub const UART_BAUD_RATE: usize = 7200;
}
