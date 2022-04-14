# LruMap

![lrumap forbids unsafe code](https://img.shields.io/badge/unsafe-forbid-success)
![lrumap is considered alpha](https://img.shields.io/badge/status-alpha-orange)
[![crate version](https://img.shields.io/crates/v/lrumap.svg)](https://crates.io/crates/lrumap)
[![Live Build Status](https://img.shields.io/github/workflow/status/khonsulabs/lrumap/Tests/main)](https://github.com/khonsulabs/lrumap/actions?query=workflow:Tests)
[![HTML Coverage Report for `main` branch](https://khonsulabs.github.io/lrumap/coverage/badge.svg)](https://khonsulabs.github.io/lrumap/coverage/)
[![Documentation for `main` branch](https://img.shields.io/badge/docs-main-informational)](https://khonsulabs.github.io/lrumap/main/lrumap/)

A set of safe Least-Recently-Used (LRU) cache types aimed at providing flexible
map-like structures that automatically evict the least recently used key and
value when its capacity is reached.

## LRU Implementation

This crate utilizes an "arena"-style linked list implementation, where all nodes
of the linked list are stored in a `Vec`. Instead of using pointers to the
nodes, all references to a node in the linked list is done using an index into
the `Vec`.

This allows all LRU list operations to be performed in constant time and remain
very efficient. Each of the implementations in this crate use this internal LRU
linked list implementation.

## LruHashMap

The [`LruHashMap`][lruhashmap] type is an LRU implementation that internally
uses a `HashMap` to track keys. Its performance characteristics will be similar
to the underlying hash map and hash implementation.

For most users, this type will be the best choice.

This crate has no features enabled by default, but transparently can switch to
[`hashbrown`][hashbrown] and its default hasher by enabling feature `hashbrown`.

```rust
use lrumap::{LruHashMap, Removed};

let mut lru = LruHashMap::new(3);
lru.push(1, "one");
lru.push(2, "two");
lru.push(3, "three");

// The cache is now full. The next push will evict the least recently touched entry.
let removed = lru.push(4, "four");
assert_eq!(removed, Some(Removed::Evicted(1, "one")));
```

## LruBTreeMap

The [`LruBTreeMap`][lrubtreemap] type is an LRU implementation that internally
uses a `BTreeMap` to track keys. Its performance characteristics will be similar
to the underlying container implementation.

By using a `BTreeMap` to track keys, this type enables efficient range-based
queries:

```rust
use lrumap::LruBTreeMap;

let mut lru = LruBTreeMap::new(5);
lru.extend([(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)]);
assert_eq!(lru.most_recent_in_range(2..=4).unwrap().key(), &4);

// Change the order by retrieving key 2.
lru.get(&2);
assert_eq!(lru.most_recent_in_range(2..=4).unwrap().key(), &2);
```

## Why another LRU crate?

For [Nebari][nebari], we needed to introduce an LRU cache to the
`StdFileManager` to close files that haven't been used recently. Each file can
have multiple readers, leading to an issue of needing to scan the LRU map for
all values that match a specific file. In the end, [@ecton][ecton] decided an
LRU implementation that used a `BTreeMap` instead of a `HashMap` would be able
to solve this problem by offering an
[`most_recent_in_range(key)`][most-recent-in-range] function. No existing crates
seemed to offer this functionality.

[nebari]: https://github.com/khonsulabs/nebari
[ecton]: https://github.com/ecton
[most-recent-in-range]: https://khonsulabs.github.io/lrumap/main/lrumap/struct.LruBTreeMap.html#method.most_recent_in_range
[lruhashmap]: https://khonsulabs.github.io/lrumap/main/lrumap/struct.LruHashMap.html
[lrubtreemap]: https://khonsulabs.github.io/lrumap/main/lrumap/struct.LruBTreeMap.html
[hashbrown]: https://docs.rs/hashbrown/latest/hashbrown/

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), are
open-source. This repository is available under the [MIT License](./LICENSE-MIT)
or the [Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
