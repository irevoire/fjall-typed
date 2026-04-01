use std::{convert::Infallible, string::FromUtf8Error};

use fjall::Slice;

use crate::{Decode, Encode};

pub struct Str {}

impl Encode for Str {
    type Item = str;
    type Error = Infallible;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        Ok(Slice::new(&item.as_bytes()))
    }
}

impl Decode for Str {
    type Item = String;
    type Error = FromUtf8Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        String::from_utf8(bytes.to_vec())
    }
}
