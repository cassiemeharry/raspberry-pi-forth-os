pub mod console;
pub mod framebuffer;
pub mod mailbox;
pub mod uart;

#[cfg(any(feature = "rpi2", feature = "rpi3"))]
pub const P_BASE: u32 = 0x3F00_0000;

#[inline(always)]
pub unsafe fn mmio_write(reg: u32, data: u32) {
    (reg as *mut u32).write_volatile(data)
}

#[inline(always)]
pub unsafe fn mmio_read(reg: u32) -> u32 {
    (reg as *const u32).read_volatile()
}
