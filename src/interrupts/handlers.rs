use bit_field::BitField;
use core::fmt::{self, Write};
use enum_repr::EnumRepr;

use super::{ExceptionClass, ExceptionStatus};
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

#[EnumRepr(type = "u8")]
#[derive(Copy, Clone, Debug)]
pub enum DataFaultStatusCode {
    AddressSizeFaultLevel0Translation = 0b000_000,
    AddressSizeFaultLevel1Translation = 0b000_001,
    AddressSizeFaultLevel2Translation = 0b000_010,
    AddressSizeFaultLevel3Translation = 0b000_011,
    TranslationFaultLevel0 = 0b000_100,
    TranslationFaultLevel1 = 0b000_101,
    TranslationFaultLevel2 = 0b000_110,
    TranslationFaultLevel3 = 0b000_111,
    AccessFlagFaultLevel1 = 0b001_001,
    AccessFlagFaultLevel2 = 0b001_010,
    AccessFlagFaultLevel3 = 0b001_011,
    PermissionFaultLevel1 = 0b001_101,
    PermissionFaultLevel2 = 0b001_110,
    PermissionFaultLevel3 = 0b001_111,
    SyncExternalAbortNotTranslation = 0b010_000,
    SyncTagCheckFail = 0b010_001,
    SyncExternalAbortLevel0Translation = 0b010_100,
    SyncExternalAbortLevel1Translation = 0b010_101,
    SyncExternalAbortLevel2Translation = 0b010_110,
    SyncExternalAbortLevel3Translation = 0b010_111,
    SyncECCNotTranslation = 0b011_000,
    SyncECCLevel0Translation = 0b011_100,
    SyncECCLevel1Translation = 0b011_101,
    SyncECCLevel2Translation = 0b011_110,
    SyncECCLevel3Translation = 0b011_111,
    AlignmentFault = 0b100_001,
    TLBConflict = 0b110_000,
    UnsupportedAtomic = 0b110_001,
    ImplDefinedLockdown = 0b110_100,
    ImplDefinedUnsupportedAtomic = 0b110_101,
    SectionDomainFault = 0b111_101,
    PageDomainFault = 0b111_110,
}

#[no_mangle]
pub unsafe extern "C" fn handle_sync() {
    if SYNC_EXCS.fetch_add(1, Ordering::SeqCst) > 10 {
        println_semihosting!("Got too many sync exceptions!");
        loop {}
    }

    let status = ExceptionStatus::load().expect("Failed to load exception status");

    println_semihosting!("Got sync exception: {:#?}", status);

    match status.exception_class {
        Err(class) => println_semihosting!(
            "Failed to decode exception class 0b{:6b} in exception: {:#?}",
            class,
            status
        ),
        Ok(ExceptionClass::Unknown) => {
            println_semihosting!("Got unknown sync exception: {:#?}", status);
        }
        Ok(ExceptionClass::DataAbortFromSame) => {
            let was_write = status.iss.get_bit(6);
            let status_code_raw = status.iss.get_bits(0..=5) as u8;
            let status_code =
                DataFaultStatusCode::from_repr(status_code_raw).ok_or(status_code_raw);
            if !status.iss.get_bit(24) {
                println_semihosting!(
                    "Got data abort from same EL when accessing address {:p} ({}, {:?})",
                    status.fault_address,
                    if was_write { "write" } else { "read" },
                    status_code,
                );
                return;
            }
            let access_size = status.iss.get_bits(22..=23);
            let access_size_str = match access_size {
                0b00 => "byte",
                0b01 => "halfword",
                0b10 => "word",
                0b11 => "doubleword",
                _ => unreachable!(),
            };
            let sign_extension_required = status.iss.get_bit(21);
            // SRT = Register number of the Rt operand of the faulting instruction.
            let srt = status.iss.get_bits(16..=20);
            let dw_register = status.iss.get_bit(15);
            let acq_rel_semantics = status.iss.get_bit(14);
            let far_not_valid = status.iss.get_bit(10);
            let external_abort = status.iss.get_bit(9);
            let cache_maintenance = status.iss.get_bit(8);
            let s1ptw = status.iss.get_bit(7);
            println_semihosting!(
                "Got data abort from same EL accessing address {:p} ({} {}, {:?})",
                status.fault_address,
                access_size,
                if was_write { "write" } else { "read" },
                status_code,
            );
        }
        Ok(other) => {
            println_semihosting!("Got unhandled exception class {:?}: {:#?}", other, status);
        }
    }
}

unsafe fn handle_timer_irq() {}
