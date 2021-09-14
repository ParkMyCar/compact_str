use static_assertions::assert_eq_size;
use std::{mem, sync::Arc};

use super::MAX_SIZE;

const PADDING_SIZE: usize = MAX_SIZE - mem::size_of::<Arc<str>>();
const PADDING: [u8; PADDING_SIZE] = [0b11111111; PADDING_SIZE];

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeapString {
    pub padding: [u8; PADDING_SIZE],
    pub string: Arc<str>,
}

impl HeapString {
    pub fn new(text: &str) -> Self {
        let padding = PADDING;
        let string = text.into();

        HeapString {
            padding,
            string,
        }
    }
}

assert_eq_size!(HeapString, String);
