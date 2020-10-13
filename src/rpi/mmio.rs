use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(all(target_pointer_width = "64", feature = "rpi3"))] {
        pub const P_BASE_OFFSET: usize = 0x0000_0000_3F00_0000;
        pub const P_BASE_PHYSICAL_ADDR: usize = P_BASE_OFFSET;
        // pub const P_BASE: usize = (0xFFFF << 48) | P_BASE_OFFSET;
        pub const P_BASE: usize = P_BASE_OFFSET;
    } else {
        compile_error!("TODO: define peripheral base for this device");
    }
}

#[inline(always)]
pub unsafe fn read(reg: usize) -> u32 {
    (reg as *const u32).read_volatile()
}

#[inline(always)]
pub unsafe fn write(reg: usize, data: u32) {
    (reg as *mut u32).write_volatile(data)
}
