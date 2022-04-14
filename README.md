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

## Open-source Licenses

This project, like all projects from [Khonsu Labs](https://khonsulabs.com/), are
open-source. This repository is available under the [MIT License](./LICENSE-MIT)
or the [Apache License 2.0](./LICENSE-APACHE).

To learn more about contributing, please see [CONTRIBUTING.md](./CONTRIBUTING.md).
