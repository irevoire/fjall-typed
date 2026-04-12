use std::{borrow::Cow, convert::Infallible, marker::PhantomData, ops::Deref};

use crate::{
    codec::{Decode, Encode},
    Error, Guard, Keyspace,
};

/// Typed optimistic tx keyspace. See [`fjall::OptimisticTxKeyspace`] for more informations on what is a keyspace.
/// Contrarily to the original fjall::OptimisticTxKeyspace, this one is typed.
/// You must specify a codec for the key and value.
/// ```no_run
/// # let db: fjall::OptimisticTxDatabase = todo!();
/// use fjall_typed::OptimisticTxKeyspace;
/// use fjall_typed::codec::{U8, Str};
///
/// let ks = db
///   .keyspace("my_items", fjall::KeyspaceCreateOptions::default)
///   .unwrap();
/// // Here we wrap the original keyspace into a fjall_typed keyspace.
/// // This one indicates that it maps `u8` with strings.
/// let ks = OptimisticTxKeyspace::<U8, Str>::new(ks);
/// ```
#[repr(transparent)]
pub struct OptimisticTxKeyspace<'a, Key, Value>(
    Cow<'a, fjall::OptimisticTxKeyspace>,
    PhantomData<(Key, Value)>,
);

impl<'a, Key, Value> Clone for OptimisticTxKeyspace<'a, Key, Value> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<'a, Key, Value> Deref for OptimisticTxKeyspace<'a, Key, Value> {
    type Target = fjall::OptimisticTxKeyspace;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, Key, Value> OptimisticTxKeyspace<'a, Key, Value> {
    /// Create a typed keyspace from a [`fjall::OptimisticTxDatabase`].
    /// ```no_run
    /// # let db: fjall::OptimisticTxDatabase = todo!();
    /// use fjall_typed::OptimisticTxKeyspace;
    /// use fjall_typed::codec::{U8, Str};
    ///
    /// let ks = db
    ///   .keyspace("my_items", fjall::KeyspaceCreateOptions::default)
    ///   .unwrap();
    /// // Here we wrap the original keyspace into a fjall_typed keyspace.
    /// // This one indicates that it maps `u8` with strings.
    /// let ks = OptimisticTxKeyspace::<U8, Str>::new(ks);
    /// ```
    #[must_use]
    #[inline]
    pub fn new(ks: fjall::OptimisticTxKeyspace) -> Self {
        Self(Cow::Owned(ks), PhantomData)
    }

    /// Return the inner [`Keyspace`].
    pub fn as_keyspace<'b>(&'b self) -> Keyspace<'b, Key, Value> {
        let ks = self.0.as_ref();
        Keyspace(Cow::Borrowed(ks.as_ref()), PhantomData)
    }

    /// Clone the inner `Arc` to decorelate this keyspace from the one it's been derived from.
    /// Can be useful if you're storing a keyspace after remapping one of its type.
    pub fn to_owned(self) -> OptimisticTxKeyspace<'static, Key, Value> {
        OptimisticTxKeyspace::new(self.0.into_owned())
    }

    /// Change the codec of the key.
    /// If you want to store this new keyspace, call [`Self::to_owned()`].
    ///
    /// ```no_run
    /// # let ks = todo!();
    /// use fjall_typed::OptimisticTxKeyspace;
    /// use fjall_typed::codec::{U8, Str, I8};
    ///
    /// let ks = OptimisticTxKeyspace::<U8, Str>::new(ks);
    /// let ks = ks.remap_key::<I8>();
    /// let s: String = ks.get(&-2).unwrap().unwrap();
    /// ```
    #[must_use]
    #[inline]
    pub fn remap_key<NK>(&'a self) -> OptimisticTxKeyspace<'a, NK, Value> {
        OptimisticTxKeyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    /// Change the codec of the value.
    /// If you want to store this new keyspace, call [`Self::to_owned()`].
    ///
    /// ```no_run
    /// # let ks = todo!();
    /// use fjall_typed::OptimisticTxKeyspace;
    /// use fjall_typed::codec::{U8, Str, Unit};
    ///
    /// let ks = OptimisticTxKeyspace::<U8, Str>::new(ks);
    /// let ks = ks.remap_value::<Unit>();
    /// let s: () = ks.get(&2).unwrap().unwrap();
    /// ```
    #[must_use]
    #[inline]
    pub fn remap_value<NV>(&'a self) -> OptimisticTxKeyspace<'a, Key, NV> {
        OptimisticTxKeyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    /// Change the codec of the key and value.
    /// If you want to store this new keyspace, call [`Self::to_owned()`].
    ///
    /// ```no_run
    /// # let ks = todo!();
    /// use fjall_typed::OptimisticTxKeyspace;
    /// use fjall_typed::codec::{U8, Str, I8, Unit};
    ///
    /// let ks = OptimisticTxKeyspace::<U8, Str>::new(ks);
    /// let ks = ks.remap_key_value::<I8, Unit>();
    /// let s: () = ks.get(&-2).unwrap().unwrap();
    /// ```
    #[must_use]
    #[inline]
    pub fn remap_key_value<NK, NV>(&'a self) -> OptimisticTxKeyspace<'a, NK, NV> {
        OptimisticTxKeyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }
}

impl<'a, Key: Encode, Value: Encode> OptimisticTxKeyspace<'a, Key, Value> {
    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::insert`].
    #[must_use]
    #[inline]
    pub fn insert(
        &self,
        key: &Key::Item,
        value: &Value::Item,
    ) -> Result<(), Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        let value = Value::encode(value).map_err(Error::Value)?;

        self.0.insert(key, value).map_err(Error::Fjall)
    }
}

impl<'a, Key: Encode, Value> OptimisticTxKeyspace<'a, Key, Value> {
    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::contains_key`].
    #[must_use]
    #[inline]
    pub fn contains_key(&self, key: &Key::Item) -> Result<bool, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.contains_key(key).map_err(Error::Fjall)
    }

    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::size_of`].
    #[must_use]
    #[inline]
    pub fn size_of(&self, key: &Key::Item) -> Result<Option<u32>, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.size_of(key).map_err(Error::Fjall)
    }

    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::remove`].
    #[must_use]
    #[inline]
    pub fn remove(&self, key: &Key::Item) -> Result<(), Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.remove(key).map_err(Error::Fjall)
    }
}

impl<'a, Key, Value: Decode> OptimisticTxKeyspace<'a, Key, Value> {
    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::first_key_value`].
    #[must_use]
    #[inline]
    pub fn first_key_value(&self) -> Option<Guard<Key, Value>> {
        self.0.first_key_value().map(Guard::new)
    }

    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::last_key_value`].
    #[must_use]
    #[inline]
    pub fn last_key_value(&self) -> Option<Guard<Key, Value>> {
        self.0.last_key_value().map(Guard::new)
    }
}

impl<'a, Key: Encode, Value: Decode> OptimisticTxKeyspace<'a, Key, Value> {
    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::get`].
    #[must_use]
    #[inline]
    pub fn get(
        &self,
        key: &Key::Item,
    ) -> Result<Option<Value::Item>, Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        match self.0.get(key).map_err(Error::Fjall)? {
            Some(value) => Value::decode(value).map(Some).map_err(Error::Value),
            None => Ok(None),
        }
    }

    /// This function has the same semantics of [`fjall::OptimisticTxKeyspace::take`].
    #[must_use]
    #[inline]
    pub fn take(
        &self,
        key: &Key::Item,
    ) -> Result<Option<Value::Item>, Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        match self.0.take(key).map_err(Error::Fjall)? {
            Some(value) => Value::decode(value).map(Some).map_err(Error::Value),
            None => Ok(None),
        }
    }
}
