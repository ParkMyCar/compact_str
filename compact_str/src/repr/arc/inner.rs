use core::ptr;
use core::sync::atomic::AtomicUsize;
use std::alloc;

const UNKNOWN: usize = 0;
pub type StrBuffer = [u8; UNKNOWN];

/// A `NonNull<ArcStringHeader>`, but it must have provenance for the entire allocation
/// including the string buffer after the header.
///
/// This pointer must always be valid for reads of the header.
#[derive(Clone, Copy)]
pub struct ArcStringPtr {
    ptr: ptr::NonNull<ArcStringHeader>,
}

#[repr(C)]
#[derive(Debug)]
pub struct ArcStringHeader {
    pub ref_count: AtomicUsize,
    pub capacity: usize,
}

impl ArcStringPtr {
    pub fn with_capacity(capacity: usize) -> Self {
        let ptr = Self::alloc(capacity);
        let raw_ptr = ptr.as_ptr();

        // SAFETY: We just allocated an instance of `ArcStringHeader` and checked to make sure it
        // wasn't null, so we know it's aligned properly, that it points to an instance of
        // `ArcStringHeader` and that the "lifetime" is valid
        unsafe {
            ptr::addr_of_mut!((*raw_ptr).ref_count).write(AtomicUsize::new(1));
            ptr::addr_of_mut!((*raw_ptr).capacity).write(capacity);
        }

        // We have initialized the header above, so it is now valid for reads of the header
        Self { ptr }
    }

    fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).unwrap();
        alloc::Layout::new::<ArcStringHeader>()
            .extend(buffer_layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub fn alloc(capacity: usize) -> ptr::NonNull<ArcStringHeader> {
        let layout = Self::layout(capacity);
        debug_assert!(layout.size() > 0);

        // SAFETY: `alloc(...)` has undefined behavior if the layout is zero-sized, but we know the
        // size of the layout is greater than 0 because it contains the header
        let raw_ptr = unsafe { alloc::alloc(layout) as *mut ArcStringHeader };

        // `alloc::alloc` returns a null pointer if allocation failed, handle it
        match ptr::NonNull::new(raw_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout),
        }
    }

    /// Deallocate the entire header and buffer.
    /// # Safety
    /// The pointer must not be used anymore after calling this.
    pub unsafe fn dealloc(&mut self) {
        // SAFETY: We know the pointer is non-null and it is properly aligned
        let capacity = self.ptr.as_ref().capacity;
        let layout = Self::layout(capacity);

        // SAFETY: `self.ptr` must always be valid, and it uses the same layout
        // we defined above. Also we know the pointer is non-null and we use the same global
        // allocator as we did in `Self::alloc(...)`
        alloc::dealloc(self.ptr.as_ptr() as *mut u8, layout);
    }

    pub fn str_buf_ptr(&self) -> *const u8 {
        // SAFETY: `self.ptr` must always be valid
        // We offset it by the size of the header, so this must always be in bounds or one past the
        // end
        unsafe { self.ptr.as_ptr().add(1).cast::<u8>() as *const u8 }
    }

    pub fn str_buf_ptr_mut(&mut self) -> *mut u8 {
        // SAFETY: `self.ptr` must always be valid
        // We offset it by the size of the header, so this must always be in bounds or one past the
        // end
        unsafe { self.ptr.as_ptr().add(1).cast::<u8>() }
    }

    pub fn header(&self) -> &ArcStringHeader {
        // SAFETY: `self.ptr` must always be valid
        unsafe { self.ptr.as_ref() }
    }
}
