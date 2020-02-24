use alloc::{boxed::Box, vec::Vec};
use bit_field::BitField;
use bitflags::bitflags;
use core::{fmt, marker::PhantomData};

use super::{PageTable, levels::*};

bitflags! {
    pub struct DescriptorFlags: u64 {
        const VALID = 1 << 0;

        const PAGE_TABLE_FLAG = 1 << 1;

        const ATTR_INDEX_DEVICE_NGNRNE = 0;
        const ATTR_INDEX_NORMAL_NC = 1;

        const NON_SECURE = 1 << 5;

        const EL1_RW_EL0_NONE = 0b00 << 6;
        const EL1_RW_EL0_RW = 0b01 << 6;
        const EL1_R0_EL0_NONE = 0b10 << 6;
        const EL1_R0_EL0_RO = 0b11 << 6;

        const ACCESS = 1 << 10;
        const NOT_GLOBAL = 1 << 11;

        const PRIVILEGED_EXECUTE_NEVER = 1 << 53;
        const EXECUTE_NEVER = 1 << 54;

        const NORMAL_FLAGS = ((1 << 0) | (1 << 2) | (1 << 10));
        const DEVICE_FLAGS = ((1 << 0) | (0 << 2) | (1 << 10));
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PageTableDescriptor<L> {
    value: u64,
    // Describes the level the descriptor is placed in.
    level: PhantomData<L>,
}

impl<L: Copy> fmt::Debug for PageTableDescriptor<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_unused() {
            f.debug_struct("PageTableDescriptor")
                .field("unused", &true)
                .field("value", &self.value)
                .finish()
        } else if self.is_table_ptr() {
            f.debug_struct("PageTableDescriptor")
                .field(
                    "table_pointer",
                    &((self.value & 0x0000_FFFF_FFFF_F000) as *const PageTable<L>),
                )
                .field("attr_index", &(self.value.get_bits(2..=4)))
                .field("non_secure", &(self.value.get_bit(5)))
                .field("access_permission", &(self.value.get_bits(6..=7)))
                .field("sharability", &(self.value.get_bits(8..=9)))
                .field("access", &(self.value.get_bit(10)))
                .field("not_global", &(self.value.get_bit(11)))
                .finish()
        } else {
            f.debug_struct("PageTableDescriptor")
                .field(
                    "physical_address",
                    &((self.value & 0x0000_FFFF_FFFF_0000) as *const [u8; 4096]),
                )
                .field("attr_index", &(self.value.get_bits(2..=4)))
                .field("non_secure", &(self.value.get_bit(5)))
                .field("access_permission", &(self.value.get_bits(6..=7)))
                .field("sharability", &(self.value.get_bits(8..=9)))
                .field("access", &(self.value.get_bit(10)))
                .field("not_global", &(self.value.get_bit(11)))
                .finish()
        }
    }
}

impl<L: Copy> PageTableDescriptor<L> {
    pub const fn zero() -> PageTableDescriptor<L> {
        PageTableDescriptor {
            value: 0,
            level: PhantomData,
        }
    }

    pub const fn new_unchecked(value: u64) -> PageTableDescriptor<L> {
        PageTableDescriptor {
            value,
            level: PhantomData,
        }
    }

    pub fn is_unused(self) -> bool {
        (self.value & 1) == 0
    }

    pub fn set_unused(&mut self) {
        self.value = 0;
    }

    pub fn is_block_mem(self) -> bool {
        !DescriptorFlags::from_bits_truncate(self.value).contains(DescriptorFlags::PAGE_TABLE_FLAG)
    }

    pub fn is_table_ptr(self) -> bool {
        (self.value >> 1) & 1 == 1
    }

    fn block_attrs(self) -> Option<BlockAttributes> {
        if self.is_block_mem() {
            Some(BlockAttributes {})
        } else {
            None
        }
    }

    pub fn address(self) -> u64 {
        self.value & 0x0000_7FFF_FFFF_F000
    }
}

impl<L: PageTableLevelHasNext> PageTableDescriptor<L> {
    pub const fn new_page_table_with_flags(phys: usize, flags: DescriptorFlags) -> Self {
        PageTableDescriptor {
            value: (phys as u64)
                | flags.bits()
                | DescriptorFlags::PAGE_TABLE_FLAG.bits()
                | DescriptorFlags::VALID.bits(),
            level: PhantomData,
        }
    }

    pub const fn new_page_table(phys: usize) -> Self {
        PageTableDescriptor::new_page_table_with_flags(phys, DescriptorFlags::empty())
    }

    pub fn get_table(self) -> Option<*mut PageTable<L::Next>> {
        if self.is_table_ptr() {
            let ptr: u64 = self.value & 0x0000_FFFF_FFFF_F000;
            Some(ptr as *mut PageTable<L::Next>)
        } else {
            None
        }
    }

    pub unsafe fn ensure_table<'a>(
        &mut self,
        tables: &mut Vec<Box<PageTable<L::Next>>>,
        virt_addr: usize,
    ) -> *mut PageTable<L::Next>
    where
        <L as PageTableLevelHasNext>::Next: 'a,
    {
        self.get_table().unwrap_or_else(move || {
            let new_table = super::get_table_for_virt(tables, virt_addr);
            *self = PageTableDescriptor::new_page_table(
                new_table as *mut PageTable<L::Next> as usize,
            );
            println!("Pointing desciptor {:?} at table {:p}", self, new_table as *mut _);
            new_table
        })
    }
}


pub trait CanMapBlocks: Sized {
    fn new_block_mem_with_flags(phys: usize, flags: DescriptorFlags) -> Self;

    fn new_block_mem(phys: usize) -> Self {
        Self::new_block_mem_with_flags(phys, DescriptorFlags::empty())
    }
}


impl<L: PageTableLevel1Through3> CanMapBlocks for PageTableDescriptor<L> {
    default fn new_block_mem_with_flags(phys: usize, flags: DescriptorFlags) -> Self {
        PageTableDescriptor {
            value: (phys as u64) | flags.bits() | DescriptorFlags::VALID.bits(),
            level: PhantomData,
        }
    }

}

impl CanMapBlocks for PageTableDescriptor<Bottom> {
    fn new_block_mem_with_flags(phys: usize, flags: DescriptorFlags) -> Self {
        PageTableDescriptor {
            value: (phys as u64)
                | flags.bits()
                | DescriptorFlags::PAGE_TABLE_FLAG.bits()
                | DescriptorFlags::VALID.bits(),
            level: PhantomData,
        }
    }
}

pub struct BlockAttributes {}
