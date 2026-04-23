
pub struct StackArena<const N: usize> {
    #[allow(dead_code)]
    val: std::cell::UnsafeCell<[std::mem::MaybeUninit<u8>; N]>,
    offset: std::cell::UnsafeCell<usize>
}

impl<const N: usize> StackArena<N> {
    #[inline(always)]
    pub fn new() -> Self {
        let val = std::cell::UnsafeCell::new([std::mem::MaybeUninit::uninit(); N]);
        Self {
            val,
            offset: std::cell::UnsafeCell::new(0),
        }
    }
    
    #[inline(always)]
    pub fn reset(&mut self) {
        unsafe { *self.offset.get() = 0 };
    }
}

unsafe impl<const N: usize> std::alloc::Allocator for StackArena<N> {
    #[inline(always)]
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let base_ptr = self.val.get() as *const u8;
        let offset = unsafe { *self.offset.get() };
        
        let cursor = unsafe { base_ptr.byte_add(offset) };
        
        let align = layout.align();
        let size = layout.size();
        
        let aligned_addr = cursor.addr().checked_next_multiple_of(align)
                .ok_or(std::alloc::AllocError)?;
                
        let aligned_ptr = base_ptr.with_addr(aligned_addr);
        let end_ptr = unsafe { aligned_ptr.byte_add(size) };
           
        if unsafe { end_ptr.byte_offset_from(base_ptr) } as usize > N {
            return Err(std::alloc::AllocError);
        }
        
        unsafe { *self.offset.get() = end_ptr.byte_offset_from(base_ptr) as usize };

        Ok(unsafe {
            std::ptr::NonNull::new_unchecked(std::ptr::slice_from_raw_parts_mut(
                aligned_ptr.cast_mut(),
                size,
            ))
        }) 
    }
    
    #[inline(always)]
    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) { }
}
