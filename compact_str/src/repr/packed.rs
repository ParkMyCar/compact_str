use super::MAX_SIZE;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PackedString {
    buffer: [u8; MAX_SIZE],
}

impl PackedString {
    #[inline]
    pub fn new(text: &str) -> Self {
        debug_assert_eq!(text.len(), MAX_SIZE);
        debug_assert!(text.as_bytes()[0] <= 127);

        let mut buffer = [0u8; MAX_SIZE];
        buffer[..text.len()].copy_from_slice(text.as_bytes());

        PackedString { buffer }
    }

    #[inline]
    pub const fn new_const(text: &str) -> Self {
        if text.len() != MAX_SIZE {
            // HACK: This allows us to make assertions within a `const fn` without requiring nightly,
            // see unstable `const_panic` feature. This results in a build failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["Provided string has a length greater than MAX_SIZE!"][42];
        }
        if text.as_bytes()[0] > 127 {
            // HACK: This allows us to make assertions within a `const fn` without requiring nightly,
            // see unstable `const_panic` feature. This results in a build failure, not a runtime panic
            #[allow(clippy::no_effect)]
            #[allow(unconditional_panic)]
            ["leading character of packed string isn't ASCII!"][42];
        }

        let mut buffer = [0u8; MAX_SIZE];
        let mut i = 0;
        while i < text.len() {
            buffer[i] = text.as_bytes()[i];
            i += 1;
        }

        PackedString { buffer }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: You can only construct a PackedString via a &str
        unsafe { ::std::str::from_utf8_unchecked(&self.buffer) }
    }
}

static_assertions::assert_eq_size!(PackedString, String);
