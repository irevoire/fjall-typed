use std::{
    borrow::Cow,
    convert::Infallible,
    marker::PhantomData,
    ops::{Bound, Deref, RangeBounds},
};

use crate::{
    codec::{Decode, Encode},
    Error, Guard, Iter,
};

/// Typed keyspace. See [`fjall::Keyspace`] for more informations on what is a keyspace.
/// Contrarily to the original fjall::Keyspace, this one is typed.
/// You must specify a codec for the key and value.
/// ```no_run
/// # let db: fjall::Database = todo!();
/// use fjall_typed::Keyspace;
/// use fjall_typed::codec::{U8, Str};
///
/// let ks = db
///   .keyspace("my_items", fjall::KeyspaceCreateOptions::default)
///   .unwrap();
/// // Here we wrap the original keyspace into a fjall_typed keyspace.
/// // This one indicates that it maps `u8` with strings.
/// let ks = Keyspace::<U8, Str>::new(ks);
/// ```
#[repr(transparent)]
pub struct Keyspace<'a, Key, Value>(Cow<'a, fjall::Keyspace>, PhantomData<(Key, Value)>);

impl<'a, Key, Value> Clone for Keyspace<'a, Key, Value> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<'a, Key, Value> Deref for Keyspace<'a, Key, Value> {
    type Target = fjall::Keyspace;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, Key, Value> Keyspace<'a, Key, Value> {
    /// Create a typed keyspace from a fjall::Keyspace.
    /// ```no_run
    /// # let db: fjall::Database = todo!();
    /// use fjall_typed::Keyspace;
    /// use fjall_typed::codec::{U8, Str};
    ///
    /// let ks = db
    ///   .keyspace("my_items", fjall::KeyspaceCreateOptions::default)
    ///   .unwrap();
    /// // Here we wrap the original keyspace into a fjall_typed keyspace.
    /// // This one indicates that it maps `u8` with strings.
    /// let ks = Keyspace::<U8, Str>::new(ks);
    /// ```
    pub fn new(ks: fjall::Keyspace) -> Self {
        Self(Cow::Owned(ks), PhantomData)
    }

    /// Clone the inner `Arc` to decorelate this keyspace from the one it's been derived from.
    /// Can be useful if you're storing a keyspace after remapping one of its type.
    pub fn to_owned(self) -> Keyspace<'static, Key, Value> {
        Keyspace::new(self.0.into_owned())
    }

    /// Change the codec of the key.
    /// If you want to store this new keyspace, call [`Self::to_owned()`].
    ///
    /// ```no_run
    /// # let ks = todo!();
    /// use fjall_typed::Keyspace;
    /// use fjall_typed::codec::{U8, Str, I8};
    ///
    /// let ks = Keyspace::<U8, Str>::new(ks);
    /// let ks = ks.remap_key::<I8>();
    /// let s: String = ks.get(&-2).unwrap().unwrap();
    /// ```
    pub fn remap_key<NK>(&'a self) -> Keyspace<'a, NK, Value> {
        Keyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    /// Change the codec of the value.
    /// If you want to store this new keyspace, call [`Self::to_owned()`].
    ///
    /// ```no_run
    /// # let ks = todo!();
    /// use fjall_typed::Keyspace;
    /// use fjall_typed::codec::{U8, Str, Unit};
    ///
    /// let ks = Keyspace::<U8, Str>::new(ks);
    /// let ks = ks.remap_value::<Unit>();
    /// let s: () = ks.get(&2).unwrap().unwrap();
    /// ```
    pub fn remap_value<NV>(&'a self) -> Keyspace<'a, Key, NV> {
        Keyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    /// Change the codec of the key and value.
    /// If you want to store this new keyspace, call [`Self::to_owned()`].
    ///
    /// ```no_run
    /// # let ks = todo!();
    /// use fjall_typed::Keyspace;
    /// use fjall_typed::codec::{U8, Str, I8, Unit};
    ///
    /// let ks = Keyspace::<U8, Str>::new(ks);
    /// let ks = ks.remap_key_value::<I8, Unit>();
    /// let s: () = ks.get(&-2).unwrap().unwrap();
    /// ```
    pub fn remap_key_value<NK, NV>(&'a self) -> Keyspace<'a, NK, NV> {
        Keyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    /*
       #[inline]
       #[doc = "This function has the same semantics as [`fjall::Keyspace::start_ingestion`]."]
       pub fn start_ingestion(&self) -> Result<Ingestion<'_>, fjall::Error> {
           self.0.start_ingestion()
       }
    */

    #[must_use]
    #[expect(clippy::iter_without_into_iter)]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::iter`]."]
    pub fn iter(&self) -> Iter<Key, Value> {
        Iter::new(self.0.iter())
    }

    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::first_key_value`]."]
    pub fn first_key_value(&self) -> Option<Guard<Key, Value>> {
        self.0.first_key_value().map(Guard::new)
    }

    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::last_key_value`]."]
    pub fn last_key_value(&self) -> Option<Guard<Key, Value>> {
        self.0.last_key_value().map(Guard::new)
    }
}

impl<'a, Key: Encode, Value: Encode> Keyspace<'a, Key, Value> {
    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::insert`]."]
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

impl<'a, Key: Encode, Value: Decode> Keyspace<'a, Key, Value> {
    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::get`]."]
    pub fn get(
        &self,
        key: &Key::Item,
    ) -> Result<Option<Value::Item>, Error<Key::Error, Value::Error>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        match self.0.get(key) {
            Ok(None) => Ok(None),
            Ok(Some(bytes)) => Ok(Some(Value::decode(bytes).map_err(Error::Value)?)),
            Err(err) => Err(Error::Fjall(err)),
        }
    }
}

impl<'a, Key: Encode, Value> Keyspace<'a, Key, Value> {
    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::contains_key`]."]
    pub fn contains_key(&self, key: &Key::Item) -> Result<bool, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.contains_key(key).map_err(Error::Fjall)
    }

    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::size_of`]."]
    pub fn size_of(&self, key: &Key::Item) -> Result<Option<u32>, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.size_of(key).map_err(Error::Fjall)
    }

    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::range`]."]
    pub fn range<R: RangeBounds<Key::Item>>(
        &self,
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

        Ok(Iter::new(self.0.range((start, end))))
    }

    #[must_use]
    #[inline]
    #[doc = "This function has the same semantics as [`fjall::Keyspace::prefix`]."]
    pub fn prefix(&self, prefix: &Key::Item) -> Result<Iter<Key, Value>, Key::Error> {
        let prefix = Key::encode(prefix)?;
        Ok(Iter::new(self.0.prefix(prefix)))
    }
}
