pub mod uart {
    use opentitan_macros::registers;

    #[registers("hw/ip/uart/data/uart.hjson")]
    pub struct Uart;

    pub const UART0: *const Uart = 0x40000 as *const Uart;

    pub unsafe fn test() {
        unimplemented!()
    }
}
