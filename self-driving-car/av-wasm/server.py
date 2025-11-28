#!/usr/bin/env python3
"""
Simple REST API server for AV Reasoning Demo
Uses RDFLib to load and query Turtle files
"""

from flask import Flask, request, jsonify
from flask_cors import CORS
from rdflib import Graph, Namespace, URIRef, Literal
from rdflib.plugins.sparql import prepareQuery
import time
import os

app = Flask(__name__)
CORS(app)  # Enable CORS for browser access

# Create global graph
g = Graph()

# Define namespaces
AV = Namespace("http://zenya.com/ontology/av#")
SENSOR = Namespace("http://zenya.com/ontology/sensor#")
ZENYA = Namespace("http://zenya.com/")

@app.route('/api/load', methods=['POST'])
def load_turtle():
    """Load Turtle RDF data into the graph"""
    start = time.time()

    data = request.json
    turtle_data = data.get('turtle_data', '')

    if not turtle_data:
        return jsonify({
            'success': False,
            'error': 'No turtle_data provided'
        }), 400

    try:
        # Parse turtle data
        g.parse(data=turtle_data, format='turtle')

        elapsed_ms = (time.time() - start) * 1000
        triples_count = len(g)

        print(f"✅ Loaded {triples_count} triples in {elapsed_ms:.2f}ms")

        return jsonify({
            'success': True,
            'triples_loaded': triples_count,
            'message': f'Successfully loaded {triples_count} triples',
            'execution_time_ms': elapsed_ms
        })

    except Exception as e:
        print(f"❌ Error parsing Turtle: {e}")
        return jsonify({
            'success': False,
            'error': f'Turtle parsing error: {str(e)}'
        }), 400

@app.route('/api/ask', methods=['POST'])
def execute_ask():
    """Execute SPARQL ASK query"""
    start = time.time()

    data = request.json
    sparql_query = data.get('sparql_query', '')

    if not sparql_query:
        return jsonify({
            'success': False,
            'error': 'No sparql_query provided'
        }), 400

    print(f"🔍 Executing SPARQL ASK query...")
    print(f"Query:\n{sparql_query}")

    try:
        # Execute ASK query
        result = g.query(sparql_query)

        # ASK queries return a boolean
        ask_result = bool(result)

        elapsed_us = (time.time() - start) * 1_000_000

        print(f"✅ Query executed in {elapsed_us:.2f} µs, Result: {ask_result}")

        return jsonify({
            'success': True,
            'result': ask_result,
            'execution_time_us': elapsed_us
        })

    except Exception as e:
        print(f"❌ SPARQL execution error: {e}")
        return jsonify({
            'success': False,
            'error': f'SPARQL execution error: {str(e)}'
        }), 500

@app.route('/api/select', methods=['POST'])
def execute_select():
    """Execute SPARQL SELECT query"""
    start = time.time()

    data = request.json
    sparql_query = data.get('sparql_query', '')

    if not sparql_query:
        return jsonify({
            'success': False,
            'error': 'No sparql_query provided'
        }), 400

    print(f"🔍 Executing SPARQL SELECT query...")

    try:
        # Execute SELECT query
        results = g.query(sparql_query)

        # Convert results to list of dicts
        bindings = []
        for row in results:
            binding = {}
            for var in results.vars:
                value = row[var]
                if value:
                    binding[str(var)] = str(value)
            bindings.append(binding)

        elapsed_us = (time.time() - start) * 1_000_000

        print(f"✅ Query returned {len(bindings)} bindings in {elapsed_us:.2f} µs")

        return jsonify({
            'success': True,
            'bindings': bindings,
            'count': len(bindings),
            'execution_time_us': elapsed_us
        })

    except Exception as e:
        print(f"❌ SPARQL execution error: {e}")
        return jsonify({
            'success': False,
            'error': f'SPARQL execution error: {str(e)}'
        }), 500

@app.route('/api/stats', methods=['GET'])
def get_stats():
    """Get store statistics"""
    return jsonify({
        'triples': len(g),
        'backend': 'RDFLib (Python)',
        'lookup_speed_us': 'varies',
        'memory_per_triple_bytes': 'varies'
    })

@app.route('/api/clear', methods=['POST'])
def clear():
    """Clear all triples"""
    global g
    g = Graph()

    print("🧹 Triple store cleared")

    return jsonify({
        'success': True,
        'message': 'Store cleared successfully'
    })

@app.route('/api/load-scenario/<int:scenario_id>', methods=['POST'])
def load_scenario(scenario_id):
    """Load a specific scenario from data/ directory"""
    scenario_files = {
        1: 'data/scenario1_traffic_light.ttl',
        2: 'data/scenario2_pedestrian.ttl',
        3: 'data/scenario3_school_zone.ttl'
    }

    if scenario_id not in scenario_files:
        return jsonify({
            'success': False,
            'error': f'Invalid scenario ID: {scenario_id}'
        }), 400

    file_path = scenario_files[scenario_id]

    if not os.path.exists(file_path):
        return jsonify({
            'success': False,
            'error': f'Scenario file not found: {file_path}'
        }), 404

    try:
        with open(file_path, 'r') as f:
            turtle_data = f.read()

        # Clear existing data
        global g
        g = Graph()

        # Load new scenario
        g.parse(data=turtle_data, format='turtle')

        triples_count = len(g)

        print(f"✅ Loaded scenario {scenario_id}: {triples_count} triples")

        return jsonify({
            'success': True,
            'scenario_id': scenario_id,
            'triples_loaded': triples_count,
            'message': f'Successfully loaded scenario {scenario_id}'
        })

    except Exception as e:
        print(f"❌ Error loading scenario: {e}")
        return jsonify({
            'success': False,
            'error': f'Error loading scenario: {str(e)}'
        }), 500

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({
        'status': 'healthy',
        'triples': len(g)
    })

if __name__ == '__main__':
    print("🦀 Starting AV Reasoning Engine REST API Server (Python/RDFLib)...")
    print("📍 Server running on http://localhost:8080")
    print("📂 Scenario files available:")
    print("   - POST /api/load-scenario/1 (Traffic Light)")
    print("   - POST /api/load-scenario/2 (Pedestrian)")
    print("   - POST /api/load-scenario/3 (School Zone)")
    print()

    app.run(host='0.0.0.0', port=8080, debug=True)
