use std::{borrow::Cow, convert::Infallible, marker::PhantomData, ops::RangeBounds, path::Path};

use byteview::StrView;
use fjall::{Guard, Iter, Slice};

#[derive(Debug)]
pub enum Error<KeyError, ValueError> {
    Fjall(fjall::Error),
    Key(KeyError),
    Value(ValueError),
}

pub trait Encode {
    type Item;
    type Error;

    fn encode(item: &Self::Item) -> Result<Slice, Self::Error>;
}

pub trait Decode {
    type Item;
    type Error;

    fn decode(bytes: Slice) -> Result<Self::Item, Self::Error>;
}

#[repr(transparent)]
#[derive(Clone)]
pub struct Keyspace<'a, Key, Value>(Cow<'a, fjall::Keyspace>, PhantomData<(Key, Value)>);

impl<'a, Key, Value> Keyspace<'a, Key, Value> {
    pub fn remap_key<NK>(&'a self) -> Keyspace<'a, NK, Value> {
        Keyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    pub fn remap_value<NV>(&'a self) -> Keyspace<'a, Key, NV> {
        Keyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    pub fn remap_key_value<NK, NV>(&'a self) -> Keyspace<'a, NK, NV> {
        Keyspace(Cow::Borrowed(self.0.as_ref()), PhantomData)
    }

    #[must_use]
    #[inline]
    pub fn name(&self) -> &StrView {
        self.0.name()
    }

    #[must_use]
    #[inline]
    pub fn clear(&self) -> Result<(), fjall::Error> {
        self.0.clear()
    }

    #[must_use]
    #[inline]
    pub fn fragmented_blob_bytes(&self) -> u64 {
        self.0.fragmented_blob_bytes()
    }
    /*
       /// Prepare ingestiom of a pre-sorted stream of key-value pairs into the keyspace.
       ///
       /// Prefer this method over singular inserts or write batches/transactions
       /// for maximum bulk load speed.
       ///
       /// # Errors
       ///
       /// Will return `Err` if an IO error occurs.
       ///
       /// # Panics
       ///
       /// Panics if the input iterator is not sorted in ascending order.
       #[inline]
       pub fn start_ingestion(&self) -> Result<Ingestion<'_>, fjall::Error> {
           self.0.start_ingestion()
       }
    */

    /// Creates a new keyspace.
    /// Returns the underlying LSM-tree's path.
    #[must_use]
    #[inline]
    pub fn path(&self) -> &Path {
        self.0.path()
    }

    /// Returns the disk space usage of this keyspace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// assert_eq!(0, tree.disk_space());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[must_use]
    #[inline]
    pub fn disk_space(&self) -> u64 {
        self.0.disk_space()
    }

    /// Returns an iterator that scans through the entire keyspace.
    ///
    /// Avoid using this function, or limit it as otherwise it may scan a lot of items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    /// tree.insert("f", "abc")?;
    /// tree.insert("g", "abc")?;
    /// assert_eq!(3, tree.iter().count());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[must_use]
    #[expect(clippy::iter_without_into_iter)]
    #[inline]
    pub fn iter(&self) -> Iter {
        self.0.iter()
    }

    /// Returns an iterator over a range of items.
    ///
    /// Avoid using full or unbounded ranges as they may scan a lot of items (unless limited).
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    /// tree.insert("f", "abc")?;
    /// tree.insert("g", "abc")?;
    /// assert_eq!(2, tree.range("a"..="f").count());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[inline]
    pub fn range<K: AsRef<[u8]>, R: RangeBounds<K>>(&self, range: R) -> Iter {
        self.0.range(range)
    }

    /// Returns an iterator over a prefixed set of items.
    ///
    /// Avoid using an empty prefix as it may scan a lot of items (unless limited).
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    /// tree.insert("ab", "abc")?;
    /// tree.insert("abc", "abc")?;
    /// assert_eq!(2, tree.prefix("ab").count());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[inline]
    pub fn prefix<K: AsRef<[u8]>>(&self, prefix: K) -> Iter {
        self.0.prefix(prefix)
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
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
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
    #[inline]
    pub fn approximate_len(&self) -> usize {
        self.0.approximate_len()
    }

    /// Scans the entire keyspace, returning the amount of items.
    ///
    /// # Caution
    ///
    /// This operation scans the entire keyspace: O(n) complexity!
    ///
    /// Never, under any circumstances, use .`len()` == 0 to check
    /// if the keyspace is empty, use [`Keyspace::is_empty`] instead.
    ///
    /// If you want an estimate, use [`Keyspace::approximate_len`] instead.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// assert_eq!(tree.len()?, 0);
    ///
    /// tree.insert("1", "abc")?;
    /// tree.insert("3", "abc")?;
    /// tree.insert("5", "abc")?;
    /// assert_eq!(tree.len()?, 3);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[inline]
    pub fn len(&self) -> Result<usize, fjall::Error> {
        self.0.len()
    }

    /// Returns `true` if the keyspace is empty.
    ///
    /// This operation has O(log N) complexity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// assert!(tree.is_empty()?);
    ///
    /// tree.insert("a", "abc")?;
    /// assert!(!tree.is_empty()?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[inline]
    pub fn is_empty(&self) -> Result<bool, fjall::Error> {
        self.0.is_empty()
    }

    /// Returns the first key-value pair in the keyspace.
    /// The key in this pair is the minimum key in the keyspace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("1", "abc")?;
    /// tree.insert("3", "abc")?;
    /// tree.insert("5", "abc")?;
    ///
    /// let key = tree.first_key_value().expect("item should exist").key()?;
    /// assert_eq!(&*key, "1".as_bytes());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[inline]
    pub fn first_key_value(&self) -> Option<Guard> {
        self.0.first_key_value()
    }

    /// Returns the last key-value pair in the keyspace.
    /// The key in this pair is the maximum key in the keyspace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("1", "abc")?;
    /// tree.insert("3", "abc")?;
    /// tree.insert("5", "abc")?;
    ///
    /// let key = tree.last_key_value().expect("item should exist").key()?;
    /// assert_eq!(&*key, "5".as_bytes());
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    #[inline]
    pub fn last_key_value(&self) -> Option<Guard> {
        self.0.last_key_value()
    }

    /// Returns `true` if the underlying LSM-tree is key-value-separated.
    #[must_use]
    #[inline]
    pub fn is_kv_separated(&self) -> bool {
        self.0.is_kv_separated()
    }

    #[doc(hidden)]
    #[inline]
    pub fn rotate_memtable(&self) -> Result<bool, fjall::Error> {
        self.0.rotate_memtable()
    }

    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub fn l0_table_count(&self) -> usize {
        self.0.l0_table_count()
    }

    /// Number of tables (a.k.a. SST files) in the LSM-tree.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub fn table_count(&self) -> usize {
        self.0.table_count()
    }

    /// Number of blob files in the LSM-tree.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub fn blob_file_count(&self) -> usize {
        self.0.blob_file_count()
    }

    /// Performs major compaction, blocking the caller until it's done.
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[doc(hidden)]
    #[inline]
    pub fn major_compact(&self) -> Result<(), fjall::Error> {
        self.0.major_compact()
    }
}

impl<'a, Key: Encode, Value: Encode> Keyspace<'a, Key, Value> {
    /// Inserts a key-value pair into the keyspace.
    ///
    /// Keys may be up to 65536 bytes long, values up to 2^32 bytes.
    /// Shorter keys and values result in better performance.
    ///
    /// If the key already exists, the item will be overwritten.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// tree.insert("a", "abc")?;
    ///
    /// assert!(!tree.is_empty()?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
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

impl<'a, Key: Encode, Value: Decode> Keyspace<'a, Key, Value> {
    /// Retrieves an item from the keyspace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
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
    #[inline]
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
    /// Returns `true` if the keyspace contains the specified key.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
    /// # let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;
    /// assert!(!tree.contains_key("a")?);
    ///
    /// tree.insert("a", "abc")?;
    /// assert!(tree.contains_key("a")?);
    /// #
    /// # Ok::<(), fjall::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if an IO error occurs.
    #[inline]
    pub fn contains_key(&self, key: &Key::Item) -> Result<bool, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.contains_key(key).map_err(Error::Fjall)
    }

    /// Retrieves the size of an item from the keyspace.
    ///
    /// # Examples
    ///
    /// ```
    /// # use fjall::{Database, KeyspaceCreateOptions};
    /// #
    /// # let folder = tempfile::tempdir()?;
    /// # let db = Database::builder(folder).open()?;
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
    #[inline]
    pub fn size_of(&self, key: &Key::Item) -> Result<Option<u32>, Error<Key::Error, Infallible>> {
        let key = Key::encode(key).map_err(Error::Key)?;
        self.0.size_of(key).map_err(Error::Fjall)
    }
}
