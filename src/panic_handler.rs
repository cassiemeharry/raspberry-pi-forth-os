use cfg_if::cfg_if;
use core::{
    fmt::{self, Write},
    panic::PanicInfo,
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let out = &mut crate::console::output();
    if let Some(loc) = info.location() {
        let _ = write!(
            out,
            "Panic occurred in file \"{}\" at line {}: ",
            loc.file(),
            loc.line()
        );
    } else {
        let _ = write!(
            out,
            "Panic occurred, but no location information was available: "
        );
    }

    if let Some(args) = info.message() {
        // Ignore the returned result, as we're already in the panic handler and
        // we have nowhere to report the problem.
        let _ = core::fmt::write(out, *args);
        let _ = writeln!(out, "");
    } else if let Some(msg) = info.payload().downcast_ref::<&str>() {
        let _ = writeln!(out, "{}", msg);
    } else {
        let _ = writeln!(out, "No message available");
    }

    #[cfg(feature = "semihosting")]
    qemu_exit::aarch64::exit_failure();
    #[cfg(not(feature = "semihosting"))]
    loop {}
}
