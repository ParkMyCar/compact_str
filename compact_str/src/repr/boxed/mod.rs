use core::{
    fmt,
    mem,
    ptr,
    str,
};

mod inner;
use inner::BoxStringInner;

#[repr(C)]
pub struct BoxString {
    len: usize,
    ptr: ptr::NonNull<BoxStringInner>,
}
unsafe impl Sync for BoxString {}
unsafe impl Send for BoxString {}

impl BoxString {
    #[inline]
    pub fn new(text: &str, additional: usize) -> Self {
        let len = text.len();

        let required = len + additional;
        let amortized = 3 * len / 2;
        let new_capacity = core::cmp::max(amortized, required);

        // TODO: Handle overflows in the case of __very__ large Strings
        debug_assert!(new_capacity >= len);

        let mut ptr = BoxStringInner::with_capacity(new_capacity);

        // SAFETY: We just created the `BoxStringInner` so we know the pointer is properly aligned,
        // it is non-null, points to an instance of `BoxStringInner`, and the `str_buffer`
        // is valid
        let buffer_ptr = unsafe { ptr.as_mut().buffer.as_mut_ptr() };
        // SAFETY: We know both `src` and `dest` are valid for respectively reads and writes of
        // length `len` because `len` comes from `src`, and `dest` was allocated to be at least that
        // length. We also know they're non-overlapping because `dest` is newly allocated
        unsafe { buffer_ptr.copy_from_nonoverlapping(text.as_ptr(), len) };

        BoxString { len, ptr }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        // We should never be able to programatically create an `BoxString` with a capacity less
        // than our max inline size, since then the string should be inlined
        debug_assert!(capacity >= super::MAX_SIZE);

        let len = 0;
        let ptr = BoxStringInner::with_capacity(capacity);

        BoxString { len, ptr }
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
        // SAFETY: The only way you can construct an `BoxString` is via a `&str` so it must be valid
        // UTF-8, or the caller has manually made those guarantees
        unsafe { str::from_utf8_unchecked(self.as_slice()) }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[u8] {
        &self.inner().as_bytes()[..self.len]
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.len = length;
    }

    /// Returns a shared reference to the heap allocated `BoxStringInner`
    #[inline]
    fn inner(&self) -> &BoxStringInner {
        // SAFETY: If we still have an instance of `BoxString` then we know the pointer to
        // `BoxString` is valid for at least as long as the provided ref to `self`
        unsafe { self.ptr.as_ref() }
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        BoxStringInner::dealloc(self.ptr)
    }
}

impl Clone for BoxString {
    fn clone(&self) -> Self {
        Self::new(self.as_str(), self.capacity() - self.len())
    }
}

impl fmt::Debug for BoxString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl From<&str> for BoxString {
    #[inline]
    fn from(text: &str) -> Self {
        BoxString::new(text, 0)
    }
}

impl Drop for BoxString {
    fn drop(&mut self) {
        unsafe { self.drop_inner() }
    }
}

#[cfg(test)]
mod tests {
    use super::BoxString;

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let box_str = BoxString::from(example);

        assert_eq!(box_str.as_str(), example);
        assert_eq!(box_str.len(), example.len());
    }

    #[test]
    fn test_clone_and_drop() {
        let example = "nyc";
        let one = BoxString::from(example);
        let two = one.clone();

        assert_eq!(one.as_str(), example);
        drop(one);
        assert_eq!(two.as_str(), example);

    }
}

static_assertions::const_assert_eq!(mem::size_of::<BoxString>(), 2 * mem::size_of::<usize>());
