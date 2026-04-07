use std::marker::PhantomData;

use facet::Facet;
use fjall::Slice;

use crate::codec::{Decode, Encode};

/// Encode a struct as json through the [`facet::Facet`] trait.
pub struct FacetPostcard<T>(PhantomData<T>);

impl<T: Facet<'static>> Encode for FacetPostcard<T> {
    type Item = T;
    type Error = facet_postcard::SerializeError;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error> {
        let buf = facet_postcard::to_vec(item)?;
        Ok(buf.into())
    }
}

impl<T: Facet<'static>> Decode for FacetPostcard<T> {
    type Item = T;
    type Error = facet_postcard::DeserializeError;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error> {
        facet_postcard::from_slice(&bytes)
    }
}
