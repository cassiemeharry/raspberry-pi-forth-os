use core::sync::atomic::{compiler_fence, Ordering};

use super::{
    addrs::{PhysAddr, VirtAddr},
    *,
};
use crate::rpi::mmio;

macro_rules! page_table_accessor {
    ($fn_name:ident, $marker:ty, $start:ident, $end:ident) => {
        pub unsafe fn $fn_name() -> &'static mut [PageTable<$marker>] {
            let start_ptr = &mut $start as *mut MARKER as *mut PageTable<$marker>;
            let end_ptr = &mut $end as *mut MARKER as *mut PageTable<$marker>;
            let size = ((end_ptr as usize) - (start_ptr as usize))
                / core::mem::size_of::<PageTable<$marker>>();
            core::slice::from_raw_parts_mut(start_ptr, size)
        }
    };
}

page_table_accessor!(
    global_page_tables,
    Global,
    GLOBAL_PAGE_TABLES_START,
    UPPER_PAGE_TABLES_START
);
// page_table_accessor!(
//     upper_page_tables,
//     Upper,
//     UPPER_PAGE_TABLES_START,
//     MIDDLE_PAGE_TABLES_START
// );
page_table_accessor!(
    middle_page_tables,
    Middle,
    MIDDLE_PAGE_TABLES_START,
    BOTTOM_PAGE_TABLES_START
);
page_table_accessor!(
    bottom_page_tables,
    Bottom,
    BOTTOM_PAGE_TABLES_START,
    PAGE_TABLES_END
);

extern "C" {
    fn enable_mmu();

    #[link_name = "__page_table_global"]
    static mut GLOBAL_PAGE_TABLES_START: MARKER;

    #[link_name = "__page_tables_upper"]
    static mut UPPER_PAGE_TABLES_START: MARKER;

    #[link_name = "__page_tables_middle"]
    static mut MIDDLE_PAGE_TABLES_START: MARKER;

    #[link_name = "__page_tables_bottom"]
    static mut BOTTOM_PAGE_TABLES_START: MARKER;

    #[link_name = "__page_tables_end"]
    static mut PAGE_TABLES_END: MARKER;

    type MARKER;

    #[link_name = "__start"]
    static START_WORD: MARKER;

    #[link_name = "__bss_start"]
    static mut BSS_START: MARKER;
    #[link_name = "__bss_end"]
    static BSS_END: MARKER;

    #[link_name = "__start"]
    static IMAGE_START: MARKER;
    #[link_name = "__end"]
    static IMAGE_END: MARKER;
}

// 1 GB of memory
const PHYS_MEMORY_SIZE: usize = 0x40_000_000;

// const MM_ACCESS: u64 = 0x1 << 10;
// const MM_ACCESS_PERMISSION: u64 = 0x01 << 6;

#[allow(non_upper_case_globals)]
const MT_DEVICE_nGnRnE: u64 = 0x0;
const MT_NORMAL_NC: u64 = 0x1;
#[allow(non_upper_case_globals)]
const MT_DEVICE_nGnRnE_FLAGS: u64 = 0x00;
const MT_NORMAL_NC_FLAGS: u64 = 0x44;
const MAIR_VALUE: u64 = (MT_DEVICE_nGnRnE_FLAGS
    << (8 * DescriptorFlags::ATTR_INDEX_DEVICE_NGNRNE.bits()))
    | (MT_NORMAL_NC_FLAGS << (8 * DescriptorFlags::ATTR_INDEX_NORMAL_NC.bits()));

const VA_START: usize = 0xffff_0000_0000_0000;

// const MMU_PTE_FLAGS: DescriptorFlags = MM_TYPE_PAGE | MT_NORMAL_NC | MM_ACCESS | MM_ACCESS_PERMISSION;

bitflags! {
    struct TCRFlags: u64 {
        const T0SZ_2_16 = (64 - 16) << 0;
        const T1SZ_2_16 = (64 - 16) << 16;

        const T0SZ_2_32 = (64 - 32) << 0;
        const T1SZ_2_32 = (64 - 32) << 16;

        const SH0_NON_SHARABLE = 0b00 << 8;
        const SH0_OUTER_SHARABLE = 0b10 << 8;
        const SH0_INNER_SHARABLE = 0b11 << 8;

        const SH1_NON_SHARABLE = 0b00 << 28;
        const SH1_OUTER_SHARABLE = 0b10 << 28;
        const SH1_INNER_SHARABLE = 0b11 << 28;

        const TG0_4K = 0b00 << 14;
        const TG0_16K = 0b01 << 14;
        const TG0_64K = 0b11 << 14;

        const DISABLE_TTBR1_EL1 = 1 << 23;

        const TG1_4K = 0b10 << 30;
        const TG1_16K = 0b01 << 30;
        const TG1_64K = 0b11 << 30;

        const IPS_4GB = 0b000 << 32;
        const IPS_64GB = 0b001 << 32;
        const IPS_1TB = 0b010 << 32;
        const IPS_4TB = 0b011 << 32;
        const IPS_16TB = 0b100 << 32;
        const IPS_256TB = 0b101 << 32;
        const IPS_4PB = 0b110 << 32;
    }
}

#[link_section = ".text.boot"]
#[no_mangle]
pub unsafe extern "C" fn __memory_init() {
    // Set bits 21:20 of CPACR_EL1 so the later Rust code doesn't fail due to
    // use of SIMD instructions.
    {
        let mut cpacr_el1_val: usize;
        asm!("
mrs $0, cpacr_el1
bfi $0, $1, #20, #2
msr cpacr_el1, $0"
             : "=&r"(cpacr_el1_val)
             : "r"(0b11)
             :: "volatile"
        );
    }
    compiler_fence(Ordering::SeqCst);

    crate::interrupts::exception_vector_init();

    compiler_fence(Ordering::SeqCst);

    // Zero out the .bss segment
    let start_ptr = &mut BSS_START as *mut MARKER as *mut u32;
    let length_bytes =
        (&BSS_END as *const MARKER as usize) - (&BSS_START as *const MARKER as usize);
    let mut bss_slice =
        core::slice::from_raw_parts_mut(start_ptr, length_bytes / core::mem::size_of::<u32>());
    for word in bss_slice.iter_mut() {
        *word = 0;
    }

    create_page_tables();

    let root_page_table: *const PageTable<Global> = global_page_tables().as_ptr();
    TTBR::<1, 1>::from(root_page_table)
        .set_outer_sharable()
        .set_inner_region(0)
        .set_outer_region(0)
        .install();
    TTBR::<0, 1>::from(root_page_table)
        .set_outer_sharable()
        .set_inner_region(0)
        .set_outer_region(0)
        .install();

    println!("{:?}\t{:?}", TTBR::<0, 1>::load(), TTBR::<1, 1>::load());

    let tcr_el1_value = (TCRFlags::T0SZ_2_32
        | TCRFlags::T1SZ_2_32
        | TCRFlags::TG0_4K
        | TCRFlags::TG1_4K
        | TCRFlags::IPS_1TB
        | TCRFlags::SH0_INNER_SHARABLE
        | TCRFlags::SH1_INNER_SHARABLE
        | TCRFlags::DISABLE_TTBR1_EL1)
        .bits();
    asm!("msr tcr_el1, $0" :: "r"(tcr_el1_value) :: "volatile");

    let id_aa64mmfr0_el1: u64;
    asm!("mrs $0, ID_AA64MMFR0_EL1" : "=r"(id_aa64mmfr0_el1));
    // println!("ID_AA64MMFR0_EL1 {:016X}", id_aa64mmfr0_el1);

    // print!("mair val {:016X} ", MAIR_VALUE);
    asm!("msr mair_el1, $0" :: "r"(MAIR_VALUE) :: "volatile");

    let mair_el1: usize;
    asm!("mrs $0, mair_el1" : "=r"(mair_el1));
    // println!("MAIR_EL1 {:016X} ", mair_el1);

    let ttbr0_el1: usize;
    asm!("mrs $0, ttbr0_el1" : "=r"(ttbr0_el1));
    // print!("TTBR0 {:016X} ", ttbr0_el1);

    let ttbr1_el1: usize;
    asm!("mrs $0, ttbr1_el1" : "=r"(ttbr1_el1));
    // println!("TTBR1 {:016X} ", ttbr1_el1);

    let tcr_el1: usize;
    asm!("mrs $0, tcr_el1" : "=r"(tcr_el1));
    // println!(
    //     "TCR_EL1 {:016X} ({:?})",
    //     tcr_el1,
    //     TCRFlags::from_bits(tcr_el1 as u64).unwrap()
    // );

    // macro_rules! show_page_tables {
    //     ($f:ident, $t:ty) => {
    //         for (i, page_table) in $f().iter().enumerate() {
    //             println!("Page table {}: {:p}", i, page_table as *const PageTable<$t>);
    //             let mut j = 0;
    //             for entry in page_table.entries.iter() {
    //                 if entry.is_unused() {
    //                     continue;
    //                 }
    //                 println!("\t{:03x}:\t{:x?}", j, entry);
    //                 j += 1;
    //             }
    //         }
    //     };
    // }

    // show_page_tables!(global_page_tables, Global);
    // show_page_tables!(upper_page_tables, Upper);
    // show_page_tables!(middle_page_tables, Middle);
    // show_page_tables!(bottom_page_tables, Bottom);
    let sctlr_el1_before: usize;
    asm!("mrs $0, sctlr_el1" : "=r"(sctlr_el1_before));
    print!("sctlr val {:016X} ", sctlr_el1_before);

    compiler_fence(Ordering::SeqCst);
    enable_mmu();
    compiler_fence(Ordering::SeqCst);

    let sctlr_el1_after: usize;
    asm!("mrs $0, sctlr_el1" : "=r"(sctlr_el1_after));
    println!("SCTLR_EL1 {:016X} ", sctlr_el1_after);

    let pc: usize;
    asm!("adr $0, ." : "=r"(pc));
    let example_input_addresses: [*const u32; 6] = [
        ((create_page_tables as *const u32 as usize) | 0xFFFF_FFFF_0000_0000) as *const u32,
        ((global_page_tables().as_ptr() as usize) | 0xFFFF_FFFF_0000_0000) as *const u32,
        ((&BSS_START as *const _ as *const u32 as usize) | 0xFFFF_FFFF_0000_0000) as *const u32,
        crate::rpi::mmio::P_BASE as *const u32,
        crate::rpi::mmio::P_BASE_PHYSICAL_ADDR as *const u32,
        pc as *const u32,
    ];
    for input_address in example_input_addresses.iter() {
        let input_address: *const u32 = *input_address;
        let par_el1: usize;
        asm!("
AT S1E1R, $1
mrs $0, PAR_EL1
" : "=r"(par_el1) : "r"(input_address));
        use bit_field::BitField;
        let par_el1_f = par_el1.get_bit(0);
        let par_el1_sh = par_el1.get_bits(7..=8);
        let par_el1_ns = par_el1.get_bit(9);
        let par_el1_pa = par_el1.get_bits(12..=47) << 12;
        println!(
            "Translated address {:?} to {:#016x}\n\tFailed: {:?}\tSH: {:#04b}\tNS: {:?}\tPA: {:#016X}",
            input_address, par_el1,
            par_el1_f, par_el1_sh, par_el1_ns, par_el1_pa,
        );
    }
}

#[link_section = ".text.boot"]
unsafe fn create_page_tables() {
    {
        for table in global_page_tables().iter_mut() {
            table.zero();
        }
        // for table in upper_page_tables().iter_mut() {
        //     table.zero();
        // }
        for table in middle_page_tables().iter_mut() {
            table.zero();
        }
        for table in bottom_page_tables().iter_mut() {
            table.zero();
        }
    }

    let mut page_tables = PageTables::new();

    let va_device_start = mmio::P_BASE_OFFSET;
    let va_device_end = PHYS_MEMORY_SIZE - SECTION_SIZE;

    let va_start = &IMAGE_START as *const MARKER as usize;
    let va_end = PHYS_MEMORY_SIZE;
    // assert_eq!(va_start, 0);
    super::map_memory(
        &mut page_tables,
        0,
        va_start,
        va_end,
        DescriptorFlags::VALID
            | DescriptorFlags::ATTR_INDEX_NORMAL_NC
            | DescriptorFlags::EL1_RW_EL0_NONE
            | DescriptorFlags::NON_SECURE
            | DescriptorFlags::ACCESS,
    )
    .expect("Failed to identity map image");

    super::map_memory(
        &mut page_tables,
        mmio::P_BASE_PHYSICAL_ADDR,
        va_device_start,
        va_device_end,
        DescriptorFlags::VALID
            | DescriptorFlags::ATTR_INDEX_DEVICE_NGNRNE
            | DescriptorFlags::EL1_RW_EL0_NONE
            | DescriptorFlags::NON_SECURE
            | DescriptorFlags::ACCESS,
    )
    .expect("Failed to identity map device memory");

    // PG_DIR.global.create_table_entry(&PG_DIR.upper, VA_START);
    // PG_DIR.upper.create_table_entry(&PG_DIR.middle, VA_START);
    // // PG_DIR.middle.create_table_entry(&PG_DIR.bottom, VA_START);

    // PG_DIR
    //     .middle
    //     .map_memory(0x0, VA_START, va_end, DescriptorFlags::NORMAL_FLAGS);
    // PG_DIR.middle.map_memory(
    //     mmio::P_BASE_PHYSICAL_ADDR,
    //     va_device_start,
    //     va_device_end,
    //     DescriptorFlags::DEVICE_FLAGS,
    // );
}

// #[link_section = ".text.boot"]
// #[inline(always)]
// fn create_table_entry<L1, L2>(table: &mut PageTable<L1>, next: &mut PageTable<L2>, virt_addr: usize)
// where
//     L1: PageTableLevel,
//     L2: PageTableLevel,
// {
//     let table_index = ((virt_addr as usize) >> L1::SHIFT) & (ENTRY_COUNT - 1);
//     let descriptor =
//         PageTableDescriptor::new(next as *mut PageTable<L2> as usize, MM_TYPE_PAGE_TABLE);
//     table[table_index] = descriptor;
// }

// #[link_section = ".text.boot"]
// #[inline(always)]
// fn create_block_map<L: PageTableLevel>(
//     table: &mut PageTable,
//     virt_addr: usize,
//     phys_addr: usize,
//     size: usize,
//     flags: u64,
// ) {
//     let start_index = (start >> SECTION_SHIFT) & (ENTRY_COUNT - 1);
//     let end_index = (end >> SECTION_SHIFT) & (ENTRY_COUNT - 1);
//     // Clear the bottom SECTION_SHIFT bits from the physical address.
//     let phys = (phys >> SECTION_SHIFT) << SECTION_SHIFT;
//     let base_descriptor = PageTableDescriptor((phys as u64) | flags);
//     for (i, index) in (start_index..=end_index).enumerate() {
//         let offset = i * SECTION_SIZE;
//         let descriptor = PageTableDescriptor(base_descriptor.0 + offset as u64);
//         table[index] = descriptor;
//     }
// }
