use bit_field::BitField;
use bitflags::bitflags;
use core::{
    fmt,
    iter::Step,
    marker::PhantomData,
    ops::{Index, IndexMut},
};

mod addrs;
/// Items in this module should not be accessed by the kernel outside of the
/// early boot process.
mod early_init;
mod ttbr;

pub use ttbr::TTBR;

const ENTRY_COUNT: usize = 512;

const PAGE_SHIFT: usize = 12;
const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
const TABLE_SHIFT: usize = 9;
const TABLE_SIZE: usize = 1 << TABLE_SHIFT;
const SECTION_SHIFT: usize = PAGE_SHIFT + TABLE_SHIFT;
const SECTION_SIZE: usize = 1 << SECTION_SHIFT;

pub struct PageTables<'a> {
    global: &'a mut PageTable<Global>,
    // upper: &'a mut [PageTable<Upper>],
    middle: &'a mut [PageTable<Middle>],
    bottom: &'a mut [PageTable<Bottom>],
}

impl PageTables<'static> {
    pub unsafe fn new() -> Self {
        PageTables {
            global: &mut early_init::global_page_tables()[0],
            // upper: early_init::upper_page_tables(),
            middle: early_init::middle_page_tables(),
            bottom: early_init::bottom_page_tables(),
        }
    }
}

pub trait PageTableLevel: Copy + core::fmt::Debug {
    const SHIFT: usize;

    fn block_size() -> usize {
        1 << Self::SHIFT
    }

    fn table_index(virt_addr: usize) -> PageTableIndex {
        let index = (virt_addr >> Self::SHIFT) & ((1 << TABLE_SHIFT) - 1);
        PageTableIndex::new(index as u16)
    }
}

pub trait PageTableLevelHasNext: PageTableLevel {
    type Next: PageTableLevel;
}

#[derive(Copy, Clone, Debug)]
pub struct Unknown;

#[derive(Copy, Clone, Debug)]
pub struct Global;
impl PageTableLevel for Global {
    const SHIFT: usize = PAGE_SHIFT + 2 * TABLE_SHIFT;
}

impl PageTableLevelHasNext for Global {
    type Next = Middle;
}

// #[derive(Copy, Clone, Debug)]
// pub struct Upper;
// impl PageTableLevel for Upper {
//     const SHIFT: usize = PAGE_SHIFT + 2 * TABLE_SHIFT;
// }

// impl PageTableLevelHasNext for Upper {
//     type Next = Middle;
// }

#[derive(Copy, Clone, Debug)]
pub struct Middle;
impl PageTableLevel for Middle {
    const SHIFT: usize = PAGE_SHIFT + 1 * TABLE_SHIFT;
}

impl PageTableLevelHasNext for Middle {
    type Next = Bottom;
}

#[derive(Copy, Clone, Debug)]
pub struct Bottom;
impl PageTableLevel for Bottom {
    const SHIFT: usize = PAGE_SHIFT + 0 * TABLE_SHIFT;
}

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

    pub const fn new(phys: usize, flags: DescriptorFlags) -> PageTableDescriptor<L> {
        PageTableDescriptor {
            value: (phys as u64) | flags.bits() | 1,
            level: PhantomData,
        }
    }

    pub fn is_unused(self) -> bool {
        (self.value & 1) == 0
    }

    fn set_unused(&mut self) {
        self.value = 0;
    }

    fn is_block_mem(self) -> bool {
        !DescriptorFlags::from_bits_truncate(self.value).contains(DescriptorFlags::PAGE_TABLE_FLAG)
    }

    fn is_table_ptr(self) -> bool {
        (self.value >> 1) & 1 == 1
    }

    fn block_attrs(self) -> Option<BlockAttributes> {
        if self.is_block_mem() {
            Some(BlockAttributes {})
        } else {
            None
        }
    }

    fn address(self) -> u64 {
        self.value & 0x0000_7FFF_FFFF_F000
    }
}

impl<L: PageTableLevelHasNext> PageTableDescriptor<L> {
    fn get_table(self) -> Result<*mut PageTable<L::Next>, ()> {
        if self.is_table_ptr() {
            let ptr: u64 = self.value & 0x0000_FFFF_FFFF_F000;
            Ok(ptr as *mut PageTable<L::Next>)
        } else {
            Err(())
        }
    }

    unsafe fn ensure_table(
        &mut self,
        tables: &mut [PageTable<L::Next>],
        virt_addr: usize,
    ) -> Result<*mut PageTable<L::Next>, ()> {
        self.get_table().or_else(move |()| {
            let new_table = get_table_for_virt::<L::Next>(tables, virt_addr)?;
            *self = PageTableDescriptor::new(
                new_table as *mut PageTable<L::Next> as usize,
                DescriptorFlags::PAGE_TABLE_FLAG,
            );
            Ok(new_table)
        })
    }
}

pub struct BlockAttributes {}

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

#[repr(transparent)]
pub struct PageTable<L> {
    entries: [PageTableDescriptor<L>; ENTRY_COUNT],
    level: PhantomData<L>,
}

impl<L: Copy> PageTable<L> {
    pub const fn new() -> PageTable<L> {
        PageTable {
            entries: [PageTableDescriptor::new(0, DescriptorFlags::empty()); ENTRY_COUNT],
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

#[inline]
fn align_down(addr: usize, bytes: usize) -> usize {
    let mask = !((1 << bytes) - 1);
    addr & mask
}

#[inline]
fn align_up(addr: usize, bytes: usize) -> usize {
    align_down(addr + bytes, bytes)
}

#[inline(never)]
pub unsafe fn map_memory(
    tables: &mut PageTables,
    phys_addr_start: usize,
    mut virt_addr_start: usize,
    virt_addr_end: usize,
    flags: DescriptorFlags,
) -> Result<(), ()> {
    // Align start down and end up to page size.
    let phys_addr_start = align_down(phys_addr_start, PAGE_SHIFT);
    let virt_addr_start = align_down(virt_addr_start, PAGE_SHIFT);
    let virt_addr_end = align_up(virt_addr_end, PAGE_SHIFT);

    // println!(
    //     "Attempting to map physical memory starting at {:#016x} to range {:#016x}-{:#016x}.",
    //     phys_addr_start, virt_addr_start, virt_addr_end
    // );

    let mut phys_addr = phys_addr_start;
    let mut virt_addr = virt_addr_start;

    // const UPPER_SIZE: usize = 1 << Upper::SHIFT;
    const MIDDLE_SIZE: usize = 1 << Middle::SHIFT;
    const BOTTOM_SIZE: usize = 1 << Bottom::SHIFT;

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
        let global_index = (virt_addr >> Global::SHIFT) & (ENTRY_COUNT - 1);
        let global_descriptor = &mut tables.global[global_index];
        // let upper: &mut PageTable<Upper> =
        //     &mut *global_descriptor.ensure_table(tables.upper, virt_addr)?;
        // if size >= UPPER_SIZE {
        //     upper.map_memory_block(phys_addr, virt_addr, flags);
        //     virt_addr += UPPER_SIZE;
        //     phys_addr += UPPER_SIZE;
        //     continue;
        // }

        // let upper_index = (virt_addr >> Upper::SHIFT) & (ENTRY_COUNT - 1);
        // let upper_descriptor = &mut upper[upper_index];
        let middle: &mut PageTable<Middle> =
            &mut *global_descriptor.ensure_table(tables.middle, virt_addr)?;
        // println!(
        //     "For addr {:#016x}, upper: {:p}, middle: {:p}",
        //     virt_addr, upper as *mut _, middle as *mut _
        // );
        if size >= MIDDLE_SIZE {
            middle.map_memory_block(phys_addr, virt_addr, flags);
            virt_addr += MIDDLE_SIZE;
            phys_addr += MIDDLE_SIZE;
            continue;
        }
        let middle_index = (virt_addr >> Middle::SHIFT) & (ENTRY_COUNT - 1);
        let middle_descriptor = &mut middle[middle_index];
        let bottom: &mut PageTable<Bottom> =
            &mut *middle_descriptor.ensure_table(tables.bottom, virt_addr)?;
        bottom.map_memory_block(
            phys_addr,
            virt_addr,
            flags | DescriptorFlags::PAGE_TABLE_FLAG,
        );
        virt_addr += BOTTOM_SIZE;
        phys_addr += BOTTOM_SIZE;
    }
    Ok(())
}

// Safety:
// * `phys_addr` and `virt_addr` must be 4K page aligned.
unsafe fn get_table_for_virt<L: PageTableLevel>(
    tables: &mut [PageTable<L>],
    virt_addr: usize,
) -> Result<&mut PageTable<L>, ()> {
    // Do the dumb thing and allocate a new table on any conflict. An
    // optimization later would be to follow the chain and only allocate a new
    // table if a table pointing to another slot doesn't work deeper in.
    let descriptor_index = L::table_index(virt_addr);
    // println!(
    //     "Looking for a page table to use for {} with slot {:?} free",
    //     core::any::type_name::<L>(),
    //     descriptor_index
    // );
    let mut table_index: Option<usize> = None;
    for (i, table) in tables.iter().enumerate() {
        if table[descriptor_index].is_unused() {
            // println!("\tFound match at tables index {}", i);
            table_index = Some(i);
            break;
        }
    }
    match table_index {
        Some(i) => Ok(&mut tables[i]),
        None => Err(()),
    }
}

impl<L: PageTableLevel> PageTable<L> {
    #[inline(never)]
    fn map_memory_block(&mut self, phys_addr: usize, virt_addr: usize, flags: DescriptorFlags) {
        let index = L::table_index(virt_addr);
        let phys_addr = (phys_addr >> L::SHIFT) << L::SHIFT;
        // println!(
        //     "Mapping virtual address {:#016x} to physical address {:#016x} in {} table {:p} slot {:?}",
        //     virt_addr, phys_addr, core::any::type_name::<L>(), self as *mut PageTable<L>, index
        // );
        let descriptor = PageTableDescriptor::new(phys_addr, flags);
        self[index] = descriptor;
    }
}

impl<L: PageTableLevelHasNext> PageTable<L> {
    #[inline(never)]
    pub fn create_table_entry(&mut self, next: &PageTable<L::Next>, virt_addr: usize) {
        let index = L::table_index(virt_addr);
        let descriptor = PageTableDescriptor::new(
            next as *const PageTable<L::Next> as usize,
            DescriptorFlags::PAGE_TABLE_FLAG,
        );
        self[index] = descriptor;
    }
}

impl PageTable<Unknown> {
    pub unsafe fn as_typed<L: PageTableLevel>(&self) -> &PageTable<L> {
        core::mem::transmute(self)
    }

    pub unsafe fn as_typed_mut<L: PageTableLevel>(&mut self) -> &mut PageTable<L> {
        core::mem::transmute(self)
    }
}

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
