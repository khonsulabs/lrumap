#[cfg(not(feature = "hashbrown"))]
use std::collections::{hash_map, HashMap};
use std::{
    borrow::Borrow,
    collections::hash_map::RandomState,
    fmt::Debug,
    hash::{BuildHasher, Hash},
};

#[cfg(feature = "hashbrown")]
use hashbrown::{hash_map, HashMap};

use crate::{
    lru::{EntryRef, LruCache, Removed},
    LruMap,
};

#[derive(Debug)]
pub struct LruHashMap<Key, Value, State = RandomState> {
    map: HashMap<Key, u32, State>,
    cache: LruCache<Key, Value>,
}

impl<Key, Value> LruHashMap<Key, Value, RandomState>
where
    Key: Hash + Eq + Clone,
{
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
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
    pub fn with_hasher(capacity: usize, hasher: State) -> Self {
        assert!(capacity > 0);
        Self {
            map: HashMap::with_capacity_and_hasher(capacity, hasher),
            cache: LruCache::new(capacity),
        }
    }

    pub fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        let node = self.map.get(key).copied();
        node.and_then(|node| self.cache.get(node).value())
    }

    pub fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.map
            .get(key)
            .and_then(|node| self.cache.get_without_update(*node).value())
    }

    pub fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.map
            .get(key)
            .copied()
            .map(|node| EntryRef::new(&mut self.cache, node))
    }

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

        if let Some(Removed::Expired(key, _)) = &result {
            self.map.remove(key);
        }

        result
    }
}

impl<Key, Value> LruMap<Key, Value> for LruHashMap<Key, Value, RandomState>
where
    Key: Hash + Eq + Clone,
{
    fn new(capacity: usize) -> Self {
        LruHashMap::new(capacity)
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn head(&mut self) -> Option<EntryRef<'_, Key, Value>> {
        self.cache
            .head()
            .map(|node| EntryRef::new(&mut self.cache, node))
    }

    fn tail(&mut self) -> Option<EntryRef<'_, Key, Value>> {
        self.cache
            .tail()
            .map(|node| EntryRef::new(&mut self.cache, node))
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

    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey> + Ord + Hash + Eq,
    {
        self.entry(key)
    }

    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>> {
        self.push(key, value)
    }
}
