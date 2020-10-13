use super::{PageTableIndex, PAGE_SHIFT};

pub const TABLE_SHIFT: usize = 9;
pub const TABLE_SIZE: usize = 1 << TABLE_SHIFT;
pub const SECTION_SHIFT: usize = PAGE_SHIFT + TABLE_SHIFT;
pub const SECTION_SIZE: usize = 1 << SECTION_SHIFT;

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

pub trait PageTableLevel1Through3: PageTableLevel {}

pub trait PageTableLevel0Through2: PageTableLevel {}

// #[derive(Copy, Clone, Debug)]
// pub struct Unknown;

#[derive(Copy, Clone, Debug)]
pub struct Global;

impl PageTableLevel for Global {
    const SHIFT: usize = PAGE_SHIFT + 2 * TABLE_SHIFT;
}

impl PageTableLevelHasNext for Global {
    type Next = Middle;
}

impl PageTableLevel0Through2 for Global {}

// #[derive(Copy, Clone, Debug)]
// pub struct Upper;

// impl PageTableLevel for Upper {
//     const SHIFT: usize = PAGE_SHIFT + 2 * TABLE_SHIFT;
// }

// impl PageTableLevelHasNext for Upper {
//     type Next = Middle;
// }

// impl PageTableLevel0Through2 for Upper {}

// impl PageTableLevel1Through3 for Upper {}

#[derive(Copy, Clone, Debug)]
pub struct Middle;

impl PageTableLevel for Middle {
    const SHIFT: usize = PAGE_SHIFT + 1 * TABLE_SHIFT;
}

impl PageTableLevelHasNext for Middle {
    type Next = Bottom;
}

impl PageTableLevel0Through2 for Middle {}

impl PageTableLevel1Through3 for Middle {}

#[derive(Copy, Clone, Debug)]
pub struct Bottom;

impl PageTableLevel for Bottom {
    const SHIFT: usize = PAGE_SHIFT + 0 * TABLE_SHIFT;
}

impl PageTableLevel1Through3 for Bottom {}
