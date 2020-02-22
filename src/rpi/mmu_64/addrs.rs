use bit_field::BitField;
use core::fmt;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(usize);

impl fmt::Debug for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtAddr({:x})", self.0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct InvalidVirtAddr(usize);

impl fmt::Debug for InvalidVirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "InvalidVirtAddr({:x})", self.0)
    }
}

impl VirtAddr {
    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    pub fn new(addr: u64) -> VirtAddr {
        Self::try_new(addr).expect(
            "address passed to VirtAddr::new must not contain any data in bits 48-64"
        )
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    pub fn try_new(addr: u64) -> Result<VirtAddr, InvalidVirtAddr> {
        match addr.get_bits(48..64) {
            0 | 0xffff => Ok(VirtAddr(addr as usize)),
            1 => Ok(VirtAddr::new_unchecked(addr)),
            other => Err(InvalidVirtAddr(other as usize)),
        }
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    pub const fn new_unchecked(addr: u64) -> VirtAddr {
        VirtAddr((((addr >> 16) as i64) << 16) as usize)
    }

    // pub const fn zero() -> VirtAddr {
    //     VirtAddr(0)
    // }

    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }

    #[inline(always)]
    pub fn from_ptr<T>(ptr: *const T) -> VirtAddr {
        Self::new(ptr as u64)
    }

    #[inline(always)]
    pub fn as_ptr<T>(self) -> *const T {
        self.as_usize() as *const T
    }

    #[inline(always)]
    pub fn as_mut_ptr<T>(self) -> *mut T {
        self.as_ptr::<T>() as *mut T
    }

    // #[inline(always)]
    // pub fn align_up<U: Into<u64>>(self, align: U) -> VirtAddr {
    //     VirtAddr(align_up(self.0, align.into()))
    // }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PhysAddr(usize);

impl fmt::Debug for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PhysAddr({:x})", self.0)
    }
}

impl PhysAddr {
    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    pub const fn new_unchecked(addr: u64) -> PhysAddr {
        PhysAddr(addr as usize)
    }

    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self.0
    }

    #[inline(always)]
    #[cfg(target_pointer_width = "64")]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}
