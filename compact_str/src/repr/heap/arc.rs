use std::sync::atomic::{
    AtomicUsize,
    Ordering,
};
use std::sync::Arc;

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
pub struct ArcString {
    len: usize,
    ptr: ptr::NonNull<ArcStringInner>,
}

impl ArcString {
    #[inline]
    pub fn new(text: &str, additional: usize) -> Self {
        let len = text.len();
        let capacity = len + additional;
        let mut ptr = ArcStringInner::with_capacity(capacity);

        // SAFETY: We just created the `ArcStringInner` so we know the pointer is properly aligned,
        // it is non-null, points to an instance of `ArcStringInner`, and the `str_buffer`
        // is valid
        let buffer_ptr = unsafe { ptr.as_mut().str_buffer.as_mut_ptr() };
        // SAFETY: We know both `src` and `dest` are valid for respectively reads and writes of
        // length `len` because `len` comes from `src`, and `dest` was allocated to be at least that
        // length. We also know they're non-overlapping because `dest` is newly allocated
        unsafe { buffer_ptr.copy_from_nonoverlapping(text.as_ptr(), len) };

        ArcString { len, ptr }
    }

    #[inline]
    pub unsafe fn make_mut_slice(&mut self) -> &mut [u8] {
        if self.inner().ref_count.compare_exchange(1, 0, Ordering::Acquire, Ordering::Relaxed).is_err() {
            // There is more than one reference to this underlying buffer, so we need to make a new
            // instance and decrement the count of the original by one

            // Make a new instance with the same capacity as self
            let additional = self.capacity() - self.len();
            let new = Self::new(self.as_str(), additional);

            // Assign self to our new instsance
            *self = new;

            // self.inner().
        } else {

        }
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner().capacity
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let buffer = self.inner().as_bytes();

        // SAFETY: The only way you can construct an `ArcString` is via a `&str` so it must be valid
        // UTF-8, or the caller has manually made those guarantees
        unsafe { str::from_utf8_unchecked(&buffer[..self.len]) }
    }

    #[inline]
    fn inner(&self) -> &ArcStringInner {
        // SAFETY: If we still have an instance of `ArcString` then we know the pointer to
        // `ArcStringInner` is valid for at least as long as the provided ref to `self`
        unsafe { self.ptr.as_ref() }
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        ArcStringInner::dealloc(self.ptr)
    }
}

impl Clone for ArcString {
    fn clone(&self) -> Self {
        let old_count = self.inner().ref_count.fetch_add(1, Ordering::Relaxed);
        assert!(
            old_count < MAX_REFCOUNT,
            "Program has gone wild, ref count > {}",
            MAX_REFCOUNT
        );

        ArcString {
            len: self.len,
            ptr: self.ptr,
        }
    }
}

impl Drop for ArcString {
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

impl fmt::Debug for ArcString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl From<&str> for ArcString {
    fn from(text: &str) -> Self {
        ArcString::new(text, 0)
    }
}

const UNKNOWN: usize = 0;
pub type StrBuffer = [u8; UNKNOWN];

#[repr(C)]
pub struct ArcStringInner {
    pub ref_count: AtomicUsize,
    capacity: usize,
    pub str_buffer: StrBuffer,
}

impl ArcStringInner {
    pub fn with_capacity(capacity: usize) -> ptr::NonNull<ArcStringInner> {
        let mut ptr = Self::alloc(capacity);

        // SAFETY: We just allocated an instance of `ArcStringInner` and checked to make sure it
        // wasn't null, so we know it's aligned properly, that it points to an instance of
        // `ArcStringInner` and that the "lifetime" is valid
        unsafe { ptr.as_mut().ref_count = AtomicUsize::new(1) };
        // SAFTEY: Same as above
        unsafe { ptr.as_mut().capacity = capacity };

        ptr
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: Since we have an instance of `ArcStringInner` so we know the buffer is still
        // valid, and we track the capacity with the creation and adjustment of the buffer
        unsafe { slice::from_raw_parts(self.str_buffer.as_ptr(), self.capacity) }
    }

    #[inline]
    pub unsafe fn as_mut_bytes(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.str_buffer.as_mut_ptr(), self.capacity)
    }

    fn layout(capacity: usize) -> alloc::Layout {
        let buffer_layout = alloc::Layout::array::<u8>(capacity).unwrap();
        alloc::Layout::new::<Self>()
            .extend(buffer_layout)
            .unwrap()
            .0
            .pad_to_align()
    }

    pub fn alloc(capacity: usize) -> ptr::NonNull<ArcStringInner> {
        let layout = Self::layout(capacity);
        debug_assert!(layout.size() > 0);

        // SAFETY: `alloc(...)` has undefined behavior if the layout is zero-sized, but we know the
        // size of the layout is greater than 0 because we define it (and check for it above)
        let raw_ptr = unsafe { alloc::alloc(layout) as *mut ArcStringInner };

        // Check to make sure our pointer is non-null, some allocators return null pointers instead
        // of panicking
        match ptr::NonNull::new(raw_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(layout),
        }
    }

    pub fn dealloc(ptr: ptr::NonNull<ArcStringInner>) {
        // SAFETY: We know the pointer is non-null and it is properly aligned
        let capacity = unsafe { ptr.as_ref().capacity };
        let layout = Self::layout(capacity);

        // SAFETY: There is only one way to allocate an ArcStringInner, and it uses the same layout
        // we defined above. Also we know the pointer is non-null and we use the same global
        // allocator as we did in `Self::alloc(...)`
        unsafe { alloc::dealloc(ptr.as_ptr() as *mut u8, layout) };
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;
    use proptest::strategy::Strategy;

    use super::ArcString;

    #[test]
    fn test_empty() {
        let empty = "";
        let arc_str = ArcString::from(empty);

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
        let arc_str = ArcString::from(long);

        assert_eq!(arc_str.as_str(), long);
        assert_eq!(arc_str.len, long.len());
    }

    #[test]
    fn test_clone_and_drop() {
        let example = "hello world!";
        let arc_str_1 = ArcString::from(example);
        let arc_str_2 = arc_str_1.clone();

        drop(arc_str_1);

        assert_eq!(arc_str_2.as_str(), example);
        assert_eq!(arc_str_2.len, example.len());
    }

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let arc_str = ArcString::from(example);

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
        #[cfg_attr(miri, ignore)]
        fn test_strings_roundtrip(word in rand_unicode()) {
            let arc_str = ArcString::from(word.as_str());
            prop_assert_eq!(&word, arc_str.as_str());
        }
    }
}

static_assertions::const_assert_eq!(mem::size_of::<ArcString>(), 2 * mem::size_of::<usize>());
// Note: Although the compiler sees `ArcStringInner` as being 16 bytes, it's technically unsized
// because it contains a buffer of size `capacity`. We manually track the size of this buffer so
// `ArcString` can only be two words long
static_assertions::const_assert_eq!(
    mem::size_of::<ArcStringInner>(),
    2 * mem::size_of::<usize>()
);
