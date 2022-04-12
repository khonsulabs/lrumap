mod hashed;
pub mod lru;
mod ordered;

use std::{borrow::Borrow, hash::Hash};

use lru::{EntryRef, Removed};

pub use crate::{hashed::*, ordered::*};

pub trait LruMap<Key, Value> {
    fn new(capacity: usize) -> Self;

    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn head(&mut self) -> Option<EntryRef<'_, Key, Value>>;
    fn tail(&mut self) -> Option<EntryRef<'_, Key, Value>>;

    fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq;
    fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq;
    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq;

    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>>;
}

#[cfg(test)]
mod tests;
