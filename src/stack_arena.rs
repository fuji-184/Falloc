pub struct StackArena<const N: usize> {
    val: [std::mem::MaybeUninit<u8>; N],
    offsite: std::cell::UnsafeCell<usize>,
}

impl<const N: usize> StackArena<N> {
    #[inline(always)]
    pub fn new() -> Self {
        const UNINIT: std::mem::MaybeUninit<u8> = std::mem::MaybeUninit::uninit();
        Self {
            val: [UNINIT; N],
            offsite: std::cell::UnsafeCell::new(0),
        }
    }

    #[inline(always)]
    pub fn alloc<T>(&self) -> &mut std::mem::MaybeUninit<T> {
        unsafe {
            let offset = self.offsite.get();
            let align = std::mem::align_of::<T>();
            let size = std::mem::size_of::<T>();
            let val_ptr = self.val.as_ptr() as *mut u8;
            let ptr = val_ptr.add(*offset);
            let align_offset = ptr.align_offset(align);
            let start = *offset + align_offset;
            let end = start + size;
            debug_assert!(end <= N, "Arena outs of memory");
            let dst = val_ptr.add(start) as *mut std::mem::MaybeUninit<T>;
            *offset = end;
            &mut *dst
        }
    }

    #[inline(always)]
    pub fn reset(&self) {
        unsafe {
            *self.offsite.get() = 0;
        }
    }
}
