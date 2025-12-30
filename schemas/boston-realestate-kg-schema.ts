/**
 * Boston Real Estate KG Dataset Schema
 *
 * This schema defines the mapping between:
 * - Knowledge Graph (RDF triples in KGDB)
 * - SQL Tables (Snowflake/BigQuery)
 *
 * HyperFederate uses this schema to generate graph_search() CTEs
 * that join SPARQL results with SQL tables.
 *
 * Sample data source: City of Boston Open Data (data.boston.gov)
 * License: PDDL (Public Domain Dedication and License)
 */

// ============================================================================
// KG Ontology Definition
// ============================================================================

export const KG_ONTOLOGY = {
  namespace: 'http://boston.gov/property#',
  prefixes: {
    'prop': 'http://boston.gov/property#',
    'rdf': 'http://www.w3.org/1999/02/22-rdf-syntax-ns#',
    'rdfs': 'http://www.w3.org/2000/01/rdf-schema#',
    'xsd': 'http://www.w3.org/2001/XMLSchema#',
    'geo': 'http://www.w3.org/2003/01/geo/wgs84_pos#',
    'owl': 'http://www.w3.org/2002/07/owl#'
  },
  classes: [
    'Property',
    'Neighborhood',
    'PropertyType'
  ],
  properties: [
    'address',
    'assessedValue',
    'yearBuilt',
    'bedrooms',
    'bathrooms',
    'squareFeet',
    'lotSize',
    'locatedIn',
    'hasType',
    'adjacentTo',
    'priceInfluencedBy',
    'lat',
    'long'
  ]
} as const

// ============================================================================
// SQL Schema Definition (Snowflake/BigQuery compatible)
// ============================================================================

export const SQL_SCHEMA = {
  snowflake: {
    database: 'HYPERMIND_DB',
    schema: 'BOSTON_REALESTATE',
    warehouse: 'COMPUTE_WH'
  },
  bigquery: {
    project: 'hypermind-analytics',
    dataset: 'boston_realestate'
  },
  tables: {
    Properties: {
      name: 'properties',
      kgClass: 'Property',
      primaryKey: 'property_id',
      columns: {
        property_id: { type: 'VARCHAR', kgPredicate: 'id' },
        address: { type: 'VARCHAR', kgPredicate: 'address' },
        assessed_value: { type: 'DECIMAL', kgPredicate: 'assessedValue' },
        year_built: { type: 'INT', kgPredicate: 'yearBuilt' },
        bedrooms: { type: 'INT', kgPredicate: 'bedrooms' },
        bathrooms: { type: 'DECIMAL', kgPredicate: 'bathrooms' },
        square_feet: { type: 'INT', kgPredicate: 'squareFeet' },
        lot_size: { type: 'DECIMAL', kgPredicate: 'lotSize' },
        latitude: { type: 'DECIMAL', kgPredicate: 'lat' },
        longitude: { type: 'DECIMAL', kgPredicate: 'long' },
        neighborhood_id: { type: 'VARCHAR' },
        property_type_id: { type: 'VARCHAR' }
      },
      foreignKeys: [
        { column: 'neighborhood_id', references: 'Neighborhoods.neighborhood_id' },
        { column: 'property_type_id', references: 'PropertyTypes.type_id' }
      ]
    },
    Neighborhoods: {
      name: 'neighborhoods',
      kgClass: 'Neighborhood',
      primaryKey: 'neighborhood_id',
      columns: {
        neighborhood_id: { type: 'VARCHAR', kgPredicate: 'id' },
        name: { type: 'VARCHAR', kgPredicate: 'label' },
        median_value: { type: 'DECIMAL' },
        latitude: { type: 'DECIMAL', kgPredicate: 'lat' },
        longitude: { type: 'DECIMAL', kgPredicate: 'long' }
      }
    },
    PropertyTypes: {
      name: 'property_types',
      kgClass: 'PropertyType',
      primaryKey: 'type_id',
      columns: {
        type_id: { type: 'VARCHAR', kgPredicate: 'id' },
        name: { type: 'VARCHAR', kgPredicate: 'label' },
        description: { type: 'VARCHAR' }
      }
    },
    NeighborhoodAdjacency: {
      name: 'neighborhood_adjacency',
      kgClass: 'adjacentTo',
      primaryKey: 'id',
      columns: {
        id: { type: 'INT' },
        neighborhood_a: { type: 'VARCHAR' },
        neighborhood_b: { type: 'VARCHAR' }
      }
    },
    PriceInfluences: {
      name: 'price_influences',
      kgClass: 'priceInfluencedBy',
      primaryKey: 'id',
      columns: {
        id: { type: 'INT' },
        property_id: { type: 'VARCHAR' },
        influenced_by_id: { type: 'VARCHAR' },
        influence_score: { type: 'DECIMAL' }
      }
    }
  }
}

// ============================================================================
// TypeScript Types (generated from schema)
// ============================================================================

export interface PropertyRow {
  property_id: string
  address: string
  assessed_value: number
  year_built: number
  bedrooms: number
  bathrooms: number
  square_feet: number
  lot_size: number
  latitude: number
  longitude: number
  neighborhood_id: string
  property_type_id: string
}

export interface NeighborhoodRow {
  neighborhood_id: string
  name: string
  median_value: number
  latitude: number
  longitude: number
}

export interface PropertyTypeRow {
  type_id: string
  name: string
  description: string
}

export interface NeighborhoodAdjacencyRow {
  id: number
  neighborhood_a: string
  neighborhood_b: string
}

export interface PriceInfluenceRow {
  id: number
  property_id: string
  influenced_by_id: string
  influence_score: number
}

// ============================================================================
// KG ↔ SQL Mapping Functions
// ============================================================================

export function toRdfUri(tableName: keyof typeof SQL_SCHEMA.tables, id: string): string {
  const ns = KG_ONTOLOGY.namespace
  return `<${ns}${tableName}/${id}>`
}

export function fromRdfUri(uri: string): { table: string; id: string } | null {
  const ns = KG_ONTOLOGY.namespace
  if (!uri.startsWith(`<${ns}`)) return null
  const path = uri.slice(ns.length + 1, -1) // Remove < and >
  const [table, id] = path.split('/')
  return table && id ? { table, id } : null
}

// ============================================================================
// HyperFederate graph_search() CTE Generator
// ============================================================================

export function generateGraphSearchCTE(
  sparqlQuery: string,
  joinTable: keyof typeof SQL_SCHEMA.tables,
  joinColumn: string,
  target: 'snowflake' | 'bigquery' = 'snowflake'
): string {
  const table = SQL_SCHEMA.tables[joinTable]
  const fullTableName = target === 'snowflake'
    ? `${SQL_SCHEMA.snowflake.database}.${SQL_SCHEMA.snowflake.schema}.${table.name}`
    : `\`${SQL_SCHEMA.bigquery.project}.${SQL_SCHEMA.bigquery.dataset}.${table.name}\``

  return `
WITH kg AS (
  SELECT * FROM graph_search('${sparqlQuery}')
)
SELECT
  t.*,
  kg.*
FROM ${fullTableName} t
JOIN kg ON t.${joinColumn} = kg.id
`
}

// ============================================================================
// Sample SPARQL Queries for Boston Real Estate
// ============================================================================

export const SAMPLE_QUERIES = {
  // Get all properties with details
  allProperties: `
    PREFIX prop: <${KG_ONTOLOGY.namespace}>
    SELECT ?property ?address ?value ?bedrooms ?neighborhood WHERE {
      ?property a prop:Property .
      ?property prop:address ?address .
      ?property prop:assessedValue ?value .
      OPTIONAL { ?property prop:bedrooms ?bedrooms }
      OPTIONAL { ?property prop:locatedIn ?n . ?n rdfs:label ?neighborhood }
    }
  `,

  // High-value properties (> $2M)
  highValueProperties: `
    PREFIX prop: <${KG_ONTOLOGY.namespace}>
    SELECT ?property ?address ?value ?neighborhood WHERE {
      ?property a prop:Property .
      ?property prop:address ?address .
      ?property prop:assessedValue ?value .
      ?property prop:locatedIn ?n .
      ?n rdfs:label ?neighborhood .
      FILTER(?value > 2000000)
    }
    ORDER BY DESC(?value)
  `,

  // Properties with geographic coordinates
  propertiesWithGeo: `
    PREFIX prop: <${KG_ONTOLOGY.namespace}>
    PREFIX geo: <http://www.w3.org/2003/01/geo/wgs84_pos#>
    SELECT ?property ?address ?lat ?lng ?value WHERE {
      ?property a prop:Property .
      ?property prop:address ?address .
      ?property geo:lat ?lat .
      ?property geo:long ?lng .
      OPTIONAL { ?property prop:assessedValue ?value }
    }
  `,

  // Neighborhoods adjacent to Back Bay
  adjacentToBackBay: `
    PREFIX prop: <${KG_ONTOLOGY.namespace}>
    SELECT ?neighbor ?label WHERE {
      prop:BackBay prop:adjacentTo ?neighbor .
      ?neighbor rdfs:label ?label .
    }
  `,

  // Properties with price influences
  priceInfluences: `
    PREFIX prop: <${KG_ONTOLOGY.namespace}>
    SELECT ?property ?address ?influenced_by ?influenced_address WHERE {
      ?property prop:priceInfluencedBy ?influenced_by .
      ?property prop:address ?address .
      ?influenced_by prop:address ?influenced_address .
    }
  `,

  // Historic properties (built before 1900)
  historicProperties: `
    PREFIX prop: <${KG_ONTOLOGY.namespace}>
    SELECT ?property ?address ?year ?neighborhood WHERE {
      ?property a prop:Property .
      ?property prop:address ?address .
      ?property prop:yearBuilt ?year .
      ?property prop:locatedIn ?n .
      ?n rdfs:label ?neighborhood .
      FILTER(?year < 1900)
    }
    ORDER BY ?year
  `
}

// ============================================================================
// HyperFederate Sample Queries (SQL + KG)
// ============================================================================

export const HYPERFEDERATE_QUERIES = {
  // Join KG properties with Snowflake MLS listings
  propertiesWithMLS: `
WITH kg AS (
  SELECT * FROM graph_search('
    PREFIX prop: <http://boston.gov/property#>
    SELECT ?id ?address ?value ?neighborhood WHERE {
      ?property a prop:Property .
      ?property prop:address ?address .
      ?property prop:assessedValue ?value .
      ?property prop:locatedIn ?n .
      ?n rdfs:label ?neighborhood .
    }
  ')
)
SELECT
  kg.address,
  kg.value AS assessed_value,
  kg.neighborhood,
  mls.listing_price,
  mls.days_on_market,
  (mls.listing_price - kg.value) AS price_premium
FROM HYPERMIND_DB.BOSTON_REALESTATE.mls_listings mls
JOIN kg ON mls.property_address = kg.address
WHERE kg.value > 1000000
ORDER BY price_premium DESC
`,

  // Properties with geographic data for mapping
  propertiesForMap: `
WITH kg AS (
  SELECT * FROM graph_search('
    PREFIX prop: <http://boston.gov/property#>
    PREFIX geo: <http://www.w3.org/2003/01/geo/wgs84_pos#>
    SELECT ?id ?address ?lat ?lng ?value ?type WHERE {
      ?property a prop:Property .
      ?property prop:address ?address .
      ?property geo:lat ?lat .
      ?property geo:long ?lng .
      OPTIONAL { ?property prop:assessedValue ?value }
      OPTIONAL { ?property prop:hasType ?t . ?t rdfs:label ?type }
    }
  ')
)
SELECT
  kg.*,
  n.name AS neighborhood_name,
  n.median_value AS neighborhood_median
FROM HYPERMIND_DB.BOSTON_REALESTATE.neighborhoods n
JOIN HYPERMIND_DB.BOSTON_REALESTATE.properties p ON p.neighborhood_id = n.neighborhood_id
JOIN kg ON p.address = kg.address
`
}

// ============================================================================
// DDL for Snowflake
// ============================================================================

export const SNOWFLAKE_DDL = `
-- Snowflake DDL for Boston Real Estate Dataset
-- Database: HYPERMIND_DB
-- Schema: BOSTON_REALESTATE

CREATE DATABASE IF NOT EXISTS HYPERMIND_DB;
CREATE SCHEMA IF NOT EXISTS HYPERMIND_DB.BOSTON_REALESTATE;

-- Properties table
CREATE OR REPLACE TABLE HYPERMIND_DB.BOSTON_REALESTATE.properties (
  property_id VARCHAR PRIMARY KEY,
  address VARCHAR NOT NULL,
  assessed_value DECIMAL(12, 2),
  year_built INT,
  bedrooms INT,
  bathrooms DECIMAL(3, 1),
  square_feet INT,
  lot_size DECIMAL(10, 2),
  latitude DECIMAL(9, 6),
  longitude DECIMAL(9, 6),
  neighborhood_id VARCHAR,
  property_type_id VARCHAR,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP(),
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP()
);

-- Neighborhoods table
CREATE OR REPLACE TABLE HYPERMIND_DB.BOSTON_REALESTATE.neighborhoods (
  neighborhood_id VARCHAR PRIMARY KEY,
  name VARCHAR NOT NULL,
  median_value DECIMAL(12, 2),
  latitude DECIMAL(9, 6),
  longitude DECIMAL(9, 6),
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP()
);

-- Property types table
CREATE OR REPLACE TABLE HYPERMIND_DB.BOSTON_REALESTATE.property_types (
  type_id VARCHAR PRIMARY KEY,
  name VARCHAR NOT NULL,
  description VARCHAR,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP()
);

-- Neighborhood adjacency table (for graph relationships)
CREATE OR REPLACE TABLE HYPERMIND_DB.BOSTON_REALESTATE.neighborhood_adjacency (
  id INT AUTOINCREMENT PRIMARY KEY,
  neighborhood_a VARCHAR NOT NULL,
  neighborhood_b VARCHAR NOT NULL,
  UNIQUE(neighborhood_a, neighborhood_b)
);

-- Price influences table (for graph relationships)
CREATE OR REPLACE TABLE HYPERMIND_DB.BOSTON_REALESTATE.price_influences (
  id INT AUTOINCREMENT PRIMARY KEY,
  property_id VARCHAR NOT NULL,
  influenced_by_id VARCHAR NOT NULL,
  influence_score DECIMAL(5, 4),
  UNIQUE(property_id, influenced_by_id)
);

-- MLS Listings (external data source for federation)
CREATE OR REPLACE TABLE HYPERMIND_DB.BOSTON_REALESTATE.mls_listings (
  listing_id VARCHAR PRIMARY KEY,
  property_address VARCHAR NOT NULL,
  listing_price DECIMAL(12, 2),
  days_on_market INT,
  listing_status VARCHAR,
  list_date DATE,
  agent_name VARCHAR
);

-- Create foreign key constraints
ALTER TABLE HYPERMIND_DB.BOSTON_REALESTATE.properties
  ADD CONSTRAINT fk_neighborhood FOREIGN KEY (neighborhood_id)
  REFERENCES HYPERMIND_DB.BOSTON_REALESTATE.neighborhoods(neighborhood_id);

ALTER TABLE HYPERMIND_DB.BOSTON_REALESTATE.properties
  ADD CONSTRAINT fk_property_type FOREIGN KEY (property_type_id)
  REFERENCES HYPERMIND_DB.BOSTON_REALESTATE.property_types(type_id);

-- Create indexes for performance
CREATE INDEX idx_properties_neighborhood ON HYPERMIND_DB.BOSTON_REALESTATE.properties(neighborhood_id);
CREATE INDEX idx_properties_value ON HYPERMIND_DB.BOSTON_REALESTATE.properties(assessed_value);
CREATE INDEX idx_properties_geo ON HYPERMIND_DB.BOSTON_REALESTATE.properties(latitude, longitude);
`

// ============================================================================
// DDL for BigQuery
// ============================================================================

export const BIGQUERY_DDL = `
-- BigQuery DDL for Boston Real Estate Dataset
-- Project: hypermind-analytics
-- Dataset: boston_realestate

-- Create dataset
CREATE SCHEMA IF NOT EXISTS \`hypermind-analytics.boston_realestate\`;

-- Properties table
CREATE TABLE IF NOT EXISTS \`hypermind-analytics.boston_realestate.properties\` (
  property_id STRING NOT NULL,
  address STRING NOT NULL,
  assessed_value FLOAT64,
  year_built INT64,
  bedrooms INT64,
  bathrooms FLOAT64,
  square_feet INT64,
  lot_size FLOAT64,
  latitude FLOAT64,
  longitude FLOAT64,
  neighborhood_id STRING,
  property_type_id STRING,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP(),
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP()
);

-- Neighborhoods table
CREATE TABLE IF NOT EXISTS \`hypermind-analytics.boston_realestate.neighborhoods\` (
  neighborhood_id STRING NOT NULL,
  name STRING NOT NULL,
  median_value FLOAT64,
  latitude FLOAT64,
  longitude FLOAT64,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP()
);

-- Property types table
CREATE TABLE IF NOT EXISTS \`hypermind-analytics.boston_realestate.property_types\` (
  type_id STRING NOT NULL,
  name STRING NOT NULL,
  description STRING,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP()
);

-- Neighborhood adjacency table
CREATE TABLE IF NOT EXISTS \`hypermind-analytics.boston_realestate.neighborhood_adjacency\` (
  id INT64 NOT NULL,
  neighborhood_a STRING NOT NULL,
  neighborhood_b STRING NOT NULL
);

-- Price influences table
CREATE TABLE IF NOT EXISTS \`hypermind-analytics.boston_realestate.price_influences\` (
  id INT64 NOT NULL,
  property_id STRING NOT NULL,
  influenced_by_id STRING NOT NULL,
  influence_score FLOAT64
);

-- MLS Listings table
CREATE TABLE IF NOT EXISTS \`hypermind-analytics.boston_realestate.mls_listings\` (
  listing_id STRING NOT NULL,
  property_address STRING NOT NULL,
  listing_price FLOAT64,
  days_on_market INT64,
  listing_status STRING,
  list_date DATE,
  agent_name STRING
);
`

// ============================================================================
// Sample Data Loader
// ============================================================================

export const SAMPLE_DATA = {
  neighborhoods: [
    { neighborhood_id: 'back_bay', name: 'Back Bay', median_value: 3500000, latitude: 42.3503, longitude: -71.0810 },
    { neighborhood_id: 'beacon_hill', name: 'Beacon Hill', median_value: 2800000, latitude: 42.3588, longitude: -71.0707 },
    { neighborhood_id: 'south_end', name: 'South End', median_value: 1650000, latitude: 42.3412, longitude: -71.0723 },
    { neighborhood_id: 'jamaica_plain', name: 'Jamaica Plain', median_value: 850000, latitude: 42.3097, longitude: -71.1152 },
    { neighborhood_id: 'dorchester', name: 'Dorchester', median_value: 650000, latitude: 42.2878, longitude: -71.0671 },
    { neighborhood_id: 'charlestown', name: 'Charlestown', median_value: 1200000, latitude: 42.3782, longitude: -71.0602 },
    { neighborhood_id: 'brighton', name: 'Brighton', median_value: 750000, latitude: 42.3509, longitude: -71.1619 },
    { neighborhood_id: 'east_boston', name: 'East Boston', median_value: 625000, latitude: 42.3702, longitude: -71.0389 },
    { neighborhood_id: 'roxbury', name: 'Roxbury', median_value: 550000, latitude: 42.3152, longitude: -71.0882 },
    { neighborhood_id: 'south_boston', name: 'South Boston', median_value: 950000, latitude: 42.3381, longitude: -71.0476 }
  ],
  propertyTypes: [
    { type_id: 'single_family', name: 'Single Family', description: 'Detached single-family home' },
    { type_id: 'condo', name: 'Condo', description: 'Condominium unit' },
    { type_id: 'multi_family', name: 'Multi-Family', description: 'Multi-unit residential property' },
    { type_id: 'commercial', name: 'Commercial', description: 'Commercial property' }
  ],
  properties: [
    { property_id: 'BB001', address: '165 Marlborough Street', assessed_value: 2850000, year_built: 1875, bedrooms: 3, bathrooms: 2.5, square_feet: 2400, latitude: 42.3512, longitude: -71.0842, neighborhood_id: 'back_bay', property_type_id: 'condo' },
    { property_id: 'BB002', address: '298 Commonwealth Avenue', assessed_value: 8500000, year_built: 1890, bedrooms: 6, bathrooms: 5.0, square_feet: 6800, latitude: 42.3498, longitude: -71.0867, neighborhood_id: 'back_bay', property_type_id: 'single_family' },
    { property_id: 'BB003', address: '45 Newbury Street', assessed_value: 4200000, year_built: 1868, bedrooms: 0, bathrooms: 2.0, square_feet: 3200, latitude: 42.3525, longitude: -71.0760, neighborhood_id: 'back_bay', property_type_id: 'commercial' },
    { property_id: 'BH001', address: '72 Pinckney Street', assessed_value: 3950000, year_built: 1830, bedrooms: 4, bathrooms: 3.5, square_feet: 3100, latitude: 42.3593, longitude: -71.0684, neighborhood_id: 'beacon_hill', property_type_id: 'single_family' },
    { property_id: 'BH002', address: '15 Chestnut Street', assessed_value: 1650000, year_built: 1845, bedrooms: 2, bathrooms: 1.5, square_feet: 1800, latitude: 42.3587, longitude: -71.0718, neighborhood_id: 'beacon_hill', property_type_id: 'condo' },
    { property_id: 'JP001', address: '42 Sedgwick Street', assessed_value: 1250000, year_built: 1920, bedrooms: 4, bathrooms: 2.0, square_feet: 2200, latitude: 42.3089, longitude: -71.1168, neighborhood_id: 'jamaica_plain', property_type_id: 'single_family' },
    { property_id: 'SE001', address: '534 Tremont Street', assessed_value: 2400000, year_built: 1885, bedrooms: 8, bathrooms: 4.0, square_feet: 4800, latitude: 42.3405, longitude: -71.0732, neighborhood_id: 'south_end', property_type_id: 'multi_family' },
    { property_id: 'SE002', address: '88 Waltham Street', assessed_value: 895000, year_built: 1910, bedrooms: 2, bathrooms: 1.0, square_feet: 1100, latitude: 42.3418, longitude: -71.0715, neighborhood_id: 'south_end', property_type_id: 'condo' }
  ]
}
