use std::marker::PhantomData;
use std::{
    borrow,
    cmp,
    fmt,
    hash,
    mem,
    ops,
    ptr,
    slice,
};

use crate::repr::{
    outlined_drop,
    NonMaxU8,
    Repr,
    MAX_SIZE,
};
use crate::CompactString;

crate::asserts::assert_size_eq!(CompactString, CompactCow, Option<CompactCow>);

const BORROWED_FLAG: NonMaxU8 = NonMaxU8::V253;
const HEAP_FLAG: NonMaxU8 = NonMaxU8::V254;

#[repr(C)]
pub struct CompactCow<'a> {
    data: *const u8,
    len: usize,

    #[cfg(target_pointer_width = "64")]
    pad0: u32,
    pad1: u16,
    pad2: u8,
    discriminant: NonMaxU8,

    phantom: PhantomData<&'a str>,
}

unsafe impl Send for CompactCow<'_> {}
unsafe impl Sync for CompactCow<'_> {}

impl<'a> CompactCow<'a> {
    /// Construct a new [`CompactCow`] from a [`CompactString`].
    ///
    /// The new [`CompactCow`] has `'static` lifetime, and will become the owner of the supplied
    /// string.
    ///
    /// This method works the same as Calling [`CompactCow::from(some_compact_string)`](From).
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::{CompactCow, CompactString};
    /// let string = CompactString::from("Hey, world!");
    /// let cow = CompactCow::from_compact(string);
    /// assert_eq!("Hey, world!", cow);
    /// ```
    #[must_use]
    #[inline]
    pub const fn from_compact(value: CompactString) -> CompactCow<'static> {
        // SAFETY: we own `value`, and the representation is compatible
        unsafe { mem::transmute(value) }
    }

    /// Construct a new [`CompactCow`] from a [`String`].
    ///
    /// The new [`CompactCow`] has `'static` lifetime, and will become the owner of the supplied
    /// string.
    ///
    /// This method works the same as Calling [`CompactCow::from(some_string)`](From).
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::{CompactCow, CompactString};
    /// let string = CompactString::from("Hey, world!");
    /// let cow = CompactCow::from_compact(string);
    /// assert_eq!("Hey, world!", cow);
    /// ```
    #[must_use]
    #[inline]
    pub fn from_string(value: String) -> CompactCow<'static> {
        CompactString::from(value).into()
    }

    /// Borrow an [`str`] for construct a new [`CompactCow`].
    ///
    /// The new [`CompactCow`] has the same lifetime as the argument.
    /// If the string is short enough to be inlined, it is.
    ///
    /// This method works the same as Calling [`CompactCow::from(string_reference)`](From).
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::{CompactCow, CompactString};
    /// let string = "Hey, world!";
    /// let cow = CompactCow::from_str(string);
    /// assert_eq!("Hey, world!", cow);
    /// ```
    #[must_use]
    pub const fn from_str(value: &str) -> Self {
        if value.len() <= MAX_SIZE {
            Self::from_compact(CompactString::new_inline(value))
        } else {
            CompactCow {
                data: value.as_ptr(),
                len: value.len(),
                #[cfg(target_pointer_width = "64")]
                pad0: 0,
                pad1: 0,
                pad2: 0,
                discriminant: BORROWED_FLAG,
                phantom: PhantomData,
            }
        }
    }

    /// Returns `true` if the [`CompactCow`] borrows its data.
    ///
    /// This method works the same as [`!compact_cow.is_owned()`](CompactCow::is_owned).
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCow;
    /// let cow = CompactCow::from("Hello, world! How are you today?");
    /// assert!(cow.is_borrowed());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_borrowed(&self) -> bool {
        matches!(self.discriminant, BORROWED_FLAG)
    }

    /// Returns `true` if the [`CompactCow`] owns its data.
    ///
    /// This method works the same as [`!compact_cow.is_borrowed()`](CompactCow::is_borrowed).
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCow;
    /// let cow = CompactCow::from(String::from("Hello, world! How are you today?"));
    /// assert!(cow.is_owned());
    /// ```
    #[must_use]
    #[inline]
    pub const fn is_owned(&self) -> bool {
        !self.is_borrowed()
    }

    /// Retrieve a reference to the contained [`str`] data.
    ///
    /// This method works the same as [`&*compact_cow`](ops::Deref).
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCow;
    /// let cow = CompactCow::from("Hello, world! How are you today?");
    /// assert_eq!(cow.as_str(), "Hello, world! How are you today?");
    /// ```
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self.discriminant {
            BORROWED_FLAG => {
                // SAFETY: we just checked that `self.is_borrowed()`
                unsafe { self.uncheched_as_borrowed() }
            }
            HEAP_FLAG => {
                // SAFETY: we just checked that `self.is_owned()`
                let compact = unsafe { self.unchecked_as_owned() };
                // SAFETY: we just checked that the owned data is on the heap
                let heap = unsafe { &compact.repr.as_union().heap };
                heap.string.as_str()
            }
            _ => {
                // SAFETY: we just checked that `self.is_owned()`
                let compact = unsafe { self.unchecked_as_owned() };
                // SAFETY: we just checked that the owned data is stored inline
                let inline = unsafe { &compact.repr.as_union().inline };
                inline.as_str()
            }
        }
    }

    /// Retrieve a mutable reference to the contained [`str`] data.
    ///
    /// This method works the same as [`&mut *compact_cow`](ops::DerefMut),
    /// and is slightly faster than calling `compact_cow.to_mut().as_mut_str()`.
    ///
    /// If the data was borrowed before calling this method, it gets copied to be owned.
    ///
    /// # See also
    ///
    /// * [`CompactCow::to_mut()`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCow;
    /// let mut cow = CompactCow::from("Hello, world! How are you today?");
    /// cow.as_mut_str().make_ascii_uppercase();
    /// assert_eq!(cow, "HELLO, WORLD! HOW ARE YOU TODAY?");
    /// ```
    #[must_use]
    pub fn as_mut_str(&mut self) -> &mut str {
        let (len, slice) = loop {
            match self.discriminant {
                BORROWED_FLAG => {
                    // SAFETY: we just checked that `self.is_borrowed()`
                    unsafe { self.make_owned() };
                    // Continue the loop. The data is owned now, so this case won't be hit again.
                }
                HEAP_FLAG => {
                    // SAFETY: we just checked that `self.is_owned()`
                    let compact = unsafe { self.unchecked_as_mut_owned() };
                    // SAFETY: we just checked that the owned data is on the heap
                    let heap = unsafe { &mut compact.repr.as_union_mut().heap };
                    // SAFETY: we'll only return the first `len()` bytes
                    break (heap.string.len(), unsafe { heap.string.as_mut_slice() });
                }
                _ => {
                    // SAFETY: we just checked that `self.is_owned()`
                    let compact = unsafe { self.unchecked_as_mut_owned() };
                    // SAFETY: we just checked that the owned data is stored inline
                    let inline = unsafe { &mut compact.repr.as_union_mut().inline };
                    // SAFETY: we'll only return the first `len()` bytes
                    break (inline.len(), unsafe { inline.as_mut_slice() });
                }
            }
        };
        // SAFETY: we owned data must be a valid string
        unsafe { std::str::from_utf8_unchecked_mut(&mut slice[..len]) }
    }

    /// Make the data owned if needed, and return a mutable reference to it.
    ///
    /// # See also
    ///
    /// * [`CompactCow::as_mut_str()`]
    ///
    /// # Examples
    ///
    /// ```
    /// # use compact_str::CompactCow;
    /// let mut cow = CompactCow::from("Hello, world! How are you today?");
    /// cow.to_mut().truncate(13);
    /// assert_eq!(cow, "Hello, world!");
    /// ```
    // Maybe the user only wants to make the data owned, but doesn't care about the data itself.
    #[allow(clippy::must_use_candidate)]
    #[inline]
    pub fn to_mut(&mut self) -> &mut CompactString {
        self.ensure_owned();
        // SAFETY: we ensured that `self.is_owned()`
        unsafe { self.unchecked_as_mut_owned() }
    }

    /// Get a reference to the borrowed [`str`].
    ///
    /// Returns `None` is the data is not borrowed.
    #[must_use]
    #[inline]
    pub fn get_borrowed(&self) -> Option<&'a str> {
        match self.is_borrowed() {
            true => {
                // SAFETY: we just checked that `self.is_borrowed()`
                Some(unsafe { self.uncheched_as_borrowed() })
            }
            false => None,
        }
    }

    /// Get a reference to the borrowed [`CompactCow`].
    ///
    /// Returns `None` is the data is not owned.
    #[inline]
    pub fn get_owned(&self) -> Option<&CompactString> {
        match self.is_owned() {
            true => {
                // SAFETY: we just checked that `self.is_owned()`
                Some(unsafe { self.unchecked_as_owned() })
            }
            false => None,
        }
    }

    /// Get a mutable reference to the borrowed [`CompactCow`].
    ///
    /// Returns `None` is the data is not owned.
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut CompactString> {
        match self.is_owned() {
            true => {
                // SAFETY: we just checked that `self.is_owned()`
                Some(unsafe { self.unchecked_as_mut_owned() })
            }
            false => None,
        }
    }

    /// Extract the owned [`str`] data as [`CompactString`].
    ///
    /// If the data was not owned before calling this method, it gets copied.
    #[inline]
    pub fn into_owned(mut self) -> CompactString {
        self.ensure_owned();
        // SAFETY: we ensured that `self.is_owned()`
        unsafe { self.unchecked_owned_into_inner() }
    }

    /// Extract the owned [`str`] data as [`String`].
    ///
    /// If the data was not owned before calling this method, it gets copied.
    pub fn into_string(self) -> String {
        match self.discriminant {
            BORROWED_FLAG => {
                // SAFETY: we just checked that `self.is_borrowed()`
                let borrowed = unsafe { self.uncheched_as_borrowed() };
                let result = borrowed.to_owned();

                // No need to call drop for borrowed data.
                mem::forget(self);

                result
            }
            HEAP_FLAG => {
                // SAFETY: we just checked that `self.is_owned()`
                //         we know that both both representations are compatible
                let compact: CompactString = unsafe { mem::transmute(self) };
                compact.into_string()
            }
            _ => {
                // SAFETY: we just checked that `self.is_owned()`
                let compact = unsafe { self.unchecked_as_owned() };
                // SAFETY: we just checked that the owned data is stored inline
                let inline = unsafe { &compact.repr.as_union().inline };
                let result = inline.as_str().to_owned();

                // No need to call drop for inline data.
                mem::forget(self);

                result
            }
        }
    }

    /// SAFETY: caller must ensure that `self.is_owned()`
    #[inline]
    const unsafe fn unchecked_as_owned(&self) -> &CompactString {
        &*(self as *const Self).cast()
    }

    /// SAFETY: caller must ensure that `self.is_owned()`
    #[inline]
    unsafe fn unchecked_as_mut_owned(&mut self) -> &mut CompactString {
        &mut *(self as *mut Self).cast()
    }

    /// SAFETY: caller must ensure that `self.is_owned()`
    #[inline]
    const unsafe fn unchecked_owned_into_inner(self) -> CompactString {
        mem::transmute(mem::ManuallyDrop::new(self))
    }

    /// SAFETY: caller must ensure that `self.is_borrowed()`
    #[inline]
    unsafe fn uncheched_as_borrowed(&self) -> &'a str {
        let slice = slice::from_raw_parts(self.data, self.len);
        core::str::from_utf8_unchecked(slice)
    }

    #[inline]
    fn ensure_owned(&mut self) {
        if self.is_borrowed() {
            unsafe { self.make_owned() };
        }
    }

    /// SAFETY: the caller has to ensure that `self.is_borrowed()`
    unsafe fn make_owned(&mut self) {
        let s = self.uncheched_as_borrowed();
        let mut s = mem::ManuallyDrop::new(Repr::new(s));
        // SAFETY: both representations are compatible
        //         the old data was borrowed, so it does not need to be dropped
        ptr::swap((self as *mut Self).cast(), &mut s);
    }
}

impl Clone for CompactCow<'_> {
    fn clone(&self) -> Self {
        if self.is_borrowed() {
            // SAFETY: we just checked that `self.is_borrowed()`
            let value = unsafe { self.uncheched_as_borrowed() };
            value.into()
        } else {
            // SAFETY: we just checked that `self.is_owned()`
            let value = unsafe { self.unchecked_as_owned() };
            value.clone().into()
        }
    }
}

impl Drop for CompactCow<'_> {
    #[inline]
    fn drop(&mut self) {
        if self.discriminant == HEAP_FLAG {
            // SAFETY: we just checked that `self.is_owned()`
            let value = unsafe { self.unchecked_as_mut_owned() };
            outlined_drop(&mut value.repr);
        }
    }
}

impl Default for CompactCow<'_> {
    #[inline]
    fn default() -> Self {
        Self::from_str("")
    }
}

impl ops::Deref for CompactCow<'_> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl ops::DerefMut for CompactCow<'_> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.to_mut()
    }
}

impl AsRef<str> for CompactCow<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for CompactCow<'_> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl borrow::Borrow<str> for CompactCow<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl hash::Hash for CompactCow<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Display for CompactCow<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl fmt::Debug for CompactCow<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

mod impl_cmp_for_compact_cow {
    use super::*;

    impl<T: AsRef<str>> cmp::PartialEq<T> for CompactCow<'_> {
        fn eq(&self, other: &T) -> bool {
            self.as_str() == other.as_ref()
        }
    }

    impl cmp::Eq for CompactCow<'_> {}

    impl<T: AsRef<str>> cmp::PartialOrd<T> for CompactCow<'_> {
        fn partial_cmp(&self, other: &T) -> Option<cmp::Ordering> {
            Some(self.as_str().cmp(other.as_ref()))
        }
    }

    impl cmp::Ord for CompactCow<'_> {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.as_str().cmp(other.as_str())
        }
    }
}

mod impl_cmp_from_compact_cow {
    use super::*;

    impl<'a> cmp::PartialEq<CompactCow<'a>> for &str {
        fn eq(&self, other: &CompactCow<'a>) -> bool {
            <str as PartialEq<str>>::eq(self, other)
        }
    }

    impl<'a> cmp::PartialOrd<CompactCow<'a>> for &str {
        fn partial_cmp(&self, other: &CompactCow<'a>) -> Option<cmp::Ordering> {
            <str as PartialOrd<str>>::partial_cmp(self, other)
        }
    }

    impl<'a> cmp::PartialEq<CompactCow<'a>> for String {
        fn eq(&self, other: &CompactCow<'a>) -> bool {
            <str as PartialEq<str>>::eq(self, other)
        }
    }

    impl<'a> cmp::PartialOrd<CompactCow<'a>> for String {
        fn partial_cmp(&self, other: &CompactCow<'a>) -> Option<cmp::Ordering> {
            <str as PartialOrd<str>>::partial_cmp(self, other)
        }
    }

    impl<'a> cmp::PartialEq<CompactCow<'a>> for borrow::Cow<'_, str> {
        fn eq(&self, other: &CompactCow<'a>) -> bool {
            <str as PartialEq<str>>::eq(self, other)
        }
    }

    impl<'a> cmp::PartialOrd<CompactCow<'a>> for borrow::Cow<'_, str> {
        fn partial_cmp(&self, other: &CompactCow<'a>) -> Option<cmp::Ordering> {
            <str as PartialOrd<str>>::partial_cmp(self, other)
        }
    }
}

mod impl_add {
    use super::*;

    impl<'a, 'b> ops::AddAssign<&'b str> for CompactCow<'a> {
        #[inline]
        fn add_assign(&mut self, rhs: &'b str) {
            if !rhs.is_empty() {
                *self.to_mut() += rhs;
            }
        }
    }

    impl<'a, 'b> ops::AddAssign<CompactCow<'b>> for CompactCow<'a> {
        #[inline]
        fn add_assign(&mut self, rhs: CompactCow<'b>) {
            *self += rhs.as_str();
        }
    }

    impl<'a, 'b> ops::Add<&'b str> for CompactCow<'a> {
        type Output = CompactCow<'a>;

        #[inline]
        fn add(mut self, rhs: &'b str) -> Self::Output {
            self += rhs;
            self
        }
    }

    impl<'a, 'b> ops::Add<CompactCow<'b>> for CompactCow<'a> {
        type Output = CompactCow<'a>;

        #[inline]
        fn add(mut self, rhs: CompactCow<'b>) -> Self::Output {
            self += rhs;
            self
        }
    }
}

mod impl_into_compact_cow {
    use super::*;

    impl<'a> From<CompactString> for CompactCow<'a> {
        #[inline]
        fn from(value: CompactString) -> Self {
            Self::from_compact(value)
        }
    }

    impl<'a> From<&'a str> for CompactCow<'a> {
        #[inline]
        fn from(value: &'a str) -> Self {
            Self::from_str(value)
        }
    }

    impl<'a> From<String> for CompactCow<'a> {
        #[inline]
        fn from(value: String) -> Self {
            Self::from_string(value)
        }
    }

    impl<'a> From<borrow::Cow<'a, str>> for CompactCow<'a> {
        fn from(value: borrow::Cow<'a, str>) -> Self {
            match value {
                borrow::Cow::Borrowed(s) => s.into(),
                borrow::Cow::Owned(s) => s.into(),
            }
        }
    }
}

mod impl_from_compact_cow {
    use super::*;

    impl<'a> From<CompactCow<'a>> for borrow::Cow<'a, str> {
        fn from(value: CompactCow<'a>) -> Self {
            match value.is_borrowed() {
                true => borrow::Cow::Borrowed(unsafe { value.uncheched_as_borrowed() }),
                false => {
                    borrow::Cow::Owned(String::from(unsafe { value.unchecked_owned_into_inner() }))
                }
            }
        }
    }

    impl<'a> From<CompactCow<'a>> for String {
        #[inline]
        fn from(value: CompactCow<'a>) -> Self {
            value.into_string()
        }
    }

    impl<'a> From<CompactCow<'a>> for CompactString {
        #[inline]
        fn from(value: CompactCow<'a>) -> Self {
            value.into_owned()
        }
    }
}

mod impl_from_iterator_for_compact_cow {
    use super::*;

    impl FromIterator<char> for CompactCow<'static> {
        #[inline]
        fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
            CompactString::from_iter(iter).into()
        }
    }

    impl FromIterator<String> for CompactCow<'static> {
        #[inline]
        fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
            CompactString::from_iter(iter).into()
        }
    }

    impl FromIterator<CompactString> for CompactCow<'static> {
        fn from_iter<T: IntoIterator<Item = CompactString>>(iter: T) -> Self {
            let mut result = CompactString::new("");
            for s in iter {
                result.push_str(s.as_str());
            }
            result.into()
        }
    }

    impl<'a> FromIterator<&'a str> for CompactCow<'static> {
        #[inline]
        fn from_iter<T: IntoIterator<Item = &'a str>>(iter: T) -> Self {
            CompactString::from_iter(iter).into()
        }
    }

    impl<'a> FromIterator<borrow::Cow<'a, str>> for CompactCow<'static> {
        #[inline]
        fn from_iter<T: IntoIterator<Item = borrow::Cow<'a, str>>>(iter: T) -> Self {
            CompactString::from_iter(iter).into()
        }
    }
}

mod impl_from_iterator_from_compact_cow {
    use super::*;

    impl<'a> FromIterator<CompactCow<'a>> for String {
        fn from_iter<T: IntoIterator<Item = CompactCow<'a>>>(iter: T) -> Self {
            let mut iter = iter.into_iter();
            match iter.next() {
                Some(s) => {
                    let mut s = s.into_string();
                    s.extend(iter);
                    s
                }
                None => String::new(),
            }
        }
    }

    impl<'a> FromIterator<CompactCow<'a>> for CompactString {
        fn from_iter<T: IntoIterator<Item = CompactCow<'a>>>(iter: T) -> Self {
            let mut iter = iter.into_iter();
            match iter.next() {
                Some(s) => {
                    let mut s = s.into_owned();
                    s.extend(iter);
                    s
                }
                None => CompactString::new(""),
            }
        }
    }

    impl<'a> FromIterator<CompactCow<'a>> for borrow::Cow<'static, str> {
        fn from_iter<T: IntoIterator<Item = CompactCow<'a>>>(iter: T) -> Self {
            String::from_iter(iter).into()
        }
    }

    impl<'a> FromIterator<CompactCow<'a>> for Repr {
        fn from_iter<T: IntoIterator<Item = CompactCow<'a>>>(iter: T) -> Self {
            crate::repr::from_as_ref_str_iterator(iter.into_iter())
        }
    }
}

mod impl_extend_for_compact_cow {
    use super::*;

    impl<'a> Extend<CompactCow<'a>> for CompactCow<'_> {
        fn extend<T: IntoIterator<Item = CompactCow<'a>>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }

    impl Extend<CompactString> for CompactCow<'_> {
        fn extend<T: IntoIterator<Item = CompactString>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }

    impl Extend<String> for CompactCow<'_> {
        fn extend<T: IntoIterator<Item = String>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }

    impl<'a> Extend<borrow::Cow<'a, str>> for CompactCow<'_> {
        fn extend<T: IntoIterator<Item = borrow::Cow<'a, str>>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }

    impl Extend<Box<str>> for CompactCow<'_> {
        fn extend<T: IntoIterator<Item = Box<str>>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }

    impl<'a> Extend<&'a str> for CompactCow<'_> {
        fn extend<T: IntoIterator<Item = &'a str>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }
}

mod impl_extend_from_compact_cow {
    use super::*;

    impl<'a> Extend<CompactCow<'a>> for String {
        fn extend<T: IntoIterator<Item = CompactCow<'a>>>(&mut self, iter: T) {
            iter.into_iter().for_each(move |s| self.push_str(&s));
        }
    }

    impl<'a> Extend<CompactCow<'a>> for borrow::Cow<'_, str> {
        fn extend<T: IntoIterator<Item = CompactCow<'a>>>(&mut self, iter: T) {
            self.to_mut().extend(iter)
        }
    }

    impl<'a> Extend<CompactCow<'a>> for CompactString {
        fn extend<T: IntoIterator<Item = CompactCow<'a>>>(&mut self, iter: T) {
            iter.into_iter().for_each(|s| self.push_str(&s));
        }
    }
}

#[cfg(feature = "serde")]
mod impl_serde {
    use serde::{
        de,
        ser,
    };

    use super::*;

    struct Visitor<'a>(PhantomData<&'a ()>);

    impl<'a, 'de: 'a> de::Visitor<'de> for Visitor<'a> {
        type Value = CompactCow<'a>;

        #[inline]
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("expected string")
        }

        #[inline]
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(CompactString::new(v).into())
        }

        #[inline]
        fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
            Ok(CompactCow::from_str(v))
        }

        #[inline]
        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(v.into())
        }
    }

    impl<'a, 'de: 'a> de::Deserialize<'de> for CompactCow<'a> {
        #[inline]
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_str(Visitor(PhantomData))
        }
    }

    impl ser::Serialize for CompactCow<'_> {
        #[inline]
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_str(self.as_str())
        }
    }

    #[cfg(test)]
    #[test]
    fn test() {
        const SHORT_VALUE: &str = "0123456789";
        const SHORT_JSON: &str = concat!(r#"{ "data": "0123456789" }"#);

        const LONG_VALUE: &str = "01234567890123456789012345";
        const LONG_JSON: &str = concat!(r#"{ "data": "01234567890123456789012345" }"#);

        const ESCAPED_SHORT_VALUE: &str = "01234 6789";
        const ESCAPED_SHORT_JSON: &str = concat!(r#"{ "data": "01234\u00206789" }"#);

        const ESCAPED_LONG_VALUE: &str = "01234 67890123456789012345";
        const ESCAPED_LONG_JSON: &str = concat!(r#"{ "data": "01234\u002067890123456789012345" }"#);

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Test<'a> {
            #[serde(borrow)]
            data: CompactCow<'a>,
        }

        let data = serde_json::from_str::<Test>(SHORT_JSON).unwrap().data;
        assert_eq!(data, SHORT_VALUE);
        assert!(data.is_owned());
        assert!(!data.as_owned().unwrap().is_heap_allocated());

        let data = serde_json::from_str::<Test>(LONG_JSON).unwrap().data;
        assert_eq!(data, LONG_VALUE);
        assert!(data.is_borrowed());

        let data = serde_json::from_str::<Test>(ESCAPED_SHORT_JSON)
            .unwrap()
            .data;
        assert_eq!(data, ESCAPED_SHORT_VALUE);
        assert!(data.is_owned());
        assert!(!data.as_owned().unwrap().is_heap_allocated());

        let data = serde_json::from_str::<Test>(ESCAPED_LONG_JSON)
            .unwrap()
            .data;
        assert_eq!(data, ESCAPED_LONG_VALUE);
        assert!(data.is_owned());
        assert!(data.as_owned().unwrap().is_heap_allocated());
    }
}
