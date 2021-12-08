use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};

use std::{
    alloc,
    fmt,
    mem,
    ptr,
    slice,
    str,
};

/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// Going above this limit will abort your program (although not
/// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
const MAX_REFCOUNT: usize = (isize::MAX) as usize;

#[repr(C)]
pub struct ArcStr {
    len: usize,
    ptr: ptr::NonNull<ArcStrInner>,
}

impl ArcStr {
    #[inline]
    pub fn new() -> Self {
        let ptr = ArcStrInner::new();

        ArcStr { len: 0, ptr }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let buffer = self.inner().as_bytes();
        unsafe { str::from_utf8_unchecked(buffer) }
    }

    #[inline]
    fn inner(&self) -> &ArcStrInner {
        unsafe { self.ptr.as_ref() }
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        ArcStrInner::dealloc(self.ptr.as_ptr())
    }
}

impl Clone for ArcStr {
    fn clone(&self) -> Self {
        let old_count = self.inner().ref_count.fetch_add(1, Ordering::Relaxed);

        if old_count > MAX_REFCOUNT {
            panic!("Program has gone wild, ref count > {}", MAX_REFCOUNT);
        }

        ArcStr {
            len: self.len,
            ptr: self.ptr,
        }
    }
}

impl Drop for ArcStr {
    fn drop(&mut self) {
        if self.inner().ref_count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        std::sync::atomic::fence(Ordering::Acquire);
        unsafe { self.drop_inner() }
    }
}

impl fmt::Debug for ArcStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl From<&str> for ArcStr {
    fn from(text: &str) -> Self {
        let len = text.len();
        let mut ptr = ArcStrInner::with_capacity(len);

        unsafe {
            ptr.as_mut()
                .str_buffer
                .as_mut_ptr()
                .copy_from(text.as_ptr(), len);
        };

        ArcStr { len, ptr }
    }
}

const UNKNOWN: usize = 0;
pub type StrBuffer = [u8; UNKNOWN];

#[repr(C)]
pub struct ArcStrInner {
    ref_count: AtomicUsize,
    capacity: usize,
    pub str_buffer: StrBuffer,
}

impl ArcStrInner {
    pub fn new() -> ptr::NonNull<ArcStrInner> {
        Self::with_capacity(0)
    }

    pub fn with_capacity(capacity: usize) -> ptr::NonNull<ArcStrInner> {
        let ptr = unsafe { Self::alloc(capacity) };

        unsafe { (*ptr).ref_count = AtomicUsize::new(1) };
        unsafe { (*ptr).capacity = capacity };

        unsafe { ptr::NonNull::new_unchecked(ptr) }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.str_buffer.as_ptr(), self.capacity) }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    unsafe fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).unwrap();
        alloc::Layout::new::<Self>()
            .extend(buffer_layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub unsafe fn alloc(capacity: usize) -> *mut ArcStrInner {
        let layout = Self::layout(capacity);
        let ptr = alloc::alloc(layout) as *mut ArcStrInner;
        if ptr.is_null() {
            alloc::handle_alloc_error(layout);
        }

        ptr
    }

    pub unsafe fn dealloc(ptr: *mut ArcStrInner) {
        let capacity = (*ptr).capacity;
        let layout = Self::layout(capacity);

        alloc::dealloc(ptr as *mut u8, layout);
    }
}

#[cfg(test)]
mod test {
    use super::ArcStr;

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let arc_str = ArcStr::from(example);

        println!("{}", arc_str.len());

        assert_eq!(arc_str.as_str(), example);
        assert_eq!(arc_str.len(), example.len());
    }
}

static_assertions::const_assert_eq!(mem::size_of::<ArcStr>(), 16);
static_assertions::const_assert_eq!(mem::size_of::<ArcStrInner>(), 16);
