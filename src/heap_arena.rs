#![feature(allocator_api)]

pub struct HeapArena {
    val: std::ptr::NonNull<u8>,
    handle: std::cell::UnsafeCell<*mut u8>,
    end: *mut u8,
    layout: std::alloc::Layout,
}

impl HeapArena {
    #[inline(always)]
    pub fn new(size: usize) -> Self {
        unsafe {
            let layout =
                std::alloc::Layout::from_size_align_unchecked(size, std::mem::align_of::<usize>());

            let start = std::alloc::alloc(layout);
            if start.is_null() {
                std::alloc::handle_alloc_error(layout);
            }

            Self {
                val: std::ptr::NonNull::new_unchecked(start),
                handle: start.into(),
                end: start.add(size),
                layout: layout,
            }
        }
    }

    #[inline(always)]
    fn inner_alloc<T>(&self, val: T) -> *mut T {
        unsafe {
            let handle = self.handle.get();
            let align = std::mem::align_of::<T>();
            //let size = std::mem::size_of::<T>();

            let handle_addr = (*handle) as usize;
            let aligned_addr = (handle_addr + (align - 1)) & !(align - 1);
            let ptr = aligned_addr as *mut T;
            let next = ptr.add(1) as *mut u8;
            debug_assert!(next <= self.end, "arena is out of memory");

            ptr.write(val);
            *handle = next;
            ptr
        }
    }

    #[inline(always)]
    pub fn alloc<'lifetime_arena, T>(&'lifetime_arena self, val: T) -> &'lifetime_arena T {
        let ptr = self.inner_alloc(val);
        unsafe { &*ptr }
    }

    #[inline(always)]
    pub fn alloc_mut<'lifetime_arena, T>(&'lifetime_arena self, val: T) -> &'lifetime_arena mut T {
        let ptr = self.inner_alloc(val);
        unsafe { &mut *ptr }
    }

    #[inline(always)]
    pub fn reset(&self) {
        let handle = self.handle.get();
        unsafe {
            *handle = self.val.as_ptr();
        }
    }
}

impl Drop for HeapArena {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(self.val.as_ptr(), self.layout);
        }
    }
}

unsafe impl std::alloc::Allocator for &HeapArena {
    #[inline(always)]
    fn allocate(
        &self,
        layout: std::alloc::Layout,
    ) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        unsafe {
            let handle = self.handle.get();
            let size = layout.size();
            let align = layout.align();

            let handle_addr = (*handle) as usize;
            let aligned_addr = (handle_addr + (align - 1)) & !(align - 1);
            let ptr = aligned_addr as *mut u8;
            let next = ptr.add(size);
            debug_assert!(next <= self.end, "arena is out of memory");
            *handle = next;

            let offset = aligned_addr - self.val.as_ptr() as usize;
            let ptr = self.val.as_ptr().add(offset);

            return Ok(std::ptr::NonNull::new_unchecked(
                std::slice::from_raw_parts_mut(ptr, size),
            ));
        }
    }

    #[inline(always)]
    unsafe fn deallocate(&self, _ptr: std::ptr::NonNull<u8>, _layout: std::alloc::Layout) {}
}
