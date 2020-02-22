// use byteorder::{ByteOrder, NativeEndian};
use core::{
    convert::TryInto,
    fmt, mem, ops, slice,
    sync::atomic::{fence, Ordering},
};

use crate::allocator::align_up;

const MAIL_BASE: usize = 0xB880;

const MAIL_FULL: u32 = 0x8000_0000;
const MAIL_EMPTY: u32 = 0x4000_0000;

// const MAPPED_REGISTERS_BASE: usize = 0x2000_0000;
const MAPPED_REGISTERS_BASE: usize = 0x3f00_0000;
// const MAPPED_REGISTERS_BASE: usize = 0x7E00_0000;

#[derive(Copy, Clone, Debug)]
struct MailboxRegisterOffsets {
    read: u8,
    peek: u8,
    sender: u8,
    status: u8,
    config: u8,
    write: u8,
}

const MAILBOX_OFFFSETS: MailboxRegisterOffsets = MailboxRegisterOffsets {
    read: 0x00,
    peek: 0x10,
    sender: 0x14,
    status: 0x18,
    config: 0x1c,
    write: 0x20,
};
//     MailboxRegisterOffsets {
//         read: 0x20,
//         peek: 0x30,
//         sender: 0x34,
//         status: 0x38,
//         config: 0x3c,
//         write: 0x40,
//     },
// ];

#[inline]
unsafe fn read_reg(base: usize, offset: u8) -> u32 {
    ((MAPPED_REGISTERS_BASE + base + offset as usize) as *const u32).read_volatile()
}

#[inline]
unsafe fn write_reg(base: usize, offset: u8, value: u32) {
    ((MAPPED_REGISTERS_BASE + base + offset as usize) as *mut u32).write_volatile(value)
}

unsafe fn read_mailbox(channel: u8) -> u32 {
    // 1. Read the status register until the empty flag is not set.
    // 2. Read data from the read register.
    // 3. If the lower four bits do not match the channel number desired repeat
    //    from 1.
    // 4. The upper 28 bits are the returned data.

    // Wait for the mailbox to be non-empty
    //     Execute a memory barrier
    //     Read MAIL0_STATUS
    //     Goto step 1 if MAIL_EMPTY bit is set
    // Execute a memory barrier
    // Read from MAIL0_READ
    // Check the channel (lowest 4 bits) of the read value for the correct channel
    // If the channel is not the one we wish to read from (i.e: 1), go to step 1
    // Return the data (i.e: the read value >> 4)

    // println!("Reading mailbox (want channel {})", channel);

    loop {
        loop {
            fence(Ordering::SeqCst);
            if read_reg(MAIL_BASE, MAILBOX_OFFFSETS.status) & MAIL_EMPTY == 0 {
                break;
            }
        }
        fence(Ordering::SeqCst);
        let data: u32 = read_reg(MAIL_BASE, MAILBOX_OFFFSETS.read);
        let read_channel = (data & 0x0F) as u8;
        let data = data >> 4;
        // println!(
        //     "Got data from mailbox: {:#8x} (from channel {})",
        //     data, read_channel
        // );
        if read_channel != channel {
            // println!("Wrong channel, trying again...");
            continue;
        }
        return data;
    }
}

unsafe fn write_mailbox(channel: u8, data: u32) {
    // 1. Read the status register until the full flag is not set.
    // 2. Write the data (shifted into the upper 28 bits) combined with the
    //    channel (in the lower four bits) to the write register.
    // println!("Writing {:#8x} to mailbox channel {}", data, channel);
    loop {
        // Wait for space
        fence(Ordering::SeqCst);
        if read_reg(MAIL_BASE, MAILBOX_OFFFSETS.status + 0x20) & MAIL_FULL == 0 {
            break;
        }
    }
    write_reg(MAIL_BASE, MAILBOX_OFFFSETS.write, data | (channel as u32));
    fence(Ordering::SeqCst);
    // println!("Finished writing to mailbox");
}

pub trait PropertyTagList: Sized {
    fn prepare(self) -> PropertyMessageWrapper<Self> {
        PropertyMessageWrapper::new(self)
    }
}

macro_rules! impl_ptl {
    ( $( $t:ident ),+ ) => {
        impl< $($t),+ > PropertyTagList for ( $(PropertyMessage< $t >, )+ )
        where $(
            $t: Sized,
        )+ {}
    };
}

impl<T: Sized> PropertyTagList for PropertyMessage<T> {}
impl_ptl!(T1);
impl_ptl!(T1, T2);
impl_ptl!(T1, T2, T3);
impl_ptl!(T1, T2, T3, T4);
impl_ptl!(T1, T2, T3, T4, T5);
impl_ptl!(T1, T2, T3, T4, T5, T6);
impl_ptl!(T1, T2, T3, T4, T5, T6, T7);
impl_ptl!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_ptl!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_ptl!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);

#[repr(C, align(16))]
#[derive(Debug)]
pub struct PropertyMessageWrapper<TL: PropertyTagList> {
    buffer_size: u32,
    code: u32,
    tags: TL,
    end: u32,
}

impl<TL: PropertyTagList> PropertyMessageWrapper<TL> {
    #[inline]
    fn new(tags: TL) -> Self {
        PropertyMessageWrapper {
            buffer_size: mem::size_of::<Self>()
                .try_into()
                .expect("Property message list size in bytes is too big to fit in a u32"),
            code: 0x0000_0000,
            tags,
            end: 0x0000_0000,
        }
    }

    fn as_quads(&self) -> &[u32] {
        let size_bytes = mem::size_of::<Self>();
        debug_assert_eq!(size_bytes % 4, 0);
        let u32_size: usize = size_bytes / 4;
        unsafe { slice::from_raw_parts((self as *const Self) as *const u32, u32_size) }
    }

    pub fn send<'a>(&'a mut self) -> Option<&'a TL>
    where
        TL: fmt::Debug,
    {
        // println!("Property message before sending over mailbox: {:#x?}", self);
        // println!(
        //     "Property message quads before sending over mailbox: {:#x?}",
        //     self.as_quads()
        // );
        const CHANNEL: u8 = Channel::PropertyTagsSend as u8;
        unsafe {
            let ptr = self as *const Self;
            let addr = ptr as usize;
            write_mailbox(CHANNEL, addr.try_into().ok()?);
            let resp_addr = read_mailbox(CHANNEL);
            // let resp_ptr = resp_addr as *const u32;
            // println!("Got response from mailbox: {:#?}", &*resp_ptr);
            // let resp_code: u32 = *resp_ptr.offset(1);
            // println!(
            //     "Property message after response {:#8x}: {:#x?}",
            //     resp_addr, self
            // );
            // {
            //     let message_quads = self.as_quads();
            //     println!("Property message words: {:#x?}", message_quads);
            // }
            if self.code != 0x8000_0000 {
                return None;
            }
            // let msg_ptr = resp_ptr.offset(2);

            // let value_buffer_size_ptr = msg_ptr.offset(1);
            // let value_buffer_size = (*value_buffer_size_ptr) as usize;
            // let value_buffer_ptr = msg_ptr.offset(3) as *const T;
            // assert_eq!(value_buffer_size, mem::size_of::<T>());
            // let value_ref = &*(value_buffer_ptr as *const T);
            // Some(value_ref)
            Some(&self.tags)
        }
    }
}

impl<TL: PropertyTagList> ops::Deref for PropertyMessageWrapper<TL> {
    type Target = TL;
    fn deref(&self) -> &TL {
        &self.tags
    }
}

impl<TL: PropertyTagList> ops::DerefMut for PropertyMessageWrapper<TL> {
    fn deref_mut(&mut self) -> &mut TL {
        &mut self.tags
    }
}

#[repr(C, align(4))]
#[derive(Debug)]
pub struct PropertyMessage<T> {
    tag: u32,
    buffer_size: u32,
    code: u32,
    buffer: T,
}

impl<T> ops::Deref for PropertyMessage<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.buffer
    }
}

impl<T> ops::DerefMut for PropertyMessage<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.buffer
    }
}

impl<T: Sized> From<(u32, T)> for PropertyMessage<T> {
    fn from((tag, buffer): (u32, T)) -> PropertyMessage<T> {
        PropertyMessage::new(tag, buffer)
    }
}

impl<T> PropertyMessage<T> {
    pub fn new(tag: u32, buffer: T) -> Self {
        let buffer_size = align_up(mem::size_of::<T>(), 4)
            .try_into()
            .expect("Property message size is too big to fit in a u32");
        PropertyMessage {
            tag,
            buffer_size,
            code: 0,
            buffer,
        }
    }
}

// impl<T: fmt::Debug> PropertyMessage<T> {
//     pub fn new(tag: u32, buffer: T) -> Self {
//         PropertyMessage {
//         }
//     }
// }

pub fn send_raw_message<T: fmt::Debug>(channel: Channel, msg: &mut T) -> Result<u32, ()> {
    let resp: u32;
    let msg_ptr = msg as *mut T;
    let msg_addr_usize = msg_ptr as usize;
    let msg_addr_u32 = msg_addr_usize.try_into().map_err(|_| ())?;
    unsafe {
        write_mailbox(channel as u8, msg_addr_u32);
        resp = read_mailbox(channel as u8);
    }
    // println!(
    //     "Got response {:#8x} after raw message send: {:#x?}",
    //     resp, msg
    // );
    Ok(resp)
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum Channel {
    Power = 0,
    Framebuffer = 1,
    VirtualUART = 2,
    VCHIQ = 3,
    LEDs = 4,
    Buttons = 5,
    TouchScreen = 6,
    Unknown7 = 7,
    PropertyTagsSend = 8,
    PropertyTagsReceive = 9,
}
