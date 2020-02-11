use core::{sync::atomic::{AtomicBool, Ordering}, fmt};

use crate::rpi::mailbox;

// pub const GET_CLOCK_STATE_TAG: u32 = 0x0003_0001;
// pub const SET_CLOCK_STATE_TAG: u32 = 0x0003_8001;
// pub const GET_CLOCK_RATE_TAG: u32 = 0x0003_0002;
// pub const SET_CLOCK_RATE_TAG: u32 = 0x0003_8002;
// pub const GET_CLOCK_MAX_RATE_TAG: u32 = 0x0003_0004;
// pub const GET_CLOCK_MIN_RATE_TAG: u32 = 0x0003_0007;

trait ToBytes {
    fn to_bytes<'a>(&'a self) -> &'a [u8];
    fn to_bytes_mut<'a>(&'a mut self) -> &'a mut [u8];
}

impl ToBytes for [u32] {
    fn to_bytes<'a>(&'a self) -> &'a [u8] {
        let ptr: *const u32 = self.as_ptr();
        let new_ptr = ptr as *const u8;
        let new_len = self.len().checked_mul(4).unwrap();
        unsafe { core::slice::from_raw_parts(new_ptr, new_len) }
    }

    fn to_bytes_mut<'a>(&'a mut self) -> &'a mut [u8] {
        let ptr: *mut u32 = self.as_mut_ptr();
        let new_ptr = ptr as *mut u8;
        let new_len = self.len().checked_mul(4).unwrap();
        unsafe { core::slice::from_raw_parts_mut(new_ptr, new_len) }
    }
}

unsafe fn uart1_init() {
    const UART1_CLOCK: u32 = 0x0000_0002;
    const UART1_POWER: u32 = 0x0000_0002;

    const GET_POWER_STATE_TAG: u32 = 0x0002_0001;
    const SET_POWER_STATE_TAG: u32 = 0x0002_8001;
    #[repr(C)]
    struct IdInfo {
        id: u32,
        info: u32,
    }

    let mut message = mailbox::Message::new(GET_POWER_STATE_TAG, IdInfo {
        id: UART1_POWER,
        info: 0,
    });
    let response = message.send(mailbox::Channel::Power).unwrap();
    let exists = ((response.info >> 1) & 1) == 0;
    let powered_on = ((response.info >> 0) & 1) == 1;
    if !exists {
        return;
    }

    if !powered_on {
        const ON: u32 = 1 << 0;
        const WAIT: u32 = 1 << 1;
        let mut message = mailbox::Message::new(SET_POWER_STATE_TAG, IdInfo {
            id: UART1_POWER,
            info: ON | WAIT,
        });
        let response = message.send(mailbox::Channel::Power).unwrap();
    }

    // const GET_CLOCK_STATE_TAG: u32 = 0x0003_0001;
    // const GET_CLOCK_MAX_RATE_TAG: u32 = 0x0003_0004;
    // const SET_CLOCK_RATE_TAG: u32 = 0x0003_8002;

    // let msg = IdInfo { id: UART1_CLOCK, info: 0 };
    // let mut message = mailbox::Message::new(GET_CLOCK_STATE_TAG, msg);
    // let result: &ClockInfo = message.send(mailbox::Channel::PropertyTagsSend).unwrap();
    // assert_eq!(result.info & 0011, 0b01);

    // let msg = IdInfo { id: UART1_CLOCK, info: 0 };
    // let mut message = mailbox::Message::new(GET_CLOCK_MAX_RATE_TAG, msg);
    // let result: &ClockInfo = message.send(mailbox::Channel::PropertyTagsSend).unwrap();
    // let max_rate = result.info;

    // let msg = IdInfo { id: UART1_CLOCK, info: 0 };
    // let mut message = mailbox::Message::new(SET_CLOCK_RATE_TAG, msg);
    // let result: &ClockInfo = message.send(mailbox::Channel::PropertyTagsSend).unwrap();
    // let actual_rate = result.info;

    // response_buffer = [0; 2];
    // response_size = 0;
    // request_response(&mut [MessageRequest {
    //     tag: SET_CLOCK_RATE_TAG,
    //     request_buffer: &[UART1_CLOCK, rate, 1].to_bytes(),
    //     response_buffer: (&mut response_buffer).to_bytes_mut(),
    //     response_size: &mut response_size,
    // }]);
    // assert_eq!(response_size, 8);
    // assert_eq!(response_buffer[0], UART1_CLOCK);
    // let _actual_rate = response_buffer[1];
}

static UART1_IS_INITING: AtomicBool = AtomicBool::new(false);
static UART1_INIT_ONCE: spin::Once<()> = spin::Once::new();

pub struct UART1 {}

impl UART1 {
    pub fn new() -> UART1 {
        if UART1_IS_INITING.swap(true, Ordering::SeqCst) {
            panic!("UART1 init is nested");
        }
        UART1_INIT_ONCE.call_once(|| unsafe { uart1_init() });
        UART1 {}
    }
}

impl fmt::Write for UART1 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        unimplemented!()
    }
}
