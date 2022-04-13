#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::cargo,
    missing_docs,
    // clippy::missing_docs_in_private_items,
    clippy::nursery,
    clippy::pedantic,
    future_incompatible,
    rust_2018_idioms,
)]
#![allow(
    clippy::missing_errors_doc, // TODO clippy::missing_errors_doc
    clippy::option_if_let_else,
    clippy::module_name_repetitions,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation
)]

mod hashed;
mod lru;
mod ordered;

use std::{borrow::Borrow, hash::Hash};

pub use crate::{
    hashed::*,
    lru::{EntryRef, Removed},
    ordered::*,
};

/// A Least Recently Used map interface that supports all map implementations
/// exposed by this crate.
pub trait LruMap<Key, Value> {
    /// Creates a new map with the maximum `capacity`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is <= 1 or > `u32::MAX`.
    fn new(capacity: usize) -> Self;

    /// Returns the number of keys present in this map.
    fn len(&self) -> usize;

    /// Retruns true if this map contains no keys.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a reference to the most recently used key.
    fn head(&mut self) -> Option<EntryRef<'_, Key, Value>>;
    /// Returns a reference to the least recently used key.
    fn tail(&mut self) -> Option<EntryRef<'_, Key, Value>>;

    /// Returns the stored value for `key`, if present.
    ///
    /// This function touches the key, making it the most recently used key.
    fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq;

    /// Returns the stored value for `key`, if present.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache.
    fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq;

    /// Returns an [`EntryRef`] for `key`, if present.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache. The [`EntryRef`] can touch the key, depending on which
    /// functions are used.
    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq;

    /// Inserts `value` for `key` into this map. If a value is already stored
    /// for this key, [`Removed::PreviousValue`] is returned with the previously
    /// stored value. If no value is currently stored and the map is full, the
    /// least recently used entry will be returned in [`Removed::Evicted`].
    /// Otherwise, `None` will be returned.
    ///
    /// This function touches the key, making it the most recently used key.
    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>>;
}

#[cfg(test)]
mod tests;
