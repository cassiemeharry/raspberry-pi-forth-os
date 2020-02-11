use core::fmt;

use crate::rpi::{P_BASE, mmio_read, mmio_write};

const GPIO_BASE: u32 = P_BASE + 0x0020_0000;
const GPIO_FSEL1: u32 = GPIO_BASE + 0x04;
const GPIO_SET0: u32 = GPIO_BASE + 0x1C;
const GPIO_CLR0: u32 = GPIO_BASE + 0x28;
const GPIO_PUD: u32 = GPIO_BASE + 0x94;
const GPIO_PUD_CLK0: u32 = GPIO_BASE + 0x98;

const AUX_ENABLES: u32 = P_BASE + 0x0021_5004;
const AUX_MU_IO_REG: u32 = P_BASE + 0x0021_5040;
const AUX_MU_IER_REG: u32 = P_BASE + 0x0021_5044;
const AUX_MU_IIR_REG: u32 = P_BASE + 0x0021_5048;
const AUX_MU_LCR_REG: u32 = P_BASE + 0x0021_504C;
const AUX_MU_MCR_REG: u32 = P_BASE + 0x0021_5050;
const AUX_MU_LSR_REG: u32 = P_BASE + 0x0021_5054;
const AUX_MU_MSR_REG: u32 = P_BASE + 0x0021_5058;
const AUX_MU_SCRATCH: u32 = P_BASE + 0x0021_505C;
const AUX_MU_CNTL_REG: u32 = P_BASE + 0x0021_5060;
const AUX_MU_STAT_REG: u32 = P_BASE + 0x0021_5064;
const AUX_MU_BAUD_REG: u32 = P_BASE + 0x0021_5068;
// // Controls actuation of pull up/down to ALL GPIO pins.
// const GPPUD: u32 = GPIO_BASE + 0x94;

// // Controls actuation of pull up/down for specific GPIO pin.
// const GPPUDCLK0: u32 = GPIO_BASE + 0x98;

// const UART0_BASE: u32 = GPIO_BASE;

// // The base address for UART.
// const UART0_BASE: u32 = GPIO_BASE + 0x1000; // for raspi2 & 3, 0x20201000 for raspi1

// // // The offsets for reach register for the UART.
// const UART0_DR: u32 = UART0_BASE + 0x00;
// // const UART0_RSRECR: u32 = UART0_BASE + 0x04;
// const UART0_FR: u32 = UART0_BASE + 0x18;
// // const UART0_ILPR: u32 = UART0_BASE + 0x20;
// const UART0_IBRD: u32 = UART0_BASE + 0x24;
// const UART0_FBRD: u32 = UART0_BASE + 0x28;
// const UART0_LCRH: u32 = UART0_BASE + 0x2C;
// const UART0_CR: u32 = UART0_BASE + 0x30;
// // const UART0_IFLS: u32 = UART0_BASE + 0x34;
// const UART0_IMSC: u32 = UART0_BASE + 0x38;
// // const UART0_RIS: u32 = UART0_BASE + 0x3C;
// // const UART0_MIS: u32 = UART0_BASE + 0x40;
// const UART0_ICR: u32 = UART0_BASE + 0x44;
// // const UART0_DMACR: u32 = UART0_BASE + 0x48;
// // const UART0_ITCR: u32 = UART0_BASE + 0x80;
// // const UART0_ITIP: u32 = UART0_BASE + 0x84;
// // const UART0_ITOP: u32 = UART0_BASE + 0x88;
// // const UART0_TDR: u32 = UART0_BASE + 0x8C;

extern "C" {
    fn delay(count: usize);
}
// #[inline(always)]
// unsafe fn delay(mut count: usize) {
//     asm!("\
// __delay_%=:
//     subs %[count], %[count], #1;
//     bne __delay_%=
// " : "=r"(count) : "0"(count) : "cc");
//     let _ = count;
// }

/// This function must only be called once.
unsafe fn uart0_init() {
    let mut selector = mmio_read(GPIO_FSEL1);
    selector &= !(7 << 12); // clean gpio14
    selector |= 2 << 12; // set alt5 for gpio14
    selector &= !(7 << 15); // clean gpio15
    selector |= 2 << 15; // set alt5 for gpio15
    mmio_write(GPIO_FSEL1, selector);

    mmio_write(GPIO_PUD, 0);
    delay(150);
    mmio_write(GPIO_PUD_CLK0, (1 << 14) | (1 << 15));
    delay(150);
    mmio_write(GPIO_PUD_CLK0, 0);

    mmio_write(AUX_ENABLES, 1); // enable mini uart
    mmio_write(AUX_MU_CNTL_REG, 0); // disable auto flow control and disable receiver and transmitter (for now)
    mmio_write(AUX_MU_IER_REG, 0); // disable receive and transmit interrupts
    mmio_write(AUX_MU_LCR_REG, 3); // enable 8-bit mode
    mmio_write(AUX_MU_MCR_REG, 0); // set RTS line to be always high
    mmio_write(AUX_MU_BAUD_REG, 270); // set baud rate to 115200

    mmio_write(AUX_MU_CNTL_REG, 3); // Now that setup is done, enable transmitter and receiver.

    // // Disable UART0.
    // mmio_write(UART0_CR, 0x0000_0000);
    // // Setup the GPIO pin 14 && 15.

    // // Disable pull up/down for all GPIO pins & delay for 150 cycles.
    // mmio_write(GPPUD, 0x0000_0000);
    // delay(150);

    // // Disable pull up/down for pin 14,15 & delay for 150 cycles.
    // mmio_write(GPPUDCLK0, (1 << 14) | (1 << 15));
    // delay(150);

    // // Write 0 to GPPUDCLK0 to make it take effect.
    // mmio_write(GPPUDCLK0, 0x0000_0000);

    // // Clear pending interrupts.
    // mmio_write(UART0_ICR, 0x0000_07FF);

    // // Set integer & fractional part of baud rate.
    // // Divider = UART_CLOCK/(16 * Baud)
    // // Fraction part register = (Fractional part * 64) + 0.5
    // // UART_CLOCK = 3000000; Baud = 115200.

    // // Divider = 3000000 / (16 * 115200) = 1.627 = ~1.
    // mmio_write(UART0_IBRD, 1);
    // // Fractional part register = (.627 * 64) + 0.5 = 40.6 = ~40.
    // mmio_write(UART0_FBRD, 40);

    // // Enable FIFO & 8 bit data transmission (1 stop bit, no parity).
    // mmio_write(UART0_LCRH, (1 << 4) | (1 << 5) | (1 << 6));

    // // Mask all interrupts.
    // mmio_write(
    //     UART0_IMSC,
    //     (1 << 1) | (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7) | (1 << 8) | (1 << 9) | (1 << 10),
    // );

    // // Enable UART0, receive & transfer part of UART.
    // mmio_write(UART0_CR, (1 << 0) | (1 << 8) | (1 << 9));
}

#[inline]
unsafe fn uart0_put_byte(byte: u8) {
    loop {
        if mmio_read(AUX_MU_LSR_REG) & 0x20 != 0 {
            break;
        }
    }
    mmio_write(AUX_MU_IO_REG, byte as u32)
}

#[inline]
unsafe fn uart0_get_byte() -> u8 {
    // Wait for UART to have received something.
    loop {
        if mmio_read(AUX_MU_LSR_REG) & 0x01 != 0 {
            break;
        }
    }
    mmio_read(AUX_MU_IO_REG) as u8
}

static UART0_INIT_ONCE: spin::Once<()> = spin::Once::new();

pub struct UART0 {}

impl UART0 {
    pub fn new() -> UART0 {
        UART0_INIT_ONCE.call_once(|| unsafe { uart0_init() });
        UART0 {}
    }

    pub fn read_byte(&mut self) -> u8 {
        unsafe { uart0_get_byte() }
    }

    pub fn write_byte(&mut self, byte: u8) {
        unsafe { uart0_put_byte(byte) }
    }
}

impl fmt::Write for UART0 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub unsafe extern "C" fn uart_read_byte() -> u8 {
    UART0::new().read_byte()
}

pub unsafe extern "C" fn uart_put_byte(byte: usize) {
    UART0::new().write_byte(byte as u8)
}
