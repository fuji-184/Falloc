
pub struct Block {
    next: Option<std::ptr::NonNull<Block>>
}

pub struct StaticPoolAllocator {
    start: std::ptr::NonNull<u8>,
    head: std::cell::UnsafeCell<Option<std::ptr::NonNull<Block>>>,
    block_size: usize,
    layout: std::alloc::Layout
}

impl StaticPoolAllocator {
    pub fn new(num_blocks: usize, block_size: usize, align_multiply: usize) -> Self {
        if align_multiply == 0 {
            panic!("Align multiply must be > 0");
        }
        
        let block_size = block_size.max(std::mem::size_of::<Block>());
        let layout = std::alloc::Layout::from_size_align(
            num_blocks * block_size,
            std::mem::align_of::<Block>() * align_multiply
        ).expect("Error in creating layout in fn new -> PoolAllocator");
        
        // SAFETY: the layout is valid, because if not it will trigger panic
        let start = unsafe { std::alloc::alloc(layout) };
        let start_ptr = std::ptr::NonNull::new(start).expect("Error OOM in fn new -> PoolAllocator");
        
        unsafe {
            for i in 0..num_blocks {
                let current_ptr = start.add(i * block_size).cast::<Block>();
                let next_ptr = if i < num_blocks - 1 {
                    Some(std::ptr::NonNull::new_unchecked(start.add((i + 1) * block_size).cast::<Block>()))
                } else {
                    None
                };
                
                (*current_ptr).next = next_ptr;
            }
        }
        
        Self {
            start: start_ptr,
            head: std::cell::UnsafeCell::new(Some(std::ptr::NonNull::new(start_ptr.as_ptr().cast::<Block>()).unwrap())),
            block_size,
            layout,
        }
        
    }
}

impl Drop for StaticPoolAllocator {
    fn drop(&mut self) {
        // SAFETY: deallocate using the pointer. The layout is saved, so it is the correct layout
        unsafe {
            std::alloc::dealloc(self.start.as_ptr(), self.layout);
        }
    }
}

unsafe impl std::alloc::Allocator for StaticPoolAllocator {
    #[inline(always)]
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let size = layout.size();
        let align = layout.align();
        let base_align = std::mem::align_of::<Block>();
        if size > self.block_size || align > base_align {
            return Err(std::alloc::AllocError);
        }
        
        let head_ptr = unsafe { &mut *self.head.get() };
        if let Some(block) = *head_ptr {
            let taken_block = block;
            unsafe { *head_ptr = block.as_ref().next };
            let ptr = taken_block.cast::<u8>();
            return Ok(std::ptr::NonNull::slice_from_raw_parts(
                ptr, self.block_size
            ));
        }
        
        Err(std::alloc::AllocError)
    }
    
    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {
        let new_block = ptr.cast::<Block>();
        
        // dereference raw pointer is unsafe
        let head_ptr = unsafe { &mut *self.head.get() };
        unsafe { (*new_block.as_ptr()).next = *head_ptr };
        
        *head_ptr = Some(new_block);
    }
}
