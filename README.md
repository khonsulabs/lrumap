# LruMap

A set of safe Least-Recently-Used (LRU) cache types aimed at providing flexible
map-like structures that automatically evict the least recently used key and
value when its capacity is reached.

## Why another LRU crate?

For [Nebari][nebari], we needed to introduce an LRU cache to the
`StdFileManager` to close files that haven't been used recently. Each file can
have multiple readers, leading to an issue of needing to scan the LRU map for
all values that match a specific file. In the end, [@ecton][ecton] decided an
LRU implementation that used a `BTreeMap` instead of a `HashMap` would be able
to solve this problem by offering an `iter_starting_at(key)` function. No
existing crates seemed to offer this functionality.

[nebari]: https://github.com/khonsulabs/nebari
[ecton]: https://github.com/ecton
[lru-rs]: https://github.com/jeromefroe/lru-rs
