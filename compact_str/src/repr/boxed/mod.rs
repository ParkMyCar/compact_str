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

    /// Reserve space for at least `additional` bytes
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        // We need at least this much space
        let new_capacity = self.len() + additional;

        // We have enough space, so there is no work to do
        if self.capacity() >= new_capacity {
            return;
        }

        // Create a new `BoxString` with enough space for at least `additional` bytes, dropping the
        // old one
        *self = BoxString::new(self.as_str(), additional);
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
    pub fn push(&mut self, ch: char) {
        let len = self.len();
        let char_len = ch.len_utf8();

        // Reserve at least enough space for the new char
        self.reserve(char_len);

        // SAFETY: We're writing a char into the slice, which is valid UTF-8
        let slice = unsafe { self.as_mut_slice() };

        // Write our char into the slice
        ch.encode_utf8(&mut slice[len..]);

        // Increment our length
        //
        // SAFETY: We just wrote `char_len` bytes into the buffer, so we know this new length is
        // valid
        unsafe { self.set_len(len + char_len) };
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        let len = self.len();
        let str_len = s.len();

        // Reserve at least enough space for the new str
        self.reserve(str_len);

        // SAFETY: We're writing a &str into the slice, which is valid UTF-8
        let slice = unsafe { self.as_mut_slice() };
        let buffer = &mut slice[len..len + str_len];

        debug_assert_eq!(buffer.len(), s.as_bytes().len());

        // Copy the string into our buffer
        buffer.copy_from_slice(s.as_bytes());
        // Incrament the length of our string
        unsafe { self.set_len(len + str_len) };
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

    /// Returns a mutable reference to the underlying buffer of bytes
    ///
    /// # Safety:
    /// * The caller must guarantee any modifications made to the buffer are valid UTF-8
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.ptr.as_mut().as_mut_bytes()
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

    #[test]
    fn test_push() {
        let example = "hello";
        let mut boxed = BoxString::from(example);

        boxed.push(' ');
        boxed.push('w');
        assert_eq!(boxed.as_str(), "hello w");
        assert_eq!(boxed.capacity(), 7);

        // Right now our len and cap are both 7, pushing 'o' should cause us to resize
        boxed.push('o');
        assert_eq!(boxed.len(), 8);
        assert_eq!(boxed.capacity(), 10);

        boxed.push('r');
        boxed.push('l');
        boxed.push('d');
        assert_eq!(boxed.len(), 11);
        assert_eq!(boxed.capacity(), 15);

        assert_eq!(boxed.as_str(), "hello world");
    }

    #[test]
    fn test_push_str() {
        let example = "hello";
        let mut boxed = BoxString::from(example);

        boxed.push_str(" world!");
        assert_eq!(boxed.as_str(), "hello world!");
        assert_eq!(boxed.len(), 12);
        assert_eq!(boxed.capacity(), 12);
    }
}

static_assertions::const_assert_eq!(mem::size_of::<BoxString>(), 2 * mem::size_of::<usize>());
