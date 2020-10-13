#![allow(incomplete_features)]
#![allow(unused)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(c_variadic)]
#![feature(const_fn)]
#![feature(const_generics)]
#![feature(const_in_array_repeat_expressions)]
#![feature(extern_types)]
#![feature(fmt_internals)] // for printf implementation
#![feature(panic_info_message)]
#![feature(ptr_offset_from)]
#![feature(specialization)]
#![feature(step_trait)]
#![no_std]

#[cfg(not(any(feature = "rpi2", feature = "rpi3")))]
compile_error!("Either feature \"rpi2\" or \"rpi3\" must be enabled for this crate.");

#[cfg(all(feature = "rpi2", feature = "rpi3"))]
compile_error!("Features \"rpi2\" and \"rpi3\" are mutually exclusive.");

extern crate alloc;
extern crate futures;

#[macro_use]
mod console;

mod allocator;
mod interrupts;
mod panic_handler;
mod rpi;

#[no_mangle]
pub fn kernel_main() {
    println!("Hello, world!");

    let s = alloc::string::String::from("It's a string on the heap!");
    println!("Got a heap-allocated string here: {}", s);

    rpi::usb::init();

    // qemu_exit::aarch64::exit_success();

    // use rpi::framebuffer::{Framebuffer, Pixel};
    // Framebuffer::with(|fb| {
    //     let mut c: u32 = 0xF0_A0_00;
    //     while c <= 0xFF_FF_FF {
    //         let color = Pixel::from(c);
    //         for y in 0..480 {
    //             for x in 0..640 {
    //                 fb[(x, y)] = color;
    //             }
    //         }
    //         c += 0x10;
    //     }
    // });
    // {
    //     // for c in "Hello, world!\nThis is a test.\n".chars() {
    //     //     rpi::console::write_char(c);
    //     // }
    //     // loop {
    //     //     use core::fmt::Write;
    //     //     for c in "abcdefghijklmnopqrstuvwxyz".chars() {
    //     //         println!("{}", c);
    //     //     }
    //     // }
    // }

    loop {}

    qemu_exit::aarch64::exit_success()
}
