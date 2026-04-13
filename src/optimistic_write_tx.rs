use std::{convert::Infallible, ops::Deref};

use fjall::{Conflict, PersistMode};

use crate::{
    codec::{Decode, Encode},
    Error, Keyspace,
};

/// Wrapper around [`fjall::OptimisticWriteTx`].
#[repr(transparent)]
pub struct OptimisticWriteTx(pub fjall::OptimisticWriteTx);

impl OptimisticWriteTx {
    /// Create a new `Self` from an original [`fjall::OptimisticWriteTx`].
    pub fn new(this: fjall::OptimisticWriteTx) -> Self {
        Self(this)
    }

    /// See [`fjall::OptimisticWriteTx::durability`].
    pub fn durability(self, mode: Option<PersistMode>) -> Self {
        Self(self.0.durability(mode))
    }

    /// See [`fjall::OptimisticWriteTx::commit`].
    pub fn commit(self) -> Result<Result<(), Conflict>, fjall::Error> {
        self.0.commit()
    }

    /// See [`fjall::OptimisticWriteTx::take`].
    pub fn take<'a, Key: Encode, Value: Decode>(
        &mut self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        key: &Key::Item,
    ) -> Result<Option<Value::Item>, Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        match self
            .0
            .take(&**keyspace.as_ref(), key)
            .map_err(Error::Fjall)?
        {
            None => Ok(None),
            Some(value) => Value::decode(value).map(Some).map_err(Error::Value),
        }
    }

    /// See [`fjall::OptimisticWriteTx::insert`].
    pub fn insert<'a, Key: Encode, Value: Encode>(
        &mut self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        key: &Key::Item,
        value: &Value::Item,
    ) -> Result<(), Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        let value = Value::encode(value).map_err(Error::Value)?;
        self.0.insert(&**keyspace.as_ref(), key, value);
        Ok(())
    }

    /// See [`fjall::OptimisticWriteTx::remove`].
    pub fn remove<'a, Key: Encode, Value>(
        &mut self,
        keyspace: impl AsRef<Keyspace<'a, Key, Value>>,
        key: &Key::Item,
    ) -> Result<(), Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.remove(&**keyspace.as_ref(), key);
        Ok(())
    }
}

impl Deref for OptimisticWriteTx {
    type Target = fjall::OptimisticWriteTx;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fjall::Readable for OptimisticWriteTx {
    fn get<K: AsRef<[u8]>>(
        &self,
        keyspace: impl AsRef<fjall::Keyspace>,
        key: K,
    ) -> fjall::Result<Option<fjall::UserValue>> {
        fjall::Readable::get(&self.0, keyspace, key)
    }

    fn contains_key<K: AsRef<[u8]>>(
        &self,
        keyspace: impl AsRef<fjall::Keyspace>,
        key: K,
    ) -> fjall::Result<bool> {
        fjall::Readable::contains_key(&self.0, keyspace, key)
    }

    fn first_key_value(&self, keyspace: impl AsRef<fjall::Keyspace>) -> Option<fjall::Guard> {
        fjall::Readable::first_key_value(&self.0, keyspace)
    }

    fn last_key_value(&self, keyspace: impl AsRef<fjall::Keyspace>) -> Option<fjall::Guard> {
        fjall::Readable::last_key_value(&self.0, keyspace)
    }

    fn size_of<K: AsRef<[u8]>>(
        &self,
        keyspace: impl AsRef<fjall::Keyspace>,
        key: K,
    ) -> fjall::Result<Option<u32>> {
        fjall::Readable::size_of(&self.0, keyspace, key)
    }

    fn iter(&self, keyspace: impl AsRef<fjall::Keyspace>) -> fjall::Iter {
        fjall::Readable::iter(&self.0, keyspace)
    }

    fn range<K: AsRef<[u8]>, R: std::ops::RangeBounds<K>>(
        &self,
        keyspace: impl AsRef<fjall::Keyspace>,
        range: R,
    ) -> fjall::Iter {
        fjall::Readable::range(&self.0, keyspace, range)
    }

    fn prefix<K: AsRef<[u8]>>(
        &self,
        keyspace: impl AsRef<fjall::Keyspace>,
        prefix: K,
    ) -> fjall::Iter {
        fjall::Readable::prefix(&self.0, keyspace, prefix)
    }
}
