# Knowledge Graph Demo

Enterprise-grade demonstration of the `rust-kgdb` TypeScript SDK featuring a realistic organizational knowledge graph.

## Overview

This sample application showcases production-ready knowledge graph capabilities including employees, departments, projects, and skills. It demonstrates the full power of SPARQL 1.1 queries with worst-case optimal join (WCOJ) execution.

## Features Demonstrated

### 1. Data Loading
- Turtle RDF format parsing
- Realistic organizational ontology
- Vocabulary management (FOAF, ORG, Schema.org)

### 2. SPARQL SELECT Queries
- **Star patterns** (WCOJ-optimized): Find all properties of a single entity
- **Chain patterns** (WCOJ-optimized): Multi-hop relationship traversal
- Complex filters and sorting

### 3. Aggregations
- COUNT, AVG aggregates
- GROUP BY and HAVING clauses
- Statistical analysis (average salary by department)

### 4. Property Paths
- Transitive closure (`+` operator)
- Organizational hierarchy traversal
- Find all direct and indirect reports

### 5. SPARQL ASK Queries
- Boolean existence checks
- Validation queries

### 6. SPARQL CONSTRUCT Queries
- Graph transformation
- Derive new knowledge from existing data
- Create simplified views

### 7. Named Graphs
- Multi-tenant data isolation
- Sensitive data segregation (compensation in separate graph)
- Cross-graph queries

### 8. Data Export
- Export to Turtle format
- File system integration
- Data portability

## Installation

```bash
cd examples/knowledge-graph-demo
npm install
```

## Running the Demo

```bash
npm start
```

Or with auto-reload during development:

```bash
npm run dev
```

## Sample Output

The demo produces color-coded console output showing:

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                     RUST-KGDB KNOWLEDGE GRAPH DEMO                           ║
║                     Enterprise Organizational Data                            ║
╚══════════════════════════════════════════════════════════════════════════════╝

================================================================================
1. Initialize GraphDB
================================================================================

Creating GraphDB instance with base URI: http://example.org/
✓ GraphDB initialized
Version: 0.1.10

================================================================================
2. Load Turtle RDF Data
================================================================================

Loading organizational knowledge graph (employees, departments, projects)...
✓ Loaded 87 triples

[... SPARQL query examples with results ...]
```

## Knowledge Graph Schema

The demo uses a realistic enterprise schema:

### Entities
- **Employees**: Names, emails, job titles, salaries, skills
- **Departments**: Engineering, Product, Backend Team, Frontend Team
- **Skills**: Technical (Rust, TypeScript, SPARQL) and Soft (Leadership, Analytics)
- **Projects**: GraphDB project, UI project with assigned team members

### Relationships
- `org:memberOf` - Employee to Department
- `ex:reportsTo` - Employee reporting hierarchy
- `ex:hasSkill` - Employee to Skill
- `ex:assignedTo` - Project to Employee
- `org:subOrganizationOf` - Department hierarchy

### Sample Queries

**Find all reports (direct and indirect) to Alice:**
```sparql
PREFIX ex: <http://example.org/>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>

SELECT ?employee ?name WHERE {
  ?employee ex:reportsTo+ ex:alice .
  ?employee foaf:name ?name .
}
```

**Average salary by department:**
```sparql
PREFIX ex: <http://example.org/>
PREFIX org: <http://www.w3.org/ns/org#>
PREFIX schema: <http://schema.org/>

SELECT ?deptName (AVG(?salary) AS ?avgSalary) (COUNT(?person) AS ?employees) WHERE {
  ?person org:memberOf ?dept .
  ?person ex:salary ?salary .
  ?dept schema:name ?deptName .
}
GROUP BY ?deptName
ORDER BY DESC(?avgSalary)
```

## Performance Characteristics

- **Lookup speed**: 2.78 µs per triple
- **Bulk insert**: 146K triples/sec
- **Memory efficiency**: 24 bytes/triple
- **Query optimization**: WCOJ for star and chain patterns (35-180x faster than traditional engines)

## File Structure

```
knowledge-graph-demo/
├── package.json          # Dependencies and scripts
├── tsconfig.json         # TypeScript configuration
├── README.md            # This file
├── src/
│   └── index.ts         # Main demo application
└── output/
    └── organization.ttl # Exported data (generated on run)
```

## Modifying the Demo

### Add Your Own Data

Edit the `organizationData` constant in `src/index.ts`:

```typescript
const myData = `
@prefix ex: <http://example.org/> .
@prefix foaf: <http://xmlns.com/foaf/0.1/> .

ex:myEntity a foaf:Person ;
    foaf:name "My Name" .
`;

db.loadTtl(myData, null);
```

### Create Custom Queries

```typescript
const myQuery = `
  PREFIX ex: <http://example.org/>
  SELECT ?s ?p ?o WHERE {
    ?s ?p ?o .
  } LIMIT 10
`;

const results = db.querySelect(myQuery);
console.log(results);
```

### Load External Files

```typescript
import * as fs from 'fs';

const ttlContent = fs.readFileSync('/path/to/your/file.ttl', 'utf-8');
db.loadTtl(ttlContent, null);
```

## Learn More

- [rust-kgdb TypeScript SDK](https://www.npmjs.com/package/rust-kgdb)
- [SPARQL 1.1 Specification](https://www.w3.org/TR/sparql11-query/)
- [RDF 1.1 Primer](https://www.w3.org/TR/rdf11-primer/)
- [Turtle Format](https://www.w3.org/TR/turtle/)

## License

MIT
