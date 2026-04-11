#![warn(missing_docs)]

//! Fjall is a log-structured embeddable key-value storage engine written in Rust.
//! Its API is similar to a `BTreeMap<[u8], [u8]>`, and this crate is a wrapper around
//! its [`Keyspace`] and [`OptimisticTxKeyspace`] that lets you type the keys and values you want to use.
//! It works by letting you specify a [`codec`] for the key and value. The codec describes the way the
//! data will be laid out on disk.
//!
//! See [`fjall`] for more info on the database.

use std::{
    convert::Infallible,
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

/// All the default codecs.
pub mod codec;
mod error;
mod keyspace;
mod optimistic_tx_keyspace;

pub use error::Error;
use fjall::Readable;
pub use keyspace::Keyspace;
pub use optimistic_tx_keyspace::OptimisticTxKeyspace;

use crate::codec::{Decode, Encode};

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

/// A typed version of [`fjall::Readable`], see the original documentation for more infos.
pub trait TypedReadable: fjall::Readable {
    /// A typed version of [`fjall::Readable::get`], see the original documentation for more infos.
    fn get<'a, Key: Encode, Value: Decode>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        key: &Key::Item,
    ) -> Result<Option<Value::Item>, Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        match fjall::Readable::get(self, &**keyspace.as_ref(), key) {
            Ok(Some(value)) => Value::decode(value).map(Some).map_err(Error::Value),
            Ok(None) => Ok(None),
            Err(e) => Err(Error::Fjall(e)),
        }
    }

    /// A typed version of [`fjall::Readable::contains_key`], see the original documentation for more infos.
    fn contains_key<'a, Key: Encode, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        key: &Key::Item,
    ) -> Result<bool, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        fjall::Readable::contains_key(self, &**keyspace.as_ref(), key).map_err(Error::Fjall)
    }

    /// A typed version of [`fjall::Readable::first_key_value`], see the original documentation for more infos.
    fn first_key_value<'a, Key, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
    ) -> Option<Guard<Key, Value>> {
        fjall::Readable::first_key_value(self, &**keyspace.as_ref()).map(Guard::new)
    }

    /// A typed version of [`fjall::Readable::last_key_value`], see the original documentation for more infos.
    fn last_key_value<'a, Key, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
    ) -> Option<Guard<Key, Value>> {
        fjall::Readable::last_key_value(self, &**keyspace.as_ref()).map(Guard::new)
    }

    /// A typed version of [`fjall::Readable::size_of`], see the original documentation for more infos.
    fn size_of<'a, Key: Encode, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        key: &Key::Item,
    ) -> Result<Option<u32>, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        fjall::Readable::size_of(self, &**keyspace.as_ref(), key).map_err(Error::Fjall)
    }

    /// A typed version of [`fjall::Readable::iter`], see the original documentation for more infos.
    fn iter<'a, Key, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
    ) -> Iter<Key, Value> {
        Iter::new(fjall::Readable::iter(self, &**keyspace.as_ref()))
    }

    /// A typed version of [`fjall::Readable::range`], see the original documentation for more infos.
    fn range<'a, Key: Encode, Value, R: RangeBounds<Key::Item>>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        range: R,
    ) -> Result<Iter<Key, Value>, Key::Error> {
        let start = match range.start_bound() {
            Bound::Included(key) => Bound::Excluded(Key::encode(key)?),
            Bound::Excluded(key) => Bound::Included(Key::encode(key)?),
            Bound::Unbounded => Bound::Unbounded,
        };
        let end = match range.end_bound() {
            Bound::Included(key) => Bound::Excluded(Key::encode(key)?),
            Bound::Excluded(key) => Bound::Included(Key::encode(key)?),
            Bound::Unbounded => Bound::Unbounded,
        };

        Ok(Iter::new(fjall::Readable::range(
            self,
            &**keyspace.as_ref(),
            (start, end),
        )))
    }

    /// A typed version of [`fjall::Readable::prefix`], see the original documentation for more infos.
    fn prefix<'a, Key: Encode, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        prefix: &Key::Item,
    ) -> Result<Iter<Key, Value>, Key::Error> {
        let prefix = Key::encode(prefix)?;

        Ok(Iter::new(fjall::Readable::prefix(
            self,
            &**keyspace.as_ref(),
            prefix,
        )))
    }

    /// A typed version of [`fjall::Readable::is_empty`], see the original documentation for more infos.
    fn is_empty<'a, Key, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
    ) -> Result<bool, fjall::Error> {
        fjall::Readable::is_empty(self, &**keyspace.as_ref())
    }

    /// A typed version of [`fjall::Readable::len`], see the original documentation for more infos.
    fn len<'a, Key, Value>(
        &self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
    ) -> Result<usize, fjall::Error> {
        fjall::Readable::len(self, &**keyspace.as_ref())
    }
}

impl<T> TypedReadable for T where T: Readable {}
