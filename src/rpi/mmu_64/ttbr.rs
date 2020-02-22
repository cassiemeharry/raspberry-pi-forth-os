use core::fmt;

use super::{PageTable, Global};

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct TTBR<const N: usize, const L: usize> {
    value: u64,
}

impl<const N: usize, const L: usize> fmt::Debug for TTBR<N, L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TTBR{}_EL{}: {:#016x}", N, L, self.value)
    }
}

// bit 1 determines sharability, and bit 5 whether it's inner (1) or outer (0).
const SHAREABLE_MASK: u64 = !((1 << 5) | (1 << 1));

// Assumption: TCR_EL1.EAE == 0
impl<const N: usize, const L: usize> TTBR<N, L> {
    pub fn set_inner_region(self, irgn: u8) -> Self {
        let irgn = irgn as u64;
        // IRGN[0] is bit 6, IRGN[1] is bit 0
        const MASK: u64 = !((1 << 6) | (1 << 0));
        let value = (self.value & MASK) | ((irgn >> 1) & 1) | ((irgn & 1) << 6);
        TTBR { value }
    }

    pub fn set_outer_region(self, region: u8) -> Self {
        let region = region as u64;
        // Region bits are [4:3]
        const MASK: u64 = !(0b11 << 3);
        let value = (self.value & MASK) | ((region & 0b11) << 3);
        TTBR { value }
    }

    pub fn set_not_sharable(self) -> Self {
        let value = (self.value & SHAREABLE_MASK) | (0 << 5) | (0 << 1);
        TTBR { value }
    }

    pub fn set_inner_shareable(self) -> Self {
        let value = (self.value & SHAREABLE_MASK) | (1 << 5) | (1 << 1);
        TTBR { value }
    }

    pub fn set_outer_sharable(self) -> Self {
        let value = self.value & SHAREABLE_MASK | (0 << 5) | (1 << 1);
        TTBR { value }
    }
}

impl<const N: usize, const L: usize> From<*const PageTable<Global>> for TTBR<N, L> {
    fn from(table_ptr: *const PageTable<Global>) -> TTBR<N, L> {
        const MASK: u64 = 0xFFFF_FFFF & !0b0111_1111;
        let value = (table_ptr as u64) & MASK;
        if value != (table_ptr as u64) {
            panic!("Global page table isn't properly aligned (got address {:p})", table_ptr);
        }
        // println!("Original TTBR{}_EL{} value (from page table ref): {:#016x}", N, L, value);
        TTBR { value }
    }
}

impl TTBR<0, 1> {
    pub fn load() -> Self {
        let value;
        unsafe { asm!("mrs $0, ttbr0_el1" : "=r"(value) ::: "volatile") };
        TTBR { value }
    }

    pub unsafe fn install(self) {
        println!("Installing TTBR0_EL1 with value {:#016x}", self.value);
        asm!("msr ttbr0_el1, $0" :: "r"(self.value) :: "volatile");
    }
}

impl TTBR<1, 1> {
    pub fn load() -> Self {
        let value;
        unsafe { asm!("mrs $0, ttbr1_el1" : "=r"(value) ::: "volatile") };
        TTBR { value }
    }

    pub unsafe fn install(self) {
        println!("Installing TTBR1_EL1 with value {:#016x}", self.value);
        asm!("msr ttbr1_el1, $0" :: "r"(self.value) :: "volatile");
    }
}
