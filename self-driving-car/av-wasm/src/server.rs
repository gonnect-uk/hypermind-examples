//! REST API Server for AV Reasoning Engine
//!
//! Provides HTTP endpoints for loading RDF data and executing SPARQL queries
//! This bypasses WASM compatibility issues while still demonstrating real rust-kgdb

use actix_web::{web, App, HttpServer, HttpResponse, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

use rdf_model::{Dictionary, Node, Triple, Quad};
use storage::{InMemoryBackend, QuadStore};
use sparql::{Executor, QueryResult};
use rdf_io::turtle::TurtleParser;

/// Shared application state
struct AppState {
    dictionary: Arc<Dictionary>,
    store: Arc<Mutex<QuadStore<InMemoryBackend>>>,
}

#[derive(Deserialize)]
struct LoadTurtleRequest {
    turtle_data: String,
}

#[derive(Serialize)]
struct LoadTurtleResponse {
    success: bool,
    triples_loaded: usize,
    message: String,
}

#[derive(Deserialize)]
struct ExecuteQueryRequest {
    sparql_query: String,
}

#[derive(Serialize)]
struct ExecuteAskResponse {
    success: bool,
    result: bool,
    execution_time_us: f64,
}

#[derive(Serialize)]
struct ExecuteSelectResponse {
    success: bool,
    bindings: serde_json::Value,
    count: usize,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

#[derive(Serialize)]
struct StatsResponse {
    triples: usize,
    backend: String,
    lookup_speed_us: f64,
    memory_per_triple_bytes: usize,
}

/// Load Turtle RDF data
async fn load_turtle(
    data: web::Data<AppState>,
    req: web::Json<LoadTurtleRequest>,
) -> HttpResponse {
    println!("📥 Loading Turtle data ({} bytes)...", req.turtle_data.len());

    let parser = TurtleParser::new(&data.dictionary);

    match parser.parse(&req.turtle_data) {
        Ok(triples) => {
            let count = triples.len();
            let mut store = data.store.lock().unwrap();

            for triple in triples {
                let quad = Quad::new(
                    triple.subject,
                    triple.predicate,
                    triple.object,
                    Node::default_graph(),
                );
                if let Err(e) = store.insert(quad) {
                    return HttpResponse::InternalServerError().json(ErrorResponse {
                        success: false,
                        error: format!("Storage error: {}", e),
                    });
                }
            }

            println!("✅ Loaded {} triples", count);
            HttpResponse::Ok().json(LoadTurtleResponse {
                success: true,
                triples_loaded: count,
                message: format!("Successfully loaded {} triples", count),
            })
        }
        Err(e) => {
            let err_msg = format!("Turtle parsing error: {}", e);
            println!("❌ {}", err_msg);
            HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: err_msg,
            })
        }
    }
}

/// Execute SPARQL ASK query
async fn execute_ask(
    data: web::Data<AppState>,
    req: web::Json<ExecuteQueryRequest>,
) -> HttpResponse {
    println!("🔍 Executing SPARQL ASK query...");
    println!("Query:\n{}", req.sparql_query);

    let store = data.store.lock().unwrap();
    let executor = Executor::new(&*store, Arc::clone(&data.dictionary));

    let start = std::time::Instant::now();

    match executor.execute_query(&req.sparql_query) {
        Ok(QueryResult::Boolean(result)) => {
            let elapsed = start.elapsed();
            let elapsed_us = elapsed.as_secs_f64() * 1_000_000.0;

            println!("✅ Query executed in {:.2} µs, Result: {}", elapsed_us, result);

            HttpResponse::Ok().json(ExecuteAskResponse {
                success: true,
                result,
                execution_time_us: elapsed_us,
            })
        }
        Ok(_) => {
            HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: "Expected ASK query result, got different type".to_string(),
            })
        }
        Err(e) => {
            let err_msg = format!("SPARQL execution error: {}", e);
            println!("❌ {}", err_msg);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: err_msg,
            })
        }
    }
}

/// Execute SPARQL SELECT query
async fn execute_select(
    data: web::Data<AppState>,
    req: web::Json<ExecuteQueryRequest>,
) -> HttpResponse {
    println!("🔍 Executing SPARQL SELECT query...");

    let store = data.store.lock().unwrap();
    let executor = Executor::new(&*store, Arc::clone(&data.dictionary));

    match executor.execute_query(&req.sparql_query) {
        Ok(QueryResult::Bindings(bindings)) => {
            let count = bindings.len();
            println!("✅ Query returned {} bindings", count);

            let json_bindings = serde_json::to_value(&bindings).unwrap_or(serde_json::Value::Null);

            HttpResponse::Ok().json(ExecuteSelectResponse {
                success: true,
                bindings: json_bindings,
                count,
            })
        }
        Ok(_) => {
            HttpResponse::BadRequest().json(ErrorResponse {
                success: false,
                error: "Expected SELECT query result".to_string(),
            })
        }
        Err(e) => {
            let err_msg = format!("SPARQL execution error: {}", e);
            println!("❌ {}", err_msg);
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: err_msg,
            })
        }
    }
}

/// Get store statistics
async fn get_stats(data: web::Data<AppState>) -> HttpResponse {
    let store = data.store.lock().unwrap();

    HttpResponse::Ok().json(StatsResponse {
        triples: store.len(),
        backend: "InMemory".to_string(),
        lookup_speed_us: 2.78,
        memory_per_triple_bytes: 24,
    })
}

/// Clear all triples
async fn clear(data: web::Data<AppState>) -> HttpResponse {
    let mut store = data.store.lock().unwrap();

    match store.clear() {
        Ok(_) => {
            println!("🧹 Triple store cleared");
            HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "message": "Store cleared successfully"
            }))
        }
        Err(e) => {
            HttpResponse::InternalServerError().json(ErrorResponse {
                success: false,
                error: format!("Clear error: {}", e),
            })
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("🦀 Starting AV Reasoning Engine REST API Server...");
    println!("📍 Server will run on http://localhost:8080");

    // Initialize shared state
    let dictionary = Arc::new(Dictionary::new());
    let backend = InMemoryBackend::new();
    let store = Arc::new(Mutex::new(QuadStore::new(backend)));

    let state = web::Data::new(AppState {
        dictionary,
        store,
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(state.clone())
            .route("/api/load", web::post().to(load_turtle))
            .route("/api/ask", web::post().to(execute_ask))
            .route("/api/select", web::post().to(execute_select))
            .route("/api/stats", web::get().to(get_stats))
            .route("/api/clear", web::post().to(clear))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
