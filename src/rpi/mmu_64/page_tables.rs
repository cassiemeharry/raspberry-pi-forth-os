use alloc::{vec, boxed::Box, vec::Vec};
use spin::{Mutex, MutexGuard, Once};

use super::{PageTable, levels::*};

struct PageTablesOwned {
    global: Box<PageTable<Global>>,
    middle: Vec<Box<PageTable<Middle>>>,
    bottom: Vec<Box<PageTable<Bottom>>>,
}

static PAGE_TABLES: Once<Mutex<PageTablesOwned>> = Once::new();

impl PageTablesOwned {
    fn with_global<F, T>(f: F) -> T
    where
        F: for<'r> FnOnce(&'r mut Self) -> T,
    {
        let mutex = PAGE_TABLES.call_once(|| {
            let global = Box::new(PageTable::new());
            let middle = Vec::with_capacity(4);
            let bottom = Vec::with_capacity(16);
            let tables = PageTablesOwned { global, middle, bottom };
            Mutex::new(tables)
        });
        f(&mut mutex.lock())
    }
}

pub struct PageTables<'a> {
    pub global: &'a mut PageTable<Global>,
    // upper: &'a mut [&'a mut PageTable<Upper>],
    pub middle: &'a mut Vec<Box<PageTable<Middle>>>,
    pub bottom: &'a mut Vec<Box<PageTable<Bottom>>>,
}

impl<'a> PageTables<'a> {
    pub fn with_page_tables<F, T>(f: F) -> T
    where
        F: for<'r> FnOnce(&mut PageTables<'r>) -> T,
    {
        PageTablesOwned::with_global(|owned| {
            let mut tables_ref = PageTables {
                global: &mut owned.global,
                middle: &mut owned.middle,
                bottom: &mut owned.bottom,
            };
            f(&mut tables_ref)
        })
    }
}
