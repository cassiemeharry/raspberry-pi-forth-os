pub mod uart0;
// #[cfg(not(feature = "rpi3"))]
pub type UART = uart0::UART0;

#[cfg(feature = "rpi3")]
pub mod uart1;
// #[cfg(feature = "rpi3")]
// pub type UART = uart1::UART1;
