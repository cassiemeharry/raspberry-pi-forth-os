use spin::{Mutex, MutexGuard};

#[cfg(target_pointer_width = "64")]
unsafe fn syscall(number: u8, parameter: usize) -> isize {
    let result: isize;
    asm!(
        "hlt 0xF000" : "={x0}"(result) : "{w0}"(number), "{x1}"(parameter) :: "volatile"
    );
    result
}

#[cfg(target_pointer_width = "32")]
unsafe fn syscall(number: u32, parameter: usize) -> isize {
    let result: isize;
    asm!(
        "hlt 0xF000" : "={r0}"(result) : "{r0}"(number), "{r1}"(parameter) :: "volatile"
    );
    result
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

static SH_TTY_OUT: Mutex<Option<usize>> = Mutex::new(None);

#[inline(never)]
#[no_mangle]
pub unsafe extern "C" fn puts(bytes: *const u8) -> i32 {
    let mut len: isize = 0;
    while *bytes.offset(len) != 0 {
        len += 1;
    }
    let bytes = core::slice::from_raw_parts(bytes, len as usize);
    match check_init() {
        Ok(()) => (),
        Err(()) => return -1,
    };
    let mut tty_lock = SH_TTY_OUT.lock();
    let handle = match *tty_lock {
        Some(handle) => handle,
        None => {
            match sys_open(b":tt\0", OpenMode::W) {
                Some(handle) => {
                    *tty_lock = Some(handle);
                    handle
                },
                None => return -1,
            }
        }
    };
    sys_write0(bytes);
    return len as i32;
}
