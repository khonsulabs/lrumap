mod hashed;
mod lru;
mod ordered;

use std::{borrow::Borrow, hash::Hash};

pub use crate::{
    hashed::*,
    lru::{EntryRef, Removed},
    ordered::*,
};

pub trait LruMap<Key, Value>
where
    Key: Ord + Eq + Hash + Clone,
{
    fn new(capacity: usize) -> Self;

    fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>;
    fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>;
    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>;
    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>>;
}

#[cfg(test)]
mod tests;
