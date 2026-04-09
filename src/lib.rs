#![warn(missing_docs)]

//! Fjall is a log-structured embeddable key-value storage engine written in Rust.
//! Its API is similar to a `BTreeMap<[u8], [u8]>`, and this crate is a wrapper around
//! its [`Keyspace`] and [`OptimisticTxKeyspace`] that lets you type the keys and values you want to use.
//! It works by letting you specify a [`codec`] for the key and value. The codec describes the way the
//! data will be laid out on disk.
//!
//! See [`fjall`] for more info on the database.

use std::{convert::Infallible, marker::PhantomData};

/// All the default codecs.
pub mod codec;
mod error;
mod keyspace;
mod optimistic_tx_keyspace;

pub use error::Error;
pub use keyspace::Keyspace;
pub use optimistic_tx_keyspace::OptimisticTxKeyspace;

/// Refer to [`fjall::Guard`] for more info.
pub struct Guard<Key, Value>(fjall::Guard, PhantomData<(Key, Value)>);

impl<Key, Value> Guard<Key, Value> {
    pub(crate) fn new(guard: fjall::Guard) -> Self {
        Self(guard, PhantomData)
    }

    /// Change the codec of the key.
    #[inline]
    #[must_use]
    pub fn remap_key_type<NKey>(self) -> Guard<NKey, Value> {
        Guard(self.0, PhantomData)
    }

    /// Change the codec of the value.
    #[inline]
    #[must_use]
    pub fn remap_value_type<NValue>(self) -> Guard<Key, NValue> {
        Guard(self.0, PhantomData)
    }

    /// Change the codec of the key and value.
    #[inline]
    #[must_use]
    pub fn remap_types<NKey, NValue>(self) -> Guard<NKey, NValue> {
        Guard(self.0, PhantomData)
    }

    /// For more informations, refer to [`fjall::Guard::size`].
    #[inline]
    #[must_use]
    pub fn size(self) -> Result<u32, fjall::Error> {
        self.0.size()
    }
}

impl<Key: codec::Decode, Value> Guard<Key, Value> {
    /// For more informations, refer to [`fjall::Guard::key`].
    #[inline]
    #[must_use]
    pub fn key(self) -> Result<Key::Item, Error<Key::Error, Infallible>> {
        let key = self.0.key().map_err(Error::Fjall)?;
        Key::decode(key).map_err(Error::Key)
    }
}

impl<Key, Value: codec::Decode> Guard<Key, Value> {
    /// For more informations, refer to [`fjall::Guard::value`].
    #[inline]
    #[must_use]
    pub fn value(self) -> Result<Value::Item, Error<Infallible, Value::Error>> {
        let value = self.0.value().map_err(Error::Fjall)?;
        Value::decode(value).map_err(Error::Value)
    }
}

impl<Key: codec::Decode, Value: codec::Decode> Guard<Key, Value> {
    /// For more informations, refer to [`fjall::Guard::into_inner`].
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> Result<(Key::Item, Value::Item), Error<Key::Error, Value::Error>> {
        let (k, v) = self.0.into_inner().map_err(Error::Fjall)?;
        Ok((
            Key::decode(k).map_err(Error::Key)?,
            Value::decode(v).map_err(Error::Value)?,
        ))
    }
}

/// Refer to [`fjall::Iter`] for more info.
pub struct Iter<Key, Value>(fjall::Iter, PhantomData<(Key, Value)>);

impl<Key, Value> Iter<Key, Value> {
    pub(crate) fn new(iter: fjall::Iter) -> Self {
        Self(iter, PhantomData)
    }

    /// Change the codec of the key.
    pub fn remap_key_type<NKey>(self) -> Iter<NKey, Value> {
        Iter(self.0, PhantomData)
    }

    /// Change the codec of the value.
    pub fn remap_value_type<NValue>(self) -> Iter<Key, NValue> {
        Iter(self.0, PhantomData)
    }

    /// Change the codec of the key and value.
    pub fn remap_types<NKey, NValue>(self) -> Iter<NKey, NValue> {
        Iter(self.0, PhantomData)
    }
}

impl<Key, Value> Iterator for Iter<Key, Value> {
    type Item = Guard<Key, Value>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(Guard::new)
    }
}

impl<Key, Value> DoubleEndedIterator for Iter<Key, Value> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(Guard::new)
    }
}
