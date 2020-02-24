use alloc::{
    alloc::{GlobalAlloc, Layout},
    collections::BTreeMap,
};
use linked_list_allocator::LockedHeap;
use spin::{Mutex, Once};

use crate::rpi::{
    mailbox::{PropertyMessage, PropertyTagList},
    mmio::P_BASE_PHYSICAL_ADDR,
};

extern "C" {
    type MARKER;

    #[link_name = "__end"]
    static IMAGE_END: MARKER;
}

struct MyAllocatorWrapper {
    inner: Once<LockedHeap>,
}

impl MyAllocatorWrapper {
    const fn new_uninit() -> Self {
        MyAllocatorWrapper { inner: Once::new() }
    }

    fn get_inner(&self) -> &LockedHeap {
        self.inner.call_once(|| {
            #[repr(C)]
            #[derive(Copy, Clone, Debug)]
            struct GetARMMemoryMessage {
                base: u32,
                size: u32,
            }
            let mut msg =
                PropertyMessage::new(0x0001_0005, GetARMMemoryMessage { base: 0, size: 0 })
                    .prepare();
            let result = match msg.send() {
                Some(result) => **result,
                None => {
                    println!("Warning: failed to get memory size from mailbox, using hardcoded defaults.");
                    GetARMMemoryMessage {
                        base: 0,
                        size: 0x3c000000.min(P_BASE_PHYSICAL_ADDR as u32),
                    }
                },
            };
            assert_eq!(result.base, 0);
            let start = (unsafe { &IMAGE_END as *const MARKER as usize } + 4096) & !(4096 - 1);
            let end = ((result.base + result.size) as usize).min(P_BASE_PHYSICAL_ADDR) - 1;
            assert!(start < end);

            let size = end - start;
            println!(
                "Initializing allocator with start {:p} and end {:p} (size: {}B)",
                start as *const MARKER, end as *const MARKER, size,
            );
            let allocator = LockedHeap::empty();
            {
                let mut lock = allocator.lock();
                unsafe { lock.init(start, size) };
            }
            allocator
        })
    }
}

unsafe impl GlobalAlloc for MyAllocatorWrapper {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut inner = self.get_inner();
        inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut inner = self.get_inner();
        inner.dealloc(ptr, layout)
    }
}

#[global_allocator]
static ALLOC: MyAllocatorWrapper = MyAllocatorWrapper::new_uninit();

#[alloc_error_handler]
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("Failed to allocate from layout {:?}", layout)
}
