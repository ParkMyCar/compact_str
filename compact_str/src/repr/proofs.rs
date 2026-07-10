use super::capacity::{Capacity, MAX_VALUE};
use super::{Repr, HEAP_MASK, LENGTH_MASK, MAX_SIZE};

/// Verify that every valid UTF-8 byte sequence that fits inline survives the unsafe
/// `InlineBuffer` to `Repr` conversion without changing its contents or metadata.
#[kani::proof]
fn inline_repr_roundtrip() {
    let bytes: [u8; MAX_SIZE] = kani::any();
    let len: usize = kani::any();

    kani::assume(len <= MAX_SIZE);
    let input = &bytes[..len];
    kani::assume(core::str::from_utf8(input).is_ok());

    // SAFETY: The preceding assumption restricts this proof to valid UTF-8 inputs.
    let text = unsafe { core::str::from_utf8_unchecked(input) };
    let repr = Repr::new(text).expect("an inline Repr does not allocate");

    assert_eq!(repr.as_str().as_bytes(), input);
    assert_eq!(repr.len(), len);
    assert_eq!(repr.capacity(), MAX_SIZE);
    assert!(!repr.is_heap_allocated());
    assert!(!repr.is_static_str());
    assert!(repr.last_byte() < HEAP_MASK);

    if len < MAX_SIZE {
        assert_eq!(repr.last_byte(), len as u8 | LENGTH_MASK);
    } else {
        // A full inline buffer stores string data in its final byte. Valid UTF-8 guarantees that
        // byte cannot overlap with the range reserved for length markers and discriminants.
        assert_eq!(repr.last_byte(), bytes[MAX_SIZE - 1]);
        assert!(repr.last_byte() < LENGTH_MASK);
    }

    kani::cover!(len == 0);
    kani::cover!(len == MAX_SIZE);
}

/// Verify the reversible portion of `Capacity`'s packed representation for every value it can
/// encode inline.
#[kani::proof]
fn capacity_roundtrip() {
    let original: usize = kani::any();
    kani::assume(original <= MAX_VALUE);

    let encoded = Capacity::new(original);
    assert!(!encoded.is_heap());

    // SAFETY: The assumption above restricts `encoded` to the inline capacity range.
    let decoded = unsafe { encoded.as_usize() };
    assert_eq!(decoded, original);

    kani::cover!(original == 0);
    kani::cover!(original == MAX_VALUE);
}

/// Exercise the concrete values at both ends of the packed capacity representation, including
/// the 32-bit sentinel used when the value itself must be stored on the heap.
#[kani::proof]
fn capacity_boundaries() {
    let zero = Capacity::new(0);
    assert!(!zero.is_heap());
    // SAFETY: Zero is in the inline capacity range.
    assert_eq!(unsafe { zero.as_usize() }, 0);

    let max = Capacity::new(MAX_VALUE);
    assert!(!max.is_heap());
    // SAFETY: `MAX_VALUE` is the upper end of the inline capacity range.
    assert_eq!(unsafe { max.as_usize() }, MAX_VALUE);

    #[cfg(target_pointer_width = "32")]
    {
        assert!(Capacity::new(MAX_VALUE + 1).is_heap());
        assert!(Capacity::new(usize::MAX).is_heap());
    }
}

/// Verify that a value outside the 64-bit packed representation's domain is rejected before it
/// can violate the unsafe encoding assumption in `Capacity::new`.
#[cfg(target_pointer_width = "64")]
#[kani::proof]
#[kani::should_panic]
fn capacity_rejects_usize_max() {
    let _ = Capacity::new(usize::MAX);
}
