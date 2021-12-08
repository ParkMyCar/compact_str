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
    pub fn as_str(&self) -> &str {
        let buffer = self.inner().as_bytes();

        // SAFETY: The only way you can construct an `ArcStr` is via a `&str` so it must be valid
        // UTF-8, or the caller has manually made those guarantees
        unsafe { str::from_utf8_unchecked(buffer) }
    }

    #[inline]
    fn inner(&self) -> &ArcStrInner {
        // SAFETY: If we still have an instance of `ArcStr` then we know the pointer to
        // `ArcStrInner` is valid for at least as long as the provided ref to `self`
        unsafe { self.ptr.as_ref() }
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        ArcStrInner::dealloc(self.ptr)
    }
}

impl Clone for ArcStr {
    fn clone(&self) -> Self {
        let old_count = self.inner().ref_count.fetch_add(1, Ordering::Relaxed);
        assert!(old_count < MAX_REFCOUNT, "Program has gone wild, ref count > {}", MAX_REFCOUNT);

        ArcStr {
            len: self.len,
            ptr: self.ptr,
        }
    }
}

impl Drop for ArcStr {
    fn drop(&mut self) {
        // This was copied from the implementation of `std::sync::Arc`
        // TODO: Better document the safety invariants here
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

        // SAFETY: We just created the `ArcStrInner` so we know the pointer is properly aligned, it
        // is non-null, points to an instance of `ArcStrInner`, and the `str_buffer` is valid
        let buffer_ptr = unsafe { ptr.as_mut().str_buffer.as_mut_ptr() };
        // SAFETY: We know both `src` and `dest` are valid for respectively reads and writes of
        // length `len` because `len` comes from `src`, and `dest` was allocated to be that
        // length. We also know they're non-overlapping because `dest` is newly allocated
        unsafe { buffer_ptr.copy_from_nonoverlapping(text.as_ptr(), len) };

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
    pub fn with_capacity(capacity: usize) -> ptr::NonNull<ArcStrInner> {
        let mut ptr = Self::alloc(capacity);

        // SAFETY: We just allocated an instance of `ArcStrInner` and checked to make sure it wasn't
        // null, so we know it's aligned properly, that it points to an instance of `ArcStrInner`
        // and that the "lifetime" is valid
        unsafe { ptr.as_mut().ref_count = AtomicUsize::new(1) };
        // SAFTEY: Same as above
        unsafe { ptr.as_mut().capacity = capacity };

        ptr
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: Since we have an instance of `ArcStrInner` so we know the buffer is still valid,
        // and we track the capacity with the creation and adjustment of the buffer
        unsafe { slice::from_raw_parts(self.str_buffer.as_ptr(), self.capacity) }
    }

    fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).unwrap();
        alloc::Layout::new::<Self>()
            .extend(buffer_layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub fn alloc(capacity: usize) -> ptr::NonNull<ArcStrInner> {
        let layout = Self::layout(capacity);
        debug_assert!(layout.size() > 0);

        // SAFETY: `alloc(...)` has undefined behavior if the layout is zero-sized, but we know the
        // size of the layout is greater than 0 because we define it (and check for it above)
        let raw_ptr = unsafe { alloc::alloc(layout) as *mut ArcStrInner };

        // Check to make sure our pointer is non-null, some allocators return null pointers instead
        // of panicking
        match ptr::NonNull::new(raw_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout),
        }
    }

    pub fn dealloc(ptr: ptr::NonNull<ArcStrInner>) {
        // SAFETY: We know the pointer is non-null and it is properly aligned
        let capacity = unsafe { ptr.as_ref().capacity };
        let layout = Self::layout(capacity);

        // SAFETY: There is only one way to allocate an ArcStrInner, and it uses the same layout
        // we defined above. Also we know the pointer is non-null and we use the same global
        // allocator as we did in `Self::alloc(...)`
        unsafe { alloc::dealloc(ptr.as_ptr() as *mut u8, layout) };
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use proptest::strategy::Strategy;

    use super::ArcStr;

    #[test]
    fn test_empty() {
        let empty = "";
        let arc_str = ArcStr::from(empty);

        assert_eq!(arc_str.as_str(), empty);
        assert_eq!(arc_str.len, empty.len());
    }

    #[test]
    fn test_long() {
        let long = "aaabbbcccdddeeefff\n
                    ggghhhiiijjjkkklll\n
                    mmmnnnooopppqqqrrr\n
                    ssstttuuuvvvwwwxxx\n
                    yyyzzz000111222333\n
                    444555666777888999000";
        let arc_str = ArcStr::from(long);

        assert_eq!(arc_str.as_str(), long);
        assert_eq!(arc_str.len, long.len());
    }

    #[test]
    fn test_clone_and_drop() {
        let example = "hello world!";
        let arc_str_1 = ArcStr::from(example);
        let arc_str_2 = arc_str_1.clone();

        drop(arc_str_1);

        assert_eq!(arc_str_2.as_str(), example);
        assert_eq!(arc_str_2.len, example.len());
    }

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let arc_str = ArcStr::from(example);

        assert_eq!(arc_str.as_str(), example);
        assert_eq!(arc_str.len, example.len());
    }

    // generates random unicode strings, upto 80 chars long
    fn rand_unicode() -> impl Strategy<Value = String> {
        proptest::collection::vec(proptest::char::any(), 0..80)
            .prop_map(|v| v.into_iter().collect())
    }

    proptest! {
        #[test]
        fn test_strings_roundtrip(word in rand_unicode()) {
            let arc_str = ArcStr::from(word.as_str());
            prop_assert_eq!(&word, arc_str.as_str());
        }
    }
}

static_assertions::const_assert_eq!(mem::size_of::<ArcStr>(), 16);
static_assertions::const_assert_eq!(mem::size_of::<ArcStrInner>(), 16);
