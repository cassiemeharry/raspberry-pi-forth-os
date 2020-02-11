use crate::{println, rpi::{P_BASE, mmio_read}};
use super::ExceptionStatus;

#[no_mangle]
pub extern "C" fn show_invalid_entry_message(_entry: usize, _esr: usize, _elr: usize) {
    // println!("Unhandled exception slot {:#02x}, ESR register: {:#16x}, ELR register: {:#16x}", entry, esr, elr);
    loop {
        unsafe { asm!("hlt 2"); }
    }
}

const IRQ_BASIC_PENDING: u32 = P_BASE + 0x0000_B200;
const IRQ_PENDING_1: u32 = P_BASE + 0x0000_B204;
const IRQ_PENDING_2: u32 = P_BASE + 0x0000_B208;
const FIQ_CONTROL: u32 = P_BASE + 0x0000_B20C;
const ENABLE_IRQS_1: u32 = P_BASE + 0x0000_B210;
const ENABLE_IRQS_2: u32 = P_BASE + 0x0000_B214;
const ENABLE_BASIC_IRQS: u32 = P_BASE + 0x0000_B218;
const DISABLE_IRQS_1: u32 = P_BASE + 0x0000_B21C;
const DISABLE_IRQS_2: u32 = P_BASE + 0x0000_B220;
const DISABLE_BASIC_IRQS: u32 = P_BASE + 0x0000_B224;

const SYSTEM_TIMER_IRQ_0: u32 = 1 << 0;
const SYSTEM_TIMER_IRQ_1: u32 = 1 << 1;
const SYSTEM_TIMER_IRQ_2: u32 = 1 << 2;
const SYSTEM_TIMER_IRQ_3: u32 = 1 << 3;

#[no_mangle]
pub unsafe extern "C" fn handle_irq() {
    let irq = mmio_read(IRQ_PENDING_1);
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

    let status = ExceptionStatus::load()
        .expect("Failed to load exception status");

    println!("Got sync exception! {:#?}", status);
}

unsafe fn handle_timer_irq() {}
