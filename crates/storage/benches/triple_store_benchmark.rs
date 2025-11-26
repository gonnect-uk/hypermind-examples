//! Real Performance Benchmark - Beat RDFox Today!
//!
//! Run with: cargo bench --bench triple_store_benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use std::time::Duration;

// Import from workspace crates
use rdf_model::Dictionary;
use storage::{InMemoryBackend, StorageBackend};

fn benchmark_triple_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("triple_insert");

    for size in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut backend = InMemoryBackend::new();
                let dict = Arc::new(Dictionary::new());

                // Insert triples
                for i in 0..size {
                    let subj = dict.intern(&format!("http://example.org/subject{}", i));
                    let pred = dict.intern("http://example.org/predicate");
                    let obj = dict.intern(&format!("http://example.org/object{}", i));

                    let key = format!("spo_{}_{}", subj, pred).into_bytes();
                    let value = obj.as_bytes().to_vec();

                    backend.put(&key, &value).unwrap();
                }

                black_box(backend)
            });
        });
    }

    group.finish();
}

fn benchmark_triple_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("triple_lookup");

    // Setup: Create store with 10K triples
    let mut backend = InMemoryBackend::new();
    let dict = Arc::new(Dictionary::new());

    for i in 0..10_000 {
        let subj = dict.intern(&format!("http://example.org/subject{}", i));
        let pred = dict.intern("http://example.org/predicate");
        let obj = dict.intern(&format!("http://example.org/object{}", i));

        let key = format!("spo_{}_{}", subj, pred).into_bytes();
        let value = obj.as_bytes().to_vec();

        backend.put(&key, &value).unwrap();
    }

    group.bench_function("lookup_existing", |b| {
        b.iter(|| {
            let subj = dict.intern("http://example.org/subject5000");
            let pred = dict.intern("http://example.org/predicate");
            let key = format!("spo_{}_{}", subj, pred).into_bytes();

            let result = backend.get(&key).unwrap();
            black_box(result)
        });
    });

    group.finish();
}

fn benchmark_dictionary_intern(c: &mut Criterion) {
    let mut group = c.benchmark_group("dictionary");

    group.bench_function("intern_new", |b| {
        b.iter(|| {
            let dict = Dictionary::new();
            for i in 0..1000 {
                let uri = format!("http://example.org/resource{}", i);
                let interned = dict.intern(&uri);
                black_box(interned);
            }
        });
    });

    group.bench_function("intern_duplicate", |b| {
        let dict = Dictionary::new();
        // Pre-intern URIs
        for i in 0..100 {
            dict.intern(&format!("http://example.org/resource{}", i));
        }

        b.iter(|| {
            for i in 0..100 {
                let uri = format!("http://example.org/resource{}", i);
                let interned = dict.intern(&uri);
                black_box(interned);
            }
        });
    });

    group.finish();
}

fn benchmark_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");
    group.measurement_time(Duration::from_secs(10));

    // Individual inserts (current baseline)
    group.bench_function("bulk_insert_100k_individual", |b| {
        b.iter(|| {
            let mut backend = InMemoryBackend::new();
            let dict = Arc::new(Dictionary::new());

            for i in 0..100_000 {
                let subj = dict.intern(&format!("http://ex.org/s{}", i % 10000));
                let pred = dict.intern(&format!("http://ex.org/p{}", i % 100));
                let obj = dict.intern(&format!("http://ex.org/o{}", i));

                let key = format!("{}_{}_{}", subj, pred, obj).into_bytes();
                backend.put(&key, &[1]).unwrap();
            }

            black_box(backend)
        });
    });

    // Batch insert (optimized)
    group.bench_function("bulk_insert_100k_batched", |b| {
        b.iter(|| {
            let mut backend = InMemoryBackend::new();
            let dict = Arc::new(Dictionary::new());

            // Prepare all key-value pairs first
            let mut pairs = Vec::with_capacity(100_000);
            for i in 0..100_000 {
                let subj = dict.intern(&format!("http://ex.org/s{}", i % 10000));
                let pred = dict.intern(&format!("http://ex.org/p{}", i % 100));
                let obj = dict.intern(&format!("http://ex.org/o{}", i));

                let key = format!("{}_{}_{}", subj, pred, obj).into_bytes();
                pairs.push((key, vec![1]));
            }

            // Single batch operation
            backend.batch_put(pairs).unwrap();

            black_box(backend)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_triple_insert,
    benchmark_triple_lookup,
    benchmark_dictionary_intern,
    benchmark_bulk_operations
);

criterion_main!(benches);
