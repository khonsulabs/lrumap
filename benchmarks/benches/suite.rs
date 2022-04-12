use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

pub fn benchmark(c: &mut Criterion) {
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

criterion_group!(suite, benchmark);
criterion_main!(suite);
