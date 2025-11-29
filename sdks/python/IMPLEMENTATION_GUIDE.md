# Python SDK Implementation Guide

This document provides step-by-step instructions for implementing the Python SDK bindings for rust-kgdb.

## Overview

The Python SDK architecture consists of three layers:

```
Python Application
    ↓
rust_kgdb (Pythonic wrapper) ← YOU IMPLEMENT THIS
    ↓
UniFFI Generated Bindings   ← AUTO-GENERATED
    ↓
mobile-ffi (Rust FFI)        ← ALREADY EXISTS
```

## Prerequisites

- Python 3.8+
- Official uniffi-bindgen tool (NOT our custom one)
- Rust toolchain with mobile-ffi crate built

## Step 1: Install UniFFI Bindgen for Python

Our custom uniffi-bindgen only supports Swift and Kotlin. For Python, use the official tool:

```bash
# Install official uniffi-bindgen (Python version)
pip install uniffi-bindgen==0.30.0

# Verify installation
uniffi-bindgen --version
```

## Step 2: Generate Python Bindings

```bash
# From repository root
uniffi-bindgen generate \
    crates/mobile-ffi/src/gonnect.udl \
    --language python \
    --out-dir sdks/python/rust_kgdb/

# This creates:
# - sdks/python/rust_kgdb/_uniffi/gonnect.py (low-level bindings)
# - sdks/python/rust_kgdb/_uniffi/__init__.py
```

## Step 3: Create High-Level Wrapper Classes

Create `sdks/python/rust_kgdb/graphdb.py`:

```python
"""
High-level Pythonic wrapper for rust-kgdb.

Provides intuitive, type-safe API with Python idioms.
"""
from typing import Optional, List, Dict, Iterator
from ._uniffi.gonnect import GraphDB as FFIGraphDB, GonnectNode, create_graph_db


class GraphDB:
    """RDF graph database with SPARQL support."""

    def __init__(self, ffi: FFIGraphDB):
        """Internal constructor. Use GraphDB.in_memory() instead."""
        self._ffi = ffi

    @staticmethod
    def in_memory() -> 'GraphDB':
        """
        Creates a new in-memory database.

        Returns:
            A new GraphDB instance

        Example:
            >>> db = GraphDB.in_memory()
            >>> assert db.is_empty()
        """
        return GraphDB(create_graph_db())

    def insert(self) -> 'InsertBuilder':
        """
        Starts building an insert operation.

        Returns:
            An InsertBuilder for chaining triple additions

        Example:
            >>> db.insert() \\
            ...     .triple(subject, predicate, object) \\
            ...     .execute()
        """
        return InsertBuilder(self._ffi)

    def query(self) -> 'QueryBuilder':
        """
        Starts building a SPARQL query.

        Returns:
            A QueryBuilder for setting query parameters

        Example:
            >>> results = db.query() \\
            ...     .sparql("SELECT ?s WHERE { ?s ?p ?o }") \\
            ...     .execute()
        """
        return QueryBuilder(self._ffi)

    def count(self) -> int:
        """
        Counts total triples in the database.

        Returns:
            Number of triples

        Example:
            >>> db.count()
            0
        """
        return self._ffi.count_triples()

    def is_empty(self) -> bool:
        """
        Checks if database is empty.

        Returns:
            True if no triples, False otherwise

        Example:
            >>> db.is_empty()
            True
        """
        return self.count() == 0

    def clear(self) -> None:
        """
        Clears all triples from database.

        Warning:
            This operation cannot be undone.

        Example:
            >>> db.clear()
            >>> assert db.is_empty()
        """
        self._ffi.clear()


class InsertBuilder:
    """Builder for inserting triples."""

    def __init__(self, ffi: FFIGraphDB):
        self._ffi = ffi
        self._triples: List[tuple] = []
        self._graph: Optional[GonnectNode] = None

    def triple(self, subject: 'Node', predicate: 'Node', obj: 'Node') -> 'InsertBuilder':
        """
        Adds a triple to insert.

        Args:
            subject: The triple's subject
            predicate: The triple's predicate
            obj: The triple's object

        Returns:
            This builder for chaining

        Example:
            >>> builder.triple(
            ...     Node.iri("http://example.org/alice"),
            ...     Node.iri("http://xmlns.com/foaf/0.1/name"),
            ...     Node.literal("Alice")
            ... )
        """
        self._triples.append((subject._to_ffi(), predicate._to_ffi(), obj._to_ffi()))
        return self

    def graph(self, graph: 'Node') -> 'InsertBuilder':
        """
        Sets the named graph for all triples.

        Args:
            graph: The named graph IRI

        Returns:
            This builder for chaining
        """
        self._graph = graph._to_ffi()
        return self

    def execute(self) -> None:
        """
        Executes the insert operation.

        Raises:
            GonnectException: If insert fails
        """
        for s, p, o in self._triples:
            self._ffi.insert_triple(s, p, o, self._graph)


class QueryBuilder:
    """Builder for SPARQL queries."""

    def __init__(self, ffi: FFIGraphDB):
        self._ffi = ffi
        self._sparql: Optional[str] = None

    def sparql(self, query: str) -> 'QueryBuilder':
        """
        Sets the SPARQL query string.

        Args:
            query: SPARQL 1.1 query

        Returns:
            This builder for chaining

        Example:
            >>> builder.sparql('''
            ...     PREFIX foaf: <http://xmlns.com/foaf/0.1/>
            ...     SELECT ?name WHERE { ?person foaf:name ?name }
            ... ''')
        """
        self._sparql = query
        return self

    def execute(self) -> 'QueryResult':
        """
        Executes the query and returns results.

        Returns:
            QueryResult containing bindings

        Raises:
            GonnectException: If query fails
        """
        if not self._sparql:
            raise ValueError("SPARQL query not set")

        bindings_json = self._ffi.execute_sparql(self._sparql)
        return QueryResult._from_json(bindings_json)


class QueryResult:
    """Query results with variable bindings."""

    def __init__(self, bindings: List['Binding']):
        self._bindings = bindings

    def __len__(self) -> int:
        """Number of results."""
        return len(self._bindings)

    def __iter__(self) -> Iterator['Binding']:
        """Iterate over bindings."""
        return iter(self._bindings)

    def __getitem__(self, index: int) -> 'Binding':
        """Get binding by index."""
        return self._bindings[index]

    def is_empty(self) -> bool:
        """Check if results are empty."""
        return len(self._bindings) == 0

    @staticmethod
    def _from_json(json_str: str) -> 'QueryResult':
        """Parse results from JSON."""
        import json
        data = json.loads(json_str) if json_str and json_str != "[]" else []
        bindings = [Binding(item) for item in data]
        return QueryResult(bindings)


class Binding:
    """Variable binding from query result."""

    def __init__(self, vars: Dict[str, str]):
        self._vars = vars

    def get(self, variable: str, default: Optional[str] = None) -> Optional[str]:
        """
        Gets value for variable name.

        Args:
            variable: Variable name (without '?' prefix)
            default: Default value if not bound

        Returns:
            Variable value or default
        """
        return self._vars.get(variable, default)

    def __getitem__(self, variable: str) -> Optional[str]:
        """Bracket notation access."""
        return self._vars.get(variable)

    def __contains__(self, variable: str) -> bool:
        """Check if variable is bound."""
        return variable in self._vars

    @property
    def variables(self) -> List[str]:
        """Get all variable names."""
        return list(self._vars.keys())


# Re-export for convenience
__all__ = ['GraphDB', 'InsertBuilder', 'QueryBuilder', 'QueryResult', 'Binding']
```

## Step 4: Create Node Factory

Create `sdks/python/rust_kgdb/node.py`:

```python
"""RDF node factory methods."""
from typing import Optional
from ._uniffi.gonnect import GonnectNode


class Node:
    """RDF node (IRI, Literal, or Blank Node)."""

    def __init__(self, ffi_node: GonnectNode):
        self._ffi_node = ffi_node

    @staticmethod
    def iri(uri: str) -> 'Node':
        """Create an IRI node."""
        return Node(GonnectNode.Iri(uri))

    @staticmethod
    def literal(value: str) -> 'Node':
        """Create a plain literal."""
        return Node(GonnectNode.Literal(value, None, None))

    @staticmethod
    def typed_literal(value: str, datatype: str) -> 'Node':
        """Create a typed literal."""
        return Node(GonnectNode.Literal(value, datatype, None))

    @staticmethod
    def lang_literal(value: str, lang: str) -> 'Node':
        """Create a language-tagged literal."""
        return Node(GonnectNode.Literal(value, None, lang))

    @staticmethod
    def integer(value: int) -> 'Node':
        """Create an integer literal."""
        return Node.typed_literal(
            str(value),
            "http://www.w3.org/2001/XMLSchema#integer"
        )

    @staticmethod
    def boolean(value: bool) -> 'Node':
        """Create a boolean literal."""
        return Node.typed_literal(
            str(value).lower(),
            "http://www.w3.org/2001/XMLSchema#boolean"
        )

    @staticmethod
    def double(value: float) -> 'Node':
        """Create a double literal."""
        return Node.typed_literal(
            str(value),
            "http://www.w3.org/2001/XMLSchema#double"
        )

    @staticmethod
    def blank(id: str) -> 'Node':
        """Create a blank node."""
        return Node(GonnectNode.BlankNode(id))

    def _to_ffi(self) -> GonnectNode:
        """Convert to FFI node."""
        return self._ffi_node


__all__ = ['Node']
```

## Step 5: Create Package Init

Create `sdks/python/rust_kgdb/__init__.py`:

```python
"""rust-kgdb Python SDK."""
from .graphdb import GraphDB, QueryResult, Binding
from .node import Node

__version__ = "0.1.2"
__all__ = ['GraphDB', 'QueryResult', 'Binding', 'Node']
```

## Step 6: Port Regression Tests

Create `sdks/python/tests/test_regression.py`:

```python
"""Regression test suite - 20 tests matching Rust SDK."""
import pytest
from rust_kgdb import GraphDB, Node


class TestRegression:
    """Comprehensive regression tests."""

    @pytest.fixture
    def db(self):
        """Create fresh database for each test."""
        return GraphDB.in_memory()

    # Test 1: Basic CRUD
    def test_basic_crud(self, db):
        """Insert single triple and query."""
        db.insert() \\
            .triple(
                Node.iri("http://example.org/test"),
                Node.iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#type"),
                Node.iri("http://example.org/TestClass")
            ) \\
            .execute()

        assert db.count() == 1

        results = db.query() \\
            .sparql("SELECT ?type WHERE { <http://example.org/test> a ?type }") \\
            .execute()

        assert len(results) == 1

    # Test 2-20: Similar structure...
    # (Full implementation would include all 20 tests)
```

## Step 7: Create setup.py

Create `sdks/python/setup.py`:

```python
"""Setup script for rust-kgdb Python SDK."""
from setuptools import setup, find_packages

setup(
    name="rust-kgdb",
    version="0.1.2",
    description="Production-ready Python bindings for rust-kgdb RDF/SPARQL database",
    long_description=open("README.md").read(),
    long_description_content_type="text/markdown",
    author="Zenya GraphDB Team",
    url="https://github.com/zenya-graphdb/rust-kgdb",
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=[
        "cffi>=1.15.0",
    ],
    extras_require={
        "dev": [
            "pytest>=7.0.0",
            "pytest-cov>=4.0.0",
            "black>=23.0.0",
            "mypy>=1.0.0",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
    ],
)
```

## Step 8: Testing

```bash
# Install in development mode
pip install -e sdks/python/[dev]

# Run tests
pytest sdks/python/tests/

# Run with coverage
pytest sdks/python/tests/ --cov=rust_kgdb --cov-report=html
```

## Implementation Checklist

- [ ] Install official uniffi-bindgen for Python
- [ ] Generate UniFFI Python bindings
- [ ] Create graphdb.py wrapper
- [ ] Create node.py factory
- [ ] Create __init__.py
- [ ] Port all 20 regression tests to pytest
- [ ] Create setup.py for pip installation
- [ ] Add type hints (mypy)
- [ ] Add docstrings (Sphinx)
- [ ] Test installation with `pip install -e .`
- [ ] Generate Sphinx documentation
- [ ] Create pyproject.toml for modern packaging

## Estimated Effort

- Binding generation: 1 hour (once uniffi-bindgen is installed)
- Wrapper classes: 4 hours
- Test porting: 3 hours
- Documentation: 2 hours
- Packaging: 1 hour

**Total: ~11 hours (1.5 days)**

## Notes

- The custom uniffi-bindgen in our repo ONLY supports Swift and Kotlin
- Use official Python uniffi-bindgen from PyPI
- Follow PEP 8 style guide
- Use type hints for better IDE support
- Sphinx documentation recommended
