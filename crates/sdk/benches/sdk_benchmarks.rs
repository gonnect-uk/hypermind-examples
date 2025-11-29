//! SDK performance benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_kgdb_sdk::{GraphDB, Node};

fn benchmark_insert_single(c: &mut Criterion) {
    c.bench_function("sdk_insert_single", |b| {
        b.iter(|| {
            let mut db = GraphDB::in_memory();
            db.insert()
                .triple(
                    Node::iri("http://example.org/alice"),
                    Node::iri("http://xmlns.com/foaf/0.1/name"),
                    Node::literal("Alice"),
                )
                .execute()
                .expect("Insert should succeed");
            black_box(db);
        });
    });
}

fn benchmark_insert_batch(c: &mut Criterion) {
    c.bench_function("sdk_insert_100", |b| {
        b.iter(|| {
            let mut db = GraphDB::in_memory();
            for i in 0..100 {
                db.insert()
                    .triple(
                        Node::iri(format!("http://example.org/person{}", i)),
                        Node::iri("http://xmlns.com/foaf/0.1/name"),
                        Node::literal(format!("Person {}", i)),
                    )
                    .execute()
                    .expect("Insert should succeed");
            }
            black_box(db);
        });
    });
}

fn benchmark_query(c: &mut Criterion) {
    let mut db = GraphDB::in_memory();
    for i in 0..100 {
        db.insert()
            .triple(
                Node::iri(format!("http://example.org/person{}", i)),
                Node::iri("http://xmlns.com/foaf/0.1/name"),
                Node::literal(format!("Person {}", i)),
            )
            .execute()
            .expect("Insert should succeed");
    }

    c.bench_function("sdk_query_select_all", |b| {
        b.iter(|| {
            let results = db
                .query()
                .sparql("SELECT ?s ?p ?o WHERE { ?s ?p ?o }")
                .execute()
                .expect("Query should succeed");
            black_box(results);
        });
    });
}

criterion_group!(benches, benchmark_insert_single, benchmark_insert_batch, benchmark_query);
criterion_main!(benches);
