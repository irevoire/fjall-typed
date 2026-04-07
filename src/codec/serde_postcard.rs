use std::marker::PhantomData;

use fjall::Slice;
use serde::{de::DeserializeOwned, Serialize};

use crate::codec::{Decode, Encode};

/// Encode a struct as [`postcard`] through the [`serde::Serialize`] and [`serde::Deserialize`] traits.
pub struct SerdePostcard<T>(PhantomData<T>);

impl<T: Serialize> Encode for SerdePostcard<T> {
    type Item = T;
    type Error = postcard::Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let buf = postcard::to_allocvec(item)?;
        Ok(buf.into())
    }
}

impl<T: DeserializeOwned> Decode for SerdePostcard<T> {
    type Item = T;
    type Error = postcard::Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        postcard::from_bytes(&bytes)
    }
}
