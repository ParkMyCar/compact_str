use static_assertions::assert_eq_size;
use std::{
    mem,
    sync::Arc,
};

use super::MAX_SIZE;
use crate::metadata::Metadata;

const PADDING_SIZE: usize = MAX_SIZE - mem::size_of::<Arc<str>>() - mem::size_of::<Metadata>();
const PADDING: [u8; PADDING_SIZE] = [0; PADDING_SIZE];

#[repr(C)]
#[derive(Debug, Clone)]
pub struct HeapString {
    pub metadata: Metadata,
    pub padding: [u8; PADDING_SIZE],
    pub string: Arc<str>,
}

impl HeapString {
    pub fn new(text: &str) -> Self {
        let metadata = Metadata::new_heap();
        let padding = PADDING;
        let string = text.into();

        HeapString {
            metadata,
            padding,
            string,
        }
    }
}

assert_eq_size!(HeapString, String);
