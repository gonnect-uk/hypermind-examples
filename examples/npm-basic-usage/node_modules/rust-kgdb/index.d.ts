/**
 * rust-kgdb TypeScript bindings
 * High-performance RDF/SPARQL database with 100% W3C compliance
 */

export interface QueryResult {
  bindings: Record<string, string>
}

export interface TripleResult {
  subject: string
  predicate: string
  object: string
}

/**
 * GraphDB - High-performance RDF/SPARQL database
 *
 * @example
 * ```typescript
 * const db = new GraphDB('http://example.org/my-app')
 *
 * db.loadTtl(`
 *   @prefix foaf: <http://xmlns.com/foaf/0.1/> .
 *   <http://example.org/alice> foaf:name "Alice" .
 * `, null)
 *
 * const results = db.querySelect('SELECT ?name WHERE { ?person foaf:name ?name }')
 * console.log(results[0].bindings.name) // "Alice"
 * ```
 */
export class GraphDB {
  /**
   * Create new in-memory GraphDB instance
   * @param appGraphUri - Default graph URI for this app
   */
  constructor(appGraphUri: string)

  /**
   * Load Turtle (TTL) RDF data
   * @param ttlContent - Turtle format RDF data
   * @param graphName - Optional named graph URI
   */
  loadTtl(ttlContent: string, graphName: string | null): void

  /**
   * Execute SPARQL SELECT query
   * @param sparql - SPARQL query string
   * @returns Array of query results with variable bindings
   */
  querySelect(sparql: string): QueryResult[]

  /**
   * Execute SPARQL query (CONSTRUCT/ASK/DESCRIBE)
   * @param sparql - SPARQL query string
   * @returns Array of triples
   */
  query(sparql: string): TripleResult[]

  /**
   * Count total triples in database
   */
  countTriples(): number

  /**
   * Clear all data from database
   */
  clear(): void

  /**
   * Get app graph URI
   */
  getGraphUri(): string
}

/**
 * Get library version
 */
export function getVersion(): string
