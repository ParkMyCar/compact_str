use core::iter::Extend;
use core::sync::atomic::Ordering;

use super::ArcString;

/// An `ArcStringWriter` provides safe mutable access to the underlying buffer of an `ArcString`.
///
/// To create an `ArcStringWriter`, we first check to make sure we have a unique reference to the
/// underlying buffer of an `ArcString`, if we don't then we create one. Knowing we hold a unique
/// reference to the `ArcString` allows us to make multiple modifications to the buffer, this is
/// particularly beneficial for something like the `core::iter::Extend` trait
pub struct ArcStringWriter<'a> {
    arc_string: &'a mut ArcString,
}

impl<'a> ArcStringWriter<'a> {
    #[inline]
    pub fn new(arc_string: &'a mut ArcString) -> Self {
        if arc_string
            .inner()
            .ref_count
            .compare_exchange(1, 0, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // There is more than one reference to this underlying buffer, so we need to make a new
            // instance and decrement the count of the original by one

            // Make a new instance with the same capacity as self
            let additional = arc_string.capacity() - arc_string.len();
            let new = ArcString::new(arc_string.as_str(), additional);

            // Assign arc_string to our new instsance, this drops the old ArcString, which
            // decrements its ref count
            *arc_string = new;
        } else {
            // We were the sole reference of either kind; bump back up the strong ref count.
            arc_string.inner().ref_count.store(1, Ordering::Release);
        }

        Self { arc_string }
    }

    /// Reserve space for at least `additional` bytes
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        // We need at least this much space
        let new_capacity = self.arc_string.len() + additional;

        // We have enough space, so there is no work to do
        if self.arc_string.capacity() >= new_capacity {
            return;
        }

        // Create a new `ArcString` with enough space for at least `additional` bytes, dropping the
        // old one
        *self.arc_string = ArcString::new(self.arc_string.as_str(), additional);
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        let len = self.arc_string.len();
        let char_len = ch.len_utf8();

        // Reserve at least enough space for the new char
        self.reserve(char_len);

        // SAFETY: We're writing a char into the slice, which is valid UTF-8
        let slice = unsafe { self.as_mut_slice() };

        // Write the char into the slice
        ch.encode_utf8(&mut slice[len..]);
        // Increment our length
        //
        // SAFETY: We just wrote `char_len` bytes into the buffer, so we know this new length is
        // valid
        unsafe { self.arc_string.set_len(len + char_len) };
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        let len = self.arc_string.len();
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
        unsafe { self.arc_string.set_len(len + str_len) };
    }

    /// Returns a reference to a mutable slice of bytes with lifetime `'a`
    ///
    /// # SAFETY
    /// * Callers must guarantee that any modifications they make to the slice are valid UTF-8
    #[inline]
    pub unsafe fn as_mut_slice(&mut self) -> &'a mut [u8] {
        self.arc_string.ptr.as_mut().as_mut_bytes()
    }

    /// Transforms the `ArcStringWriter<'a>` into a mutable slice of bytes with lifetime `'a`
    ///
    /// # SAFETY
    /// * Callers must guarantee that any modifications they make to the slice are valid UTF-8
    #[inline]
    pub unsafe fn into_mut_slice(self) -> &'a mut [u8] {
        // SAFETY: If we still have an instance of `ArcString` then we know the pointer to
        // `ArcStringInner` is valid for at least as long as the provided ref to `self`
        self.arc_string.ptr.as_mut().as_mut_bytes()
    }
}

impl<'a> Extend<char> for ArcStringWriter<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        let iterator = iter.into_iter();
        let (lower_bound, _) = iterator.size_hint();
        self.reserve(lower_bound);
        iterator.for_each(|c| self.push(c));
    }
}

impl<'c, 'a> Extend<&'c char> for ArcStringWriter<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'c char>>(&mut self, iter: T) {
        self.extend(iter.into_iter().copied());
    }
}

impl<'s, 'a> Extend<&'s str> for ArcStringWriter<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = &'s str>>(&mut self, iter: T) {
        iter.into_iter().for_each(|s| self.push_str(s));
    }
}

impl<'a> Extend<Box<str>> for ArcStringWriter<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}

impl<'a> Extend<String> for ArcStringWriter<'a> {
    #[inline]
    fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
        iter.into_iter().for_each(move |s| self.push_str(&s));
    }
}
