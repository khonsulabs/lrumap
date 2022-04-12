use std::{
    borrow::Borrow,
    collections::{btree_map, BTreeMap},
    fmt::Debug,
    hash::Hash,
};

use crate::{
    lru::{EntryRef, LruCache, Removed},
    LruMap,
};

#[derive(Debug)]
pub struct LruBTreeMap<Key, Value> {
    map: BTreeMap<Key, u32>,
    cache: LruCache<Key, Value>,
}

impl<Key, Value> LruBTreeMap<Key, Value>
where
    Key: Ord + Clone,
{
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 1);
        assert!(capacity <= usize::try_from(u32::MAX).unwrap());
        Self {
            map: BTreeMap::new(),
            cache: LruCache::new(capacity),
        }
    }

    pub fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + ?Sized,
        Key: Borrow<QueryKey>,
    {
        let node = self.map.get(key).copied();
        node.and_then(|node| self.cache.get(node).value())
    }

    pub fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.map
            .get(key)
            .and_then(|node| self.cache.get_without_update(*node).value())
    }

    pub fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Ord + ?Sized,
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

        if let Some(Removed::Expired(key, _)) = &result {
            self.map.remove(key);
        }

        result
    }
}

impl<Key, Value> LruMap<Key, Value> for LruBTreeMap<Key, Value>
where
    Key: Ord + Eq + Hash + Clone,
{
    fn new(capacity: usize) -> Self {
        Self::new(capacity)
    }

    fn get<QueryKey>(&mut self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.get(key)
    }

    fn get_without_update<QueryKey>(&self, key: &QueryKey) -> Option<&Value>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.get_without_update(key)
    }

    fn entry<QueryKey>(&mut self, key: &QueryKey) -> Option<EntryRef<'_, Key, Value>>
    where
        QueryKey: Ord + Hash + Eq + ?Sized,
        Key: Borrow<QueryKey>,
    {
        self.entry(key)
    }

    fn push(&mut self, key: Key, value: Value) -> Option<Removed<Key, Value>> {
        self.push(key, value)
    }
}
