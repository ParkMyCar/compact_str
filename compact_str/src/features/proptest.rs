//! Implements the [`proptest::arbitrary::Arbitrary`] trait for [`CompactString`]

use proptest::arbitrary::{
    Arbitrary,
    StrategyFor,
};
use proptest::prelude::*;
use proptest::strategy::{
    MapInto,
    Strategy,
};
use proptest::string::StringParam;

use crate::CompactString;

impl Arbitrary for CompactString {
    type Parameters = StringParam;
    type Strategy = MapInto<StrategyFor<String>, Self>;

    fn arbitrary_with(a: Self::Parameters) -> Self::Strategy {
        any_with::<String>(a).prop_map_into()
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use crate::CompactString;

    proptest! {
        #[test]
        #[cfg_attr(miri, ignore)]
        fn proptest_sanity(compact: CompactString) {
            let control: String = compact.clone().into();
            assert_eq!(control, compact);
        }

        /// We rely on [`proptest`]'s `String` strategy for generating a `CompactString`. When
        /// converting from a `String` into a `CompactString`, our O(1) converstion kicks in and we
        /// reuse the buffer, unless empty, and thus all non-empty strings will be heap allocated
        #[test]
        #[cfg_attr(miri, ignore)]
        fn proptest_does_not_inline_strings(compact: CompactString) {
            if compact.is_empty() {
                assert!(!compact.is_heap_allocated());
            } else {
                assert!(compact.is_heap_allocated());
            }
        }
    }
}
