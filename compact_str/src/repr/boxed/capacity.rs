use crate::repr::HEAP_MASK;

// how many bytes a `usize` occupies
const USIZE_SIZE: usize = core::mem::size_of::<usize>();
// state that describes the capacity as being stored on the heap
const CAPACITY_IS_ON_THE_HEAP: [u8; USIZE_SIZE] = [HEAP_MASK; USIZE_SIZE];

// how many bytes we can use for capacity
const SPACE_FOR_CAPACITY: usize = USIZE_SIZE - 1;
// the maximum value we're able to store, e.g. on 64-bit arch this is 2^56 - 2
//
// note: Preferably we'd used usize.pow(..) here, but that's not a `const fn`, so we need to use
// bitshift operators, and there's a lint against using them in this pattern, which IMO isn't a
// great lint
#[allow(clippy::precedence)]
const MAX_VALUE: usize = (1 << SPACE_FOR_CAPACITY * 8) - 2;

/// An integer type that uses `core::mem::size_of::<usize>() - 1` bytes to store the capacity of
/// a heap buffer.
///
/// Assumming a 64-bit arch, a [`super::BoxString`] uses 8 bytes for a pointer, 8 bytes for a
/// length, and then needs 1 byte for a discriminant. We need to store the capacity somewhere, and
/// we could store it on the heap, but we also have 7 unused bytes. [`Capacity`] handles storing a
/// value in these 7 bytes, returning an error if it's not possible, at which point we'll store the
/// capacity on the heap.
///
/// # Max Values
/// * __64-bit:__ `(2 ^ (7 * 8)) - 2 = 72_057_594_037_927_934 ~= 64 petabytes`
/// * __32-bit:__ `(2 ^ (3 * 8)) - 2 = 16_777_214             ~= 16 megabytes`
///
/// Practically speaking, on a 64-bit architecture we'll never need to store the capacity on the
/// heap, because with it's impossible to create a string that is 64 petabytes or larger. But for
/// 32-bit architectures we need to be able to store a capacity larger than 16 megabytes, since a
/// string larger than 16 megabytes probably isn't that uncommon.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Capacity {
    _buf: [u8; USIZE_SIZE],
}

impl Capacity {
    pub const fn new(capacity: usize) -> Result<Self, Self> {
        if capacity > MAX_VALUE {
            // if we need the last byte to encode this capacity then we need to put the capacity on
            // the heap. return an Error so `BoxString` can do the right thing
            Err(Capacity {
                _buf: CAPACITY_IS_ON_THE_HEAP,
            })
        } else {
            let mut bytes = capacity.to_le_bytes();
            // otherwise, we can store this capacity inline! Set the last byte to be our `HEAP_MASK`
            // for our discriminant, using the leading bytes to store the actual value
            bytes[core::mem::size_of::<usize>() - 1] = HEAP_MASK;
            Ok(Capacity { _buf: bytes })
        }
    }

    #[allow(dead_code)]
    pub fn as_usize(&self) -> Result<usize, ()> {
        if self._buf == CAPACITY_IS_ON_THE_HEAP {
            Err(())
        } else {
            let mut usize_buf = [0u8; USIZE_SIZE];
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self._buf.as_ptr(),
                    usize_buf.as_mut_ptr(),
                    SPACE_FOR_CAPACITY,
                );
            }
            Ok(usize::from_le_bytes(usize_buf))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Capacity;

    #[test]
    fn test_zero_roundtrips() {
        let og = 0;
        let cap = Capacity::new(og).unwrap();
        let after = cap.as_usize().unwrap();

        assert_eq!(og, after);
    }

    #[test]
    fn test_max_value() {
        let available_bytes = (core::mem::size_of::<usize>() - 1) as u32;
        let max_value = 2usize.pow(available_bytes * 8) - 2;

        #[cfg(target_pointer_width = "64")]
        assert_eq!(max_value, 72057594037927934);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(max_value, 16777214);

        let cap = Capacity::new(max_value).unwrap();
        let after = cap.as_usize().unwrap();

        assert_eq!(max_value, after);
    }

    #[test]
    fn test_first_invalid_value() {
        let available_bytes = (core::mem::size_of::<usize>() - 1) as u32;
        let first_invalid = 2usize.pow(available_bytes * 8) - 1;

        #[cfg(target_pointer_width = "64")]
        assert_eq!(first_invalid, 72057594037927935);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(first_invalid, 16777215);

        assert!(Capacity::new(first_invalid).is_err());
    }

    #[test]
    fn test_usize_max_fails() {
        let og = usize::MAX;
        assert!(Capacity::new(og).is_err());
    }
}

crate::asserts::assert_size_eq!(Capacity, usize);
