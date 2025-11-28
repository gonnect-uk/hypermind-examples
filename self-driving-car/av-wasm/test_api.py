#!/usr/bin/env python3
"""
Test script for AV Reasoning Engine API
Demonstrates real SPARQL execution with RDF data
"""

import requests
import json

BASE_URL = "http://localhost:8080"

def test_load_scenario():
    """Load traffic light scenario data"""
    print("📥 Loading Scenario 1: Traffic Light...")

    turtle_data = """
@prefix av: <http://zenya.com/ontology/av#> .
@prefix sensor: <http://zenya.com/ontology/sensor#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://zenya.com/vehicle/ego>
    a av:Vehicle ;
    rdfs:label "Ego Vehicle" ;
    av:hasVelocity "13.3"^^xsd:float ;
    av:positionX "-80.0"^^xsd:float ;
    av:carLength "5.0"^^xsd:float .

<http://zenya.com/traffic_light/tl_001>
    a av:TrafficLight ;
    rdfs:label "Traffic Light 001" ;
    av:state "red"^^xsd:string ;
    av:positionX "-30.0"^^xsd:float ;
    sensor:confidence "0.98"^^xsd:float ;
    av:distanceMeters "30.0"^^xsd:float .
"""

    response = requests.post(f"{BASE_URL}/api/load", json={"turtle_data": turtle_data})
    result = response.json()

    print(f"✅ Status: {result['success']}")
    print(f"   Triples loaded: {result['triples_loaded']}")
    print(f"   Execution time: {result.get('execution_time_ms', 0):.2f} ms")
    print()

    return result['success']

def test_sparql_ask():
    """Test SPARQL ASK query"""
    print("🔍 Testing SPARQL ASK Query...")

    query = """
PREFIX av: <http://zenya.com/ontology/av#>
ASK {
  ?tl a av:TrafficLight ;
      av:state "red" .
}
"""

    response = requests.post(f"{BASE_URL}/api/ask", json={"sparql_query": query})
    result = response.json()

    print(f"✅ Status: {result['success']}")
    print(f"   Result: {result['result']}")
    print(f"   Execution time: {result.get('execution_time_us', 0):.2f} µs")
    print()

    return result['result']

def test_sparql_select():
    """Test SPARQL SELECT query"""
    print("🔍 Testing SPARQL SELECT Query...")

    query = """
PREFIX av: <http://zenya.com/ontology/av#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
SELECT ?vehicle ?velocity ?label
WHERE {
  ?vehicle a av:Vehicle ;
           av:hasVelocity ?velocity ;
           rdfs:label ?label .
}
"""

    response = requests.post(f"{BASE_URL}/api/select", json={"sparql_query": query})
    result = response.json()

    print(f"✅ Status: {result['success']}")
    print(f"   Bindings returned: {result['count']}")
    print(f"   Results:")
    for binding in result['bindings']:
        print(f"      {json.dumps(binding, indent=6)}")
    print()

    return result['success']

def test_stats():
    """Get store statistics"""
    print("📊 Getting Store Statistics...")

    response = requests.get(f"{BASE_URL}/api/stats")
    result = response.json()

    print(f"   Triples in store: {result['triples']}")
    print(f"   Backend: {result['backend']}")
    print()

if __name__ == "__main__":
    print("=" * 60)
    print("🦀 AV Reasoning Engine API Test Suite")
    print("=" * 60)
    print()

    # Clear store first
    requests.post(f"{BASE_URL}/api/clear")

    # Run tests
    if test_load_scenario():
        test_stats()
        test_sparql_ask()
        test_sparql_select()

        print("=" * 60)
        print("✅ All tests completed successfully!")
        print("=" * 60)
    else:
        print("❌ Failed to load scenario data")
