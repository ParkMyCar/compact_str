use core::{
    ptr,
    slice,
};
use std::alloc;

const UNKNOWN: usize = 0;
pub type StrBuffer = [u8; UNKNOWN];

#[repr(C)]
pub struct BoxStringInner {
    pub capacity: usize,
    pub buffer: StrBuffer,
}

impl BoxStringInner {
    #[inline]
    pub fn with_capacity(capacity: usize) -> ptr::NonNull<BoxStringInner> {
        let mut ptr = Self::alloc(capacity);

        // SAFETY: We just allocated an instance of `BoxStringInner` and checked to make sure it
        // wasn't null, so we know it's aligned properly, that it points to an instance of
        // `BoxStringInner` and that the "lifetime" is valid
        unsafe { ptr.as_mut().capacity = capacity };

        ptr
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: Since we have an instance of `BoxStringInner` so we know the buffer is still
        // valid, and we track the capacity with the creation and adjustment of the buffer
        unsafe { slice::from_raw_parts(self.buffer.as_ptr(), self.capacity) }
    }

    /// Returns a mutable reference to the underlying buffer of bytes
    ///
    /// # Safety
    /// * The caller must check that any modifications made to the underlying buffer are valid UTF-8
    #[inline]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        // SAFETY: Since we have an instance of `BoxStringInner` so we know the buffer is still
        // valid, and we track the capacity with the creation and adjustment of the buffer
        slice::from_raw_parts_mut(self.buffer.as_mut_ptr(), self.capacity)
    }

    pub fn alloc(capacity: usize) -> ptr::NonNull<BoxStringInner> {
        let layout = Self::layout(capacity);
        debug_assert!(layout.size() > 0);

        // SAFETY: `alloc(...)` has undefined behavior if the layout is zero-sized, but we know the
        // size of the layout is greater than 0 because we define it (and check for it above)
        let raw_ptr = unsafe { alloc::alloc(layout) as *mut BoxStringInner };

        // Check to make sure our pointer is non-null, some allocators return null pointers instead
        // of panicking
        match ptr::NonNull::new(raw_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout),
        }
    }

    pub fn dealloc(ptr: ptr::NonNull<BoxStringInner>) {
        // SAFETY: We know the pointer is non-null and it is properly aligned
        let capacity = unsafe { ptr.as_ref().capacity };
        let layout = Self::layout(capacity);

        // SAFETY: There is only one way to allocate a BoxStringInner, and it uses the same layout
        // we defined above. Also we know the pointer is non-null and we use the same global
        // allocator as we did in `Self::alloc(...)`
        unsafe { alloc::dealloc(ptr.as_ptr() as *mut u8, layout) };
    }

    fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).expect("valid capacity");
        alloc::Layout::new::<Self>()
            .extend(buffer_layout)
            .expect("valid layout")
            .0
            .pad_to_align()
    }
}
