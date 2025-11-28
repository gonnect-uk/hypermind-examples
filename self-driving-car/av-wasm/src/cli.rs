/*!
 * rust-kgdb CLI for AV Reasoning
 * Simple command-line tool that executes SPARQL queries using rust-kgdb
 */

use parking_lot::Mutex;
use rdf_io::turtle::TurtleParser;
use rdf_model::Dictionary;
use serde::{Deserialize, Serialize};
use sparql::{Executor, QueryResult};
use std::env;
use std::io::{self, Read};
use std::sync::Arc;
use std::time::Instant;
use storage::{InMemoryBackend, QuadStore};

#[derive(Deserialize)]
struct LoadRequest {
    turtle_data: String,
}

#[derive(Serialize)]
struct LoadResponse {
    success: bool,
    triples_loaded: usize,
    execution_time_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Deserialize)]
struct QueryRequest {
    sparql_query: String,
}

#[derive(Serialize)]
struct AskResponse {
    success: bool,
    result: bool,
    execution_time_us: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize)]
struct SelectResponse {
    success: bool,
    count: usize,
    bindings: Vec<serde_json::Value>,
    execution_time_us: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

// Global state (simple for CLI)
static DICT: once_cell::sync::Lazy<Arc<Dictionary>> =
    once_cell::sync::Lazy::new(|| Arc::new(Dictionary::new()));
static STORE: once_cell::sync::Lazy<Arc<Mutex<QuadStore<InMemoryBackend>>>> =
    once_cell::sync::Lazy::new(|| {
        Arc::new(Mutex::new(QuadStore::new(InMemoryBackend::new())))
    });

fn cmd_load(input: &str) -> String {
    let start = Instant::now();

    let req: LoadRequest = match serde_json::from_str(input) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::to_string(&LoadResponse {
                success: false,
                triples_loaded: 0,
                execution_time_ms: 0.0,
                error: Some(format!("Invalid JSON: {}", e)),
            })
            .unwrap();
        }
    };

    // Parse Turtle
    let parser = TurtleParser::new(&DICT);
    let triples = match parser.parse(&req.turtle_data) {
        Ok(t) => t,
        Err(e) => {
            return serde_json::to_string(&LoadResponse {
                success: false,
                triples_loaded: 0,
                execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                error: Some(format!("Parse error: {:?}", e)),
            })
            .unwrap();
        }
    };

    // Insert triples
    let count = triples.len();
    let mut store = STORE.lock();

    for triple in triples {
        if let Err(e) = store.insert(triple) {
            return serde_json::to_string(&LoadResponse {
                success: false,
                triples_loaded: 0,
                execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                error: Some(format!("Insert error: {:?}", e)),
            })
            .unwrap();
        }
    }

    drop(store);

    serde_json::to_string(&LoadResponse {
        success: true,
        triples_loaded: count,
        execution_time_ms: start.elapsed().as_secs_f64() * 1000.0,
        error: None,
    })
    .unwrap()
}

fn cmd_ask(input: &str) -> String {
    let start = Instant::now();

    let req: QueryRequest = match serde_json::from_str(input) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::to_string(&AskResponse {
                success: false,
                result: false,
                execution_time_us: 0.0,
                error: Some(format!("Invalid JSON: {}", e)),
            })
            .unwrap();
        }
    };

    let store = STORE.lock();
    let executor = Executor::new(&*store, Arc::clone(&DICT));

    match executor.execute_query(&req.sparql_query) {
        Ok(QueryResult::Boolean(result)) => {
            drop(store);
            serde_json::to_string(&AskResponse {
                success: true,
                result,
                execution_time_us: start.elapsed().as_micros() as f64,
                error: None,
            })
            .unwrap()
        }
        Ok(_) => serde_json::to_string(&AskResponse {
            success: false,
            result: false,
            execution_time_us: start.elapsed().as_micros() as f64,
            error: Some("Query did not return ASK result".to_string()),
        })
        .unwrap(),
        Err(e) => serde_json::to_string(&AskResponse {
            success: false,
            result: false,
            execution_time_us: start.elapsed().as_micros() as f64,
            error: Some(format!("Query error: {:?}", e)),
        })
        .unwrap(),
    }
}

fn cmd_select(input: &str) -> String {
    let start = Instant::now();

    let req: QueryRequest = match serde_json::from_str(input) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::to_string(&SelectResponse {
                success: false,
                count: 0,
                bindings: vec![],
                execution_time_us: 0.0,
                error: Some(format!("Invalid JSON: {}", e)),
            })
            .unwrap();
        }
    };

    let store = STORE.lock();
    let executor = Executor::new(&*store, Arc::clone(&DICT));

    match executor.execute_query(&req.sparql_query) {
        Ok(QueryResult::Bindings(bindings)) => {
            drop(store);

            let mut json_bindings = Vec::new();
            for binding in bindings.iter().take(100) {
                let mut obj = serde_json::Map::new();
                for (var, node) in binding.iter() {
                    obj.insert(var.clone(), serde_json::Value::String(format!("{}", node)));
                }
                json_bindings.push(serde_json::Value::Object(obj));
            }

            serde_json::to_string(&SelectResponse {
                success: true,
                count: json_bindings.len(),
                bindings: json_bindings,
                execution_time_us: start.elapsed().as_micros() as f64,
                error: None,
            })
            .unwrap()
        }
        Ok(_) => serde_json::to_string(&SelectResponse {
            success: false,
            count: 0,
            bindings: vec![],
            execution_time_us: start.elapsed().as_micros() as f64,
            error: Some("Query did not return SELECT result".to_string()),
        })
        .unwrap(),
        Err(e) => serde_json::to_string(&SelectResponse {
            success: false,
            count: 0,
            bindings: vec![],
            execution_time_us: start.elapsed().as_micros() as f64,
            error: Some(format!("Query error: {:?}", e)),
        })
        .unwrap(),
    }
}

fn cmd_clear() -> String {
    *STORE.lock() = QuadStore::new(InMemoryBackend::new());
    serde_json::to_string(&serde_json::json!({
        "success": true,
        "message": "Store cleared"
    }))
    .unwrap()
}

fn cmd_stats() -> String {
    let triples = STORE.lock().size();
    serde_json::to_string(&serde_json::json!({
        "triples": triples,
        "backend": "rust-kgdb InMemory"
    }))
    .unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: av-cli <command>");
        eprintln!("Commands: load, ask, select, clear, stats");
        eprintln!("Input: JSON via stdin");
        std::process::exit(1);
    }

    let command = &args[1];

    // Read stdin
    let mut input = String::new();
    if command != "clear" && command != "stats" {
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read stdin");
    }

    // Execute command
    let output = match command.as_str() {
        "load" => cmd_load(&input),
        "ask" => cmd_ask(&input),
        "select" => cmd_select(&input),
        "clear" => cmd_clear(),
        "stats" => cmd_stats(),
        _ => {
            serde_json::to_string(&serde_json::json!({
                "success": false,
                "error": format!("Unknown command: {}", command)
            }))
            .unwrap()
        }
    };

    println!("{}", output);
}
