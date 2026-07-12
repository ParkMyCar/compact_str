//! Implements [`rayon`]'s [`FromParallelIterator`] and [`ParallelExtend`] traits for
//! [`CompactString`], so it can be built from (or extended by) a parallel iterator the same way
//! [`String`] can -- including the `Item = CompactString` variants.
//!
//! Every impl delegates the actual parallel collection to `String` (reusing rayon's optimized
//! string collectors) and then converts, so the behavior matches `String` exactly.

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;

use rayon::iter::{FromParallelIterator, IntoParallelIterator, ParallelExtend, ParallelIterator};

use crate::CompactString;

/// Mirrors both `rayon` string-collection traits for `CompactString`, for each item type that
/// rayon can already collect into a `String`.
macro_rules! impl_via_string {
    ($item:ty $(, $lt:lifetime)?) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
        impl$(<$lt>)? FromParallelIterator<$item> for CompactString {
            fn from_par_iter<I>(par_iter: I) -> Self
            where
                I: IntoParallelIterator<Item = $item>,
            {
                CompactString::from(String::from_par_iter(par_iter))
            }
        }

        #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
        impl$(<$lt>)? ParallelExtend<$item> for CompactString {
            fn par_extend<I>(&mut self, par_iter: I)
            where
                I: IntoParallelIterator<Item = $item>,
            {
                let mut buffer = String::new();
                buffer.par_extend(par_iter);
                self.push_str(&buffer);
            }
        }
    };
}

impl_via_string!(char);
impl_via_string!(&'a char, 'a);
impl_via_string!(&'a str, 'a);
impl_via_string!(String);
impl_via_string!(Box<str>);
impl_via_string!(Cow<'a, str>, 'a);

// `Item = CompactString`: rayon has no collectors for our type, so map each `CompactString` into a
// `String` first and reuse the `String` collectors.
#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
impl FromParallelIterator<CompactString> for CompactString {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = CompactString>,
    {
        CompactString::from(String::from_par_iter(
            par_iter.into_par_iter().map(CompactString::into_string),
        ))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
impl ParallelExtend<CompactString> for CompactString {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = CompactString>,
    {
        let mut buffer = String::new();
        buffer.par_extend(par_iter.into_par_iter().map(CompactString::into_string));
        self.push_str(&buffer);
    }
}

// The reverse direction: collect/extend a `String` from `CompactString`s. Permitted by the orphan
// rule because `CompactString` is a local type.
#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
impl FromParallelIterator<CompactString> for String {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = CompactString>,
    {
        String::from_par_iter(par_iter.into_par_iter().map(CompactString::into_string))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
impl ParallelExtend<CompactString> for String {
    fn par_extend<I>(&mut self, par_iter: I)
    where
        I: IntoParallelIterator<Item = CompactString>,
    {
        self.par_extend(par_iter.into_par_iter().map(CompactString::into_string));
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec::Vec;

    use rayon::prelude::*;
    use test_strategy::proptest;

    use crate::tests::rand_unicode;
    use crate::CompactString;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_par_iter_chars() {
        let compact: CompactString = vec!['h', 'e', 'l', 'l', 'o'].into_par_iter().collect();
        assert_eq!(compact, "hello");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_par_iter_str_slices() {
        let compact: CompactString = vec!["com", "pact", "_str"].par_iter().copied().collect();
        assert_eq!(compact, "compact_str");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn from_par_iter_strings() {
        let words = vec![String::from("a"), String::from("bc")];
        let compact: CompactString = words.into_par_iter().collect();
        assert_eq!(compact, "abc");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn par_extend_appends_in_place() {
        let mut compact = CompactString::from("start:");
        compact.par_extend(vec!['a', 'b', 'c'].into_par_iter());
        assert_eq!(compact, "start:abc");
    }

    #[test]
    #[cfg_attr(miri, ignore)]
    fn compact_string_items_collect_into_both() {
        let words = vec![CompactString::from("foo"), CompactString::from("bar")];
        let compact: CompactString = words.clone().into_par_iter().collect();
        let string: String = words.into_par_iter().collect();
        assert_eq!(compact, "foobar");
        assert_eq!(string, "foobar");
    }

    // Parity + ordering vs `String` across many words. Ignored under Miri, where rayon's thread
    // pool is far too slow.
    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_from_par_iter_matches_string(
        #[strategy(proptest::collection::vec(rand_unicode(), 0..20))] words: Vec<String>,
    ) {
        let compact: CompactString = words.par_iter().map(String::as_str).collect();
        let control: String = words.par_iter().map(String::as_str).collect();
        assert_eq!(compact, control);
    }

    #[proptest]
    #[cfg_attr(miri, ignore)]
    fn proptest_par_extend_matches_string(
        #[strategy(proptest::collection::vec(rand_unicode(), 0..20))] words: Vec<String>,
    ) {
        let mut compact = CompactString::from("prefix:");
        let mut control = String::from("prefix:");
        compact.par_extend(words.clone().into_par_iter());
        control.par_extend(words.into_par_iter());
        assert_eq!(compact, control);
    }
}
