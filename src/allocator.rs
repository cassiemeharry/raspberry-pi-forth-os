use core::marker::PhantomData;

#[repr(transparent)]
pub struct PhysicalAddress<T> {
    ptr: usize,
    tag: PhantomData<*const T>,
}

impl<T> PhysicalAddress<T> {
    pub const fn as_virtual_ptr(self) -> *const T {
        (self.ptr + 0x80000000) as *const T
    }

    pub const fn as_mut_virtual_ptr(self) -> *mut T {
        (self.ptr + 0x80000000) as *mut T
    }
}

// #[repr(transparent)]
// pub struct PhysicalAddressMut<T> {
//     ptr: usize,
//     tag: PhantomData<*mut T>,
// };

impl<T> From<*const T> for PhysicalAddress<T> {
    fn from(virtual_addr: *const T) -> PhysicalAddress<T> {
        (virtual_addr as usize).into()
    }
}

impl<T> From<*mut T> for PhysicalAddress<T> {
    fn from(virtual_addr: *mut T) -> PhysicalAddress<T> {
        (virtual_addr as usize).into()
    }
}

impl<T> From<usize> for PhysicalAddress<T> {
    fn from(virtual_addr: usize) -> PhysicalAddress<T> {
        unimplemented!()
    }
}

impl<T> From<PhysicalAddress<T>> for *const T {
    fn from(physical_addr: PhysicalAddress<T>) -> *const T {
        physical_addr.as_virtual_ptr()
    }
}

impl<T> From<PhysicalAddress<T>> for *mut T {
    fn from(physical_addr: PhysicalAddress<T>) -> *mut T {
        physical_addr.as_mut_virtual_ptr()
    }
}

impl<T> From<PhysicalAddress<T>> for usize {
    fn from(physical_addr: PhysicalAddress<T>) -> usize {
        physical_addr.as_virtual_ptr() as usize
    }
}

pub const fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
