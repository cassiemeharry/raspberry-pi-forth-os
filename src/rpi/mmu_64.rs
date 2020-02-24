use alloc::{boxed::Box, vec::Vec};
use bit_field::BitField;
use bitflags::bitflags;
use core::{
    fmt,
    iter::Step,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

mod addrs;
mod descriptors;
/// Items in this module should not be accessed by the kernel outside of the
/// early boot process.
mod early_init;
mod levels;
mod page_tables;
mod ttbr;

use self::{descriptors::*, levels::*, page_tables::PageTables};
pub use ttbr::TTBR;

const ENTRY_COUNT: usize = 512;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;

#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PageTableIndex(u16);

impl PageTableIndex {
    pub fn new(index: u16) -> PageTableIndex {
        assert!((index as usize) < ENTRY_COUNT);
        PageTableIndex(index)
    }
}

impl Step for PageTableIndex {
    #[inline(always)]
    fn steps_between(start: &Self, end: &Self) -> Option<usize> {
        Step::steps_between(&start.0, &end.0)
    }

    #[inline(always)]
    fn replace_one(&mut self) -> Self {
        self.0 = Step::replace_one(&mut self.0);
        self.clone()
    }
    #[inline(always)]
    fn replace_zero(&mut self) -> Self {
        self.0 = Step::replace_zero(&mut self.0);
        self.clone()
    }
    #[inline(always)]
    fn add_one(&self) -> Self {
        PageTableIndex(Step::add_one(&self.0))
    }
    #[inline(always)]
    fn sub_one(&self) -> Self {
        PageTableIndex(Step::sub_one(&self.0))
    }
    #[inline(always)]
    fn add_usize(&self, n: usize) -> Option<Self> {
        Step::add_usize(&self.0, n).map(|val| PageTableIndex(val))
    }
    #[inline(always)]
    fn sub_usize(&self, n: usize) -> Option<Self> {
        Step::sub_usize(&self.0, n).map(|val| PageTableIndex(val))
    }
}

#[repr(C, align(4096))]
pub struct PageTable<L> {
    entries: [PageTableDescriptor<L>; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L: Copy> PageTable<L> {
    pub const fn new() -> PageTable<L> {
        PageTable {
            entries: [PageTableDescriptor::zero(); ENTRY_COUNT],
            level: PhantomData,
        }
    }

    pub fn zero(&mut self) {
        for entry in self.iter_mut() {
            entry.set_unused()
        }
    }

    pub fn is_unused(&self) -> bool {
        self.entries.iter().all(|d| d.is_unused())
    }

    pub fn iter(&self) -> impl Iterator<Item = &PageTableDescriptor<L>> {
        self.entries.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PageTableDescriptor<L>> {
        self.entries.iter_mut()
    }
}

// `bytes` must be a power of 2.
#[inline]
pub fn align_down(addr: usize, bytes: usize) -> usize {
    let leading_zeros = bytes.leading_zeros() as usize;
    let bits = 64 - leading_zeros;
    align_down_bits(addr, bits)
}

// `bytes` must be a power of 2.
#[inline]
pub fn align_up(addr: usize, bytes: usize) -> usize {
    let leading_zeros = bytes.leading_zeros() as usize;
    let bits = 64 - leading_zeros;
    align_up_bits(addr, bits)
}

#[inline]
pub fn align_down_bits(addr: usize, bits: usize) -> usize {
    let mask = !((1 << bits) - 1);
    addr & mask
}

#[inline]
pub fn align_up_bits(addr: usize, bits: usize) -> usize {
    align_down_bits(addr + (1 << bits), bits)
}

// const UPPER_SIZE: usize = 1 << Upper::SHIFT;
const MIDDLE_SIZE: usize = 1 << Middle::SHIFT;
const BOTTOM_SIZE: usize = 1 << Bottom::SHIFT;

#[inline(never)]
pub unsafe fn map_memory<'a>(
    tables: &'a mut PageTables<'_>,
    phys_addr_start: usize,
    mut virt_addr_start: usize,
    virt_addr_end: usize,
    flags: DescriptorFlags,
) -> Result<(), ()> {
    // Align start down and end up to page size.
    let phys_addr_start = align_down_bits(phys_addr_start, PAGE_SHIFT);
    let virt_addr_start = align_down_bits(virt_addr_start, PAGE_SHIFT);
    let virt_addr_end = align_up_bits(virt_addr_end, PAGE_SHIFT);

    // println!(
    //     "Attempting to map physical memory starting at {:#016x} to range {:#016x}-{:#016x}.",
    //     phys_addr_start, virt_addr_start, virt_addr_end
    // );

    let mut phys_addr = phys_addr_start;
    let mut virt_addr = virt_addr_start;

    // println!(
    //     "Global page table address: {:p}",
    //     tables.global as *mut PageTable<Global>
    // );
    // println!(
    //     "Size limits:\n\tUpper:  {:#16x}\n\tMiddle: {:#16x}\n\tBottom: {:#16x}",
    //     0, MIDDLE_SIZE, BOTTOM_SIZE
    // );

    // The efficient way to do this would be to map pages as large as possible,
    // but for now, we'll map 4K pages individually for simplicity.
    while virt_addr < virt_addr_end {
        let size = virt_addr_end - virt_addr;
        // println!(
        //     "There are {:#x} bytes left to map, from {:#016x}-{:#016x}",
        //     size, virt_addr, virt_addr_end
        // );
        map_memory_inner(tables, size, &mut virt_addr, &mut phys_addr, flags);
    }
    Ok(())
}

unsafe fn map_memory_inner<'b>(
    tables: &'b mut PageTables<'_>,
    size: usize,
    virt_addr: &mut usize,
    phys_addr: &mut usize,
    flags: DescriptorFlags,
) {
    let global_index = (*virt_addr >> Global::SHIFT) & (ENTRY_COUNT - 1);
    let global_descriptor = &mut tables.global[global_index];
    // let upper: &mut PageTable<Upper> =
    //     &mut *global_descriptor.ensure_table(tables.upper, virt_addr);
    // if size >= UPPER_SIZE {
    //     upper.map_memory_block(*phys_addr, *virt_addr, flags);
    //     *virt_addr += UPPER_SIZE;
    //     *phys_addr += UPPER_SIZE;
    //     return Ok(());
    // }

    // let upper_index = (*virt_addr >> Upper::SHIFT) & (ENTRY_COUNT - 1);
    // let upper_descriptor = &mut upper[upper_index];
    let middle: *mut PageTable<Middle> =
        global_descriptor.ensure_table(&mut tables.middle, *virt_addr);
    // println!(
    //     "For addr {:#016x}, upper: {:p}, middle: {:p}",
    //     virt_addr, upper as *mut _, middle as *mut _
    // );
    let middle = &mut *middle;
    if size >= MIDDLE_SIZE {
        middle.map_memory_block(*phys_addr, *virt_addr, flags);
        *virt_addr += MIDDLE_SIZE;
        *phys_addr += MIDDLE_SIZE;
        return;
    }
    let middle_index = (*virt_addr >> Middle::SHIFT) & (ENTRY_COUNT - 1);
    let middle_descriptor = &mut middle[middle_index];
    let bottom: *mut PageTable<Bottom> =
        middle_descriptor.ensure_table(&mut tables.bottom, *virt_addr);
    let bottom = &mut *bottom;
    bottom.map_memory_block(
        *phys_addr,
        *virt_addr,
        flags | DescriptorFlags::PAGE_TABLE_FLAG,
    );
    *virt_addr += BOTTOM_SIZE;
    *phys_addr += BOTTOM_SIZE;
}

fn get_table_for_virt<'a, L: PageTableLevel>(
    tables: &'a mut Vec<Box<PageTable<L>>>,
    virt_addr: usize,
) -> &'a mut PageTable<L> {
    let virt_addr = align_down_bits(virt_addr, PAGE_SHIFT);
    // Do the dumb thing and allocate a new table on any conflict. An
    // optimization later would be to follow the chain and only allocate a new
    // table if a table pointing to another slot doesn't work deeper in.
    let descriptor_index = L::table_index(virt_addr);
    // println!(
    //     "Looking for a page table to use for {} with slot {:?} free",
    //     core::any::type_name::<L>(),
    //     descriptor_index
    // );
    for table_ref in tables.iter_mut() {
        if table_ref.is_unused() {
            &mut *table_ref;
        }
    }
    // No valid table found, so make a new one.
    let table = Box::new(PageTable::new());
    let index = tables.len();
    tables.push(table);
    &mut *tables[index]
}

impl<L: PageTableLevel1Through3> PageTable<L> {
    #[inline(never)]
    fn map_memory_block(&mut self, phys_addr: usize, virt_addr: usize, flags: DescriptorFlags) {
        let index = L::table_index(virt_addr);
        let phys_addr = (phys_addr >> L::SHIFT) << L::SHIFT;
        // println!(
        //     "Mapping virtual address {:#016x} to physical address {:#016x} in {} table {:p} slot {:?}",
        //     virt_addr, phys_addr, core::any::type_name::<L>(), self as *mut PageTable<L>, index
        // );
        let descriptor = PageTableDescriptor::<L>::new_block_mem_with_flags(phys_addr, flags);
        self[index] = descriptor;
    }
}

impl<L: PageTableLevelHasNext> PageTable<L> {
    #[inline(never)]
    pub fn create_table_entry(&mut self, next: *const PageTable<L::Next>, virt_addr: usize) {
        let index = L::table_index(virt_addr);
        let descriptor = PageTableDescriptor::new_page_table(next as usize);
        self[index] = descriptor;
    }
}

// impl PageTable<Unknown> {
//     pub unsafe fn as_typed<L: PageTableLevel>(&self) -> &PageTable<L> {
//         core::mem::transmute(self)
//     }

//     pub unsafe fn as_typed_mut<L: PageTableLevel>(&mut self) -> &mut PageTable<L> {
//         core::mem::transmute(self)
//     }
// }

impl<L> Index<usize> for PageTable<L> {
    type Output = PageTableDescriptor<L>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.entries[index]
    }
}

impl<L> IndexMut<usize> for PageTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.entries[index]
    }
}

impl<L> Index<PageTableIndex> for PageTable<L> {
    type Output = PageTableDescriptor<L>;

    fn index(&self, index: PageTableIndex) -> &Self::Output {
        &self.entries[index.0 as usize]
    }
}

impl<L> IndexMut<PageTableIndex> for PageTable<L> {
    fn index_mut(&mut self, index: PageTableIndex) -> &mut Self::Output {
        &mut self.entries[index.0 as usize]
    }
}
