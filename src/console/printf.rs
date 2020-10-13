// I attempted to implement printf in Rust, but aarch64 varargs is busted :(

extern "C" {
    fn vprintf(fmt: *const u8, ap: ...) -> i32;
}

#[no_mangle]
pub unsafe extern "C" fn printf(mut fmt: *const u8, mut args: ...) {
    use alloc::string::String;
    use core::fmt::Write;

    let mut console = crate::console::output();

    let fmt: &str = {
        let mut ptr = fmt;
        let mut length = 0;
        while *ptr != 0 && length < 1024 {
            length += 1;
            ptr = ptr.offset(1);
        }
        let bytes = core::slice::from_raw_parts(fmt, length);
        match core::str::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                write!(console, "(printf: got garbage in format string: {:?})", e);
                return;
            },
        }
    };

    let mut prefix = String::new();

    enum StateMachine<'a> {
        Normal { start: usize },
        Format {
            prefix: &'a mut String,
            u_prefix: bool,
        },
    }

    let mut state_machine = StateMachine::Normal { start: 0 };

    for (i, c) in fmt.chars().enumerate() {
        match state_machine {
            StateMachine::Normal { start } => {
                if c == '%' {
                    if start < i {
                        write!(console, "{}", &fmt[start..i]);
                    }
                    prefix.clear();
                    state_machine = StateMachine::Format {
                        prefix: &mut prefix,
                        u_prefix: false,
                    };
                }
            },
            StateMachine::Format { ref mut prefix, ref mut u_prefix } => {
                match c {
                    'u' => {
                        *u_prefix = true;
                        continue;
                    }
                    'c' => {
                        let x = args.arg::<u8>();
                        let c = x as char;
                        write!(console, "{}", c);
                        state_machine = StateMachine::Normal { start: i + 1 };
                    }
                    'd' => {
                        if *u_prefix {
                            let x = args.arg::<i64>();
                            write!(console, "{}", x);
                        } else {
                            let x = args.arg::<i32>();
                            write!(console, "{}", x);
                        }
                        state_machine = StateMachine::Normal { start: i + 1 };
                    }
                    'p' => {
                        let ptr = args.arg::<*const u8>();
                        write!(console, "{:016p}", ptr);
                        state_machine = StateMachine::Normal { start: i + 1 };
                    }
                    's' => {
                        let start_ptr = args.arg::<usize>() as *const u8;
                        // write!(console, "\n(printf: %s, ptr = {:p})", start_ptr);
                        {
                            let first_bytes = core::slice::from_raw_parts(start_ptr, 8);
                            // write!(console, "(printf: %s, first_bytes = {:?})", first_bytes);
                            let deref = (*(start_ptr as *const usize)) as *const u8;
                            let first_bytes = core::slice::from_raw_parts(deref, 8);
                            // write!(console, "(printf: %s, deref = {:p}, deref first_bytes = {:?})", deref, first_bytes);
                        }
                        let mut ptr = start_ptr;
                        let mut length = 0;
                        while *ptr != 0 && length < 1024 {
                            length += 1;
                            ptr = ptr.offset(1);
                        }
                        let bytes = core::slice::from_raw_parts(start_ptr, length);
                        let string: String = bytes.iter().map(|b| *b as char).collect();
                        write!(console, "{}", string);
                        // match core::str::from_utf8(bytes) {
                        //     Ok(s) => write!(console, "{}", string),
                        //     Err(e) => write!(
                        //         console,
                        //         "(printf: failed to convert {} bytes in %s formatter: error={:?}, ptr={:p}, bytes={:x?})",
                        //         length, e, start_ptr, bytes
                        //     ),
                        // };
                        state_machine = StateMachine::Normal { start: i + 1 };
                    }
                    'x' => {
                        if *u_prefix {
                            let x = args.arg::<u64>();
                            write!(console, "{:016x}", x);
                        } else {
                            let x = args.arg::<u32>();
                            write!(console, "{:08x}", x);
                        }
                        state_machine = StateMachine::Normal { start: i + 1 };
                    }
                    other => {
                        prefix.push(other);
                    }
                }
            }
        }
    }
    match state_machine {
        StateMachine::Normal { start } => {
            write!(console, "{}", &fmt[start..]);
        }
        StateMachine::Format { .. } => {
            write!(console, "(printf: hit end of format string while parsing '%' escape)");
        }
    }
}

// struct FormatStr<'a, T: ?Sized> {
//     prefix: &'a str,
//     value: &'a T,
// }

// impl<'a, T: fmt::Display> FormatStr<'a, T> {
//     fn new(prefix: &'a str, value: &'a T) -> Self {
//         FormatStr { prefix, value }
//     }

//     fn display(&self, f: impl FnOnce(fmt::Arguments<'a>) -> fmt::Result) -> fmt::Result {
//         use fmt::rt::v1::*;

//         let single_arg = fmt::ArgumentV1::new(self.value, <T as fmt::Display>::fmt);
//         let pieces = &[""];
//         let args = &[single_arg];

//         if self.prefix.is_empty() {
//             let formatted = fmt::Arguments::new_v1(pieces, args);
//             f(formatted)
//         } else {
//             let mut format = FormatSpec {
//                 fill: ' ',
//                 align: Alignment::Unknown,
//                 flags: 0,
//                 precision: Count::Implied,
//                 width: Count::Implied,
//             };
//             let mut number_start: Option<usize> = None;
//             for (i, c) in self.prefix.chars().enumerate() {
//                 match c {
//                     '0' => {
//                         if number_start.is_none() {
//                             format.fill = '0';
//                         }
//                     },
//                     '1'..='9' => {
//                         if number_start.is_none() {
//                             number_start = Some(i);
//                             continue;
//                         }
//                     },
//                     other => {
//                         println!("(printf: ignoring format modifier {:?})", other);
//                     },
//                 };
//                 if let Some(start) = number_start {
//                     if !('0'..='9').contains(&c) && start != i {
//                         let width_str = &self.prefix[start..i];
//                         match width_str.parse() {
//                             Ok(width) => {
//                                 format.width = Count::Is(width);
//                             },
//                             Err(err) => {
//                                 println!("(printf: bad width specifier: {:?})", width_str);
//                             },
//                         }
//                     }
//                 }
//             }
//             let fmts = &[fmt::rt::v1::Argument { position: 0, format }];
//             let formatted = fmt::Arguments::new_v1_formatted(pieces, args, fmts);
//             f(formatted)
//         }
//     }

//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         unimplemented!()
//     }
// }
