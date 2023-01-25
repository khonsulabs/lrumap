use std::collections::HashSet;
use std::fmt::Debug;
use std::marker::PhantomData;

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
            node: IterState::BeforeHead,
        }
    }

    pub fn get(&mut self, node: NodeId) -> &Node<Key, Value> {
        self.touch(node);
        &self.nodes[node.as_usize()]
    }

    pub fn get_without_touch(&self, node: NodeId) -> &Node<Key, Value> {
        &self.nodes[node.as_usize()]
    }

    pub fn get_mut(&mut self, node: NodeId) -> &mut Node<Key, Value> {
        self.touch(node);
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

    pub fn touch(&mut self, node_index: NodeId) {
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

    fn push_front(&mut self, key: Key, value: Value) -> (NodeId, Option<(Key, Value)>) {
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

    fn allocate_node(&mut self, key: Key, value: Value) -> (NodeId, Option<(Key, Value)>) {
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
                current_node = if let Some(next) = node.next {
                    next
                } else {
                    end_found = true;
                    break;
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
            Self::Occupied { key, value } => (key, value),
            Self::Vacant => unreachable!("evict called on a vacant entry"),
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
    pub const fn last_accessed(&self) -> usize {
        self.last_accessed
    }

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

    pub fn value_mut(&mut self) -> &mut Value {
        match &mut self.entry {
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
    fn cache(&self) -> &LruCache<Key, Value>;
    fn cache_mut(&mut self) -> &mut LruCache<Key, Value>;
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

    /// Returns the key of this entry.
    #[must_use]
    pub fn key(&self) -> &Key {
        self.cache.cache().get_without_touch(self.node).key()
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
        self.cache.cache_mut().get(self.node).value()
    }

    /// Touches this key, making it the most recently used key.
    pub fn touch(&mut self) {
        self.cache.cache_mut().touch(self.node);
    }

    /// Returns the value of this entry.
    ///
    /// This function does not touch the key, preserving its current position in
    /// the lru cache.
    #[must_use]
    pub fn peek_value(&self) -> &Value {
        self.cache.cache().get_without_touch(self.node).value()
    }

    /// Returns the number of changes to the cache since this key was last
    /// touched.
    #[must_use]
    pub fn staleness(&self) -> usize {
        self.cache.cache().sequence().wrapping_sub(
            self.cache
                .cache()
                .get_without_touch(self.node)
                .last_accessed,
        )
    }

    /// Returns an iterator over the least-recently used keys beginning with the
    /// current entry.
    pub fn iter(&self) -> Iter<'_, Key, Value> {
        Iter {
            cache: self.cache.cache(),
            node: IterState::StartingAt(self.node),
        }
    }

    /// Updates this reference to point to the next least recently used key in
    /// the list. Returns true if a next entry was found, or returns false if
    /// the entry is the last entry in the list.
    #[must_use]
    pub fn move_next(&mut self) -> bool {
        if let Some(next) = self.cache.cache().get_without_touch(self.node).next {
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
        if let Some(previous) = self.cache.cache().get_without_touch(self.node).previous {
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

/// A double-ended iterator over a cache's keys and values in order from most
/// recently touched to least recently touched.
#[must_use]
pub struct Iter<'a, Key, Value> {
    cache: &'a LruCache<Key, Value>,
    node: IterState,
}

enum IterState {
    BeforeHead,
    AfterTail,
    StartingAt(NodeId),
    Node(NodeId),
}

impl<'a, Key, Value> Iterator for Iter<'a, Key, Value> {
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<Self::Item> {
        let next_node = match self.node {
            IterState::BeforeHead => self.cache.head,
            IterState::StartingAt(node) => Some(node),
            IterState::Node(node) => self.cache.nodes[node.as_usize()].next,
            IterState::AfterTail => None,
        };
        if let Some(node_id) = next_node {
            let node = &self.cache.nodes[node_id.as_usize()];
            self.node = IterState::Node(node_id);
            Some((node.key(), node.value()))
        } else {
            self.node = IterState::AfterTail;
            None
        }
    }
}
impl<'a, Key, Value> DoubleEndedIterator for Iter<'a, Key, Value> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let previous_node = match self.node {
            IterState::BeforeHead => None,
            IterState::StartingAt(node) | IterState::Node(node) => {
                self.cache.nodes[node.as_usize()].previous
            }
            IterState::AfterTail => self.cache.tail,
        };
        if let Some(node_id) = previous_node {
            let node = &self.cache.nodes[node_id.as_usize()];
            self.node = IterState::Node(node_id);
            Some((node.key(), node.value()))
        } else {
            self.node = IterState::BeforeHead;
            None
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
