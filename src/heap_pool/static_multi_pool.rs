use crate::Error;

pub struct Block {
    next: Option<std::ptr::NonNull<Block>>
}

pub struct StaticMultiPoolAlloc<const N: usize> {
    start: [std::ptr::NonNull<u8>; N],
    head: [std::cell::UnsafeCell<Option<std::ptr::NonNull<Block>>>; N],
    block_size: [usize; N],
    aligns: [usize; N],
    layout: [std::alloc::Layout; N]
}

fn align_up(size: usize, align: usize) -> usize {
    (size + align - 1) & !(align - 1)
}

impl<const N: usize> StaticMultiPoolAlloc<N> {
    pub fn new(num_blocks: [usize; N], block_size: [usize; N], align_multiply: [usize; N]) -> Result<Self, Error> {
        if align_multiply.iter().any(|val| *val == 0) {
          return Err(Error::InvalidAlignment("Align multiply must be > 0"));
        }
        let block_align = std::mem::align_of::<Block>();
        let aligns: [usize; N] = align_multiply.map(|val| val * block_align);
        
        let base_block_size = std::mem::size_of::<Block>();
        let block_size: [usize; N] = std::array::from_fn(|i| {
            let size = block_size[i].max(base_block_size);
            align_up(size, block_align) 
        });
        
        let layout: [std::alloc::Layout; N] = std::array::try_from_fn(|i| std::alloc::Layout::from_size_align(
            num_blocks[i].checked_mul(block_size[i])
                .ok_or(Error::IntegerOverflow("Integer overflow, each num block * block size must fit within usize, because it will make the layout incorrect due to wrap around"))?,
            aligns[i]
        ).map_err(|err| Error::LayoutError(err) )
        )?;
        
        // SAFETY: the layout is valid, because if not it will trigger panic
        let starts: [std::ptr::NonNull<u8>; N] = std::array::try_from_fn(|i| {
        
        let start = unsafe { std::alloc::alloc(layout[i]) };
        let start_ptr = std::ptr::NonNull::new(start).ok_or(Error::OutOfMemory)?;
        
        unsafe {
            for j in 0..num_blocks[i] {
                let current_ptr = start.add(j * block_size[i]).cast::<Block>();
                let next_ptr = if j < num_blocks[i] - 1 {
                    Some(std::ptr::NonNull::new_unchecked(start.add((j + 1) * block_size[i]).cast::<Block>()))
                } else {
                    None
                };
                
                (*current_ptr).next = next_ptr;
            }
        }
        
        Ok(start_ptr)
        
        })?;
        
        let head: [std::cell::UnsafeCell<Option<std::ptr::NonNull<Block>>>; N] = std::array::from_fn(|i| {
            std::cell::UnsafeCell::new(Some(std::ptr::NonNull::new(starts[i].as_ptr().cast::<Block>()).unwrap()))
        });
        
        Ok(Self {
            start: starts,
            head,
            block_size,
            aligns,
            layout,
        })
        
    }
}

impl<const N: usize> Drop for StaticMultiPoolAlloc<N> {
    fn drop(&mut self) {
        // SAFETY: deallocate using the pointer. The layout is saved, so it is the correct layout
        for (i, val) in self.start.iter().enumerate() {
            unsafe {
                std::alloc::dealloc(val.as_ptr(), self.layout[i]);
            }
        }
    }
}

unsafe impl<const N: usize> std::alloc::Allocator for StaticMultiPoolAlloc<N> {
    #[inline(always)]
    fn allocate(&self, layout: std::alloc::Layout) -> Result<std::ptr::NonNull<[u8]>, std::alloc::AllocError> {
        let size = layout.size();
        let align = layout.align();
        //let base_align = std::mem::align_of::<Block>();
        
        let index = self.aligns.iter().enumerate().position(|(i, &val)| {
            size <= self.block_size[i] && align <= val
        });

        let i = index.ok_or(std::alloc::AllocError)?;
        
        let head_ptr = unsafe { &mut *self.head[i].get() };
        if let Some(block) = *head_ptr {
            let taken_block = block;
            unsafe { *head_ptr = block.as_ref().next };
            let ptr = taken_block.cast::<u8>();
            return Ok(std::ptr::NonNull::slice_from_raw_parts(
                ptr, self.block_size[i]
            ));
        }
        
        Err(std::alloc::AllocError)
    }
    
    #[inline(always)]
    unsafe fn deallocate(&self, ptr: std::ptr::NonNull<u8>, layout: std::alloc::Layout) {
        let new_block = ptr.cast::<Block>();
        let align = layout.align();
        
        let index = self.aligns.iter().enumerate().position(|(_, &val)| {
            align <= val
        });

        let i = index.ok_or(std::alloc::AllocError).unwrap();
        
        // dereference raw pointer is unsafe
        let head_ptr = unsafe { &mut *self.head[i].get() };
        unsafe { (*new_block.as_ptr()).next = *head_ptr };
        
        *head_ptr = Some(new_block);
    }
}
