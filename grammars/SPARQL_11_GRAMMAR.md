# SPARQL 1.1 Grammar Reference

Official specification: https://www.w3.org/TR/sparql11-query/
Apache Jena implementation: https://github.com/apache/jena/blob/main/jena-arq/Grammar/Final/sparql_11-final.jj

## Core Query Types

### SELECT Query
Filters and projects variables from the dataset.

**Syntax:**
```sparql
SELECT [DISTINCT|REDUCED] (variables | *)
WHERE { graph_pattern }
[solution_modifiers]
```

**Features:**
- DISTINCT: Eliminates duplicate solutions
- REDUCED: Permits elimination of some duplicates
- Expressions with AS: `SELECT (expr AS ?var)`
- Wildcard: `SELECT *` for all variables

### CONSTRUCT Query
Generates RDF triples based on a template.

**Syntax:**
```sparql
CONSTRUCT { triple_template }
WHERE { graph_pattern }
[solution_modifiers]
```

**Abbreviated form:**
```sparql
CONSTRUCT WHERE { graph_pattern }
```

### ASK Query
Returns boolean indicating whether pattern matches.

**Syntax:**
```sparql
ASK WHERE { graph_pattern }
```

### DESCRIBE Query
Retrieves RDF descriptions of resources.

**Syntax:**
```sparql
DESCRIBE (IRI | variable)* WHERE { graph_pattern }
```

## Graph Patterns

### Basic Graph Pattern (BGP)
Sequence of triple patterns matched against the dataset.

```sparql
?s ?p ?o .
?s rdf:type ex:Person .
?s foaf:name ?name .
```

### OPTIONAL Pattern
Makes a pattern optional (non-binding).

```sparql
OPTIONAL { ?s foaf:age ?age }
```

### UNION Pattern
Matches alternative patterns.

```sparql
{ ?s foaf:name ?name } UNION { ?s rdfs:label ?name }
```

### MINUS Pattern
Excludes solutions matching a pattern.

```sparql
MINUS { ?s ex:blocked true }
```

### GRAPH Pattern
Matches patterns in a named graph.

```sparql
GRAPH <http://example.org/graph> { ?s ?p ?o }
GRAPH ?g { ?s ?p ?o }
```

### SERVICE Pattern
Federated query to remote SPARQL endpoint.

```sparql
SERVICE <http://example.org/sparql> { ?s ?p ?o }
```

### FILTER Constraint
Boolean constraint on solutions.

```sparql
FILTER (?age > 18)
FILTER (regex(?name, "^John"))
FILTER EXISTS { ?s foaf:knows ?friend }
```

### BIND Assignment
Binds expression result to a variable.

```sparql
BIND ((?price * ?quantity) AS ?total)
```

### VALUES Data Block
Inline data for solutions.

```sparql
VALUES (?x ?y) {
  (<uri1> 1)
  (<uri2> 2)
}
```

## Property Paths

### Sequence Path
```sparql
?s foaf:knows/foaf:name ?name
```

### Alternative Path
```sparql
?s foaf:name|rdfs:label ?name
```

### Zero or More (Kleene Star)
```sparql
?s foaf:knows* ?friend
```

### One or More
```sparql
?s foaf:knows+ ?friend
```

### Zero or One
```sparql
?s foaf:knows? ?friend
```

### Inverse Path
```sparql
?s ^foaf:knows ?knower
```

### Negated Property Set
```sparql
?s !(rdf:type|rdfs:label) ?o
```

## Solution Modifiers

### GROUP BY
Groups solutions for aggregation.

```sparql
GROUP BY ?category ?type
GROUP BY (year(?date) AS ?year)
```

### HAVING
Filters grouped solutions.

```sparql
HAVING (COUNT(?item) > 5)
HAVING (AVG(?price) < 100)
```

### ORDER BY
Sorts solutions.

```sparql
ORDER BY ASC(?name)
ORDER BY DESC(?date) ?name
```

### LIMIT and OFFSET
Limits number of solutions returned.

```sparql
LIMIT 10
OFFSET 20
```

## Aggregate Functions

- **COUNT**: `COUNT(*)`, `COUNT(?var)`, `COUNT(DISTINCT ?var)`
- **SUM**: `SUM(?var)`, `SUM(DISTINCT ?var)`
- **AVG**: `AVG(?var)`, `AVG(DISTINCT ?var)`
- **MIN**: `MIN(?var)`, `MIN(DISTINCT ?var)`
- **MAX**: `MAX(?var)`, `MAX(DISTINCT ?var)`
- **SAMPLE**: `SAMPLE(?var)` - arbitrary value from group
- **GROUP_CONCAT**: `GROUP_CONCAT(?var; separator=", ")`

## Expression Operators

### Logical Operators
- `||` - OR
- `&&` - AND
- `!` - NOT

### Comparison Operators
- `=` - Equal
- `!=` - Not equal
- `<` - Less than
- `>` - Greater than
- `<=` - Less than or equal
- `>=` - Greater than or equal

### Arithmetic Operators
- `+` - Addition
- `-` - Subtraction
- `*` - Multiplication
- `/` - Division

### String Functions
- `SUBSTR(str, start, length)`
- `STRLEN(str)`
- `CONCAT(str1, str2, ...)`
- `UCASE(str)` - uppercase
- `LCASE(str)` - lowercase
- `CONTAINS(str, substr)`
- `STRSTARTS(str, prefix)`
- `STRENDS(str, suffix)`
- `REPLACE(str, pattern, replacement)`
- `REGEX(str, pattern, flags?)`

### Numeric Functions
- `ABS(num)`
- `CEIL(num)`
- `FLOOR(num)`
- `ROUND(num)`

### Type Testing
- `isIRI(term)`
- `isBlank(term)`
- `isLiteral(term)`
- `isNumeric(term)`
- `BOUND(?var)` - test if variable is bound
- `DATATYPE(literal)`
- `LANG(literal)`

### Date/Time Functions
- `NOW()` - current datetime
- `YEAR(datetime)`
- `MONTH(datetime)`
- `DAY(datetime)`
- `HOURS(datetime)`
- `MINUTES(datetime)`
- `SECONDS(datetime)`

### Hash Functions
- `MD5(str)`
- `SHA1(str)`
- `SHA256(str)`
- `SHA384(str)`
- `SHA512(str)`

## SPARQL Update Operations

### INSERT DATA
Insert ground triples.

```sparql
INSERT DATA {
  <http://example.org/s> <http://example.org/p> "value" .
}
```

### DELETE DATA
Delete ground triples.

```sparql
DELETE DATA {
  <http://example.org/s> <http://example.org/p> "value" .
}
```

### DELETE WHERE
Pattern-based deletion.

```sparql
DELETE WHERE {
  ?s ?p ?o .
  FILTER (?o = "old_value")
}
```

### INSERT/DELETE with WHERE
Complex updates with pattern matching.

```sparql
DELETE { ?s foaf:name ?old }
INSERT { ?s foaf:name ?new }
WHERE {
  ?s foaf:name ?old .
  BIND (UCASE(?old) AS ?new)
}
```

### LOAD
Load RDF from URI into graph.

```sparql
LOAD <http://example.org/data.ttl>
LOAD <http://example.org/data.ttl> INTO GRAPH <http://example.org/graph>
```

### CLEAR
Remove all triples from graph.

```sparql
CLEAR GRAPH <http://example.org/graph>
CLEAR DEFAULT
```

### DROP
Remove graph entirely.

```sparql
DROP GRAPH <http://example.org/graph>
DROP SILENT GRAPH <http://example.org/graph>
```

### ADD/MOVE/COPY
Graph-to-graph operations.

```sparql
COPY <graph1> TO <graph2>
MOVE <graph1> TO <graph2>
ADD <graph1> TO <graph2>
```

## Variables and Terms

### Variables
- `?name` - question mark syntax
- `$name` - dollar sign syntax (alternative)

### IRIs
- `<http://example.org/resource>` - full IRI
- `ex:resource` - prefixed name

### Literals
- `"string"` - plain string
- `"string"@en` - with language tag
- `"123"^^xsd:integer` - with datatype
- `123` - numeric literal (integer)
- `123.45` - numeric literal (decimal)
- `1.23e4` - numeric literal (double)
- `true`, `false` - boolean literals

### Blank Nodes
- `_:label` - labeled blank node
- `[]` - anonymous blank node
- `[ foaf:name "John" ]` - blank node with properties

### RDF Collections
- `(item1 item2 item3)` - list notation

## PREFIX Declarations

```sparql
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX foaf: <http://xmlns.com/foaf/0.1/>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT * WHERE {
  ?s rdf:type foaf:Person .
}
```

## BASE Declaration

```sparql
BASE <http://example.org/>

SELECT * WHERE {
  <resource> ?p ?o .
}
```

## Implementation Notes

- Apache Jena uses JavaCC for parser generation
- Grammar file: `sparql_11-final.jj`
- Parser supports SPARQL 1.0, 1.1, and ARQ extensions
- ARQ adds: aggregates, property functions, assignment, negation
- Full support for property paths including negation and sequences
