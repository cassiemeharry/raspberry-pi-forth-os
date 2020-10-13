use spin::{Mutex, MutexGuard};

#[cfg(all(feature = "semihosting", target_pointer_width = "64"))]
unsafe fn syscall(number: u8, parameter: usize) -> isize {
    let result: isize;
    asm!(
        "hlt 0xF000" : "={x0}"(result) : "{w0}"(number), "{x1}"(parameter) :: "volatile"
    );
    result
}

#[cfg(all(feature = "semihosting", target_pointer_width = "32"))]
unsafe fn syscall(number: u32, parameter: usize) -> isize {
    let result: isize;
    asm!(
        "hlt 0xF000" : "={r0}"(result) : "{r0}"(number), "{r1}"(parameter) :: "volatile"
    );
    result
}

#[cfg(not(feature = "semihosting"))]
unsafe fn syscall(number: u32, parameter: usize) -> isize {
    panic!("Attempted semihosting call {:?} with parameter {:?}", number, parameter)
}

fn check_init() -> Result<(), ()> {
    static SH_STATUS: Mutex<isize> = Mutex::new(0);

    let mut lock = SH_STATUS.lock();
    if *lock != 0 {
        return Ok(());
    }
    Ok(())
}

#[repr(usize)]
enum OpenMode {
    R = 0,
    RB = 1,
    RPlus = 2,
    RPlusB = 3,
    W = 4,
    WB = 5,
    WPlus = 6,
    WPlusB = 7,
    A = 8,
    AB = 9,
    APlus = 10,
    APlusB = 11,
}

#[must_use]
fn sys_open(filename: &[u8], mode: OpenMode) -> Option<usize> {
    debug_assert_eq!(filename[filename.len() - 1], 0);

    #[repr(C)]
    struct OpenParamBlock {
        bytes: *const u8,
        mode: usize,
        len: usize,
    }
    // #[cfg(target_pointer_width = "32")]
    // assert_eq!(core::mem::size_of::<OpenParamBlock>(), 3 * 32);
    // #[cfg(target_pointer_width = "64")]
    // assert_eq!(core::mem::size_of::<OpenParamBlock>(), 3 * 64);

    #[used]
    let size = core::mem::size_of::<OpenParamBlock>();

    let mut params = OpenParamBlock {
        bytes: filename.as_ptr(),
        mode: mode as usize,
        len: filename.len(),
    };
    let params_ptr = &params as *const OpenParamBlock;
    let result = unsafe { syscall(0x01, params_ptr as usize) };
    if result >= 0 {
        Some(result as usize)
    } else {
        None
    }
}

#[inline(never)]
fn sys_write0(bytes: &[u8]) {
    debug_assert_eq!(bytes[bytes.len() - 1], 0);

    let _result = unsafe { syscall(0x04, bytes.as_ptr() as usize) };
}
