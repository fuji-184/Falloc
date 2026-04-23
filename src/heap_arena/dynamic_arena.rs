
pub struct Block {
    ptr: *mut u8,
    cursor: *mut u8,
    end: *mut u8,
    layout: std::alloc::Layout,
    next: Option<Box<Block>>
}

pub struct DynamicHeapArena {
    head: std::cell::UnsafeCell<Block>
}

impl DynamicHeapArena {
    #[inline(always)]
    pub fn new(size: usize) -> Self {
        
        let layout = unsafe { std::alloc::Layout::from_size_align_unchecked(size, std::mem::align_of::<usize>()) };

        let start = unsafe { std::alloc::alloc(layout) };
            if start.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            Self {
                head: std::cell::UnsafeCell::new(Block {
                    ptr: start,
                    cursor: start,
                    end: unsafe { start.add(size) },
                    layout: layout,
                    next: None
                })
            }
        
    }
    
    unsafe fn grows(&self, min_size: usize) {
        println!("grow");
        let head = unsafe { &mut *self.head.get() };
        let new_size = min_size.max(head.layout.size() * 2);
        let layout = std::alloc::Layout::from_size_align(new_size, 8).unwrap();
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() { std::alloc::handle_alloc_error(layout); }

        let old_block = unsafe { std::ptr::replace(head, Block {
            ptr,
            layout,
            cursor: ptr,
            end: ptr.add(new_size),
            next: None,
        }) };
        
        head.next = Some(Box::new(old_block));
    }

    #[inline(always)]
    pub fn reset(&mut self) {
        let mut current = unsafe { &mut *self.head.get() };
        loop {
            current.cursor = current.ptr;
            if let Some(ref mut next_block) = current.next {
                current = next_block.as_mut();
            } else {
                break;
            }
        }
    }
}

impl Drop for DynamicHeapArena {
    #[inline(always)]
    fn drop(&mut self) {
        let mut current = unsafe { std::ptr::replace(
            self.head.get(),
            Block {
                ptr: std::ptr::null_mut(),
                cursor: std::ptr::null_mut(),
                end: std::ptr::null_mut(),
                layout: std::alloc::Layout::from_size_align(1, 1).unwrap(),
                next: None,
            },
        ) };
        loop {
            if !current.ptr.is_null() {
                unsafe { std::alloc::dealloc(current.ptr, current.layout) };
            }

            if let Some(next_block) = current.next {
                current = *next_block;
            } else {
                break;
            }
        }
        
    }
}

unsafe impl std::alloc::Allocator for &DynamicHeapArena {
    #[inline(always)]
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let head = unsafe { &mut *self.head.get() };
        let align = layout.align();

        let handle_addr = head.cursor as usize;
        let aligned_addr = handle_addr.checked_next_multiple_of(align)
                .ok_or(std::alloc::AllocError)?;
        let size = layout.size();
        let end_addr = head.end as usize;
        if aligned_addr.checked_add(size).unwrap_or(usize::MAX) > end_addr {
            unsafe { self.grows(size) };
            return self.allocate(layout);
        }
        
        let ptr = head.cursor.map_addr(|_| aligned_addr);
        let next = unsafe { ptr.add(size) };
        head.cursor = next as *mut u8;
            
        return Ok(unsafe {
            std::ptr::NonNull::new_unchecked(
                std::ptr::slice_from_raw_parts_mut(ptr, size),
            )
        });
        
    }

    #[inline(always)]
    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {}
}
