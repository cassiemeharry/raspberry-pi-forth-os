use alloc::boxed::Box;
use cfg_if::cfg_if;
use core::fmt::{self, Write};

#[macro_export]
macro_rules! print {
    () => {{
        use core::fmt::Write;
        let mut out = &mut crate::console::output();
        write!(out, "").unwrap();
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut out = &mut crate::console::output();
        write!(out, $($arg)*).unwrap();
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        use core::fmt::Write;
        let mut out = &mut crate::console::output();
        writeln!(out, "").unwrap();
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut out = &mut crate::console::output();
        writeln!(out, $($arg)*).unwrap();
    }};
}

#[macro_export]
macro_rules! println_semihosting {
    () => {{
        use core::fmt::Write;
        let mut out = &mut crate::console::output_prefer_semihosting();
        writeln!(out, "").unwrap();
    }};
    ($($arg:tt)*) => {{
        use core::fmt::Write;
        let mut out = &mut crate::console::output_prefer_semihosting();
        writeln!(out, $($arg)*).unwrap();
    }};
}

#[cfg(feature = "semihosting")]
pub fn output_prefer_semihosting() -> impl Write {
    struct BufferWriter<'a> {
        buffer: &'a mut [u8],
    }
    impl<'a> Write for BufferWriter<'a> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let mut index = 0;
            for byte in s.bytes() {
                if index >= self.buffer.len() - 1 {
                    return Err(fmt::Error);
                }
                self.buffer[index] = byte;
                index += 1;
            }
            self.buffer[index] = 0;
            unsafe {
                crate::rpi::semihosting::puts(self.buffer.as_mut_ptr());
            }
            Ok(())
        }
    }
    static mut BUFFER: [u8; 256] = [0; 256];
    let mut writer: BufferWriter<'static> = unsafe {
        BufferWriter {
            buffer: &mut BUFFER,
        }
    };
    writer
}

#[cfg(not(feature = "semihosting"))]
pub fn output_prefer_semihosting() -> impl Write {
    output()
}

pub fn output() -> impl Write {
    use crate::rpi::{console::Console, uart::UART};
    struct BoxWriter {
        inner: Box<dyn Write + 'static>,
    }
    impl Write for BoxWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.inner.write_str(s)
        }

        fn write_char(&mut self, c: char) -> fmt::Result {
            self.inner.write_char(c)
        }

        fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
            self.inner.write_fmt(args)
        }
    }

    // return UART::new();

    let inner = match Console::new() {
        Some(console) => Box::new(console) as Box<dyn Write + 'static>,
        None => Box::new(UART::new()) as Box<dyn Write + 'static>,
    };
    BoxWriter { inner }
}

mod c_funcs {
    #[inline(never)]
    #[no_mangle]
    pub unsafe extern "C" fn puts(bytes: *const u8) -> i32 {
        let mut len: isize = 0;
        while *bytes.offset(len) != 0 {
            len += 1;
        }
        let bytes = core::slice::from_raw_parts(bytes, len as usize);
        match core::str::from_utf8(bytes) {
            Ok(s) => print!("{}", s),
            Err(err) => print!("(puts: bad UTF-8: {}){:?}", err, bytes),
        };
        return len as i32;
    }

    #[no_mangle]
    pub unsafe extern "C" fn putchar(byte: u8) {
        print!("{}", byte as char);
    }
}
