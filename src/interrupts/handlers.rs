use core::fmt::{self, Write};

use super::ExceptionStatus;
use crate::{
    println,
    rpi::mmio::{self, P_BASE},
};

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum ExceptionEntry {
    SyncEL1t = 0,
    IRQEL1t = 1,
    FIQEL1t = 2,
    ErrorEL1t = 3,
    SyncEL1h = 4,
    IRQEL1h = 5,
    FIQEL1h = 6,
    ErrorEL1h = 7,
    SyncEL064 = 8,
    IRQEL064 = 9,
    FIQEL064 = 10,
    ErrorEL064 = 11,
    SyncEL032 = 12,
    IRQEL032 = 13,
    FIQEL032 = 14,
    ErrorEL032 = 15,
}

#[no_mangle]
pub unsafe extern "C" fn show_invalid_entry_message(entry: ExceptionEntry) -> ! {
    let status = ExceptionStatus::load().expect("Failed to load exception status");

    println!(
        "Landed on undefined exception handler (entry {:?}), status = {:#?}",
        entry, status
    );

    loop {}
}

const IRQ_BASIC_PENDING: usize = P_BASE + 0x0000_B200;
const IRQ_PENDING_1: usize = P_BASE + 0x0000_B204;
const IRQ_PENDING_2: usize = P_BASE + 0x0000_B208;
const FIQ_CONTROL: usize = P_BASE + 0x0000_B20C;
const ENABLE_IRQS_1: usize = P_BASE + 0x0000_B210;
const ENABLE_IRQS_2: usize = P_BASE + 0x0000_B214;
const ENABLE_BASIC_IRQS: usize = P_BASE + 0x0000_B218;
const DISABLE_IRQS_1: usize = P_BASE + 0x0000_B21C;
const DISABLE_IRQS_2: usize = P_BASE + 0x0000_B220;
const DISABLE_BASIC_IRQS: usize = P_BASE + 0x0000_B224;

const SYSTEM_TIMER_IRQ_0: u32 = 1 << 0;
const SYSTEM_TIMER_IRQ_1: u32 = 1 << 1;
const SYSTEM_TIMER_IRQ_2: u32 = 1 << 2;
const SYSTEM_TIMER_IRQ_3: u32 = 1 << 3;

#[no_mangle]
pub unsafe extern "C" fn handle_irq() {
    let irq = mmio::read(IRQ_PENDING_1);
    match irq {
        SYSTEM_TIMER_IRQ_1 => handle_timer_irq(),
        _ => println!("Unknown pending irq: {:x}", irq),
    };
}

use core::sync::atomic::{AtomicUsize, Ordering};
static SYNC_EXCS: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
pub unsafe extern "C" fn handle_sync() {
    println!("Handling sync exception");
    if SYNC_EXCS.fetch_add(1, Ordering::SeqCst) > 10 {
        panic!("Got too many sync exceptions!");
    }

    let status = ExceptionStatus::load().expect("Failed to load exception status");

    println!("Got sync exception! {:#?}", status);
}

unsafe fn handle_timer_irq() {}
