use crate::SmartStr;

#[test]
fn sanity_test() {
    let small_str = SmartStr::new("hello world");
    assert_eq!(small_str, "hello world");
    // small_str is 11 characters, which should always be allocated on the stack
    assert!(!small_str.is_heap_allocated());

    let large_str = SmartStr::new("I am a cool str that is 42 characters long");
    assert_eq!(large_str, "I am a cool str that is 42 characters long");
    // large_str is 42 characters, which should always be allocated on the heap
    assert!(large_str.is_heap_allocated());
}

#[cfg(test)]
mod randomized {
    use crate::SmartStr;
    use proptest::prelude::*;

    #[cfg(target_pointer_width = "64")]
    const INLINED_SIZE: usize = 23;
    #[cfg(target_pointer_width = "32")]
    const INLINED_SIZE: usize = 11;

    proptest! {
        #[test]
        fn test_form_strings_correctly(word in "[.*]{0,1000}") {
            let smartstr = SmartStr::new(&word);

            // strings should be equal
            prop_assert_eq!(&word, &smartstr);

            // strings with length INLINED_SIZE bytes or less should not be heap allocatated
            match word.len() {
                0..=INLINED_SIZE => prop_assert!(!smartstr.is_heap_allocated()),
                _ => prop_assert!(smartstr.is_heap_allocated()),
            }
        }
    }
}
