use std::borrow::Borrow;
#[cfg(not(feature = "hashbrown"))]
use std::collections::{hash_map, hash_map::RandomState as DefaultState, HashMap};
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

#[cfg(feature = "hashbrown")]
use hashbrown::{
    hash_map::{self, DefaultHashBuilder as DefaultState},
    HashMap,
};

use crate::lru::{EntryCache, EntryRef, IntoIter, LruCache, NodeId, Removed};
use crate::LruMap;

/// A Least Recently Used map with fixed capacity that stores keys using a
/// `HashMap` internally. Inserting and querying has similar performance to
/// using a `HashMap`, but internally this data structure keeps track of the
/// order in which the keys were last touched.
///
/// When inserting a new key and the map is at-capacity, the least recently used
/// key will be evicted to make room for the new key.
///
/// To avoid `unsafe`, this crate must store each entry's key twice. This means
/// that `Key` must implement `Clone`. If you're using expensive-to-clone keys,
/// consider wrapping the key in an `Rc`/`Arc` or using an alternate LRU crate.
#[derive(Debug)]
#[must_use]
pub struct LruHashMap<Key, Value, State = DefaultState> {
    map: HashMap<Key, NodeId, State>,
    cache: LruCache<Key, Value>,
}

impl<Key, Value> LruHashMap<Key, Value, DefaultState>
where
    Key: Hash + Eq + Clone,
{
    /// Creates a new map with the maximum `capacity`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is <= 1.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 1);
        Self {
            map: HashMap::with_capacity(capacity),
            cache: LruCache::new(capacity),
        }
    }
}

impl<Key, Value, State> LruHashMap<Key, Value, State>
where
    Key: Hash + Eq + Clone,
    State: BuildHasher,
{
    /// Creates a new map with the maximum `capacity` and `hasher`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is <= 1
    pub fn with_hasher(capacity: usize, hasher: State) -> Self {
        assert!(capacity > 1);
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hasher),
            cache: LruCache::new(capacity),
        }
    }

    /// Returns the stored value for `key`, if present.
    ///
    /// This function touches the key, making it the most recently used key.
    pub fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        let node = self.map.get(key).copied();
        node.map(|node| self.cache.get(node).value())
    }

    /// Returns the stored value for `key`, if present.
    ///
    /// This function touches the key, making it the most recently used key.
    pub fn get_mut<QueryKey>(&mut self, key: &QueryKey) -> Option<&mut Value>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        let node = self.map.get(key).copied();
        node.map(|node| self.cache.get_mut(node).value_mut())
    }

    /// Returns the stored value for `key`, if present.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache.
    pub fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.map
            .get(key)
            .map(|node| self.cache.get_without_touch(*node).value())
    }

    /// Returns an [`EntryRef`] for `key`, if present.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache. The [`EntryRef`] can touch the key, depending on which
    /// functions are used.
    ///
    /// ```rust
    /// use lrumap::{LruHashMap, LruMap, Removed};
    ///
    /// let mut lru = LruHashMap::new(3);
    /// lru.push(1, 1);
    /// lru.push(2, 2);
    /// lru.push(3, 3);
    ///
    /// // The cache has been updated once since entry 2 was touched.
    /// let mut entry = lru.entry(&2).unwrap();
    /// assert_eq!(entry.staleness(), 1);
    /// // Peeking the value will not update the entry's position.
    /// assert_eq!(entry.peek_value(), &2);
    /// assert_eq!(entry.staleness(), 1);
    /// // Querying the value or touching the entry will move it to the
    /// // front of the cache.
    /// assert_eq!(entry.value(), &2);
    /// assert_eq!(entry.staleness(), 0);
    ///
    /// assert_eq!(lru.head().unwrap().key(), &2);
    /// ```
    pub fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Self, Key, Value>>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.map
            .get(key)
            .copied()
            .map(|node| EntryRef::new(self, node))
    }

    /// Inserts `value` for `key` into this map. If a value is already stored
    /// for this key, [`Removed::PreviousValue`] is returned with the previously
    /// stored value. If no value is currently stored and the map is full, the
    /// least recently used entry will be returned in [`Removed::Evicted`].
    /// Otherwise, `None` will be returned.
    ///
    /// This function touches the key, making it the most recently used key.
    ///
    /// ```rust
    /// use lrumap::{LruHashMap, LruMap, Removed};
    ///
    /// let mut lru = LruHashMap::new(3);
    /// lru.push(1, 1);
    /// lru.push(2, 2);
    /// lru.push(3, 3);
    ///
    /// // The cache is now full. The next push will evict an entry.
    /// let removed = lru.push(4, 4);
    /// assert_eq!(removed, Some(Removed::Evicted(1, 1)));
    ///
    /// // This leaves the cache with 4 as the most recent key, and 2 as the
    /// // least recent key.
    /// assert_eq!(lru.head().unwrap().key(), &4);
    /// assert_eq!(lru.tail().unwrap().key(), &2);
    /// ```
    pub fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>> {
        // Create the new entry for this key/value pair, which also puts it at
        // the front of the LRU
        // let existing_entry = self.map.entry(key.clone());
        let entry = self.map.entry(key.clone());

        if let hash_map::Entry::Occupied(entry) = &entry {
            let node_ref = *entry.get();
            // Swap the value out.
            let value = self.cache.get_mut(node_ref).replace_value(value);

            return Some(Removed::PreviousValue(value));
        }

        // Key is not currently contained. Create a new node.
        let (node, result) = self.cache.push(key, value);

        // Insert the node
        entry.or_insert(node);

        if let Some(Removed::Evicted(key, _)) = &result {
            self.map.remove(key);
        }

        result
    }

    /// Pushes all items from `iterator` into this map. If there are more
    /// entries in the iterator than capacity remaining, keys will be evicted as
    /// needed.
    ///
    /// This function is equivalent to a for loop calling [`Self::push()`].
    ///
    /// ```rust
    /// use lrumap::{LruHashMap, LruMap};
    ///
    /// let mut lru = LruHashMap::new(3);
    /// lru.extend([(1, 1), (2, 2), (3, 3), (4, 4)]);
    ///
    /// assert_eq!(lru.head().unwrap().key(), &4);
    /// assert_eq!(lru.tail().unwrap().key(), &2);
    /// ```
    pub fn extend<IntoIter: IntoIterator<Item = (Key, Value)>>(&mut self, iterator: IntoIter) {
        for (key, value) in iterator {
            self.push(key, value);
        }
    }
}

impl<Key, Value> LruMap<Key, Value> for LruHashMap<Key, Value, DefaultState>
where
    Key: Hash + Eq + Clone,
{
    fn new(capacity: usize) -> Self {
        Self::new(capacity)
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn head(&mut self) -> Option<EntryRef<'_, Self, Key, Value>> {
        self.cache.head().map(|node| EntryRef::new(self, node))
    }

    fn tail(&mut self) -> Option<EntryRef<'_, Self, Key, Value>> {
        self.cache.tail().map(|node| EntryRef::new(self, node))
    }

    fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq,
    {
        self.get(key)
    }

    fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq,
    {
        self.get_without_update(key)
    }

    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Self, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq,
    {
        self.entry(key)
    }

    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>> {
        self.push(key, value)
    }

    fn iter(&self) -> crate::lru::Iter<'_, Key, Value> {
        self.cache.iter()
    }

    fn extend<IntoIter: IntoIterator<Item = (Key, Value)>>(&mut self, iterator: IntoIter) {
        self.extend(iterator);
    }
}

impl<Key, Value, State> EntryCache<Key, Value> for LruHashMap<Key, Value, State>
where
    Key: Hash + Eq + Clone,
    State: BuildHasher,
{
    fn cache(&self) -> &LruCache<Key, Value> {
        &self.cache
    }

    fn cache_mut(&mut self) -> &mut LruCache<Key, Value> {
        &mut self.cache
    }

    fn remove(&mut self, node: NodeId) -> ((Key, Value), Option<NodeId>, Option<NodeId>) {
        let ((key, value), next, previous) = self.cache.remove(node);
        self.map.remove(&key);
        ((key, value), next, previous)
    }
}

impl<Key, Value, State> IntoIterator for LruHashMap<Key, Value, State>
where
    Key: Hash + Eq + Clone,
    State: BuildHasher,
{
    type IntoIter = IntoIter<Key, Value>;
    type Item = (Key, Value);

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::from(self.cache)
    }
}
