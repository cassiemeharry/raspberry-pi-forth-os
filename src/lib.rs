#![allow(incomplete_features)]
#![allow(unused)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(extern_types)]
#![feature(panic_info_message)]
#![feature(ptr_offset_from)]
#![feature(step_trait)]
#![no_std]

#[cfg(not(any(feature = "rpi2", feature = "rpi3")))]
compile_error!("Either feature \"rpi2\" or \"rpi3\" must be enabled for this crate.");

#[cfg(all(feature = "rpi2", feature = "rpi3"))]
compile_error!("Features \"rpi2\" and \"rpi3\" are mutually exclusive.");

// extern crate panic_halt;

mod allocator;
#[macro_use]
mod console;
mod interrupts;
mod panic_handler;
mod rpi;

#[no_mangle]
pub fn kernel_main() {
    println!("Hello, world!");

    qemu_exit::aarch64::exit_success();

    // use rpi::framebuffer::{Framebuffer, Pixel};
    // Framebuffer::with(|fb| {
    //     let colors = [
    //         Pixel::from((0xFF, 0xFF, 0xFF)),
    //         Pixel::from((0x00, 0x00, 0x00)),
    //     ];
    //     for color in colors.iter().cycle() {
    //         for y in 0..480 {
    //             for x in 0..640 {
    //                 fb[(x, y)] = color.clone();
    //             }
    //         }
    //     }
    // });
    {
        for c in "Hello, world!\nThis is a test.\n".chars() {
            rpi::console::write_char(c);
        }
        // let mut console = rpi::console::Console::new();
        // loop {
        //     use core::fmt::Write;
        //     for c in "abcdefghijklmnopqrstuvwxyz".chars() {
        //         write!(&mut console, "{}", c);
        //     }
        // }
    }

    qemu_exit::aarch64::exit_success()
}
