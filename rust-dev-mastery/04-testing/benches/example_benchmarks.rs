use criterion::{black_box, criterion_group, criterion_main, Criterion};
use testing_mastery::p07_benchmarking::{BenchStore, concat_format, concat_push, concat_collect};

fn bench_store_insert(c: &mut Criterion) {
    c.bench_function("store_insert", |b| {
        let mut store = BenchStore::new();
        let mut i = 0u64;
        b.iter(|| {
            store.insert(&format!("key-{}", i), &format!("value-{}", i));
            i += 1;
        });
    });
}

fn bench_store_get(c: &mut Criterion) {
    let store = BenchStore::with_entries(1000);
    c.bench_function("store_get", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let key = format!("key-{}", i % 1000);
            black_box(store.get(&key));
            i += 1;
        });
    });
}

fn bench_concat(c: &mut Criterion) {
    let mut group = c.benchmark_group("concat");
    group.bench_function("format", |b| b.iter(|| black_box(concat_format(100))));
    group.bench_function("push", |b| b.iter(|| black_box(concat_push(100))));
    group.bench_function("collect", |b| b.iter(|| black_box(concat_collect(100))));
    group.finish();
}

criterion_group!(benches, bench_store_insert, bench_store_get, bench_concat);
criterion_main!(benches);
