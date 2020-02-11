use core::{fmt::Write, panic::PanicInfo};

use crate::rpi::uart::UART;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = UART::new();

    if let Some(loc) = info.location() {
        let _ = write!(
            &mut uart,
            "Panic occurred in file \"{}\" at line {}: ",
            loc.file(),
            loc.line()
        );
    } else {
        let _ = write!(
            &mut uart,
            "Panic occurred, but no location information was available: "
        );
    }

    if let Some(args) = info.message() {
        // Ignore the returned result, as we're already in the panic handler and
        // we have nowhere to report the problem.
        let _ = core::fmt::write(&mut uart, *args);
        let _ = writeln!(&mut uart, "");
    } else if let Some(msg) = info.payload().downcast_ref::<&str>() {
        let _ = writeln!(&mut uart, "{}", msg);
    } else {
        let _ = writeln!(&mut uart, "No message available");
    }

    loop {
        unsafe { asm!("wfe") };
    }
}
