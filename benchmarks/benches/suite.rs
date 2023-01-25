use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::prelude::SliceRandom;
use rand::thread_rng;

fn benchmark(c: &mut Criterion) {
    push(c);
    get(c);
}

fn push(c: &mut Criterion) {
    let mut group = c.benchmark_group("push");

    let mut entries = Vec::new();
    for i in 0_u32..=2000 {
        entries.push(i);
    }
    for i in (0_u32..=2000).rev() {
        entries.push(i);
    }

    group.bench_function("ordered-lru", |b| {
        let mut lru = lrumap::LruBTreeMap::new(1000);
        b.iter_batched(
            || entries.clone(),
            |batch: Vec<u32>| {
                for i in batch {
                    lru.push(i, i);
                }
            },
            BatchSize::NumIterations(u64::try_from(entries.len()).unwrap()),
        )
    });

    group.bench_function("hashed-lru", |b| {
        let mut lru = lrumap::LruHashMap::new(1000);
        b.iter_batched(
            || entries.clone(),
            |batch: Vec<u32>| {
                for i in batch {
                    lru.push(i, i);
                }
            },
            BatchSize::NumIterations(u64::try_from(entries.len()).unwrap()),
        )
    });

    group.bench_function("lru", |b| {
        let mut lru = lru::LruCache::new(1000);
        b.iter_batched(
            || &entries[..],
            |batch: &[u32]| {
                for i in batch {
                    lru.push(i, i);
                }
            },
            BatchSize::NumIterations(u64::try_from(entries.len()).unwrap()),
        )
    });
}

fn get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    let mut ordered = lrumap::LruBTreeMap::new(1000);
    let mut unordered = lrumap::LruHashMap::new(1000);
    let mut lru = lru::LruCache::new(1000);
    let mut indicies = Vec::new();
    for i in 0_u32..=1000 {
        indicies.push(i);
        ordered.push(i, i);
        unordered.push(i, i);
        lru.push(i, i);
    }

    indicies.shuffle(&mut thread_rng());

    group.bench_function("ordered-lru", |b| {
        b.iter_batched(
            || &indicies[..],
            |indicies: &[u32]| {
                for i in indicies {
                    ordered.get(i);
                }
            },
            BatchSize::NumIterations(u64::try_from(indicies.len()).unwrap()),
        )
    });

    group.bench_function("hashed-lru", |b| {
        b.iter_batched(
            || &indicies[..],
            |indicies: &[u32]| {
                for i in indicies {
                    unordered.get(i);
                }
            },
            BatchSize::NumIterations(u64::try_from(indicies.len()).unwrap()),
        )
    });

    group.bench_function("lru", |b| {
        b.iter_batched(
            || &indicies[..],
            |indicies: &[u32]| {
                for i in indicies {
                    lru.get(i);
                }
            },
            BatchSize::NumIterations(u64::try_from(indicies.len()).unwrap()),
        )
    });
}

criterion_group!(suite, benchmark);
criterion_main!(suite);
