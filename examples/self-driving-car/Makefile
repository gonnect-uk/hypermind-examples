.PHONY: help build start stop demo clean test health ts-setup ts-start ts-stop ts-demo

# Colors for output
GREEN  = \033[0;32m
YELLOW = \033[0;33m
BLUE   = \033[0;34m
CYAN   = \033[0;36m
NC     = \033[0m # No Color

help: ## Show this help message
	@echo "$(GREEN)🚗 Self-Driving Car Demo - rust-kgdb v0.1.3$(NC)"
	@echo ""
	@echo "$(CYAN)📘 TYPESCRIPT SDK BACKEND (DEFAULT):$(NC)"
	@echo "  $(GREEN)ts-demo      $(NC) Setup and start TypeScript SDK backend + open demo"
	@echo "  $(GREEN)ts-start     $(NC) Start TypeScript SDK backend server"
	@echo "  $(GREEN)ts-stop      $(NC) Stop TypeScript SDK backend server"
	@echo "  $(GREEN)ts-setup     $(NC) Install npm dependencies"
	@echo ""
	@echo "$(YELLOW)Other Commands:$(NC)"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | grep -v 'ts-' | awk 'BEGIN {FS = ":.*?## "}; {printf "  $(GREEN)%-12s$(NC) %s\n", $$1, $$2}'

build: ## Build av-server binary (release mode)
	@echo "$(GREEN)🔨 Building av-server...$(NC)"
	cd av-cli-standalone && cargo build --bin av-server --features server --release
	@echo "$(GREEN)✅ Build complete!$(NC)"

start: ## Start av-server in background
	@echo "$(GREEN)🚀 Starting av-server...$(NC)"
	@if lsof -i :8080 > /dev/null 2>&1; then \
		echo "$(YELLOW)⚠️  Port 8080 already in use. Stopping existing server...$(NC)"; \
		$(MAKE) stop; \
		sleep 1; \
	fi
	@nohup target/release/av-server > av-server.log 2>&1 & echo $$! > av-server.pid
	@sleep 2
	@if curl -s http://localhost:8080/health > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Server started successfully at http://localhost:8080$(NC)"; \
		echo "$(GREEN)📊 Stats: http://localhost:8080/stats$(NC)"; \
	else \
		echo "$(YELLOW)❌ Server failed to start. Check av-server.log$(NC)"; \
		exit 1; \
	fi

stop: ## Stop av-server
	@echo "$(YELLOW)🛑 Stopping av-server...$(NC)"
	@if [ -f av-server.pid ]; then \
		kill -9 $$(cat av-server.pid) 2>/dev/null || true; \
		rm av-server.pid; \
		echo "$(GREEN)✅ Server stopped$(NC)"; \
	else \
		pkill -9 av-server 2>/dev/null || true; \
		echo "$(GREEN)✅ No server running$(NC)"; \
	fi

demo: start ## Start server and open demo in browser
	@echo "$(GREEN)🌐 Opening demo...$(NC)"
	@sleep 1
	@open DEMO_RUST_KGDB.html || xdg-open DEMO_RUST_KGDB.html 2>/dev/null || echo "$(YELLOW)Please open: file://$(PWD)/DEMO_RUST_KGDB.html$(NC)"
	@echo ""
	@echo "$(GREEN)✅ Demo ready!$(NC)"
	@echo "$(GREEN)📍 URL: file://$(PWD)/DEMO_RUST_KGDB.html$(NC)"
	@echo "$(GREEN)🔌 Backend: http://localhost:8080$(NC)"
	@echo ""
	@echo "$(YELLOW)To stop the server, run: make stop$(NC)"

clean: stop ## Stop server and clean build artifacts
	@echo "$(YELLOW)🧹 Cleaning...$(NC)"
	@rm -f av-server.log av-server.pid
	@cd av-cli-standalone && cargo clean
	@echo "$(GREEN)✅ Clean complete$(NC)"

test: ## Run test SPARQL queries
	@echo "$(GREEN)🧪 Testing SPARQL queries...$(NC)"
	@echo ""
	@echo "$(YELLOW)1. Load test data:$(NC)"
	@curl -s -X POST http://localhost:8080/load -H "Content-Type: application/json" -d '{"turtle_data": "@prefix av: <http://zenya.com/ontology/av#> .\n@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\n<http://zenya.com/vehicle/ego>\n    a av:Vehicle ;\n    av:velocity \"13.3\"^^xsd:float ;\n    av:positionX \"-80.0\"^^xsd:float ."}' | jq -r 'if .success then "✅ Loaded \(.triples_loaded) triples in \(.execution_time_ms)ms" else "❌ Failed: \(.error)" end'
	@echo ""
	@echo "$(YELLOW)2. Test SPARQL ASK query:$(NC)"
	@curl -s -X POST http://localhost:8080/ask -H "Content-Type: application/json" -d '{"sparql_query": "PREFIX av: <http://zenya.com/ontology/av#>\nPREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\nASK { ?v rdf:type av:Vehicle . ?v av:velocity ?speed }"}' | jq -r 'if .success then "✅ Query result: \(.result) (executed in \(.execution_time_us)µs)" else "❌ Failed: \(.error)" end'
	@echo ""
	@echo "$(GREEN)✅ Tests complete$(NC)"

health: ## Check server health
	@echo "$(GREEN)🏥 Checking server health...$(NC)"
	@if curl -s http://localhost:8080/health | jq -e '.status == "healthy"' > /dev/null 2>&1; then \
		echo "$(GREEN)✅ Server is healthy$(NC)"; \
		curl -s http://localhost:8080/stats | jq; \
	else \
		echo "$(YELLOW)❌ Server is not responding$(NC)"; \
		exit 1; \
	fi

logs: ## Show server logs
	@if [ -f av-server.log ]; then \
		tail -f av-server.log; \
	else \
		echo "$(YELLOW)No log file found. Is the server running?$(NC)"; \
	fi

# Default target (use TypeScript backend!)
.DEFAULT_GOAL := ts-demo

# ===== TYPESCRIPT SDK BACKEND =====

ts-setup: ## Install TypeScript dependencies
	@echo "$(CYAN)📦 Installing TypeScript dependencies...$(NC)"
	@npm install
	@echo "$(GREEN)✅ Dependencies installed$(NC)"

ts-start: ## Start TypeScript SDK backend server
	@echo "$(CYAN)📘 Starting TypeScript SDK backend...$(NC)"
	@if lsof -i :8080 > /dev/null 2>&1; then \
		echo "$(YELLOW)⚠️  Port 8080 already in use. Stopping...$(NC)"; \
		$(MAKE) ts-stop; \
		sleep 1; \
	fi
	@nohup npm start > typescript-backend.log 2>&1 & echo $$! > typescript-backend.pid
	@sleep 3
	@if curl -s http://localhost:8080/health > /dev/null 2>&1; then \
		echo "$(GREEN)✅ TypeScript SDK backend started at http://localhost:8080$(NC)"; \
		curl -s http://localhost:8080/health | jq; \
	else \
		echo "$(YELLOW)❌ Server failed to start. Check typescript-backend.log$(NC)"; \
		exit 1; \
	fi

ts-stop: ## Stop TypeScript SDK backend server
	@echo "$(YELLOW)🛑 Stopping TypeScript backend...$(NC)"
	@if [ -f typescript-backend.pid ]; then \
		kill -9 $$(cat typescript-backend.pid) 2>/dev/null || true; \
		rm typescript-backend.pid; \
		echo "$(GREEN)✅ Server stopped$(NC)"; \
	else \
		pkill -9 -f "ts-node typescript_backend.ts" 2>/dev/null || true; \
		pkill -9 -f "node typescript_backend.js" 2>/dev/null || true; \
		echo "$(GREEN)✅ No server running$(NC)"; \
	fi

ts-demo: ts-start ## Start TypeScript SDK backend and open demo
	@echo "$(CYAN)🌐 Opening demo...$(NC)"
	@sleep 1
	@open DEMO_RUST_KGDB.html || xdg-open DEMO_RUST_KGDB.html 2>/dev/null || echo "$(YELLOW)Please open: file://$(PWD)/DEMO_RUST_KGDB.html$(NC)"
	@echo ""
	@echo "$(GREEN)✅ Demo ready!$(NC)"
	@echo "$(GREEN)📍 URL: file://$(PWD)/DEMO_RUST_KGDB.html$(NC)"
	@echo "$(CYAN)📘 Backend: TypeScript SDK (NAPI-RS + in-memory)$(NC)"
	@echo "$(GREEN)🔌 API: http://localhost:8080$(NC)"
	@echo ""
	@echo "$(YELLOW)To stop the server, run: make ts-stop$(NC)"

# Default target (use TypeScript backend now!)
.DEFAULT_GOAL := ts-demo
