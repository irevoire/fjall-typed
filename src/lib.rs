use std::marker::PhantomData;

pub mod codec;
mod error;
mod keyspace;

pub use error::Error;
pub use keyspace::Keyspace;

pub struct Guard<Key, Value>(fjall::Guard, PhantomData<(Key, Value)>);

impl<Key, Value> Guard<Key, Value> {
    pub fn new(guard: fjall::Guard) -> Self {
        Self(guard, PhantomData)
    }

    pub fn remap_key_type<NKey>(self) -> Guard<NKey, Value> {
        Guard(self.0, PhantomData)
    }

    pub fn remap_value_type<NValue>(self) -> Guard<Key, NValue> {
        Guard(self.0, PhantomData)
    }

    pub fn remap_types<NKey, NValue>(self) -> Guard<NKey, NValue> {
        Guard(self.0, PhantomData)
    }
}

pub struct Iter<Key, Value>(fjall::Iter, PhantomData<(Key, Value)>);

impl<Key, Value> Iter<Key, Value> {
    pub fn new(iter: fjall::Iter) -> Self {
        Self(iter, PhantomData)
    }

    pub fn remap_key_type<NKey>(self) -> Iter<NKey, Value> {
        Iter(self.0, PhantomData)
    }

    pub fn remap_value_type<NValue>(self) -> Iter<Key, NValue> {
        Iter(self.0, PhantomData)
    }

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
