use core::iter::Extend;
use core::mem::ManuallyDrop;
use core::{
    fmt,
    ptr,
    slice,
    str,
};

mod capacity;
use capacity::Capacity;

mod inner;

const MIN_SIZE: usize = core::mem::size_of::<usize>() / 2;

#[repr(C)]
pub struct BoxString {
    ptr: ptr::NonNull<u8>,
    len: usize,
    cap: Capacity,
}
unsafe impl Sync for BoxString {}
unsafe impl Send for BoxString {}

impl BoxString {
    #[inline]
    pub fn new(text: &str) -> Self {
        let len = text.len();

        // Always allocate at least a few bytes
        //
        // Note: practically we should never try to create an empty `BoxString`, since we inline
        // short strings
        let capacity = core::cmp::max(len, MIN_SIZE);

        // SAFETY: `Self::alloc_ptr(...)` requires that capacity is non-zero. Above we set capacity
        // to be at least size_of::<usize>, so we know it'll be non-zero.
        let (cap, ptr) = unsafe { BoxString::alloc_ptr(capacity) };

        // SAFETY: We know both `src` and `dest` are valid for respectively reads and writes of
        // length `len` because `len` comes from `src`, and `dest` was allocated to be at least that
        // length. We also know they're non-overlapping because `dest` is newly allocated
        #[cfg(target_pointer_width = "64")]
        unsafe {
            ptr.as_ptr().copy_from_nonoverlapping(text.as_ptr(), len)
        };

        #[cfg(not(target_pointer_width = "64"))]
        {
            let write_ptr = if cap.is_heap() {
                unsafe { ptr.as_ptr().add(core::mem::size_of::<usize>()) }
            } else {
                ptr.as_ptr()
            };
            unsafe { write_ptr.copy_from_nonoverlapping(text.as_ptr(), len) };
        }

        BoxString { len, ptr, cap }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let len = 0;

        // Always allocate at least a few bytes
        //
        // Note: practically we should never try to create an empty `BoxString`, since we inline
        // short strings
        let capacity = core::cmp::max(capacity, MIN_SIZE);

        // SAFETY: `Self::alloc_ptr(...)` requires that capacity is non-zero. Above we set capacity
        // to be at least size_of::<usize>, so we know it'll be non-zero.
        let (cap, ptr) = unsafe { BoxString::alloc_ptr(capacity) };

        BoxString { len, ptr, cap }
    }

    #[inline(always)]
    unsafe fn alloc_ptr(capacity: usize) -> (Capacity, ptr::NonNull<u8>) {
        #[cfg(target_pointer_width = "64")]
        let (cap, ptr) = {
            debug_assert!(capacity <= capacity::MAX_VALUE);

            let cap = Capacity::new_unchecked(capacity);
            let ptr = inner::inline_capacity::alloc(capacity);
            (cap, ptr)
        };

        #[cfg(not(target_pointer_width = "64"))]
        let (cap, ptr) = match Capacity::new(capacity) {
            Ok(cap) => {
                let ptr = inner::inline_capacity::alloc(capacity);
                (cap, ptr)
            }
            Err(cap) => {
                let ptr = inner::heap_capacity::alloc(capacity);
                // write our capacity onto the heap
                core::ptr::copy_nonoverlapping(
                    capacity.to_le_bytes().as_ptr(),
                    ptr.as_ptr(),
                    core::mem::size_of::<usize>(),
                );
                (cap, ptr)
            }
        };

        (cap, ptr)
    }

    #[inline]
    pub fn with_additional(text: &str, additional: usize) -> Self {
        let len = text.len();

        let required = len + additional;
        let amortized = 3 * len / 2;
        let new_capacity = core::cmp::max(amortized, required);

        // TODO: Handle overflows in the case of __very__ large Strings
        debug_assert!(new_capacity >= len);

        // Create the `BoxString` with our determined capacity
        let mut new = BoxString::with_capacity(new_capacity);

        // SAFETY: We're writing a &str which is valid UTF-8
        let buffer = unsafe { new.as_mut_slice() };
        buffer[..len].copy_from_slice(text.as_bytes());

        // SAFETY: We just wrote `len` bytes into our buffer
        unsafe { new.set_len(len) };

        new
    }

    #[inline]
    pub fn from_string(s: String) -> Self {
        match Capacity::new(s.capacity()) {
            // Note: We should never hit this case when using BoxString with CompactString
            Ok(_) if s.capacity() == 0 => BoxString::new(""),
            Ok(cap) => {
                // Don't drop `s` to avoid a double free
                let mut s = ManuallyDrop::new(s.into_bytes());
                let len = s.len();
                let raw_ptr = s.as_mut_ptr();

                let ptr = ptr::NonNull::new(raw_ptr).expect("string with capacity has null ptr?");
                // create a new BoxString with our parts!
                BoxString { len, ptr, cap }
            }
            Err(_) => BoxString::new(s.as_str()),
        }
    }

    #[inline]
    pub fn from_box_str(b: Box<str>) -> Self {
        match Capacity::new(b.len()) {
            // Note: We should never hit this case when using BoxString with CompactString
            Ok(_) if b.len() == 0 => BoxString::new(""),
            Ok(cap) => {
                let len = b.len();
                // Don't drop the box here
                let raw_ptr = Box::into_raw(b).cast::<u8>();

                let ptr = ptr::NonNull::new(raw_ptr).expect("string with capacity has null ptr?");
                // create a new BoxString with our parts!
                BoxString { len, ptr, cap }
            }
            Err(_) => BoxString::new(&b),
        }
    }

    /// Reserve space for at least `additional` bytes
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        // We need at least this much space
        let len = self.len();
        let required = len + additional;

        // We have enough space, so there is no work to do
        if self.capacity() >= required {
            return;
        }

        // We need to reserve additional space, so create a new BoxString with additional space
        let new = BoxString::with_additional(self.as_str(), additional);

        // Set our new BoxString as self
        *self = new;
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        #[cfg(target_pointer_width = "64")]
        {
            debug_assert!(self.cap.as_usize().is_ok());
            unsafe { self.cap.as_usize_unchecked() }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            match self.cap.as_usize() {
                // capacity is on the stack, just return it!
                Ok(cap) => cap,
                // capacity is on the heap, we need to read it back
                Err(_) => {
                    let mut usize_buf = [0u8; core::mem::size_of::<usize>()];
                    // copy bytes from the heap into our buffer
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            self.ptr.as_ptr() as *const u8,
                            usize_buf.as_mut_ptr(),
                            core::mem::size_of::<usize>(),
                        );
                    };
                    // interpret those bytes as a usize
                    usize::from_le_bytes(usize_buf)
                }
            }
        }
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
        &self.as_buffer()[..self.len]
    }

    /// Returns a mutable reference to the underlying buffer of bytes
    ///
    /// # Safety:
    /// * The caller must guarantee any modifications made to the buffer are valid UTF-8
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &mut [u8] {
        self.as_mut_buffer()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, length: usize) {
        self.len = length;
    }

    #[inline]
    fn as_buffer(&self) -> &[u8] {
        #[cfg(target_pointer_width = "64")]
        {
            debug_assert!(self.cap.as_usize().is_ok());

            // SAFETY: Since we have an instance of `BoxStringInner` so we know the buffer is still
            // valid. Also since we're on a 64-bit arch, it's practically impossible for our
            // capacity to be stored on the heap
            unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.cap.as_usize_unchecked()) }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            match self.cap.as_usize() {
                // capacity is on the stack, so read the entire buffer!
                Ok(cap) => unsafe { slice::from_raw_parts(self.ptr.as_ptr(), cap) },
                // capacity is on the heap, we need to read the capacity, and then read the buffer
                // starting from an offset
                Err(_) => {
                    // read our first few bytes to get our capacity
                    let mut usize_buf = [0u8; core::mem::size_of::<usize>()];
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            self.ptr.as_ptr() as *const u8,
                            usize_buf.as_mut_ptr(),
                            core::mem::size_of::<usize>(),
                        );
                    };
                    let cap = usize::from_le_bytes(usize_buf);

                    // read `cap` bytes from our buffer, starting at an offset
                    let buf_start = unsafe { self.ptr.as_ptr().add(core::mem::size_of::<usize>()) };
                    unsafe { slice::from_raw_parts(buf_start, cap) }
                }
            }
        }
    }

    #[inline]
    unsafe fn as_mut_buffer(&mut self) -> &mut [u8] {
        #[cfg(target_pointer_width = "64")]
        {
            debug_assert!(self.cap.as_usize().is_ok());

            // SAFETY: Since we have an instance of `BoxStringInner` so we know the buffer is still
            // valid. Also since we're on a 64-bit arch, it's practically impossible for our
            // capacity to be stored on the heap
            slice::from_raw_parts_mut(self.ptr.as_ptr(), self.cap.as_usize_unchecked())
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            match self.cap.as_usize() {
                // capacity is on the stack, so read the entire buffer!
                Ok(cap) => slice::from_raw_parts_mut(self.ptr.as_ptr(), cap),
                // capacity is on the heap, we need to read the capacity, and then read the buffer
                // starting from an offset
                Err(_) => {
                    // read our first few bytes to get our capacity
                    let mut usize_buf = [0u8; core::mem::size_of::<usize>()];
                    core::ptr::copy_nonoverlapping(
                        self.ptr.as_ptr() as *const u8,
                        usize_buf.as_mut_ptr(),
                        core::mem::size_of::<usize>(),
                    );
                    let cap = usize::from_le_bytes(usize_buf);

                    // read `cap` bytes from our buffer, starting at an offset
                    let buf_start = self.ptr.as_ptr().add(core::mem::size_of::<usize>());
                    slice::from_raw_parts_mut(buf_start, cap)
                }
            }
        }
    }

    #[inline(never)]
    unsafe fn drop_inner(&mut self) {
        #[cfg(target_pointer_width = "64")]
        {
            inner::inline_capacity::dealloc(self.ptr, self.capacity())
        }

        #[cfg(not(target_pointer_width = "64"))]
        match self.cap.as_usize() {
            Ok(cap) => inner::inline_capacity::dealloc(self.ptr, cap),
            Err(_) => {
                // read our first few bytes to get our capacity
                let mut usize_buf = [0u8; core::mem::size_of::<usize>()];
                core::ptr::copy_nonoverlapping(
                    self.ptr.as_ptr() as *const u8,
                    usize_buf.as_mut_ptr(),
                    core::mem::size_of::<usize>(),
                );
                let cap = usize::from_le_bytes(usize_buf);

                inner::heap_capacity::dealloc(self.ptr, cap)
            }
        }
    }
}

impl Clone for BoxString {
    fn clone(&self) -> Self {
        // Create a new BoxString
        let len = self.len();
        let mut new = Self::with_capacity(self.capacity());

        // Write the existing String into it
        // SAFETY: We're writing a &str which we know is valid UTF-8
        let buffer = unsafe { new.as_mut_slice() };
        buffer[..len].copy_from_slice(self.as_slice());
        // SAFETY: We just wrote `len` bytes into our buffer
        unsafe { new.set_len(len) };

        new
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
        BoxString::new(text)
    }
}

impl Drop for BoxString {
    fn drop(&mut self) {
        unsafe { self.drop_inner() }
    }
}

impl Extend<char> for BoxString {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(|c| self.push(c));
    }
}

impl<'c> Extend<&'c char> for BoxString {
    #[inline]
    fn extend<T: IntoIterator<Item = &'c char>>(&mut self, iter: T) {
        self.extend(iter.into_iter().copied());
    }
}

impl<'s> Extend<&'s str> for BoxString {
    #[inline]
    fn extend<T: IntoIterator<Item = &'s str>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s));
    }
}

impl Extend<Box<str>> for BoxString {
    #[inline]
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(&s));
    }
}

impl Extend<String> for BoxString {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use test_strategy::proptest;

    use super::BoxString;
    use crate::tests::rand_unicode;

    const SIXTEEN_MB: usize = 16 * 1024 * 1024;

    #[test]
    fn test_sanity() {
        let example = "hello world!";
        let box_str = BoxString::from(example);

        assert_eq!(box_str.as_str(), example);
        assert_eq!(box_str.len(), example.len());
    }

    #[test]
    fn test_empty() {
        let box_string = BoxString::new("");
        assert_eq!(box_string.as_str(), "");
    }

    #[test]
    fn test_push() {
        let example = "hello world";
        let mut boxed = BoxString::from(example);

        boxed.push('!');
        assert_eq!(boxed.as_str(), "hello world!");
        assert_eq!(boxed.len(), 12);
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
    fn test_box_string_capacity() {
        let example = "hello";
        let mut boxed = BoxString::from(example);

        // Starts with a capacity equal to length
        assert_eq!(boxed.capacity(), 5);

        boxed.push(' ');
        // Immediate reallocate to 1.5 * capacity
        assert_eq!(boxed.len(), 6);
        assert_eq!(boxed.capacity(), 7);

        boxed.push('w');
        boxed.push('o');
        // Right now our len and cap are both 7, pushing 'o' should cause us to resize
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
    fn test_string_capacity() {
        let example = "hello";
        let mut std_string = String::from(example);

        // `std::String` starts with capacity equal to length
        assert_eq!(std_string.capacity(), 5);

        // then doubles when re-allocating
        std_string.push(' ');
        assert_eq!(std_string.capacity(), 10);

        std_string.push('w');
        std_string.push('o');
        std_string.push('r');
        std_string.push('l');

        // after pushing an 11th element, we double capacity again
        std_string.push('d');
        assert_eq!(std_string.capacity(), 20);

        std_string.push('!');
    }

    #[test]
    fn test_from_string_parts() {
        let s = String::from("hello world!");
        let box_string = BoxString::from_string(s.clone());

        assert_eq!(s.as_str(), box_string.as_str());
    }

    #[test]
    fn test_from_string_parts_empty() {
        let s = String::from("");
        let box_string = BoxString::from_string(s.clone());

        assert_eq!(s.as_str(), box_string.as_str());
    }

    #[test]
    fn test_32_bit_max_inline_cap() {
        // 65 is the ASCII value of 'A'
        // `SIXTEEN_MB - 2` is the max value we can store for capacity inline, when on 32-bit archs
        let word_buf = vec![65; SIXTEEN_MB - 2];
        let string = String::from_utf8(word_buf).unwrap();

        let box_string = BoxString::new(&string);

        // make sure the capacity was able to be stored inline
        assert_eq!(box_string.cap.as_usize(), Ok(SIXTEEN_MB - 2));
        // assert the strings are equal
        assert_eq!(&string, box_string.as_str());
    }

    #[test]
    fn test_32_bit_max_inline_cap_with_modification() {
        // 65 is the ASCII value of 'A'
        // `SIXTEEN_MB - 2` is the max value we can store for capacity inline, when on 32-bit archs
        let word_buf = vec![65; SIXTEEN_MB - 2];
        let string = String::from_utf8(word_buf).unwrap();

        let mut box_string = BoxString::new(&string);

        // make sure the capacity was able to be stored inline
        assert_eq!(box_string.cap.as_usize(), Ok(SIXTEEN_MB - 2));
        // assert the strings are equal
        assert_eq!(&string, box_string.as_str());

        // push a single character
        box_string.push('!');
        // assert the string is still correct
        assert_eq!(&format!("{}!", string), box_string.as_str());

        // on 32-bit archs the capacity will be stored on the heap
        #[cfg(target_pointer_width = "32")]
        assert!(box_string.cap.as_usize().is_err());
        // on 64-bit archs it's still inlined
        #[cfg(not(target_pointer_width = "32"))]
        assert!(box_string.cap.as_usize().is_ok());

        // push a string
        box_string.push_str("hello!");

        // on 32-bit archs the capacity will still be stored on the heap, since the capacity hasn't
        // changed
        #[cfg(target_pointer_width = "32")]
        assert!(box_string.cap.as_usize().is_err());
        // on 64-bit archs it's still inlined
        #[cfg(not(target_pointer_width = "32"))]
        assert!(box_string.cap.as_usize().is_ok());

        // assert the string is still correct
        assert_eq!(&format!("{}!hello!", string), box_string.as_str());
    }

    #[test]
    fn test_32_bit_min_heap_cap() {
        // 65 is the ASCII value of 'A'
        // `SIXTEEN_MB - 1` is the min value for capacity that gets stored on the heap
        let word_buf = vec![65; SIXTEEN_MB - 1];
        let string = String::from_utf8(word_buf).unwrap();

        let mut box_string = BoxString::new(&string);

        // on 32-bit archs the capacity will be stored on the heap
        #[cfg(target_pointer_width = "32")]
        assert!(box_string.cap.as_usize().is_err());
        // on 64-bit archs it's still inlined
        #[cfg(not(target_pointer_width = "32"))]
        assert_eq!(box_string.cap.as_usize(), Ok(SIXTEEN_MB - 1));

        // assert the strings are equal
        assert_eq!(&string, box_string.as_str());

        // push a single character
        box_string.push('!');
        // assert the string is still correct
        assert_eq!(&format!("{}!", string), box_string.as_str());

        // on 32-bit archs the capacity will be stored on the heap
        #[cfg(target_pointer_width = "32")]
        assert!(box_string.cap.as_usize().is_err());
        // on 64-bit archs it's still inlined
        #[cfg(not(target_pointer_width = "32"))]
        assert!(box_string.cap.as_usize().is_ok());

        // push a string
        box_string.push_str("hello!");

        // on 32-bit archs the capacity will still be stored on the heap, since the capacity hasn't
        // changed
        #[cfg(target_pointer_width = "32")]
        assert!(box_string.cap.as_usize().is_err());
        // on 64-bit archs it's still inlined
        #[cfg(not(target_pointer_width = "32"))]
        assert!(box_string.cap.as_usize().is_ok());

        // assert the string is still correct
        assert_eq!(&format!("{}!hello!", string), box_string.as_str());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_strings_roundtrip(#[strategy(rand_unicode())] word: String) {
        let box_str = BoxString::from(word.as_str());
        prop_assert_eq!(&word, box_str.as_str());
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn test_from_string(#[strategy(rand_unicode())] word: String) {
        let s: String = word.clone();
        let box_str = BoxString::from_string(s);

        prop_assert_eq!(&word, box_str.as_str());
    }
}

crate::asserts::assert_size_eq!(BoxString, String);
