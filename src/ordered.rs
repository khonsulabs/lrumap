use std::{
    borrow::Borrow,
    collections::{btree_map, BTreeMap},
    fmt::Debug,
    hash::Hash,
    ops::RangeBounds,
};

use crate::{
    lru::{EntryCache, EntryRef, IntoIter, LruCache, NodeId, Removed},
    LruMap,
};

/// A Least Recently Used map with fixed capacity that stores keys using a
/// [`BTreeMap`] internally. Inserting and querying has similar performance to
/// using a [`BTreeMap`], but internally this data structure keeps track of the
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
pub struct LruBTreeMap<Key, Value> {
    map: BTreeMap<Key, NodeId>,
    cache: LruCache<Key, Value>,
}

impl<Key, Value> LruBTreeMap<Key, Value>
where
    Key: Ord + Clone,
{
    /// Creates a new map with the maximum `capacity`.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is <= 1 or > `u32::MAX`.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 1);
        assert!(capacity <= usize::try_from(u32::MAX).unwrap());
        Self {
            map: BTreeMap::new(),
            cache: LruCache::new(capacity),
        }
    }

    /// Returns the stored value for `key`, if present.
    ///
    /// This function touches the key, making it the most recently used key.
    pub fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + ?Sized,
        Key: Borrow<QueryKey>,
    {
        let node = self.map.get(key).copied();
        node.map(|node| self.cache.get(node).value())
    }

    /// Returns the stored value for `key`, if present.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache.
    pub fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + ?Sized,
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
    /// use lrumap::{LruBTreeMap, LruMap, Removed};
    ///
    /// let mut lru = LruBTreeMap::new(3);
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
        QueryKey: Ord + ?Sized,
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
    /// use lrumap::{LruBTreeMap, LruMap, Removed};
    ///
    /// let mut lru = LruBTreeMap::new(3);
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

        if let btree_map::Entry::Occupied(entry) = &entry {
            let node_ref = *entry.get();
            // Swap the value out.
            let value = self.cache.get_mut(node_ref).replace_value(value);

            return Some(Removed::PreviousValue(value));
        }

        // Key is not currently contained. Create a new node.
        let (node, result) = self.cache.push(key, value);

        // Insert the node into the BTreeMap
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
    /// use lrumap::{LruBTreeMap, LruMap};
    ///
    /// let mut lru = LruBTreeMap::new(3);
    /// lru.extend([(1, 1), (2, 2), (3, 3), (4, 4)]);
    ///
    /// assert_eq!(lru.head().unwrap().key(), &4);
    /// assert_eq!(lru.tail().unwrap().key(), &2);
    /// ```
    pub fn extend<IntoIter: IntoIterator<Item = (Key, Value)>>(&mut self, iterator: IntoIter) {
        for (key, value) in iterator {
            let (node_id, removed) = self.cache.push(key.clone(), value);
            if let Some(Removed::Evicted(evicted_key, _)) = removed {
                self.map.remove(&evicted_key);
            }
            self.map.insert(key, node_id);
        }
    }

    /// Returns the most recently touched entry with a key within `range`.
    ///
    /// This function uses [`BTreeMap::range`] to identify all entries that
    /// match the given range. For each returned entry, the entry's
    /// [staleness](EntryRef::staleness) is compared, and the least stale entry
    /// is returned. If no keys match the range, `None` is returned.
    ///
    /// This function does not touch any keys, preserving the current order of
    /// the lru cache. The [`EntryRef`] returned can be used to peek, touch, or
    /// remove the entry.
    ///
    /// ```rust
    /// use lrumap::LruBTreeMap;
    ///
    /// let mut lru = LruBTreeMap::new(5);
    /// lru.extend([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);
    ///
    /// assert_eq!(lru.most_recent_in_range(2..=4).unwrap().key(), &4);
    /// // Change the order by retrieving key 2.
    /// lru.get(&2);
    /// assert_eq!(lru.most_recent_in_range(2..=4).unwrap().key(), &2);
    /// ```
    pub fn most_recent_in_range<QueryKey, Range>(
        &mut self,
        range: Range,
    ) -> Option<EntryRef<'_, Self, Key, Value>>
    where
        QueryKey: Ord + ?Sized,
        Key: Borrow<QueryKey>,
        Range: RangeBounds<QueryKey>,
    {
        self.most_recent_in_range_where(range, |_, _| true)
    }

    /// Returns the most recently touched entry with a key within `range` that
    /// passes the `condition` check.
    ///
    /// This function uses [`BTreeMap::range`] to identify all entries that
    /// match the given range. Each key and value that matches is passed to
    /// `condition`. For each entry where `condition` returns true, the
    /// [staleness](EntryRef::staleness) is compared, and the least stale entry
    /// is returned. If no keys match the range, `None` is returned.
    ///
    /// This function does not touch any keys, preserving the current order of
    /// the lru cache. The [`EntryRef`] returned can be used to peek, touch, or
    /// remove the entry.
    ///
    /// ```rust
    /// use lrumap::LruBTreeMap;
    ///
    /// let mut lru = LruBTreeMap::<u32, u16>::new(5);
    /// lru.extend([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);
    ///
    /// let condition = |key: &u32, value: &u16| key == &3 || value == &4;
    /// assert_eq!(lru.most_recent_in_range_where(2..=4, condition).unwrap().key(), &4);
    ///
    /// // Change the order by retrieving key 2. However, 2 doesn't meet the
    /// // condition, so the result is unchanged.
    /// lru.get(&2);
    /// assert_eq!(lru.most_recent_in_range_where(2..=4, condition).unwrap().key(), &4);
    ///
    /// // Request 3, moving it to the front. Since 3 matches the condition, the
    /// // result is now 3.
    /// lru.get(&3);
    /// assert_eq!(lru.most_recent_in_range_where(2..=4, condition).unwrap().key(), &3);
    /// ```
    pub fn most_recent_in_range_where<QueryKey, Range, Condition>(
        &mut self,
        range: Range,
        mut condition: Condition,
    ) -> Option<EntryRef<'_, Self, Key, Value>>
    where
        QueryKey: Ord + ?Sized,
        Key: Borrow<QueryKey>,
        Range: RangeBounds<QueryKey>,
        Condition: for<'key, 'value> FnMut(&'key Key, &'value Value) -> bool,
    {
        let mut closest_node = None;
        let mut closest_staleness = usize::MAX;
        for (_, &node_id) in self.map.range(range) {
            let node = self.cache.get_without_touch(node_id);
            if condition(node.key(), node.value()) {
                let staleness = self.cache.sequence().wrapping_sub(node.last_accessed());
                if staleness < closest_staleness {
                    closest_staleness = staleness;
                    closest_node = Some(node_id);
                }
            }
        }
        closest_node.map(|node| EntryRef::new(self, node))
    }
}

impl<Key, Value> LruMap<Key, Value> for LruBTreeMap<Key, Value>
where
    Key: Ord + Clone,
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

    fn iter(&self) -> crate::lru::Iter<'_, Key, Value> {
        self.cache.iter()
    }

    fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Eq + Hash,
    {
        self.get(key)
    }

    fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Eq + Hash,
    {
        self.get_without_update(key)
    }

    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Self, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Eq + Hash,
    {
        self.entry(key)
    }

    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>> {
        self.push(key, value)
    }

    fn extend<IntoIter: IntoIterator<Item = (Key, Value)>>(&mut self, iterator: IntoIter) {
        self.extend(iterator);
    }
}

impl<Key, Value> EntryCache<Key, Value> for LruBTreeMap<Key, Value>
where
    Key: Ord + Clone,
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

impl<Key, Value> IntoIterator for LruBTreeMap<Key, Value>
where
    Key: Ord + Clone,
{
    type Item = (Key, Value);

    type IntoIter = IntoIter<Key, Value>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::from(self.cache)
    }
}

#[test]
fn most_recent_in_range_test() {
    let mut lru = LruBTreeMap::new(5);
    lru.extend([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);

    assert_eq!(lru.most_recent_in_range(2..=4).unwrap().key(), &4);
    lru.get(&2);
    assert_eq!(lru.most_recent_in_range(2..=4).unwrap().key(), &2);
    assert_eq!(
        lru.most_recent_in_range_where(2..=4, |key: &u32, _value: &u16| key != &2)
            .unwrap()
            .key(),
        &4
    );
}
