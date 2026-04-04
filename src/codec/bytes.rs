use std::convert::Infallible;

use fjall::Slice;

use crate::codec::{Decode, Encode};

pub enum Bytes {}

impl Encode for Bytes {
    type Item = [u8];
    type Error = Infallible;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        Ok(Slice::new(item))
    }
}

impl Decode for Bytes {
    type Item = Slice;
    type Error = Infallible;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        Ok(bytes)
    }
}
