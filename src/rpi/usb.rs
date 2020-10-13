extern "C" {
    fn usb_init() -> i32;
    fn usb_stop() -> i32;

    type USB_DEVICE;
}

pub fn init() {
    let result = unsafe { usb_init() };
    println!("Got result from USB init: {:?}", result);
}
