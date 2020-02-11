use core::{fmt, ops};
use spin::Mutex;

use super::mailbox::{self, Channel, PropertyMessage, PropertyTagList};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Pixel {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl From<(u8, u8, u8)> for Pixel {
    fn from((r, g, b): (u8, u8, u8)) -> Pixel {
        Pixel { r, g, b }
    }
}

#[repr(C)]
pub struct Framebuffer {
    buffer: &'static mut [u8],
    width: u32,
    height: u32,
    pitch: u32,
    depth: u32,
}

impl fmt::Debug for Framebuffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Framebuffer: {}x{} (pitch: {}, depth: {}) at {:p}",
            self.width,
            self.height,
            self.pitch,
            self.depth,
            self.buffer.as_ptr(),
        )
    }
}

impl Framebuffer {
    /// Call some code with the framebuffer initialized and ready to use. If the
    /// framebuffer initialization fails, this will skip calling `f`.
    pub fn with<F>(f: F)
    where
        F: for<'r> FnOnce(&'r mut Framebuffer),
    {
        let mut fb_opt = FRAMEBUFFER.lock();
        match fb_opt.as_mut() {
            Some(fb) => f(fb),
            None => {
                #[repr(C)]
                #[derive(Clone, Debug)]
                struct FBInitMessage {
                    // 0-3 width Width of the requested frame buffer. My code
                    // uses a value of 640 here.
                    width: u32,
                    // 4-7 height Height of the requested frame buffer. My code
                    // uses a value of 480 here.
                    height: u32,
                    // 8-11 virtual_width Virtual Width -- easiest thing to do
                    // is to set this to width. I'm not entirely certain what
                    // this does (perhaps rescales?).
                    v_width: u32,
                    // 12-15 virtual_height Virtual Height -- easiest thing to
                    // do is to set this to height. I'm not entirely certain
                    // what this does (perhaps rescales?).
                    v_height: u32,
                    // 16-19 pitch Number of bytes between each row of the frame
                    // buffer. This is set by the GPU; in my code, I set it to
                    // zero before passing the structure to the GPU.
                    pitch: u32,
                    // 20-23 depth The number of bits per pixel of the requested
                    // frame buffer. I have not managed to make this work with
                    // anything other than a value of 24, however the Linux
                    // source seems to use 16 bit?!
                    depth: u32,
                    // 24-27 x_offset Offset in the x direction. The easiest
                    // thing to do is to set this to zero. I'm not entirely
                    // certain exactly what this does.
                    x_offset: u32,
                    // 28-31 y_offset Offset in the y direction. The easiest
                    // thing to do is to set this to zero. I'm not entirely
                    // certain exactly what this does.
                    y_offset: u32,
                    // 32-35 pointer The pointer to the frame buffer into which
                    // your code should write. This is set by the GPU. I set
                    // this to zero before passing the structure to the GPU.
                    pointer: u32,
                    // 36-39 size The size of the frame buffer. Set by the GPU.
                    // I set this to zero before passing the structure to the
                    // GPU. }
                    size: u32,
                }

                const USE_PROPERTY_INTERFACE: bool = true;
                let mut fb = if USE_PROPERTY_INTERFACE {
                    #[repr(C)]
                    #[derive(Debug)]
                    struct AllocBuffer {
                        addr: u32,
                        size: u32,
                    }
                    let width: u32 = 640;
                    let height: u32 = 480;
                    let mut set_message = (
                        // Allocate buffer
                        PropertyMessage::new(0x0004_0001, AllocBuffer { addr: 0, size: 0 }),
                        // Set size
                        PropertyMessage::new(0x0004_8003, (width, height)),
                        // Set virtual buffer width/height
                        PropertyMessage::new(0x0004_8004, (width, height)),
                        // Set depth
                        PropertyMessage::new(0x0004_8005, 24_u32),
                        // Set pixel order (0 = BGR, 1 = RGB)
                        PropertyMessage::new(0x0004_8006, 1_u32),
                        // Set pitch
                        PropertyMessage::new(0x0004_8008, core::mem::align_of::<Pixel>() as u32),
                    )
                        .prepare();
                    let buffer: &'static mut [u8] = match set_message.send() {
                        Some(set_result) => {
                            let buf_result = &set_result.0;
                            if buf_result.addr == 0 {
                                println!("Failed to initialize framebuffer (got null address back from GPU)");
                                return;
                            }
                            if buf_result.size == 0 {
                                println!("Failed to initialize framebuffer (got zero-size buffer back from GPU)");
                                return;
                            }
                            let size_result = &set_result.1;
                            unsafe {
                                core::slice::from_raw_parts_mut(
                                    buf_result.addr as usize as *mut u8,
                                    (size_result.0 as usize) * (size_result.1 as usize) * 3,
                                    // buf_result.size as usize,
                                )
                            }
                        }
                        None => {
                            println!("Failed to initialize framebuffer");
                            return;
                        }
                    };

                    let mut get_message = (
                        // Get size
                        PropertyMessage::new(0x0004_0003, (width, height)),
                        // Get depth
                        PropertyMessage::new(0x0004_0005, 24_u32),
                        // Get pixel order
                        PropertyMessage::new(0x0004_0006, 0xFFFF_FFFF_u32),
                        // Get virtual buffer width/height
                        PropertyMessage::new(0x0004_0004, (0_u32, 0_u32)),
                        // Get pitch
                        PropertyMessage::new(0x0004_0008, 0_u32),
                    )
                        .prepare();
                    match get_message.send() {
                        Some(result) => {
                            println!("Got result: {:#x?}", result);
                            let size = *result.0;
                            let depth = *result.1;
                            let pixel_order = *result.2;
                            let virtual_size = *result.3;
                            let pitch = *result.4;
                            assert_eq!(size, (width, height));
                            assert_eq!(depth, 24);
                            assert_eq!(pixel_order, 1);
                            assert_eq!(virtual_size, (width, height));
                            assert!(pitch > 0);
                            Framebuffer {
                                buffer,
                                width: size.0,
                                height: size.1,
                                depth,
                                pitch,
                            }
                        }
                        None => {
                            println!("Failed to initialize framebuffer");
                            return;
                        }
                    }
                } else {
                    const FB_CHANNEL: Channel = Channel::Framebuffer;
                    let mut msg = FBInitMessage {
                        width: 640,
                        height: 480,
                        v_width: 640,
                        v_height: 640,
                        pitch: 0,
                        depth: 24,
                        x_offset: 0,
                        y_offset: 0,
                        pointer: 0,
                        size: 0,
                    };
                    let send_result = mailbox::send_raw_message(FB_CHANNEL, &mut msg);
                    match send_result {
                        Ok(0) => (),
                        Ok(other) => {
                            println!("Got unexpected framebuffer response message {:?}", other);
                            return;
                        }
                        Err(()) => {
                            println!("Failed to initialize framebuffer");
                            return;
                        }
                    };
                    Framebuffer {
                        buffer: unsafe {
                            core::slice::from_raw_parts_mut(
                                msg.pointer as *mut u8,
                                msg.size as usize,
                            )
                        },
                        width: msg.width,
                        height: msg.height,
                        pitch: msg.pitch,
                        depth: msg.depth,
                    }
                };
                f(&mut fb);
                *fb_opt = Some(fb);
            }
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[Pixel] {
        assert_eq!(self.depth, 24);
        let addr = self.buffer.as_ptr() as *const Pixel;
        let size = self.buffer.len() / (self.depth as usize / 8);
        unsafe { core::slice::from_raw_parts(addr, size) }
    }

    pub fn pixels_mut(&mut self) -> &mut [Pixel] {
        assert_eq!(self.depth, 24);
        let addr = self.buffer.as_mut_ptr() as *mut Pixel;
        let size = self.buffer.len() / (self.depth as usize / 8);
        unsafe { core::slice::from_raw_parts_mut(addr, size) }
    }

    fn buffer_index(&self, x: u32, y: u32) -> usize {
        // y * pitch + x * 3 + rgb_channel, where rgb_channel is 0 for red, 1 for green, and 2 for blue.
        assert_eq!(self.depth, 24);
        let y_offset = y as usize * self.pitch as usize;
        let x_offset = x as usize * (self.depth / 8) as usize;
        y_offset + x_offset
    }
}

impl ops::Index<(u32, u32)> for Framebuffer {
    type Output = Pixel;

    fn index(&self, (x, y): (u32, u32)) -> &Pixel {
        let index = self.buffer_index(x, y);
        let slice = &self.buffer[index..index + 3];
        let ptr = slice.as_ptr();
        unsafe { &*(ptr as *const Pixel) }
    }
}

impl ops::IndexMut<(u32, u32)> for Framebuffer {
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut Pixel {
        assert_eq!(self.depth, 24);
        let index = ((y * self.pitch) + (x * (self.depth / 8))) as usize;
        let slice = &mut self.buffer[index..index + 3];
        let ptr = slice.as_mut_ptr();
        unsafe { &mut *(ptr as *mut Pixel) }
    }
}

static FRAMEBUFFER: spin::Mutex<Option<Framebuffer>> = Mutex::new(None);
