use std::{collections::HashSet, fmt::Debug};

pub struct LruCache<Key, Value> {
    nodes: Vec<Node<Key, Value>>,
    head: Option<u32>,
    tail: Option<u32>,
    vacant: Option<u32>,
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

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn head(&self) -> Option<u32> {
        self.head
    }

    pub fn tail(&self) -> Option<u32> {
        self.tail
    }

    pub fn get(&mut self, node: u32) -> &Node<Key, Value> {
        self.move_node_to_front(node);
        &self.nodes[node as usize]
    }

    pub fn get_without_update(&self, node: u32) -> &Node<Key, Value> {
        &self.nodes[node as usize]
    }

    pub fn get_mut(&mut self, node: u32) -> &mut Node<Key, Value> {
        self.move_node_to_front(node);
        &mut self.nodes[node as usize]
    }

    pub fn push(&mut self, key: Key, value: Value) -> (u32, Option<Removed<Key, Value>>) {
        let (node, result) = if self.head.is_some() {
            self.push_front(key, value)
        } else {
            // First node of the list.
            self.allocate_node(key, value)
        };
        (
            node,
            result.map(|(key, value)| Removed::Expired(key, value)),
        )
    }

    pub fn move_node_to_front(&mut self, node_index: u32) {
        if self.head == Some(node_index) {
            // No-op.
            return;
        }

        self.sequence += 1;

        // An entry already exists. Reuse the node.
        self.nodes[node_index as usize].last_accessed = self.sequence;

        // Update the next pointer's previous to this node's previous.
        if let Some(next_node) = self.nodes[node_index as usize].next {
            self.nodes[next_node as usize].previous = self.nodes[node_index as usize].previous;
        }

        // Update the previous pointer's next to this
        if let Some(previous_node_ref) = self.nodes[node_index as usize].previous.take() {
            self.nodes[previous_node_ref as usize].next =
                self.nodes[node_index as usize].next.take();
            if self.tail == Some(node_index) {
                self.tail = Some(previous_node_ref);
            }
        }

        // Move this node to the front
        {
            let head = self.head.unwrap();
            debug_assert!(self.nodes[head as usize].previous.is_none());
            self.nodes[head as usize].previous = Some(node_index);
        }

        self.nodes[node_index as usize].next = self.head;
        self.head = Some(node_index);
    }

    pub fn push_front(&mut self, key: Key, value: Value) -> (u32, Option<(Key, Value)>) {
        let (node, removed) = self.allocate_node(key, value);
        self.sequence += 1;
        let mut entry = &mut self.nodes[node as usize];
        entry.last_accessed = self.sequence;
        entry.next = Some(self.head.unwrap());

        let mut previous_head = &mut self.nodes[self.head.unwrap() as usize];
        debug_assert!(previous_head.previous.is_none());
        previous_head.previous = Some(node as u32);
        self.head = Some(node);
        (node, removed)
    }

    pub fn allocate_node(&mut self, key: Key, value: Value) -> (u32, Option<(Key, Value)>) {
        if let Some(vacant) = self.vacant {
            // Pull a node off the vacant list.
            self.vacant = self.nodes[vacant as usize].next;
            self.nodes[vacant as usize].next = None;
            self.nodes[vacant as usize].entry = Entry::Occupied { key, value };
            self.length += 1;
            (vacant, None)
        } else if self.nodes.len() == self.nodes.capacity() {
            // Expire the least recently used key (tail).
            let index = self.tail.unwrap();

            if let Some(previous) = self.nodes[index as usize].previous {
                self.nodes[previous as usize].next = None;
            }
            self.nodes[index as usize].previous = None;

            let mut entry = Entry::Occupied { key, value };
            std::mem::swap(&mut entry, &mut self.nodes[index as usize].entry);

            (index, entry.into())
        } else {
            // We have capacity to fill.
            let index = self.nodes.len() as u32;
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
                let node = &self.nodes[current_node as usize];
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

impl<Key, Value> From<Entry<Key, Value>> for Option<(Key, Value)> {
    fn from(entry: Entry<Key, Value>) -> Self {
        match entry {
            Entry::Occupied { key, value } => Some((key, value)),
            Entry::Vacant => None,
        }
    }
}

pub struct Node<Key, Value> {
    last_accessed: usize,
    previous: Option<u32>,
    next: Option<u32>,
    entry: Entry<Key, Value>,
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
    pub fn key(&self) -> Option<&Key> {
        match &self.entry {
            Entry::Occupied { key, .. } => Some(key),
            Entry::Vacant => None,
        }
    }

    pub fn value(&self) -> Option<&Value> {
        match &self.entry {
            Entry::Occupied { value, .. } => Some(value),
            Entry::Vacant => None,
        }
    }

    pub fn replace_value(&mut self, mut new_value: Value) -> Value {
        match &mut self.entry {
            Entry::Occupied { value, .. } => {
                std::mem::swap(value, &mut new_value);
                new_value
            }
            Entry::Vacant => unreachable!("invalid state"),
        }
    }
}

#[derive(Debug)]
pub struct EntryRef<'a, Key, Value> {
    cache: &'a mut LruCache<Key, Value>,
    node: u32,
    accessed: bool,
}

impl<'a, Key, Value> EntryRef<'a, Key, Value> {
    pub(crate) fn new(cache: &'a mut LruCache<Key, Value>, node: u32) -> Self {
        Self {
            node,
            cache,
            accessed: false,
        }
    }

    pub fn id(&self) -> u32 {
        self.node
    }

    pub fn key(&self) -> &Key {
        self.cache.nodes[self.node as usize].key().unwrap()
    }

    pub fn value(&mut self) -> &Value {
        if !self.accessed {
            self.accessed = true;
            self.cache.move_node_to_front(self.node);
        }
        self.cache.nodes[self.node as usize].value().unwrap()
    }

    pub fn peek_value(&self) -> &Value {
        self.cache.nodes[self.node as usize].value().unwrap()
    }

    pub fn staleness(&self) -> usize {
        self.cache
            .sequence
            .wrapping_sub(self.cache.nodes[self.node as usize].last_accessed)
    }

    pub fn move_next(&mut self) -> bool {
        if let Some(next) = self.cache.nodes[self.node as usize].next {
            self.node = next;
            self.accessed = false;
            true
        } else {
            false
        }
    }

    pub fn move_previous(&mut self) -> bool {
        if let Some(previous) = self.cache.nodes[self.node as usize].previous {
            self.node = previous;
            self.accessed = false;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Removed<Key, Value> {
    PreviousValue(Value),
    Expired(Key, Value),
}
