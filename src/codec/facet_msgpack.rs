use std::marker::PhantomData;

use facet::Facet;
use fjall::Slice;

use crate::codec::{Decode, Encode};

/// Encode a struct as [msgpack](https://msgpack.org/) through the [`facet::Facet`] trait.
pub struct FacetMsgpack<T>(PhantomData<T>);

impl<T: Facet<'static>> Encode for FacetMsgpack<T> {
    type Item = T;
    type Error = facet_format::SerializeError<facet_msgpack::MsgPackSerializeError>;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let buf = facet_msgpack::to_vec(item)?;
        Ok(buf.into())
    }
}

impl<T: Facet<'static>> Decode for FacetMsgpack<T> {
    type Item = T;
    type Error = facet_msgpack::DeserializeError;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        facet_msgpack::from_slice(&bytes)
    }
}
