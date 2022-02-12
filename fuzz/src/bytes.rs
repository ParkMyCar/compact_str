use std::ops::Deref;

use arbitrary::Arbitrary;
use bytes::Buf;

/// A non contiguous collection of bytes
#[derive(Arbitrary, Debug)]
pub struct NonContiguous<'a>(Vec<&'a [u8]>);

impl Buf for NonContiguous<'_> {
    fn remaining(&self) -> usize {
        self.0.iter().fold(0, |acc, slice| acc + slice.len())
    }

    fn chunk(&self) -> &[u8] {
        todo!()
    }

    fn advance(&mut self, cnt: usize) {
        todo!()
    }
}
