use std::convert::Infallible;

use fjall::Slice;

mod bytes;
pub use bytes::*;
mod lazy;
pub use lazy::*;
mod integer;
pub use integer::*;
mod str;
pub use str::*;
mod unit;
pub use unit::*;
#[cfg(feature = "facet_json")]
mod facet_json;
#[cfg(feature = "facet_json")]
pub use facet_json::*;
#[cfg(feature = "facet_postcard")]
mod facet_postcard;
#[cfg(feature = "facet_postcard")]
pub use facet_postcard::*;
#[cfg(feature = "facet_msgpack")]
mod facet_msgpack;
#[cfg(feature = "facet_msgpack")]
pub use facet_msgpack::*;
#[cfg(feature = "serde_json")]
mod serde_json;
#[cfg(feature = "serde_json")]
pub use serde_json::*;
#[cfg(feature = "serde_postcard")]
mod serde_postcard;
#[cfg(feature = "serde_postcard")]
pub use serde_postcard::*;

/// Dummy codec if you don't know yet which codec will be used
pub enum Unspecified {}

pub trait Encode {
    type Item: ?Sized;
    type Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error>;
}

pub trait Decode {
    type Item;
    type Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error>;
}

/// Convenient struct to ignore the decoding part and return the unit type instead.
/// Can be useful when working with [`crate::Guard`], but keep in mind that it still does a useless allocation.
/// Ideally, you should use the [`crate::Keyspace::contains_key`] or [`crate::Keyspace::size_of`] methods instead.
pub enum DecodeIgnore {}

impl Decode for DecodeIgnore {
    type Item = ();
    type Error = Infallible;

    fn decode(_: Slice) -> Result<Self::Item, Self::Error> {
        Ok(())
    }
}
