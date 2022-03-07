use crate::repr::HEAP_MASK;

// how many bytes a `usize` occupies
const USIZE_SIZE: usize = core::mem::size_of::<usize>();

const SPACE_FOR_CAPACITY: usize = USIZE_SIZE - 1;
// state that describes the capacity as being stored on the heap
const CAPACITY_IS_ON_THE_HEAP: [u8; USIZE_SIZE] = [0b11111111; USIZE_SIZE];

const FIRST_INVALID_VALUE: usize = (1 << SPACE_FOR_CAPACITY * 8) - 1;

#[derive(Debug, PartialEq, Eq)]
pub struct Capacity {
    _buf: [u8; USIZE_SIZE],
}

impl Capacity {
    pub const fn new(capacity: usize) -> Result<Self, Self> {
        if capacity >= FIRST_INVALID_VALUE {
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
