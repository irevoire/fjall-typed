use std::{borrow::Cow, convert::Infallible, marker::PhantomData, path::PathBuf};

use fjall::{Slice, UserKey, UserValue};

use crate::{
    codec::{Decode, Encode},
    Error, Guard,
};

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

impl<'a, Key, Value> OptimisticTxKeyspace<'a, Key, Value> {
    pub fn new(ks: fjall::OptimisticTxKeyspace) -> Self {
        Self(Cow::Owned(ks), PhantomData)
    }

    pub fn remap_key<NK>(&'a self) -> OptimisticTxKeyspace<'a, NK, Value> {
        OptimisticTxKeyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    pub fn remap_value<NV>(&'a self) -> OptimisticTxKeyspace<'a, Key, NV> {
        OptimisticTxKeyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    pub fn remap_key_value<NK, NV>(&'a self) -> OptimisticTxKeyspace<'a, NK, NV> {
        OptimisticTxKeyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    /// Returns the underlying LSM-tree's path.
    #[must_use]
    pub fn path(&self) -> PathBuf {
        self.0.path()
    }

    /// Approximates the amount of items in the keyspace.
    ///
    /// For update- or delete-heavy workloads, this value will
    /// diverge from the real value, but is a O(1) operation.
    ///
    /// For insert-only workloads (e.g. logs, time series)
    /// this value is reliable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// assert_eq!(tree.approximate_len(), 0);
    ///
    /// tree.insert("1", "abc")?;
    /// assert_eq!(tree.approximate_len(), 1);
    ///
    /// tree.remove("1")?;
    /// // Oops! approximate_len will not be reliable here
    /// assert_eq!(tree.approximate_len(), 2);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[must_use]
    pub fn approximate_len(&self) -> usize {
        self.0.approximate_len()
    }

    /// Atomically updates an item and returns the previous value.
    ///
    /// Returning `None` removes the item if it existed before.
    ///
    /// The operation will run wrapped in a transaction.
    ///
    /// # Note
    ///
    /// The provided closure can be called multiple times as this function
    /// automatically retries on conflict. Since this is an `FnMut`, make sure
    /// it is idempotent and will not cause side-effects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, Slice, KeyspaceCreateOptions};
    /// # use std::sync::Arc;
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// let prev = tree.fetch_update("a", |_| Some(Slice::from(*b"def")))?.unwrap();
    /// assert_eq!(b"abc", &*prev);
    ///
    /// let item = tree.get("a")?;
    /// assert_eq!(Some("def".as_bytes().into()), item);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// # use std::sync::Arc;
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// let prev = tree.fetch_update("a", |_| None)?.unwrap();
    /// assert_eq!(b"abc", &*prev);
    ///
    /// let item = tree.get("a")?;
    /// assert!(item.is_none());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    pub fn fetch_update<K: Into<UserKey>, F: FnMut(Option<&UserValue>) -> Option<UserValue>>(
        &self,
        key: K,
        f: F,
    ) -> Result<Option<UserValue>, fjall::Error> {
        self.0.fetch_update(key, f)
    }

    /// Atomically updates an item and returns the new value.
    ///
    /// Returning `None` removes the item if it existed before.
    ///
    /// The operation will run wrapped in a transaction.
    ///
    /// # Note
    ///
    /// The provided closure can be called multiple times as this function
    /// automatically retries on conflict. Since this is an `FnMut`, make sure
    /// it is idempotent and will not cause side-effects.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, Slice, KeyspaceCreateOptions};
    /// # use std::sync::Arc;
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// let updated = tree.update_fetch("a", |_| Some(Slice::from(*b"def")))?.unwrap();
    /// assert_eq!(b"def", &*updated);
    ///
    /// let item = tree.get("a")?;
    /// assert_eq!(Some("def".as_bytes().into()), item);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// # use std::sync::Arc;
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// let updated = tree.update_fetch("a", |_| None)?;
    /// assert!(updated.is_none());
    ///
    /// let item = tree.get("a")?;
    /// assert!(item.is_none());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    pub fn update_fetch<K: Into<UserKey>, F: FnMut(Option<&UserValue>) -> Option<UserValue>>(
        &self,
        key: K,
        f: F,
    ) -> Result<Option<Slice>, fjall::Error> {
        self.0.update_fetch(key, f)
    }
}

impl<'a, Key: Encode, Value: Encode> OptimisticTxKeyspace<'a, Key, Value> {
    /// Inserts a key-value pair into the keyspace.
    ///
    /// Keys may be up to 65536 bytes long, values up to 2^32 bytes.
    /// Shorter keys and values result in better performance.
    ///
    /// If the key already exists, the item will be overwritten.
    ///
    /// The operation will run wrapped in a transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// assert!(!db.read_tx().is_empty(&tree)?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
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
    /// Returns `true` if the keyspace contains the specified key.
    ///
    /// The operation will run wrapped in a read snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "my_value")?;
    ///
    /// assert!(tree.contains_key("a")?);
    /// assert!(!tree.contains_key("b")?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    pub fn contains_key(&self, key: &Key::Item) -> Result<bool, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.contains_key(key).map_err(Error::Fjall)
    }

    /// Retrieves the size of an item from the keyspace.
    ///
    /// The operation will run wrapped in a read snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "my_value")?;
    ///
    /// let len = tree.size_of("a")?.unwrap_or_default();
    /// assert_eq!("my_value".len() as u32, len);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    pub fn size_of(&self, key: &Key::Item) -> Result<Option<u32>, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.size_of(key).map_err(Error::Fjall)
    }

    /// Removes an item from the keyspace.
    ///
    /// The key may be up to 65536 bytes long.
    /// Shorter keys result in better performance.
    ///
    /// The operation will run wrapped in a transaction.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    /// assert!(!db.read_tx().is_empty(&tree)?);
    ///
    /// tree.remove("a")?;
    /// assert!(db.read_tx().is_empty(&tree)?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    pub fn remove(&self, key: &Key::Item) -> Result<(), Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.remove(key).map_err(Error::Fjall)
    }
}

impl<'a, Key, Value: Decode> OptimisticTxKeyspace<'a, Key, Value> {
    /// Returns the first key-value pair in the keyspace.
    /// The key in this pair is the minimum key in the keyspace.
    ///
    /// The operation will run wrapped in a read snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "my_value")?;
    /// tree.insert("b", "my_value")?;
    ///
    /// assert_eq!(b"a", &*tree.first_key_value().unwrap().key()?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[must_use]
    pub fn first_key_value(&self) -> Option<Guard<Key, Value>> {
        self.0.first_key_value().map(Guard::new)
    }

    /// Returns the last key-value pair in the keyspace.
    /// The key in this pair is the maximum key in the keyspace.
    ///
    /// The operation will run wrapped in a read snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "my_value")?;
    /// tree.insert("b", "my_value")?;
    ///
    /// assert_eq!(b"b", &*tree.last_key_value().unwrap().key()?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[must_use]
    pub fn last_key_value(&self) -> Option<Guard<Key, Value>> {
        self.0.last_key_value().map(Guard::new)
    }
}

impl<'a, Key: Encode, Value: Decode> OptimisticTxKeyspace<'a, Key, Value> {
    /// Retrieves an item from the keyspace.
    ///
    /// The operation will run wrapped in a read snapshot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "my_value")?;
    ///
    /// let item = tree.get("a")?;
    /// assert_eq!(Some("my_value".as_bytes().into()), item);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
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

    /// Removes an item and returns its value if it existed.
    ///
    /// The operation will run wrapped in a transaction.
    ///
    /// ```
    /// # use fjall::{OptimisticTxDatabase, KeyspaceCreateOptions, Readable};
    /// # use std::sync::Arc;
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = OptimisticTxDatabase::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// let taken = tree.take("a")?.unwrap();
    /// assert_eq!(b"abc", &*taken);
    ///
    /// let item = tree.get("a")?;
    /// assert!(item.is_none());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
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
