#!/usr/bin/env python3
"""
rust-kgdb Python SDK Setup
High-performance RDF/SPARQL database with 100% W3C compliance
"""

from setuptools import setup, find_packages
from pathlib import Path

# Read README
readme_file = Path(__file__).parent / "README.md"
long_description = readme_file.read_text(encoding="utf-8") if readme_file.exists() else ""

setup(
    name="rust-kgdb",
    version="0.1.8",
    description="High-performance RDF/SPARQL database with 100% W3C compliance and WCOJ execution",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Gonnect Team",
    author_email="noreply@gonnect.ai",
    url="https://github.com/gonnect-uk/rust-kgdb",
    project_urls={
        "Documentation": "https://github.com/gonnect-uk/rust-kgdb/tree/main/docs",
        "Source": "https://github.com/gonnect-uk/rust-kgdb",
        "Issues": "https://github.com/gonnect-uk/rust-kgdb/issues",
    },
    packages=["rust_kgdb_py"],
    package_data={
        "rust_kgdb_py": ["*.dylib", "*.so", "*.dll", "*.py"],
    },
    include_package_data=True,
    python_requires=">=3.8",
    install_requires=[],
    extras_require={
        "dev": [
            "pytest>=7.0",
            "pytest-cov>=4.0",
            "black>=23.0",
            "mypy>=1.0",
        ],
    },
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: Apache Software License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Rust",
        "Topic :: Database",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    keywords=["rdf", "sparql", "semantic-web", "knowledge-graph", "database", "triplestore"],
    license="Apache-2.0",
    zip_safe=False,
)
