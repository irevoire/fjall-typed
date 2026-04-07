use std::marker::PhantomData;

use fjall::Slice;
use serde::{de::DeserializeOwned, Serialize};

use crate::codec::{Decode, Encode};

pub struct SerdeJson<T>(PhantomData<T>);

impl<T: Serialize> Encode for SerdeJson<T> {
    type Item = T;
    type Error = serde_json::Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let buf = serde_json::to_vec(item)?;
        Ok(buf.into())
    }
}

impl<T: DeserializeOwned> Decode for SerdeJson<T> {
    type Item = T;
    type Error = serde_json::Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        serde_json::from_slice(&bytes)
    }
}
