//! # Solution 08: Idempotency Benchmark
//!
//! Complete benchmark suite comparing idempotency strategies.

use criterion::{black_box, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory idempotency check using a HashMap.
pub fn check_in_memory(
    store: &HashMap<String, String>,
    key: &str,
) -> bool {
    !store.contains_key(key)
}

/// Redis SET NX idempotency check.
pub async fn check_redis(pool: &deadpool_redis::Pool, key: &str) -> Result<bool, String> {
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Connection error: {e}"))?;

    let was_set: bool = redis::cmd("SET")
        .arg(key)
        .arg("processing")
        .arg("NX")
        .arg("EX")
        .arg(60)
        .query_async(&mut *conn)
        .await
        .map_err(|e| format!("Redis error: {e}"))?;

    Ok(was_set)
}

/// SQLite unique constraint idempotency check.
pub async fn check_sqlite(pool: &sqlx::SqlitePool, key: &str) -> Result<bool, String> {
    let result = sqlx::query(
        "INSERT INTO idempotency_keys (key, result, expires_at)
         VALUES (?1, NULL, datetime('now', '+3600 seconds'))
         ON CONFLICT(key) DO NOTHING",
    )
    .bind(key)
    .execute(pool)
    .await
    .map_err(|e| format!("SQLite error: {e}"))?;

    Ok(result.rows_affected() > 0)
}

/// Set up an in-memory SQLite database for benchmarking.
pub async fn setup_sqlite() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create SQLite pool");

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS idempotency_keys (
            key TEXT PRIMARY KEY,
            result TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            expires_at TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    pool
}

/// Run all benchmarks. Called from the bench binary.
pub fn run_benchmarks(c: &mut Criterion) {
    bench_in_memory_latency(c);
    bench_redis_latency(c);
    bench_sqlite_latency(c);
    bench_concurrent_throughput(c);
    bench_hit_vs_miss(c);
}

/// Pre-populate a HashMap with N keys for "duplicate" checks.
fn populate_hashmap(n: usize) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for i in 0..n {
        map.insert(format!("key-{i}"), "cached".to_string());
    }
    map
}

/// Pre-populate SQLite with N keys for "duplicate" checks.
async fn populate_sqlite(pool: &sqlx::SqlitePool, n: usize) {
    for i in 0..n {
        sqlx::query(
            "INSERT OR IGNORE INTO idempotency_keys (key, result, expires_at)
             VALUES (?1, 'cached', datetime('now', '+3600 seconds'))",
        )
        .bind(format!("key-{i}"))
        .execute(pool)
        .await
        .expect("insert");
    }
}

fn bench_in_memory_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("in_memory_latency");

    let store = populate_hashmap(10_000);

    group.bench_function("hit_existing_key", |b| {
        b.iter(|| {
            check_in_memory(black_box(&store), black_box("key-5000"))
        })
    });

    group.bench_function("miss_new_key", |b| {
        b.iter(|| {
            check_in_memory(black_box(&store), black_box("new-key-99999"))
        })
    });

    group.finish();
}

fn bench_redis_latency(c: &mut Criterion) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return,
    };

    let pool = match rt.block_on(async {
        let cfg = deadpool_redis::Config::from_url("redis://127.0.0.1:6379");
        cfg.builder()
            .expect("builder")
            .max_size(16)
            .build()
    }) {
        Ok(p) => p,
        Err(_) => {
            eprintln!("SKIP: Redis not available for benchmarking");
            return;
        }
    };

    let mut group = c.benchmark_group("redis_latency");
    group.sample_size(100);

    let key_counter = Arc::new(Mutex::new(0u64));

    group.bench_function("set_nx_new_key", |b| {
        let counter = key_counter.clone();
        b.iter(|| {
            let mut count = counter.lock().unwrap();
            *count += 1;
            let key = format!("bench:redis:{}", *count);
            rt.block_on(check_redis(black_box(&pool), black_box(&key)))
        })
    });

    rt.block_on(async {
        let mut conn = pool.get().await.unwrap();
        let _: () = redis::cmd("SET")
            .arg("bench:redis:existing")
            .arg("cached")
            .query_async(&mut *conn)
            .await
            .unwrap();
    });

    group.bench_function("hit_existing_key", |b| {
        b.iter(|| {
            rt.block_on(check_redis(black_box(&pool), black_box("bench:redis:existing")))
        })
    });

    group.finish();
}

fn bench_sqlite_latency(c: &mut Criterion) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return,
    };

    let pool = rt.block_on(setup_sqlite());
    rt.block_on(populate_sqlite(&pool, 10_000));

    let mut group = c.benchmark_group("sqlite_latency");
    group.sample_size(100);

    let key_counter = Arc::new(Mutex::new(0u64));

    group.bench_function("insert_new_key", |b| {
        let counter = key_counter.clone();
        b.iter(|| {
            let mut count = counter.lock().unwrap();
            *count += 1;
            let key = format!("bench:sqlite:{}", *count);
            rt.block_on(check_sqlite(black_box(&pool), black_box(&key)))
        })
    });

    group.bench_function("conflict_existing_key", |b| {
        b.iter(|| {
            rt.block_on(check_sqlite(black_box(&pool), black_box("key-5000")))
        })
    });

    group.finish();
}

fn bench_concurrent_throughput(c: &mut Criterion) {
    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => return,
    };

    let mut group = c.benchmark_group("concurrent_throughput");

    for concurrency in [1, 10, 50, 100] {
        group.throughput(Throughput::Elements(concurrency as u64));
        group.bench_with_input(
            BenchmarkId::new("in_memory", concurrency),
            &concurrency,
            |b, &conc| {
                let store: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));
                b.iter(|| {
                    let store = store.clone();
                    rt.block_on(async move {
                        let barrier = Arc::new(tokio::sync::Barrier::new(conc));
                        let mut handles = Vec::new();

                        for _ in 0..conc {
                            let b = barrier.clone();
                            let s = store.clone();
                            handles.push(tokio::spawn(async move {
                                b.wait().await;
                                let map = s.lock().unwrap();
                                map.contains_key("shared-key")
                            }));
                        }

                        for h in handles {
                            let _ = h.await;
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_hit_vs_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("hit_vs_miss");

    let store = populate_hashmap(10_000);

    group.bench_function("memory_hit", |b| {
        b.iter(|| check_in_memory(black_box(&store), black_box("key-5000")))
    });

    group.bench_function("memory_miss", |b| {
        b.iter(|| check_in_memory(black_box(&store), black_box("nonexistent-key")))
    });

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(_) => {
            group.finish();
            return;
        }
    };

    if let Ok(pool) = rt.block_on(async {
        let pool = setup_sqlite().await;
        populate_sqlite(&pool, 10_000).await;
        Ok::<_, String>(pool)
    }) {
        group.bench_function("sqlite_hit", |b| {
            b.iter(|| rt.block_on(check_sqlite(black_box(&pool), black_box("key-5000"))))
        });

        let counter = Arc::new(Mutex::new(0u64));
        group.bench_function("sqlite_miss", |b| {
            let c = counter.clone();
            b.iter(|| {
                let mut count = c.lock().unwrap();
                *count += 1;
                let key = format!("miss-key-{}", *count);
                rt.block_on(check_sqlite(black_box(&pool), black_box(&key)))
            })
        });
    }

    group.finish();
}
