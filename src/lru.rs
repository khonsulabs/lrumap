use std::{collections::HashSet, fmt::Debug, marker::PhantomData};

pub struct LruCache<Key, Value> {
    nodes: Vec<Node<Key, Value>>,
    head: Option<NodeId>,
    tail: Option<NodeId>,
    vacant: Option<NodeId>,
    sequence: usize,
    length: usize,
}

impl<Key, Value> LruCache<Key, Value> {
    pub fn new(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            vacant: None,
            sequence: 0,
            length: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.length
    }

    pub const fn sequence(&self) -> usize {
        self.sequence
    }

    pub const fn head(&self) -> Option<NodeId> {
        self.head
    }

    pub const fn tail(&self) -> Option<NodeId> {
        self.tail
    }

    pub const fn iter(&self) -> Iter<'_, Key, Value> {
        Iter {
            cache: self,
            node: self.head,
        }
    }

    pub fn get(&mut self, node: NodeId) -> &Node<Key, Value> {
        self.move_node_to_front(node);
        &self.nodes[node.as_usize()]
    }

    pub fn get_without_update(&self, node: NodeId) -> &Node<Key, Value> {
        &self.nodes[node.as_usize()]
    }

    pub fn get_mut(&mut self, node: NodeId) -> &mut Node<Key, Value> {
        self.move_node_to_front(node);
        &mut self.nodes[node.as_usize()]
    }

    pub fn push(&mut self, key: Key, value: Value) -> (NodeId, Option<Removed<Key, Value>>) {
        let (node, result) = if self.head.is_some() {
            self.push_front(key, value)
        } else {
            // First node of the list.
            self.allocate_node(key, value)
        };
        (
            node,
            result.map(|(key, value)| Removed::Evicted(key, value)),
        )
    }

    pub fn move_node_to_front(&mut self, node_index: NodeId) {
        if self.head == Some(node_index) {
            // No-op.
            return;
        }

        self.sequence += 1;

        // An entry already exists. Reuse the node.
        self.nodes[node_index.as_usize()].last_accessed = self.sequence;

        // Update the next pointer to the current head.
        let mut next = self.head;
        std::mem::swap(&mut next, &mut self.nodes[node_index.as_usize()].next);
        // Get and clear the previous node, as this node is going to be the new
        // head.
        let previous = self.nodes[node_index.as_usize()].previous.take().unwrap();
        // Update the previous pointer's next to the previous next value.
        self.nodes[previous.as_usize()].next = next;
        if self.tail == Some(node_index) {
            // If this is the tail, update the tail to the previous node.
            self.tail = Some(previous);
        } else {
            // Otherwise, we need to update the next node's previous to point to
            // this node's former previous.
            self.nodes[next.unwrap().as_usize()].previous = Some(previous);
        }

        // Move this node to the front
        self.nodes[self.head.unwrap().as_usize()].previous = Some(node_index);

        self.head = Some(node_index);
    }

    pub fn push_front(&mut self, key: Key, value: Value) -> (NodeId, Option<(Key, Value)>) {
        let (node, removed) = self.allocate_node(key, value);
        self.sequence += 1;
        let mut entry = &mut self.nodes[node.as_usize()];
        entry.last_accessed = self.sequence;
        entry.next = Some(self.head.unwrap());

        let mut previous_head = &mut self.nodes[self.head.unwrap().as_usize()];
        debug_assert!(previous_head.previous.is_none());
        previous_head.previous = Some(node);
        self.head = Some(node);
        (node, removed)
    }

    pub fn allocate_node(&mut self, key: Key, value: Value) -> (NodeId, Option<(Key, Value)>) {
        if let Some(vacant) = self.vacant {
            // Pull a node off the vacant list.
            self.vacant = self.nodes[vacant.as_usize()].next;
            self.nodes[vacant.as_usize()].next = None;
            self.nodes[vacant.as_usize()].entry = Entry::Occupied { key, value };
            self.length += 1;
            if self.head.is_none() {
                self.head = Some(vacant);
                self.tail = Some(vacant);
            }
            (vacant, None)
        } else if self.nodes.len() == self.nodes.capacity() {
            // Expire the least recently used key (tail).
            let index = self.tail.unwrap();
            self.tail = self.nodes[index.as_usize()].previous;
            if let Some(previous) = self.tail {
                self.nodes[previous.as_usize()].next = None;
            }
            self.nodes[index.as_usize()].previous = None;

            let mut entry = Entry::Occupied { key, value };
            std::mem::swap(&mut entry, &mut self.nodes[index.as_usize()].entry);

            (index, entry.into())
        } else {
            // We have capacity to fill.
            let index = NodeId(self.nodes.len() as u32);
            self.length += 1;
            self.nodes.push(Node {
                last_accessed: self.sequence,
                previous: None,
                next: None,
                entry: Entry::Occupied { key, value },
            });
            if self.head.is_none() {
                self.head = Some(index);
                self.tail = Some(index);
            }
            (index, None)
        }
    }

    pub fn remove(&mut self, node: NodeId) -> ((Key, Value), Option<NodeId>, Option<NodeId>) {
        self.length -= 1;
        let removed = self.nodes[node.as_usize()].entry.evict();
        let mut next = self.vacant;
        std::mem::swap(&mut next, &mut self.nodes[node.as_usize()].next);
        let previous = self.nodes[node.as_usize()].previous.take();

        if let Some(previous) = previous {
            self.nodes[previous.as_usize()].next = next;
        }
        if let Some(next) = next {
            self.nodes[next.as_usize()].previous = previous;
        }

        if self.tail == Some(node) {
            self.tail = previous;
        }

        if self.head == Some(node) {
            self.head = next;
        }

        self.vacant = Some(node);

        (removed, next, previous)
    }

    // pub fn pop_tail(&mut self) -> Option<(u32, (Key, Value))> {
    //     let vacated_entry = match (&mut self.head, &mut self.tail) {
    //         (Some(head), Some(tail)) if head == tail => {
    //             self.length -= 1;
    //             // Last node. Reset the list to None.
    //             let last_node = *head;
    //             self.head = None;
    //             self.tail = None;
    //             if let Some(vacant_head) = self.vacant {
    //                 self.nodes[last_node as usize].next = Some(vacant_head);
    //             }
    //             self.vacant = Some(last_node);
    //             Some(last_node)
    //         }
    //         (Some(_), Some(tail)) => {
    //             self.length -= 1;
    //             // First, get the node before the tail.
    //             let tail_node = &mut self.nodes[*tail as usize];
    //             debug_assert!(tail_node.next.is_none());
    //             let mut node_ref = tail_node.previous.take().unwrap();
    //             // Clear the next pointer
    //             let mut new_tail = &mut self.nodes[node_ref as usize];
    //             new_tail.next = None;
    //             // Next, set the previous node to the tail
    //             std::mem::swap(&mut node_ref, tail);
    //             // Move the previous tail to the vacant list
    //             match &mut self.vacant {
    //                 Some(last_vacant) => {
    //                     self.nodes[node_ref as usize].next = Some(*last_vacant);
    //                 }
    //                 None => self.vacant = Some(node_ref),
    //             }
    //             Some(node_ref)
    //         }
    //         _ => None,
    //     };

    //     vacated_entry.and_then(|node_ref| {
    //         // Vacate the slot
    //         let mut previous_entry = Entry::Vacant;
    //         std::mem::swap(
    //             &mut previous_entry,
    //             &mut self.nodes[node_ref as usize].entry,
    //         );
    //         match previous_entry {
    //             Entry::Occupied { key, value } => Some((node_ref, (key, value))),
    //             Entry::Vacant => None,
    //         }
    //     })
    // }
}

impl<Key, Value> Debug for LruCache<Key, Value>
where
    Key: Debug,
    Value: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        if let Some(head) = self.head {
            let mut seen_nodes = HashSet::new();
            let mut current_node = head;
            let mut end_found = false;
            while seen_nodes.insert(current_node) {
                let node = &self.nodes[current_node.as_usize()];
                list.entry(node);
                current_node = match node.next {
                    Some(next) => next,
                    None => {
                        end_found = true;
                        break;
                    }
                };
            }

            assert!(end_found, "cycle detected");
        }

        list.finish()
    }
}

#[derive(Debug)]
enum Entry<Key, Value> {
    Occupied { key: Key, value: Value },
    Vacant,
}

impl<Key, Value> Entry<Key, Value> {
    fn evict(&mut self) -> (Key, Value) {
        let mut entry = Self::Vacant;
        std::mem::swap(&mut entry, self);
        match entry {
            Entry::Occupied { key, value } => (key, value),
            Entry::Vacant => unreachable!("evict called on a vacant entry"),
        }
    }
}

impl<Key, Value> From<Entry<Key, Value>> for Option<(Key, Value)> {
    fn from(entry: Entry<Key, Value>) -> Self {
        match entry {
            Entry::Occupied { key, value } => Some((key, value)),
            Entry::Vacant => None,
        }
    }
}

pub struct Node<Key, Value> {
    entry: Entry<Key, Value>,
    previous: Option<NodeId>,
    next: Option<NodeId>,
    last_accessed: usize,
}

impl<Key, Value> Debug for Node<Key, Value>
where
    Key: Debug,
    Value: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Node");

        if let Entry::Occupied { key, value } = &self.entry {
            debug.field("key", key);
            debug.field("value", value);
        }
        debug.field("last_accessed", &self.last_accessed);

        debug.finish()
    }
}

impl<Key, Value> Node<Key, Value> {
    pub fn key(&self) -> &Key {
        match &self.entry {
            Entry::Occupied { key, .. } => key,
            Entry::Vacant => unreachable!("EntryRef can't be made against Vacant"),
        }
    }

    pub fn value(&self) -> &Value {
        match &self.entry {
            Entry::Occupied { value, .. } => value,
            Entry::Vacant => unreachable!("EntryRef can't be made against Vacant"),
        }
    }

    pub fn replace_value(&mut self, mut new_value: Value) -> Value {
        match &mut self.entry {
            Entry::Occupied { value, .. } => {
                std::mem::swap(value, &mut new_value);
                new_value
            }
            Entry::Vacant => unreachable!("EntryRef can't be made against Vacant"),
        }
    }
}

/// A reference to an entry in a Least Recently Used map.
#[derive(Debug)]
pub struct EntryRef<'a, Cache, Key, Value>
where
    Cache: EntryCache<Key, Value>,
{
    cache: &'a mut Cache,
    node: NodeId,
    accessed: bool,
    _phantom: PhantomData<(Key, Value)>,
}

pub trait EntryCache<Key, Value> {
    fn node(&self, id: NodeId) -> &Node<Key, Value>;
    fn move_node_to_front(&mut self, id: NodeId);
    fn sequence(&self) -> usize;
    fn remove(&mut self, node: NodeId) -> ((Key, Value), Option<NodeId>, Option<NodeId>);
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct NodeId(u32);

impl NodeId {
    const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl<'a, Cache, Key, Value> EntryRef<'a, Cache, Key, Value>
where
    Cache: EntryCache<Key, Value>,
{
    pub(crate) fn new(cache: &'a mut Cache, node: NodeId) -> Self {
        Self {
            node,
            cache,
            accessed: false,
            _phantom: PhantomData,
        }
    }

    /// Returns the unique index of this node. This function should only be used
    /// for debugging purposes.
    #[must_use]
    pub fn id(&self) -> NodeId {
        self.node
    }

    /// Returns the key of this entry.
    #[must_use]
    pub fn key(&self) -> &Key {
        self.cache.node(self.node).key()
    }

    /// Returns the value of this entry.
    ///
    /// This function touches the key, making it the most recently used key.
    /// This function only touches the key once. Subsequent calls will return
    /// the value without touching the key. This remains true until
    /// `move_next()` or `move_previous()` are invoked.
    #[must_use]
    pub fn value(&mut self) -> &Value {
        if !self.accessed {
            self.accessed = true;
            self.touch();
        }
        self.cache.node(self.node).value()
    }

    /// Touches this key, making it the most recently used key.
    pub fn touch(&mut self) {
        self.cache.move_node_to_front(self.node);
    }

    /// Returns the value of this entry.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache.
    #[must_use]
    pub fn peek_value(&self) -> &Value {
        self.cache.node(self.node).value()
    }

    /// Returns the number of changes to the cache since this key was last
    /// touched.
    #[must_use]
    pub fn staleness(&self) -> usize {
        self.cache
            .sequence()
            .wrapping_sub(self.cache.node(self.node).last_accessed)
    }

    /// Updates this reference to point to the next least recently used key in
    /// the list. Returns true if a next entry was found, or returns false if
    /// the entry is the last entry in the list.
    #[must_use]
    pub fn move_next(&mut self) -> bool {
        if let Some(next) = self.cache.node(self.node).next {
            self.node = next;
            self.accessed = false;
            true
        } else {
            false
        }
    }

    /// Updates this reference to point to the next most recently used key in
    /// the list. Returns true if a previous entry was found, or returns false
    /// if the entry is the first entry in the list.
    #[must_use]
    pub fn move_previous(&mut self) -> bool {
        if let Some(previous) = self.cache.node(self.node).previous {
            self.node = previous;
            self.accessed = false;
            true
        } else {
            false
        }
    }

    fn remove_with_direction(mut self, move_next: bool) -> ((Key, Value), Option<Self>) {
        let (removed, next, previous) = self.cache.remove(self.node);
        let new_self = match (move_next, next, previous) {
            (true, Some(next), _) => {
                self.node = next;
                Some(self)
            }
            (false, _, Some(previous)) => {
                self.node = previous;
                Some(self)
            }
            _ => None,
        };
        (removed, new_self)
    }

    /// Removes and returns the current entry's key and value.
    #[must_use]
    pub fn take(self) -> (Key, Value) {
        let (removed, _) = self.remove_with_direction(true);
        removed
    }

    /// Removes and returns the current entry's key and value. If this was not
    /// the last entry, the next entry's [`EntryRef`] will be returned.
    #[must_use]
    pub fn take_and_move_next(self) -> ((Key, Value), Option<Self>) {
        self.remove_with_direction(true)
    }

    /// Removes and returns the current entry's key and value. If this was not
    /// the first entry, the previous entry's [`EntryRef`] will be returned.
    #[must_use]
    pub fn take_and_move_previous(self) -> ((Key, Value), Option<Self>) {
        self.remove_with_direction(false)
    }

    /// Removes the current entry. If this was not the last entry, the next
    /// entry's [`EntryRef`] will be returned.
    #[must_use]
    pub fn remove_moving_next(self) -> Option<Self> {
        let (_, new_self) = self.take_and_move_next();
        new_self
    }

    /// Removes the current entry. If this was not the first entry, the previous
    /// entry's [`EntryRef`] will be returned.
    #[must_use]
    pub fn remove_moving_previous(self) -> Option<Self> {
        let (_, new_self) = self.take_and_move_previous();
        new_self
    }
}

/// A removed value or entry.
#[derive(Debug, Eq, PartialEq)]
pub enum Removed<Key, Value> {
    /// The previously stored value for the key that was written to.
    PreviousValue(Value),
    /// An entry was evicted to make room for the key that was written to.
    Evicted(Key, Value),
}

/// An iterator over a cache's keys and values in order from most recently
/// touched to least recently touched.
pub struct Iter<'a, Key, Value> {
    cache: &'a LruCache<Key, Value>,
    node: Option<NodeId>,
}

impl<'a, Key, Value> Iterator for Iter<'a, Key, Value> {
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        match self.node {
            Some(node) => {
                self.node = self.cache.nodes[node.as_usize()].next;
                Some((
                    self.cache.nodes[node.as_usize()].key(),
                    self.cache.nodes[node.as_usize()].value(),
                ))
            }
            None => None,
        }
    }
}

pub struct IntoIter<Key, Value> {
    cache: LruCache<Key, Value>,
}

impl<Key, Value> From<LruCache<Key, Value>> for IntoIter<Key, Value> {
    fn from(cache: LruCache<Key, Value>) -> Self {
        Self { cache }
    }
}

impl<Key, Value> Iterator for IntoIter<Key, Value> {
    type Item = (Key, Value);

    fn next(&mut self) -> Option<Self::Item> {
        self.cache.head().map(|node| {
            let (removed, ..) = self.cache.remove(node);
            removed
        })
    }
}
