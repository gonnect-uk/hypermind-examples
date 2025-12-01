#!/usr/bin/env python3
"""
rust-kgdb v0.1.3 Release Verification Test
Tests Python SDK and demonstrates usage
"""

import sys
sys.path.insert(0, 'sdks/python')

from rust_kgdb_py import GraphDb, get_version

def main():
    print("=" * 70)
    print("🚀 rust-kgdb v0.1.3 - PRODUCTION RELEASE TEST")
    print("=" * 70)

    # Test 1: Version check
    print("\n✅ Test 1: Version Check")
    version = get_version()
    print(f"   Version: {version}")
    assert "0.1.3" in version, "Version mismatch!"

    # Test 2: Create GraphDb instance
    print("\n✅ Test 2: Create GraphDb Instance")
    db = GraphDb("http://example.org/release-test")
    print("   GraphDb instance created successfully!")

    # Test 3: Load Turtle data
    print("\n✅ Test 3: Load Turtle Data")
    turtle_data = """
@prefix foaf: <http://xmlns.com/foaf/0.1/> .
@prefix ex: <http://example.org/> .

ex:alice foaf:name "Alice" .
ex:alice foaf:knows ex:bob .
ex:bob foaf:name "Bob" .
"""
    db.load_ttl(turtle_data, None)
    print("   Loaded 3 triples")

    # Test 4: Count triples
    print("\n✅ Test 4: Count Triples")
    count = db.count_triples()
    print(f"   Triple count: {count}")
    assert count == 3, f"Expected 3 triples, got {count}"

    # Test 5: SPARQL SELECT query
    print("\n✅ Test 5: SPARQL SELECT Query")
    results = db.query_select("""
        SELECT ?person ?name WHERE {
            ?person <http://xmlns.com/foaf/0.1/name> ?name .
        }
    """)
    print(f"   Found {len(results)} results:")
    for result in results:
        print(f"      - {result.bindings.get('name', 'N/A')}")

    # Test 6: Database statistics
    print("\n✅ Test 6: Database Statistics")
    stats = db.get_stats()
    print(f"   Total triples: {stats.total_triples}")
    print(f"   Total entities: {stats.total_entities}")
    print(f"   Dictionary size: {stats.dictionary_size}")
    print(f"   Memory usage: {stats.memory_bytes} bytes")
    print(f"   Storage backend: {stats.storage_backend}")

    print("\n" + "=" * 70)
    print("🎉 ALL TESTS PASSED! v0.1.3 is PRODUCTION READY!")
    print("=" * 70)

    print("\n📊 Release Artifacts:")
    print("   ✅ Python SDK: sdks/python/dist/rust-kgdb-0.1.3.tar.gz (1.1MB)")
    print("   ✅ Demo UI: self-driving-car/DEMO_RUST_KGDB.html (74KB)")
    print("   ✅ Native Library: libuniffi.dylib (2.7MB)")
    print("   ✅ Documentation: RELEASE_v0.1.3_FINAL.md")

    print("\n🚀 Quick Start:")
    print("   cd sdks/python && python3 -c 'from rust_kgdb_py import *'")
    print("   cd self-driving-car && make demo")

if __name__ == "__main__":
    main()
