use cfg_if::cfg_if;

pub mod console;
pub mod framebuffer;
pub mod mailbox;
pub mod mmio;

cfg_if! {
    if #[cfg(feature = "rpi3")] {
        pub mod mmu_64;
        pub use mmu_64 as mmu;
    }
}

#[cfg(feature = "semihosting")]
pub mod semihosting;
pub mod uart;
pub mod usb;
