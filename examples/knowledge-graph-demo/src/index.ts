/**
 * Enterprise Knowledge Graph Demo
 *
 * This application demonstrates the rust-kgdb TypeScript SDK using a realistic
 * organizational knowledge graph with employees, departments, projects, and skills.
 *
 * Features demonstrated:
 * - Loading Turtle RDF data
 * - SPARQL SELECT queries (star and chain patterns optimized with WCOJ)
 * - SPARQL ASK queries
 * - SPARQL CONSTRUCT queries
 * - Aggregations (COUNT, AVG, GROUP BY)
 * - Property paths (transitive closure for org hierarchy)
 * - Named graphs (multi-tenant data isolation)
 * - Data export to Turtle format
 */

import { GraphDB } from 'rust-kgdb';
import * as fs from 'fs';
import * as path from 'path';

// Define our vocabulary URIs
const EX = 'http://example.org/';
const FOAF = 'http://xmlns.com/foaf/0.1/';
const ORG = 'http://www.w3.org/ns/org#';
const SCHEMA = 'http://schema.org/';

// ANSI color codes for pretty console output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  cyan: '\x1b[36m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
};

function printSection(title: string) {
  console.log(`\n${colors.bright}${colors.cyan}${'='.repeat(80)}${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}${title}${colors.reset}`);
  console.log(`${colors.bright}${colors.cyan}${'='.repeat(80)}${colors.reset}\n`);
}

function printQuery(query: string) {
  console.log(`${colors.yellow}Query:${colors.reset}`);
  console.log(`${colors.blue}${query}${colors.reset}\n`);
}

function printResults(results: any) {
  console.log(`${colors.green}Results:${colors.reset}`);
  console.log(JSON.stringify(results, null, 2));
  console.log();
}

// Sample organizational knowledge graph in Turtle format
const organizationData = `
@prefix ex: <${EX}> .
@prefix foaf: <${FOAF}> .
@prefix org: <${ORG}> .
@prefix schema: <${SCHEMA}> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# Departments
ex:engineering a org:OrganizationalUnit ;
    org:identifier "ENG" ;
    schema:name "Engineering Department" ;
    org:hasSubOrganization ex:backend-team, ex:frontend-team .

ex:product a org:OrganizationalUnit ;
    org:identifier "PROD" ;
    schema:name "Product Department" .

ex:backend-team a org:OrganizationalUnit ;
    org:identifier "BACKEND" ;
    schema:name "Backend Team" ;
    org:subOrganizationOf ex:engineering .

ex:frontend-team a org:OrganizationalUnit ;
    org:identifier "FRONTEND" ;
    schema:name "Frontend Team" ;
    org:subOrganizationOf ex:engineering .

# Employees
ex:alice a foaf:Person ;
    foaf:name "Alice Johnson" ;
    foaf:mbox <mailto:alice@example.org> ;
    org:memberOf ex:engineering ;
    schema:jobTitle "VP of Engineering" ;
    ex:salary "180000"^^xsd:decimal ;
    ex:yearsExperience "12"^^xsd:integer ;
    ex:hasSkill ex:skill-rust, ex:skill-architecture, ex:skill-leadership .

ex:bob a foaf:Person ;
    foaf:name "Bob Smith" ;
    foaf:mbox <mailto:bob@example.org> ;
    org:memberOf ex:backend-team ;
    schema:jobTitle "Senior Backend Engineer" ;
    ex:reportsTo ex:alice ;
    ex:salary "150000"^^xsd:decimal ;
    ex:yearsExperience "8"^^xsd:integer ;
    ex:hasSkill ex:skill-rust, ex:skill-sparql, ex:skill-databases .

ex:carol a foaf:Person ;
    foaf:name "Carol Williams" ;
    foaf:mbox <mailto:carol@example.org> ;
    org:memberOf ex:frontend-team ;
    schema:jobTitle "Senior Frontend Engineer" ;
    ex:reportsTo ex:alice ;
    ex:salary "145000"^^xsd:decimal ;
    ex:yearsExperience "7"^^xsd:integer ;
    ex:hasSkill ex:skill-typescript, ex:skill-react, ex:skill-graphql .

ex:david a foaf:Person ;
    foaf:name "David Brown" ;
    foaf:mbox <mailto:david@example.org> ;
    org:memberOf ex:backend-team ;
    schema:jobTitle "Backend Engineer" ;
    ex:reportsTo ex:bob ;
    ex:salary "120000"^^xsd:decimal ;
    ex:yearsExperience "4"^^xsd:integer ;
    ex:hasSkill ex:skill-rust, ex:skill-docker, ex:skill-kubernetes .

ex:eve a foaf:Person ;
    foaf:name "Eve Davis" ;
    foaf:mbox <mailto:eve@example.org> ;
    org:memberOf ex:product ;
    schema:jobTitle "Product Manager" ;
    ex:salary "140000"^^xsd:decimal ;
    ex:yearsExperience "6"^^xsd:integer ;
    ex:hasSkill ex:skill-roadmapping, ex:skill-analytics, ex:skill-agile .

# Skills
ex:skill-rust a schema:DefinedTerm ;
    schema:name "Rust Programming" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-sparql a schema:DefinedTerm ;
    schema:name "SPARQL Query Language" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-databases a schema:DefinedTerm ;
    schema:name "Database Systems" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-typescript a schema:DefinedTerm ;
    schema:name "TypeScript" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-react a schema:DefinedTerm ;
    schema:name "React Framework" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-graphql a schema:DefinedTerm ;
    schema:name "GraphQL" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-docker a schema:DefinedTerm ;
    schema:name "Docker" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-kubernetes a schema:DefinedTerm ;
    schema:name "Kubernetes" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-architecture a schema:DefinedTerm ;
    schema:name "Software Architecture" ;
    schema:inDefinedTermSet ex:technical-skills .

ex:skill-leadership a schema:DefinedTerm ;
    schema:name "Technical Leadership" ;
    schema:inDefinedTermSet ex:soft-skills .

ex:skill-roadmapping a schema:DefinedTerm ;
    schema:name "Product Roadmapping" ;
    schema:inDefinedTermSet ex:soft-skills .

ex:skill-analytics a schema:DefinedTerm ;
    schema:name "Analytics" ;
    schema:inDefinedTermSet ex:soft-skills .

ex:skill-agile a schema:DefinedTerm ;
    schema:name "Agile Methodologies" ;
    schema:inDefinedTermSet ex:soft-skills .

# Projects
ex:project-graphdb a schema:SoftwareApplication ;
    schema:name "Knowledge Graph Database" ;
    schema:description "High-performance RDF database with SPARQL support" ;
    ex:usesSkill ex:skill-rust, ex:skill-sparql, ex:skill-databases ;
    ex:assignedTo ex:bob, ex:david .

ex:project-ui a schema:SoftwareApplication ;
    schema:name "Admin Dashboard" ;
    schema:description "Enterprise admin dashboard with React" ;
    ex:usesSkill ex:skill-typescript, ex:skill-react ;
    ex:assignedTo ex:carol .
`;

async function main() {
  console.log(`${colors.bright}${colors.magenta}
╔══════════════════════════════════════════════════════════════════════════════╗
║                     RUST-KGDB KNOWLEDGE GRAPH DEMO                           ║
║                     Enterprise Organizational Data                            ║
╚══════════════════════════════════════════════════════════════════════════════╝
${colors.reset}`);

  // Create database instance
  printSection('1. Initialize GraphDB');
  console.log('Creating GraphDB instance with base URI: http://example.org/');
  const db = new GraphDB('http://example.org/');
  console.log(`${colors.green}✓ GraphDB initialized${colors.reset}`);
  console.log(`Version: ${db.getVersion()}`);

  // Load organizational data
  printSection('2. Load Turtle RDF Data');
  console.log('Loading organizational knowledge graph (employees, departments, projects)...');
  db.loadTtl(organizationData, null);
  const tripleCount = db.countTriples(null);
  console.log(`${colors.green}✓ Loaded ${tripleCount} triples${colors.reset}`);

  // Example 1: SPARQL SELECT - Star pattern (WCOJ-optimized)
  printSection('3. SPARQL SELECT: Star Pattern Query');
  console.log('Find all information about Alice (star pattern - WCOJ-optimized for efficiency)');
  const starQuery = `
    PREFIX ex: <${EX}>
    PREFIX foaf: <${FOAF}>
    PREFIX schema: <${SCHEMA}>

    SELECT ?property ?value WHERE {
      ex:alice ?property ?value .
    }
    ORDER BY ?property
  `;
  printQuery(starQuery);
  const starResults = db.querySelect(starQuery);
  printResults(starResults);

  // Example 2: SPARQL SELECT - Chain pattern (WCOJ-optimized)
  printSection('4. SPARQL SELECT: Chain Pattern Query');
  console.log('Find employees who report to someone in the Engineering department');
  const chainQuery = `
    PREFIX ex: <${EX}>
    PREFIX foaf: <${FOAF}>
    PREFIX org: <${ORG}>

    SELECT ?employee ?manager ?dept WHERE {
      ?employee ex:reportsTo ?manager .
      ?manager org:memberOf ?dept .
      ?dept org:identifier "ENG" .
    }
  `;
  printQuery(chainQuery);
  const chainResults = db.querySelect(chainQuery);
  printResults(chainResults);

  // Example 3: Aggregation - Average salary by department
  printSection('5. SPARQL Aggregation: Average Salary by Department');
  const aggQuery = `
    PREFIX ex: <${EX}>
    PREFIX org: <${ORG}>
    PREFIX schema: <${SCHEMA}>
    PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

    SELECT ?deptName (AVG(?salary) AS ?avgSalary) (COUNT(?person) AS ?employees) WHERE {
      ?person org:memberOf ?dept .
      ?person ex:salary ?salary .
      ?dept schema:name ?deptName .
    }
    GROUP BY ?deptName
    ORDER BY DESC(?avgSalary)
  `;
  printQuery(aggQuery);
  const aggResults = db.querySelect(aggQuery);
  printResults(aggResults);

  // Example 4: Property path - Transitive reporting hierarchy
  printSection('6. SPARQL Property Path: Organizational Hierarchy');
  console.log('Find all direct and indirect reports to Alice (transitive closure)');
  const pathQuery = `
    PREFIX ex: <${EX}>
    PREFIX foaf: <${FOAF}>

    SELECT ?employee ?name WHERE {
      ?employee ex:reportsTo+ ex:alice .
      ?employee foaf:name ?name .
    }
    ORDER BY ?name
  `;
  printQuery(pathQuery);
  const pathResults = db.querySelect(pathQuery);
  printResults(pathResults);

  // Example 5: SPARQL ASK - Check existence
  printSection('7. SPARQL ASK: Check if Rust skill exists');
  const askQuery = `
    PREFIX ex: <${EX}>
    PREFIX schema: <${SCHEMA}>

    ASK {
      ?skill schema:name "Rust Programming" .
    }
  `;
  printQuery(askQuery);
  const askResult = db.queryAsk(askQuery);
  console.log(`${colors.green}Result: ${askResult ? 'YES' : 'NO'}${colors.reset}\n`);

  // Example 6: SPARQL CONSTRUCT - Create derived graph
  printSection('8. SPARQL CONSTRUCT: Employee Skill Network');
  console.log('Construct a simplified graph of employee-skill relationships');
  const constructQuery = `
    PREFIX ex: <${EX}>
    PREFIX foaf: <${FOAF}>
    PREFIX schema: <${SCHEMA}>

    CONSTRUCT {
      ?person foaf:name ?name .
      ?person ex:hasExpertise ?skillName .
    } WHERE {
      ?person foaf:name ?name .
      ?person ex:hasSkill ?skill .
      ?skill schema:name ?skillName .
    }
  `;
  printQuery(constructQuery);
  const constructResult = db.queryConstruct(constructQuery);
  console.log(`${colors.green}✓ Constructed ${constructResult} triples${colors.reset}\n`);

  // Example 7: Named graphs - Multi-tenant data
  printSection('9. Named Graphs: Multi-Tenant Data Isolation');
  console.log('Load sensitive compensation data into a separate named graph');

  const compensationData = `
    @prefix ex: <${EX}> .
    @prefix schema: <${SCHEMA}> .

    ex:alice schema:bonus "25000"^^<http://www.w3.org/2001/XMLSchema#decimal> .
    ex:bob schema:bonus "18000"^^<http://www.w3.org/2001/XMLSchema#decimal> .
    ex:carol schema:bonus "17000"^^<http://www.w3.org/2001/XMLSchema#decimal> .
  `;

  db.loadTtl(compensationData, 'http://example.org/compensation');
  console.log(`${colors.green}✓ Loaded compensation data into named graph${colors.reset}`);

  const namedGraphQuery = `
    PREFIX ex: <${EX}>
    PREFIX foaf: <${FOAF}>
    PREFIX schema: <${SCHEMA}>

    SELECT ?name ?salary ?bonus WHERE {
      ?person foaf:name ?name .
      ?person ex:salary ?salary .
      GRAPH <http://example.org/compensation> {
        ?person schema:bonus ?bonus .
      }
    }
    ORDER BY DESC(?salary)
  `;
  printQuery(namedGraphQuery);
  const namedGraphResults = db.querySelect(namedGraphQuery);
  printResults(namedGraphResults);

  // Example 8: Complex query - Skills match for project
  printSection('10. Complex Query: Find Candidates for GraphDB Project');
  console.log('Find employees with required skills for the GraphDB project');
  const complexQuery = `
    PREFIX ex: <${EX}>
    PREFIX foaf: <${FOAF}>
    PREFIX schema: <${SCHEMA}>

    SELECT ?name (COUNT(?requiredSkill) AS ?matchingSkills) WHERE {
      ex:project-graphdb ex:usesSkill ?requiredSkill .
      ?person ex:hasSkill ?requiredSkill .
      ?person foaf:name ?name .
    }
    GROUP BY ?name
    HAVING (COUNT(?requiredSkill) >= 2)
    ORDER BY DESC(?matchingSkills)
  `;
  printQuery(complexQuery);
  const complexResults = db.querySelect(complexQuery);
  printResults(complexResults);

  // Example 9: Export to file
  printSection('11. Export Data to Turtle Format');
  const outputDir = path.join(__dirname, '..', 'output');
  const outputFile = path.join(outputDir, 'organization.ttl');

  // Create output directory if it doesn't exist
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  // Export default graph to Turtle
  const exported = db.exportTtl(null);
  fs.writeFileSync(outputFile, exported);
  console.log(`${colors.green}✓ Exported default graph to: ${outputFile}${colors.reset}`);
  console.log(`File size: ${(exported.length / 1024).toFixed(2)} KB`);

  // Summary statistics
  printSection('12. Summary Statistics');
  const statsQuery = `
    PREFIX foaf: <${FOAF}>
    PREFIX org: <${ORG}>
    PREFIX ex: <${EX}>
    PREFIX schema: <${SCHEMA}>

    SELECT
      (COUNT(DISTINCT ?person) AS ?totalEmployees)
      (COUNT(DISTINCT ?dept) AS ?totalDepartments)
      (COUNT(DISTINCT ?skill) AS ?totalSkills)
      (COUNT(DISTINCT ?project) AS ?totalProjects)
    WHERE {
      {
        ?person a foaf:Person .
      } UNION {
        ?dept a org:OrganizationalUnit .
      } UNION {
        ?skill a schema:DefinedTerm .
      } UNION {
        ?project a schema:SoftwareApplication .
      }
    }
  `;
  printQuery(statsQuery);
  const stats = db.querySelect(statsQuery);
  printResults(stats);

  console.log(`\n${colors.bright}${colors.magenta}${'='.repeat(80)}${colors.reset}`);
  console.log(`${colors.bright}${colors.green}Demo completed successfully!${colors.reset}`);
  console.log(`${colors.bright}${colors.magenta}${'='.repeat(80)}${colors.reset}\n`);

  console.log(`${colors.cyan}Key Features Demonstrated:${colors.reset}`);
  console.log(`  ✓ RDF data loading (Turtle format)`);
  console.log(`  ✓ SPARQL SELECT queries (star and chain patterns with WCOJ optimization)`);
  console.log(`  ✓ SPARQL ASK queries (boolean results)`);
  console.log(`  ✓ SPARQL CONSTRUCT queries (graph transformation)`);
  console.log(`  ✓ Aggregations (COUNT, AVG, GROUP BY, HAVING)`);
  console.log(`  ✓ Property paths (transitive closure with +)`);
  console.log(`  ✓ Named graphs (data isolation)`);
  console.log(`  ✓ Data export (Turtle format)`);
  console.log(`  ✓ Complex joins and filters\n`);

  console.log(`${colors.cyan}Performance Characteristics:${colors.reset}`);
  console.log(`  • Lookup speed: 2.78 µs per triple`);
  console.log(`  • Bulk insert: 146K triples/sec`);
  console.log(`  • Memory: 24 bytes/triple`);
  console.log(`  • WCOJ-optimized joins for star and chain patterns\n`);

  console.log(`${colors.yellow}Next Steps:${colors.reset}`);
  console.log(`  1. Modify the knowledge graph data in this file`);
  console.log(`  2. Experiment with different SPARQL queries`);
  console.log(`  3. Try loading your own Turtle files`);
  console.log(`  4. Explore the exported data in output/organization.ttl\n`);
}

// Run the demo
main().catch((error) => {
  console.error(`${colors.bright}\x1b[31mError:${colors.reset}`, error);
  process.exit(1);
});
