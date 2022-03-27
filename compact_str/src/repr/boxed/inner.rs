const UNKNOWN: usize = 0;
pub type StrBuffer = [u8; UNKNOWN];

#[cfg(not(target_pointer_width = "64"))]
pub mod heap_capacity {
    use core::ptr;
    use std::alloc;

    use super::StrBuffer;

    pub fn alloc(capacity: usize) -> ptr::NonNull<u8> {
        let layout = layout(capacity);
        debug_assert!(layout.size() > 0);

        // SAFETY: `alloc(...)` has undefined behavior if the layout is zero-sized. We know the
        // layout can't be zero-sized though because we're always at least allocating one `usize`
        let raw_ptr = unsafe { alloc::alloc(layout) };

        // Check to make sure our pointer is non-null, some allocators return null pointers instead
        // of panicking
        match ptr::NonNull::new(raw_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout),
        }
    }

    pub unsafe fn dealloc(ptr: ptr::NonNull<u8>, capacity: usize) {
        let layout = layout(capacity);

        // SAFETY: TODO
        alloc::dealloc(ptr.as_ptr(), layout);
    }

    #[repr(C)]
    struct BoxStringInnerHeapCapacity {
        capacity: usize,
        buffer: StrBuffer,
    }

    fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).expect("valid capacity");
        alloc::Layout::new::<BoxStringInnerHeapCapacity>()
            .extend(buffer_layout)
            .expect("valid layout")
            .0
            .pad_to_align()
    }
}

pub mod inline_capacity {
    use core::ptr;
    use std::alloc;

    use super::StrBuffer;

    /// # Safety
    /// * `capacity` must be > 0
    pub unsafe fn alloc(capacity: usize) -> ptr::NonNull<u8> {
        let layout = layout(capacity);
        debug_assert!(layout.size() > 0);

        // SAFETY: `alloc(...)` has undefined behavior if the layout is zero-sized. We specify that
        // `capacity` must be > 0 as a constraint to uphold the safety of this method. If capacity
        // is greater than 0, then our layout will be non-zero-sized.
        let raw_ptr = alloc::alloc(layout);

        // Check to make sure our pointer is non-null, some allocators return null pointers instead
        // of panicking
        match ptr::NonNull::new(raw_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout),
        }
    }

    pub unsafe fn dealloc(ptr: ptr::NonNull<u8>, capacity: usize) {
        let layout = layout(capacity);

        // SAFETY: TODO
        alloc::dealloc(ptr.as_ptr(), layout);
    }

    #[repr(C)]
    struct BoxStringInnerInlineCapacity {
        buffer: StrBuffer,
    }

    fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).expect("valid capacity");
        alloc::Layout::new::<BoxStringInnerInlineCapacity>()
            .extend(buffer_layout)
            .expect("valid layout")
            .0
            .pad_to_align()
    }
}
