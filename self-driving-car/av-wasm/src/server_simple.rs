/*!
 * Simple HTTP REST API for rust-kgdb AV Reasoning Engine
 * Uses tiny_http (synchronous) to avoid actix-web/mio compilation issues
 */

use parking_lot::Mutex;
use rdf_io::turtle::TurtleParser;
use rdf_model::Dictionary;
use serde::{Deserialize, Serialize};
use sparql::{Executor, QueryResult};
use std::io::Read;
use std::sync::Arc;
use std::time::Instant;
use storage::{InMemoryBackend, QuadStore};
use tiny_http::{Header, Method, Response, Server};

// ===== REQUEST/RESPONSE TYPES =====

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

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    backend: String,
    triples: usize,
}

#[derive(Serialize)]
struct StatsResponse {
    triples: usize,
    backend: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

// ===== APPLICATION STATE =====

struct AppState {
    dictionary: Arc<Dictionary>,
    store: Mutex<QuadStore<InMemoryBackend>>,
}

impl AppState {
    fn new() -> Self {
        let dictionary = Arc::new(Dictionary::new());
        let backend = InMemoryBackend::new();
        let store = QuadStore::new(backend);

        AppState {
            dictionary,
            store: Mutex::new(store),
        }
    }

    fn clear(&self) {
        let backend = InMemoryBackend::new();
        let new_store = QuadStore::new(backend);
        *self.store.lock() = new_store;
    }

    fn triple_count(&self) -> usize {
        self.store.lock().size()
    }
}

// ===== HTTP HANDLERS =====

fn handle_health(state: &AppState) -> String {
    let response = HealthResponse {
        status: "healthy".to_string(),
        backend: "rust-kgdb InMemory".to_string(),
        triples: state.triple_count(),
    };
    serde_json::to_string(&response).unwrap()
}

fn handle_stats(state: &AppState) -> String {
    let response = StatsResponse {
        triples: state.triple_count(),
        backend: "rust-kgdb InMemory".to_string(),
    };
    serde_json::to_string(&response).unwrap()
}

fn handle_clear(state: &AppState) -> String {
    state.clear();
    serde_json::to_string(&serde_json::json!({
        "success": true,
        "message": "Store cleared"
    }))
    .unwrap()
}

fn handle_load(state: &AppState, body: &str) -> String {
    let start = Instant::now();

    let req: LoadRequest = match serde_json::from_str(body) {
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

    // Parse Turtle data
    let parser = TurtleParser::new(&state.dictionary);
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

    // Insert triples into store
    let count = triples.len();
    let mut store = state.store.lock();

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

fn handle_ask(state: &AppState, body: &str) -> String {
    let start = Instant::now();

    let req: QueryRequest = match serde_json::from_str(body) {
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

    let store = state.store.lock();
    let executor = Executor::new(&*store, Arc::clone(&state.dictionary));

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

fn handle_select(state: &AppState, body: &str) -> String {
    let start = Instant::now();

    let req: QueryRequest = match serde_json::from_str(body) {
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

    let store = state.store.lock();
    let executor = Executor::new(&*store, Arc::clone(&state.dictionary));

    match executor.execute_query(&req.sparql_query) {
        Ok(QueryResult::Bindings(bindings)) => {
            drop(store);

            // Convert bindings to JSON
            let mut json_bindings = Vec::new();
            for binding in bindings.iter().take(100) {
                // Limit to 100 results
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

// ===== MAIN SERVER =====

fn main() {
    println!("🦀 Starting rust-kgdb AV Reasoning Engine REST API Server...");
    println!("📍 Server will run on http://localhost:8080");

    let state = Arc::new(AppState::new());

    let server = Server::http("0.0.0.0:8080").expect("Failed to start server");
    println!("✅ Server started successfully!");
    println!("📊 Endpoints:");
    println!("   GET  /health        - Health check");
    println!("   POST /api/load      - Load Turtle RDF data");
    println!("   POST /api/ask       - Execute SPARQL ASK query");
    println!("   POST /api/select    - Execute SPARQL SELECT query");
    println!("   POST /api/clear     - Clear all triples");
    println!("   GET  /api/stats     - Get store statistics");
    println!();

    for request in server.incoming_requests() {
        let state = Arc::clone(&state);
        let method = request.method();
        let url = request.url().to_string();

        // CORS headers
        let cors_headers = vec![
            Header::from_bytes(&b"Access-Control-Allow-Origin"[..], &b"*"[..]).unwrap(),
            Header::from_bytes(
                &b"Access-Control-Allow-Methods"[..],
                &b"GET, POST, OPTIONS"[..],
            )
            .unwrap(),
            Header::from_bytes(
                &b"Access-Control-Allow-Headers"[..],
                &b"Content-Type"[..],
            )
            .unwrap(),
        ];

        // Handle OPTIONS (preflight)
        if *method == Method::Options {
            let response = Response::from_string("").with_header(cors_headers[0].clone());
            let response = response.with_header(cors_headers[1].clone());
            let response = response.with_header(cors_headers[2].clone());
            let _ = request.respond(response);
            continue;
        }

        // Route handling
        let (status_code, body) = match (method, url.as_str()) {
            (&Method::Get, "/health") => (200, handle_health(&state)),
            (&Method::Get, "/api/stats") => (200, handle_stats(&state)),
            (&Method::Post, "/api/clear") => (200, handle_clear(&state)),
            (&Method::Post, "/api/load") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();
                (200, handle_load(&state, &body))
            }
            (&Method::Post, "/api/ask") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();
                (200, handle_ask(&state, &body))
            }
            (&Method::Post, "/api/select") => {
                let mut body = String::new();
                request.as_reader().read_to_string(&mut body).unwrap();
                (200, handle_select(&state, &body))
            }
            _ => (
                404,
                serde_json::to_string(&ErrorResponse {
                    success: false,
                    error: "Not found".to_string(),
                })
                .unwrap(),
            ),
        };

        println!("{} {} -> {}", method, url, status_code);

        let response = Response::from_string(body)
            .with_status_code(status_code)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            )
            .with_header(cors_headers[0].clone())
            .with_header(cors_headers[1].clone())
            .with_header(cors_headers[2].clone());

        let _ = request.respond(response);
    }
}
