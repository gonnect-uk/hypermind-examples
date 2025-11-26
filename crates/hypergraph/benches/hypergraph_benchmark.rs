//! Performance Benchmarks for Hypergraph Implementation
//!
//! Measures performance of core operations to ensure they meet production requirements.
//! Targets:
//! - O(1) node/edge lookup
//! - O(1) incident edges (via index)
//! - O(d) neighbor traversal (d = degree)
//! - O(n) BFS (n = nodes visited)
//! - O(m) pattern matching (m = edges)
//!
//! Run with: cargo bench --package hypergraph --bench hypergraph_benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hypergraph::Hypergraph;

fn benchmark_add_node(c: &mut Criterion) {
    let mut group = c.benchmark_group("node_operations");

    group.bench_function("add_node", |b| {
        b.iter(|| {
            let mut hg = Hypergraph::new();
            for _ in 0..100 {
                black_box(hg.add_node());
            }
        })
    });

    group.bench_function("add_labeled_node", |b| {
        b.iter(|| {
            let mut hg = Hypergraph::new();
            for i in 0..100 {
                black_box(hg.add_labeled_node(format!("Node{}", i)));
            }
        })
    });

    group.finish();
}

fn benchmark_add_hyperedge(c: &mut Criterion) {
    let mut group = c.benchmark_group("edge_operations");

    for arity in [2, 3, 4, 5].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(arity),
            arity,
            |b, &arity| {
                b.iter_batched(
                    || {
                        let mut hg = Hypergraph::new();
                        let nodes: Vec<_> = (0..arity).map(|_| hg.add_node()).collect();
                        (hg, nodes)
                    },
                    |(mut hg, nodes)| {
                        for _ in 0..100 {
                            black_box(hg.add_hyperedge(nodes.clone(), true));
                        }
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    group.finish();
}

fn benchmark_retrieve_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("retrieve_operations");

    group.bench_function("get_node", |b| {
        let mut hg = Hypergraph::new();
        let nodes: Vec<_> = (0..100).map(|_| hg.add_node()).collect();

        b.iter(|| {
            for node in &nodes {
                black_box(hg.get_node(*node));
            }
        })
    });

    group.bench_function("get_hyperedge", |b| {
        let mut hg = Hypergraph::new();
        let n1 = hg.add_node();
        let n2 = hg.add_node();
        let edges: Vec<_> = (0..100)
            .map(|_| hg.add_hyperedge(vec![n1, n2], true))
            .collect();

        b.iter(|| {
            for edge in &edges {
                black_box(hg.get_hyperedge(*edge));
            }
        })
    });

    group.finish();
}

fn benchmark_incident_edges(c: &mut Criterion) {
    let mut group = c.benchmark_group("incident_edges");

    group.bench_function("get_incident_edges", |b| {
        let mut hg = Hypergraph::new();
        let hub = hg.add_node();
        let neighbors: Vec<_> = (0..100).map(|_| {
            let n = hg.add_node();
            hg.add_hyperedge(vec![hub, n], true);
            n
        }).collect();

        b.iter(|| {
            black_box(hg.get_incident_edges(hub));
        })
    });

    group.finish();
}

fn benchmark_neighbors(c: &mut Criterion) {
    let mut group = c.benchmark_group("neighbor_operations");

    group.bench_function("get_neighbors", |b| {
        let mut hg = Hypergraph::new();
        let hub = hg.add_node();
        for _ in 0..50 {
            let n = hg.add_node();
            hg.add_hyperedge(vec![hub, n], true);
        }

        b.iter(|| {
            black_box(hg.get_neighbors(hub));
        })
    });

    group.finish();
}

fn benchmark_bfs_traversal(c: &mut Criterion) {
    let mut group = c.benchmark_group("traversal");

    group.bench_function("bfs_linear_chain", |b| {
        let mut hg = Hypergraph::new();
        let mut prev = hg.add_node();
        for _ in 0..100 {
            let next = hg.add_node();
            hg.add_hyperedge(vec![prev, next], false);
            prev = next;
        }

        let start = hg.nodes.keys().next().copied().unwrap();
        b.iter(|| {
            black_box(hg.bfs(start));
        })
    });

    group.bench_function("bfs_star_topology", |b| {
        let mut hg = Hypergraph::new();
        let hub = hg.add_node();
        for _ in 0..100 {
            let n = hg.add_node();
            hg.add_hyperedge(vec![hub, n], false);
        }

        b.iter(|| {
            black_box(hg.bfs(hub));
        })
    });

    group.finish();
}

fn benchmark_shortest_path(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathfinding");

    group.bench_function("shortest_path_linear_20", |b| {
        let mut hg = Hypergraph::new();
        let mut nodes = vec![hg.add_node()];
        for _ in 0..20 {
            let n = hg.add_node();
            hg.add_hyperedge(vec![nodes[nodes.len() - 1], n], false);
            nodes.push(n);
        }

        let start = nodes[0];
        let end = nodes[nodes.len() - 1];

        b.iter(|| {
            black_box(hg.shortest_path(start, end));
        })
    });

    group.bench_function("shortest_path_grid_10x10", |b| {
        let mut hg = Hypergraph::new();
        let mut grid = Vec::new();

        // Create 10x10 grid
        for i in 0..10 {
            let mut row = Vec::new();
            for j in 0..10 {
                row.push(hg.add_node());
            }
            grid.push(row);
        }

        // Add edges (no diagonal)
        for i in 0..10 {
            for j in 0..10 {
                if i + 1 < 10 {
                    hg.add_hyperedge(vec![grid[i][j], grid[i + 1][j]], false);
                }
                if j + 1 < 10 {
                    hg.add_hyperedge(vec![grid[i][j], grid[i][j + 1]], false);
                }
            }
        }

        let start = grid[0][0];
        let end = grid[9][9];

        b.iter(|| {
            black_box(hg.shortest_path(start, end));
        })
    });

    group.finish();
}

fn benchmark_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    group.bench_function("find_edges_1000_triples", |b| {
        let mut hg = Hypergraph::new();
        let subjects: Vec<_> = (0..10).map(|_| hg.add_node()).collect();
        let predicates: Vec<_> = (0..10).map(|_| hg.add_node()).collect();
        let objects: Vec<_> = (0..10).map(|_| hg.add_node()).collect();

        // Create 1000 triples
        for s in &subjects {
            for p in &predicates {
                for o in &objects {
                    hg.add_hyperedge(vec![*s, *p, *o], true);
                }
            }
        }

        let pattern_subject = subjects[0];
        b.iter(|| {
            black_box(hg.find_edges(&[Some(pattern_subject), None, None]));
        })
    });

    group.bench_function("find_edges_predicate_wildcard", |b| {
        let mut hg = Hypergraph::new();
        let subjects: Vec<_> = (0..5).map(|_| hg.add_node()).collect();
        let predicates: Vec<_> = (0..20).map(|_| hg.add_node()).collect();
        let objects: Vec<_> = (0..5).map(|_| hg.add_node()).collect();

        // Create 500 triples
        for s in &subjects {
            for p in &predicates {
                for o in &objects {
                    hg.add_hyperedge(vec![*s, *p, *o], true);
                }
            }
        }

        b.iter(|| {
            black_box(hg.find_edges(&[None, None, None]));
        })
    });

    group.finish();
}

fn benchmark_subgraph_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("subgraph_extraction");

    group.bench_function("subgraph_50_nodes", |b| {
        let mut hg = Hypergraph::new();
        let nodes: Vec<_> = (0..50).map(|_| hg.add_node()).collect();

        // Add edges
        for i in 0..nodes.len() - 1 {
            hg.add_hyperedge(vec![nodes[i], nodes[i + 1]], false);
        }

        let subgraph_nodes = &nodes[0..25];

        b.iter(|| {
            black_box(hg.subgraph(subgraph_nodes));
        })
    });

    group.bench_function("subgraph_100_nodes", |b| {
        let mut hg = Hypergraph::new();
        let nodes: Vec<_> = (0..100).map(|_| hg.add_node()).collect();

        // Add edges
        for i in 0..nodes.len() - 1 {
            hg.add_hyperedge(vec![nodes[i], nodes[i + 1]], false);
        }

        let subgraph_nodes = &nodes[0..50];

        b.iter(|| {
            black_box(hg.subgraph(subgraph_nodes));
        })
    });

    group.finish();
}

fn benchmark_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("statistics");

    group.bench_function("stats_1000_nodes_1000_edges", |b| {
        let mut hg = Hypergraph::new();
        let nodes: Vec<_> = (0..1000).map(|_| hg.add_node()).collect();

        // Add 1000 edges
        for i in 0..1000 {
            let n1 = nodes[i % nodes.len()];
            let n2 = nodes[(i + 1) % nodes.len()];
            hg.add_hyperedge(vec![n1, n2], i % 2 == 0);
        }

        b.iter(|| {
            black_box(hg.stats());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_add_node,
    benchmark_add_hyperedge,
    benchmark_retrieve_operations,
    benchmark_incident_edges,
    benchmark_neighbors,
    benchmark_bfs_traversal,
    benchmark_shortest_path,
    benchmark_pattern_matching,
    benchmark_subgraph_extraction,
    benchmark_stats,
);

criterion_main!(benches);
