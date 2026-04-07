use std::marker::PhantomData;

use fjall::Slice;
use serde::{de::DeserializeOwned, Serialize};

use crate::codec::{Decode, Encode};

/// Encode a struct as [`postcard`] through the [`serde::Serialize`] and [`serde::Deserialize`] traits.
pub struct SerdeMsgpack<T>(PhantomData<T>);

impl<T: Serialize> Encode for SerdeMsgpack<T> {
    type Item = T;
    type Error = rmp_serde::Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let buf = rmp_serde::to_vec(item)?;
        Ok(buf.into())
    }
}

impl<T: DeserializeOwned> Decode for SerdeMsgpack<T> {
    type Item = T;
    type Error = rmp_serde::Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        rmp_serde::from_bytes(&bytes)
    }
}
