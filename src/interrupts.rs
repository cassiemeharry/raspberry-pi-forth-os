use bitflags::bitflags;
use core::sync::atomic::{AtomicUsize, Ordering};
use enum_repr::EnumRepr;

pub mod handlers;

#[repr(C)]
struct InterruptHandler {
    func_code: [u32; 32]
}

#[repr(C)]
struct ExceptionVectorTablePart {
    sync: InterruptHandler,
    irq: InterruptHandler,
    fiq: InterruptHandler,
    s_error: InterruptHandler,
}

#[repr(C)]
struct ExceptionVectorTable {
    current_el_sp0: ExceptionVectorTablePart,
    current_el_spx: ExceptionVectorTablePart,
    lower_el_aarch64: ExceptionVectorTablePart,
    lower_el_aarch32: ExceptionVectorTablePart,
}

extern "C" {
    #[link_name = "vectors"]
    #[no_mangle]
    static VECTORS: [ExceptionVectorTable; 4];
}

#[cfg(target_arch = "aarch64")]
pub fn exception_vector_init() {
    unsafe {
        let addr = &VECTORS as *const [ExceptionVectorTable; 4] as usize;
        asm!("msr vbar_el1, $0" :: "r"(addr) :: "volatile");
    }
}

static INT_MASK_LEVEL_D: AtomicUsize = AtomicUsize::new(0);
static INT_MASK_LEVEL_A: AtomicUsize = AtomicUsize::new(0);
static INT_MASK_LEVEL_I: AtomicUsize = AtomicUsize::new(0);
static INT_MASK_LEVEL_F: AtomicUsize = AtomicUsize::new(0);

bitflags! {
    pub struct ExceptionMask: usize {
        const D = 0b1000;
        const A = 0b0100;
        const I = 0b0010;
        const F = 0b0001;
    }
}

#[inline]
pub fn with_masked_interrupts<F, T, const MASK: ExceptionMask>(f: F) -> T
where
    F: FnOnce() -> T,
{
    for (flag, counter) in [
        (ExceptionMask::D, &INT_MASK_LEVEL_D),
        (ExceptionMask::A, &INT_MASK_LEVEL_A),
        (ExceptionMask::I, &INT_MASK_LEVEL_I),
        (ExceptionMask::F, &INT_MASK_LEVEL_F),
    ].iter() {
        if MASK.contains(*flag) {
            let prev_mask_level = counter.fetch_add(1, Ordering::SeqCst);
            if prev_mask_level == 0 {
                unsafe {
                    asm!("msr daifclr, $0" :: "i"(flag));
                }
            }
        }
    }
    let result = f();
    for (flag, counter) in [
        (ExceptionMask::D, &INT_MASK_LEVEL_D),
        (ExceptionMask::A, &INT_MASK_LEVEL_A),
        (ExceptionMask::I, &INT_MASK_LEVEL_I),
        (ExceptionMask::F, &INT_MASK_LEVEL_F),
    ].iter() {
        if MASK.contains(*flag) {
            let prev_mask_level = counter.fetch_sub(1, Ordering::SeqCst);
            if prev_mask_level == 1 {
                unsafe {
                    asm!("msr daifclr, $0" :: "i"(flag));
                }
            }
        }
    }
    result
}

#[EnumRepr(type = "u16")]
#[derive(Copy, Clone, Debug)]
enum ExceptionClass {
    Unknown = 0b000000,
    TrappedWFx = 0b000001,
    TrappedMcrCoprocF = 0b000011,
    TrappedMcrrCoprocF = 0b000100,
    TrappedMcrCoprocE = 0b000101,
    TrappedLdcStr = 0b000110,
    TrappedSveSIMDFp = 0b000111,
    TrappedVMRS = 0b001000,
    TrappedPAUTHN = 0b001001,
    TrappedMrrcE = 0b001100,
    IllegalExecutionState = 0b001110,
    SVCFromAarch32 = 0b010001,
    HVCFromAarch32 = 0b010010,
    SMCFromAarch32 = 0b010011,
    SVCFromAarch64 = 0b010101,
    HVCFromAarch64 = 0b010110,
    SMCFromAarch64 = 0b010111,
    TrappedOtherSystemInstr = 0b011000,
    TrappedSVE = 0b011001,
    TrappedERET = 0b011010,
    ImplDefinedToEL3 = 0b011111,
    InstructionAbortFromLower = 0b100000,
    InstructionAbortFromSame = 0b100001,
    PCAlignmentFault = 0b100010,
    DataAbortFromLower = 0b100100,
    DataAbortFromSame = 0b100101,
    SPAlignmentFault = 0b100110,
    TrappedFPFromAarch32 = 0b101000,
    TrappedFPFromAarch64 = 0b101100,
    SErrorInterrupt = 0b101111,
    BreakpointFromLower = 0b110000,
    BreakpointFromSame = 0b110001,
    StepExceptionFromLower = 0b110010,
    StepExceptionFromSame = 0b110011,
    WatchpointExceptionFromLower = 0b110100,
    Watchpoint = 0b110101,
    BKPTFromAarch32 = 0b111000,
    VectorCATCHFromAarch32 = 0b111010,
    BRKFromAarch64 = 0b111100,
}

#[derive(Clone, Debug)]
pub struct ExceptionStatus {
    level: u8,
    exception_class: Result<ExceptionClass, u16>,
    instr_is_quad: bool,
    iss: u32,
    fault_address: *mut (),
    exception_link: *mut (),
}

impl ExceptionStatus {
    pub unsafe fn load() -> Option<ExceptionStatus> {
        let raw_el: usize;
        asm!("mrs $0, CurrentEL" : "=r"(raw_el));
        let level = ((raw_el >> 2) & 0b11) as u8;
        match level {
            0 => None,
            1 => {
                let exception_syndrome: u32;
                asm!("mrs $0, ESR_EL1" : "=r"(exception_syndrome));
                let ec = (exception_syndrome >> 26) as u16;
                let il = (exception_syndrome >> 25) & 1;
                let iss = exception_syndrome & 0x01FF_FFFF;
                let fault_address: *mut ();
                asm!("mrs $0, FAR_EL1" : "=r"(fault_address));
                let exception_link: *mut ();
                asm!("mrs $0, ELR_EL1" : "=r"(exception_link));
                Some(ExceptionStatus {
                    level,
                    exception_class: ExceptionClass::from_repr(ec).ok_or(ec),
                    instr_is_quad: il == 1,
                    iss,
                    fault_address,
                    exception_link,
                })
            },
            2 => {
                let exception_syndrome: u32;
                asm!("mrs $0, ESR_EL2" : "=r"(exception_syndrome));
                let ec = (exception_syndrome >> 26) as u16;
                let il = (exception_syndrome >> 25) & 1;
                let iss = exception_syndrome & 0x01FF_FFFF;
                let fault_address: *mut ();
                asm!("mrs $0, FAR_EL2" : "=r"(fault_address));
                let exception_link: *mut ();
                asm!("mrs $0, ELR_EL2" : "=r"(exception_link));
                Some(ExceptionStatus {
                    level,
                    exception_class: ExceptionClass::from_repr(ec).ok_or(ec),
                    instr_is_quad: il == 1,
                    iss,
                    fault_address,
                    exception_link,
                })
            },
            3 => {
                let exception_syndrome: u32;
                asm!("mrs $0, ESR_EL3" : "=r"(exception_syndrome));
                let ec = (exception_syndrome >> 26) as u16;
                let il = (exception_syndrome >> 25) & 1;
                let iss = exception_syndrome & 0x01FF_FFFF;
                let fault_address: *mut ();
                asm!("mrs $0, FAR_EL3" : "=r"(fault_address));
                let exception_link: *mut ();
                asm!("mrs $0, ELR_EL3" : "=r"(exception_link));
                Some(ExceptionStatus {
                    level,
                    exception_class: ExceptionClass::from_repr(ec).ok_or(ec),
                    instr_is_quad: il == 1,
                    iss,
                    fault_address,
                    exception_link,
                })
            },
            _ => unreachable!(),
        }
    }
}

#[inline]
fn current_exception_level() -> u8 {
    let mut el: usize;
    unsafe {
        asm!("mrs $0, CPSR" : "=r"(el));
    }
    (el & 0x0F) as u8
}
