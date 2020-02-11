#![allow(incomplete_features)]
#![feature(asm)]
#![feature(const_generics)]
#![feature(panic_info_message)]
#![no_std]

#[macro_export]
macro_rules! println {
    () => {{
        use core::fmt::Write;
        let mut uart = crate::rpi::uart::UART::new();
        writeln!(&mut uart, "").unwrap();
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut uart = crate::rpi::uart::UART::new();
        writeln!(&mut uart, $($arg)*).unwrap();
    }};
}

// extern crate panic_halt;

mod allocator;
mod interrupts;
mod panic_handler;
mod rpi;

#[no_mangle]
pub fn kernel_main() {
    #[cfg(not(any(feature = "rpi2", feature = "rpi3")))]
    compile_error!("Either feature \"rpi2\" or \"rpi3\" must be enabled for this crate.");

    #[cfg(all(feature = "rpi2", feature = "rpi3"))]
    compile_error!("Features \"rpi2\" and \"rpi3\" are mutually exclusive.");

    interrupts::exception_vector_init();

    println!("Hello, world!");

    use rpi::framebuffer::{Framebuffer, Pixel};
    println!("Getting framebuffer...");
    Framebuffer::with(|fb| {
        println!("Got framebuffer: {:?}", fb);
        let target = 200;
        for offset in 0..target {
            fb[(offset, offset)] = Pixel::from((0xFF, 0x00, 0x00));
        }
        println!("Finished drawing line on framebuffer");
    });
    println!("Handed back framebuffer");
    {
        for c in "Hello, world!\nThis is a test.\n".chars() {
            rpi::console::write_char(c);
        }
        use core::fmt::Write;
        let mut console = rpi::console::Console::new();
        for line_number in 0..100 {
            writeln!(&mut console, "Line {}", line_number);
        }
        writeln!(&mut console, "\u{80}");
    }

    println!("All done, going into wait loop");
    loop {
        unsafe { asm!("wfe") };
    }
}
