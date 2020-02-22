use cfg_if::cfg_if;
use core::fmt::{self, Write};

pub fn output() -> impl Write {
    cfg_if! {
        if #[cfg(feature = "semihosting")] {
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
        } else {
            use crate::rpi::uart::UART;
            let mut uart = UART::new();
            uart
        }
    }
}

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
