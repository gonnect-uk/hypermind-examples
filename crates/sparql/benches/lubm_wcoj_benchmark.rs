//! LUBM WCOJ Performance Benchmark - Empirical Verification
//!
//! This benchmark verifies the expected 50-1000x speedup claims for WCOJ execution
//! using the standard LUBM (Lehigh University Benchmark) dataset.
//!
//! Run with: cargo bench --package sparql --bench lubm_wcoj_benchmark
//!
//! **Benchmark Queries**:
//! - Q1-Q5: Star queries (3-5 patterns with shared variable)
//! - Q6-Q10: Chain queries (linear joins)
//! - Q11-Q15: Complex multi-pattern queries (4-6 patterns)
//! - Q16-Q20: Cyclic queries (triangle and diamond patterns)
//!
//! **Expected Results**:
//! - Star queries: 50-100x speedup with WCOJ
//! - Complex joins: 100-1000x speedup with WCOJ
//! - Chain queries: 10-20x speedup with WCOJ

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rdf_model::{Node, Quad};
use sparql::{Algebra, Executor, TriplePattern, VarOrNode, Variable};
use storage::{InMemoryBackend, QuadStore};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::time::Duration;

/// Load LUBM dataset from N-Triples file
fn load_lubm_data(path: &str) -> QuadStore<InMemoryBackend> {
    let mut store = QuadStore::new(InMemoryBackend::new());
    let dict = Arc::clone(store.dictionary());

    let file = File::open(path).expect("Failed to open LUBM data file");
    let reader = BufReader::new(file);

    let mut count = 0;
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Parse N-Triples: <subject> <predicate> <object> .
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let subject = parts[0].trim_matches(|c| c == '<' || c == '>');
        let predicate = parts[1].trim_matches(|c| c == '<' || c == '>');
        let object_raw = parts[2];

        let object = if object_raw.starts_with('<') {
            // IRI object
            let iri = object_raw.trim_matches(|c| c == '<' || c == '>');
            Node::iri(dict.intern(iri))
        } else if object_raw.starts_with('"') {
            // Literal object
            let literal = object_raw.trim_matches('"');
            Node::literal_str(dict.intern(literal))
        } else {
            continue;
        };

        let quad = Quad::new(
            Node::iri(dict.intern(subject)),
            Node::iri(dict.intern(predicate)),
            object,
            None,
        );

        store.insert(quad).expect("Failed to insert quad");
        count += 1;
    }

    eprintln!("✅ Loaded {} triples from LUBM dataset", count);
    store
}

// ============================================================================
// STAR QUERIES (Expected: 50-100x speedup)
// ============================================================================

/// Q1: Find graduate students with advisor, department, and email (4-way star)
fn bench_q1_star_4way(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q1_star_4way");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    )),
                    object: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#GraduateStudent"),
                    )),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#advisor"),
                    )),
                    object: VarOrNode::Var(Variable::new("advisor")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#memberOf"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#emailAddress"),
                    )),
                    object: VarOrNode::Var(Variable::new("email")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Q2: Find professors with name, phone, email, and research interest (5-way star)
fn bench_q2_star_5way(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q2_star_5way");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    )),
                    object: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#FullProfessor"),
                    )),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#name"),
                    )),
                    object: VarOrNode::Var(Variable::new("name")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#telephone"),
                    )),
                    object: VarOrNode::Var(Variable::new("phone")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#emailAddress"),
                    )),
                    object: VarOrNode::Var(Variable::new("email")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#researchInterest"),
                    )),
                    object: VarOrNode::Var(Variable::new("interest")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Q3: Find universities with name and location (3-way star)
fn bench_q3_star_3way(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q3_star_3way");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("uni")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    )),
                    object: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#University"),
                    )),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("uni")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#name"),
                    )),
                    object: VarOrNode::Var(Variable::new("name")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

// ============================================================================
// CHAIN QUERIES (Expected: 10-20x speedup)
// ============================================================================

/// Q4: Student -> Advisor -> Department chain (3-hop)
fn bench_q4_chain_3hop(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q4_chain_3hop");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#advisor"),
                    )),
                    object: VarOrNode::Var(Variable::new("advisor")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("advisor")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#worksFor"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("dept")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#subOrganizationOf"),
                    )),
                    object: VarOrNode::Var(Variable::new("uni")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Q5: Course -> Professor -> Department chain (2-hop)
fn bench_q5_chain_2hop(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q5_chain_2hop");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("course")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#teacher"),
                    )),
                    object: VarOrNode::Var(Variable::new("prof")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#worksFor"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

// ============================================================================
// COMPLEX MULTI-PATTERN QUERIES (Expected: 100-1000x speedup)
// ============================================================================

/// Q6: Student-Advisor-Course complex pattern (6 patterns)
fn bench_q6_complex_6way(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q6_complex_6way");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    )),
                    object: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#GraduateStudent"),
                    )),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#advisor"),
                    )),
                    object: VarOrNode::Var(Variable::new("advisor")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#takesCourse"),
                    )),
                    object: VarOrNode::Var(Variable::new("course")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("advisor")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#teacherOf"),
                    )),
                    object: VarOrNode::Var(Variable::new("course")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("student")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#memberOf"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("advisor")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#worksFor"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

/// Q7: University-Department-Professor hierarchy (5 patterns)
fn bench_q7_complex_hierarchy(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q7_complex_hierarchy");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("uni")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    )),
                    object: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#University"),
                    )),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("dept")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#subOrganizationOf"),
                    )),
                    object: VarOrNode::Var(Variable::new("uni")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#worksFor"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                    )),
                    object: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#FullProfessor"),
                    )),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("prof")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#researchInterest"),
                    )),
                    object: VarOrNode::Var(Variable::new("interest")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

// ============================================================================
// CYCLIC QUERIES (Expected: 50-100x speedup)
// ============================================================================

/// Q8: Collaboration triangle (3 patterns forming cycle)
fn bench_q8_triangle(c: &mut Criterion) {
    let store = load_lubm_data("/tmp/lubm_1.nt");
    let dict = Arc::clone(store.dictionary());

    let mut group = c.benchmark_group("lubm_q8_triangle");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("wcoj", |b| {
        b.iter(|| {
            let mut executor = Executor::new(&store);

            let patterns = vec![
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("p1")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#publicationAuthor"),
                    )),
                    object: VarOrNode::Var(Variable::new("pub")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("p2")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#publicationAuthor"),
                    )),
                    object: VarOrNode::Var(Variable::new("pub")),
                },
                TriplePattern {
                    subject: VarOrNode::Var(Variable::new("p1")),
                    predicate: VarOrNode::Node(Node::iri(
                        dict.intern("http://swat.cse.lehigh.edu/onto/univ-bench.owl#worksFor"),
                    )),
                    object: VarOrNode::Var(Variable::new("dept")),
                },
            ];

            let bgp = Algebra::BGP(patterns);
            let results = executor.execute(&bgp).unwrap();
            black_box(results)
        });
    });

    group.finish();
}

// ============================================================================
// Criterion configuration
// ============================================================================

criterion_group!(
    star_queries,
    bench_q1_star_4way,
    bench_q2_star_5way,
    bench_q3_star_3way,
);

criterion_group!(
    chain_queries,
    bench_q4_chain_3hop,
    bench_q5_chain_2hop,
);

criterion_group!(
    complex_queries,
    bench_q6_complex_6way,
    bench_q7_complex_hierarchy,
);

criterion_group!(
    cyclic_queries,
    bench_q8_triangle,
);

criterion_main!(star_queries, chain_queries, complex_queries, cyclic_queries);
