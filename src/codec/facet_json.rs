use std::marker::PhantomData;

use facet::Facet;
use facet_format::SerializeError;
use facet_json::{DeserializeError, JsonSerializeError};
use fjall::Slice;

use crate::codec::{Decode, Encode};

pub struct FacetJson<T>(PhantomData<T>);

impl<T: Facet<'static>> Encode for FacetJson<T> {
    type Item = T;
    type Error = SerializeError<JsonSerializeError>;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let buf = facet_json::to_vec(item)?;
        Ok(buf.into())
    }
}

impl<T: Facet<'static>> Decode for FacetJson<T> {
    type Item = T;
    type Error = DeserializeError;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        facet_json::from_slice(&bytes)
    }
}
