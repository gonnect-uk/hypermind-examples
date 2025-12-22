/**
 * Datalog Example for rust-kgdb TypeScript SDK
 *
 * Demonstrates Datalog reasoning capabilities including:
 * - Adding facts and rules using JSON format
 * - Semi-naive evaluation
 * - Recursive rule evaluation (transitive closure)
 * - Querying derived facts
 *
 * IMPORTANT: The DatalogProgram API uses JSON strings for facts and rules.
 * This enables a flexible, language-agnostic interface.
 */

import { DatalogProgram, evaluateDatalog, queryDatalog, GraphDB } from 'rust-kgdb';

// =============================================================================
// Example 1: Basic Facts and Rules
// =============================================================================

function basicDatalogExample() {
    console.log('=== Basic Datalog Facts and Rules ===\n');

    const program = new DatalogProgram();

    // Add facts: parent(X, Y) means X is parent of Y
    // Facts use JSON format: {"predicate": "name", "terms": ["arg1", "arg2"]}
    program.addFact(JSON.stringify({ predicate: 'parent', terms: ['alice', 'bob'] }));
    program.addFact(JSON.stringify({ predicate: 'parent', terms: ['alice', 'carol'] }));
    program.addFact(JSON.stringify({ predicate: 'parent', terms: ['bob', 'david'] }));
    program.addFact(JSON.stringify({ predicate: 'parent', terms: ['bob', 'eve'] }));
    program.addFact(JSON.stringify({ predicate: 'parent', terms: ['carol', 'frank'] }));

    console.log('Facts added:');
    console.log('  parent(alice, bob)');
    console.log('  parent(alice, carol)');
    console.log('  parent(bob, david)');
    console.log('  parent(bob, eve)');
    console.log('  parent(carol, frank)');
    console.log(`  Total facts: ${program.factCount()}\n`);

    // Add rule: grandparent(X, Z) :- parent(X, Y), parent(Y, Z)
    // Rules use JSON format: {"head": {...}, "body": [...]}
    program.addRule(JSON.stringify({
        head: { predicate: 'grandparent', terms: ['X', 'Z'] },
        body: [
            { predicate: 'parent', terms: ['X', 'Y'] },
            { predicate: 'parent', terms: ['Y', 'Z'] }
        ]
    }));

    console.log('Rule added:');
    console.log('  grandparent(X, Z) :- parent(X, Y), parent(Y, Z)');
    console.log(`  Total rules: ${program.ruleCount()}\n`);

    // Evaluate to derive all facts using semi-naive evaluation
    const evalResult = evaluateDatalog(program);
    console.log('Evaluation completed.');
    console.log(`  Result: ${evalResult}\n`);

    // Query grandparents
    const grandparents = queryDatalog(program, 'grandparent');
    const results = JSON.parse(grandparents);
    console.log('Query: grandparent(?who, ?grandchild)');
    console.log('Results:');
    for (const result of results) {
        console.log(`  ${result.terms[0]} is grandparent of ${result.terms[1]}`);
    }
    console.log();
}

// =============================================================================
// Example 2: Recursive Rules (Transitive Closure)
// =============================================================================

function recursiveDatalogExample() {
    console.log('=== Recursive Rules (Transitive Closure) ===\n');

    const program = new DatalogProgram();

    // Add edge facts for a graph
    program.addFact(JSON.stringify({ predicate: 'edge', terms: ['a', 'b'] }));
    program.addFact(JSON.stringify({ predicate: 'edge', terms: ['b', 'c'] }));
    program.addFact(JSON.stringify({ predicate: 'edge', terms: ['c', 'd'] }));
    program.addFact(JSON.stringify({ predicate: 'edge', terms: ['d', 'e'] }));
    program.addFact(JSON.stringify({ predicate: 'edge', terms: ['a', 'c'] })); // shortcut

    console.log('Graph edges: a->b, b->c, c->d, d->e, a->c\n');

    // Base case: path(X, Y) :- edge(X, Y)
    program.addRule(JSON.stringify({
        head: { predicate: 'path', terms: ['X', 'Y'] },
        body: [{ predicate: 'edge', terms: ['X', 'Y'] }]
    }));

    // Recursive case: path(X, Z) :- edge(X, Y), path(Y, Z)
    program.addRule(JSON.stringify({
        head: { predicate: 'path', terms: ['X', 'Z'] },
        body: [
            { predicate: 'edge', terms: ['X', 'Y'] },
            { predicate: 'path', terms: ['Y', 'Z'] }
        ]
    }));

    console.log('Rules added:');
    console.log('  path(X, Y) :- edge(X, Y)         % base case');
    console.log('  path(X, Z) :- edge(X, Y), path(Y, Z)  % recursive case\n');

    // Evaluate using semi-naive algorithm
    const evalResult = evaluateDatalog(program);
    console.log(`Semi-naive evaluation completed: ${evalResult}\n`);

    // Query all paths from 'a'
    const paths = queryDatalog(program, 'path');
    const results = JSON.parse(paths);

    console.log('Query: All paths');
    console.log('Results (nodes reachable from each source):');
    for (const result of results) {
        console.log(`  ${result.terms[0]} can reach ${result.terms[1]}`);
    }
    console.log();
}

// =============================================================================
// Example 3: Fraud Detection Rules
// =============================================================================

function fraudDetectionExample() {
    console.log('=== Fraud Detection Rules ===\n');

    const program = new DatalogProgram();

    // Add transaction facts: transaction(id, sender, receiver, amount)
    program.addFact(JSON.stringify({
        predicate: 'transaction',
        terms: ['tx001', 'account_a', 'account_b', '50000']
    }));
    program.addFact(JSON.stringify({
        predicate: 'transaction',
        terms: ['tx002', 'account_b', 'account_c', '48000']
    }));
    program.addFact(JSON.stringify({
        predicate: 'transaction',
        terms: ['tx003', 'account_c', 'account_a', '45000']
    }));

    // Add account facts: account(id, jurisdiction)
    program.addFact(JSON.stringify({ predicate: 'account', terms: ['account_a', 'Panama'] }));
    program.addFact(JSON.stringify({ predicate: 'account', terms: ['account_b', 'BVI'] }));
    program.addFact(JSON.stringify({ predicate: 'account', terms: ['account_c', 'Cayman'] }));

    // Add high-risk jurisdiction facts
    program.addFact(JSON.stringify({ predicate: 'high_risk_jurisdiction', terms: ['Panama'] }));
    program.addFact(JSON.stringify({ predicate: 'high_risk_jurisdiction', terms: ['BVI'] }));
    program.addFact(JSON.stringify({ predicate: 'high_risk_jurisdiction', terms: ['Cayman'] }));

    console.log('Facts loaded:');
    console.log('  3 transactions forming circular pattern');
    console.log('  3 accounts in offshore jurisdictions\n');

    // Rule: sends_to(A, B) :- transaction(_, A, B, _)
    program.addRule(JSON.stringify({
        head: { predicate: 'sends_to', terms: ['A', 'B'] },
        body: [{ predicate: 'transaction', terms: ['_', 'A', 'B', '_'] }]
    }));

    // Rule: circular_flow(A, B, C) :- sends_to(A, B), sends_to(B, C), sends_to(C, A)
    program.addRule(JSON.stringify({
        head: { predicate: 'circular_flow', terms: ['A', 'B', 'C'] },
        body: [
            { predicate: 'sends_to', terms: ['A', 'B'] },
            { predicate: 'sends_to', terms: ['B', 'C'] },
            { predicate: 'sends_to', terms: ['C', 'A'] }
        ]
    }));

    // Rule: offshore_account(A) :- account(A, J), high_risk_jurisdiction(J)
    program.addRule(JSON.stringify({
        head: { predicate: 'offshore_account', terms: ['A'] },
        body: [
            { predicate: 'account', terms: ['A', 'J'] },
            { predicate: 'high_risk_jurisdiction', terms: ['J'] }
        ]
    }));

    console.log('Fraud detection rules:');
    console.log('  sends_to(A, B) :- transaction(_, A, B, _)');
    console.log('  circular_flow(A, B, C) :- sends_to(A, B), sends_to(B, C), sends_to(C, A)');
    console.log('  offshore_account(A) :- account(A, J), high_risk_jurisdiction(J)\n');

    // Evaluate
    evaluateDatalog(program);

    // Query circular flows
    const circularFlows = queryDatalog(program, 'circular_flow');
    const circularResults = JSON.parse(circularFlows);
    console.log('Circular payment patterns detected:');
    for (const result of circularResults) {
        console.log(`  ${result.terms[0]} -> ${result.terms[1]} -> ${result.terms[2]} -> ${result.terms[0]}`);
    }

    // Query offshore accounts
    const offshoreAccounts = queryDatalog(program, 'offshore_account');
    const offshoreResults = JSON.parse(offshoreAccounts);
    console.log('\nOffshore accounts flagged:');
    for (const result of offshoreResults) {
        console.log(`  ${result.terms[0]}`);
    }
    console.log();
}

// =============================================================================
// Example 4: Access Control Rules
// =============================================================================

function accessControlExample() {
    console.log('=== Access Control Rules ===\n');

    const program = new DatalogProgram();

    // User roles
    program.addFact(JSON.stringify({ predicate: 'role', terms: ['alice', 'admin'] }));
    program.addFact(JSON.stringify({ predicate: 'role', terms: ['bob', 'developer'] }));
    program.addFact(JSON.stringify({ predicate: 'role', terms: ['carol', 'developer'] }));
    program.addFact(JSON.stringify({ predicate: 'role', terms: ['david', 'viewer'] }));

    // Resource permissions by role
    program.addFact(JSON.stringify({ predicate: 'permission', terms: ['admin', 'read'] }));
    program.addFact(JSON.stringify({ predicate: 'permission', terms: ['admin', 'write'] }));
    program.addFact(JSON.stringify({ predicate: 'permission', terms: ['admin', 'delete'] }));
    program.addFact(JSON.stringify({ predicate: 'permission', terms: ['developer', 'read'] }));
    program.addFact(JSON.stringify({ predicate: 'permission', terms: ['developer', 'write'] }));
    program.addFact(JSON.stringify({ predicate: 'permission', terms: ['viewer', 'read'] }));

    console.log('Access control facts loaded\n');

    // Rule: can_do(User, Action) :- role(User, Role), permission(Role, Action)
    program.addRule(JSON.stringify({
        head: { predicate: 'can_do', terms: ['User', 'Action'] },
        body: [
            { predicate: 'role', terms: ['User', 'Role'] },
            { predicate: 'permission', terms: ['Role', 'Action'] }
        ]
    }));

    console.log('Access control rules:');
    console.log('  can_do(User, Action) :- role(User, Role), permission(Role, Action)\n');

    evaluateDatalog(program);

    // Query all permissions
    const permissions = queryDatalog(program, 'can_do');
    const results = JSON.parse(permissions);

    console.log('Derived permissions:');
    const permissionsByUser: { [key: string]: string[] } = {};
    for (const result of results) {
        const user = result.terms[0];
        const action = result.terms[1];
        if (!permissionsByUser[user]) {
            permissionsByUser[user] = [];
        }
        permissionsByUser[user].push(action);
    }

    for (const [user, actions] of Object.entries(permissionsByUser)) {
        console.log(`  ${user} can: ${actions.join(', ')}`);
    }
    console.log();
}

// =============================================================================
// Example 5: Integration with RDF Graph
// =============================================================================

function rdfIntegrationExample() {
    console.log('=== RDF Graph + Datalog Integration ===\n');

    // Create RDF graph
    const db = new GraphDB('http://example.org/company');

    // Load company data in Turtle format
    const ttl = `
        @prefix ex: <http://example.org/> .
        @prefix foaf: <http://xmlns.com/foaf/0.1/> .
        @prefix org: <http://www.w3.org/ns/org#> .

        ex:alice a foaf:Person ;
            foaf:name "Alice" ;
            org:memberOf ex:engineering .

        ex:bob a foaf:Person ;
            foaf:name "Bob" ;
            org:memberOf ex:engineering ;
            org:reportsTo ex:alice .

        ex:carol a foaf:Person ;
            foaf:name "Carol" ;
            org:memberOf ex:sales .

        ex:engineering a org:OrganizationalUnit ;
            org:name "Engineering Department" .

        ex:sales a org:OrganizationalUnit ;
            org:name "Sales Department" .
    `;

    db.loadTtl(ttl, null);
    console.log(`RDF graph loaded with ${db.countTriples()} triples\n`);

    // Query to extract facts for Datalog
    const membershipQuery = `
        PREFIX org: <http://www.w3.org/ns/org#>
        SELECT ?person ?dept WHERE {
            ?person org:memberOf ?dept .
        }
    `;

    const memberships = db.querySelect(membershipQuery);

    // Create Datalog program from SPARQL results
    const program = new DatalogProgram();

    console.log('Converting RDF to Datalog facts:');
    for (const row of memberships) {
        const person = row.bindings.person.split('/').pop() || '';
        const dept = row.bindings.dept.split('/').pop() || '';
        program.addFact(JSON.stringify({ predicate: 'member_of', terms: [person, dept] }));
        console.log(`  member_of(${person}, ${dept})`);
    }
    console.log();

    // Add rule: colleague(X, Y) :- member_of(X, Dept), member_of(Y, Dept)
    program.addRule(JSON.stringify({
        head: { predicate: 'colleague', terms: ['X', 'Y'] },
        body: [
            { predicate: 'member_of', terms: ['X', 'Dept'] },
            { predicate: 'member_of', terms: ['Y', 'Dept'] }
        ]
    }));

    console.log('Reasoning rule: colleague(X, Y) :- member_of(X, Dept), member_of(Y, Dept)\n');

    evaluateDatalog(program);

    // Query colleagues
    const colleagues = queryDatalog(program, 'colleague');
    const results = JSON.parse(colleagues);
    console.log('Colleagues (same department):');
    for (const result of results) {
        if (result.terms[0] !== result.terms[1]) {  // Exclude self-pairs
            console.log(`  ${result.terms[0]} and ${result.terms[1]}`);
        }
    }
    console.log();
}

// =============================================================================
// Run All Examples
// =============================================================================

function main() {
    console.log('╔════════════════════════════════════════════════════════════════════╗');
    console.log('║             Datalog Reasoning Examples - rust-kgdb SDK             ║');
    console.log('╠════════════════════════════════════════════════════════════════════╣');
    console.log('║  Using DatalogProgram with JSON-based fact/rule API                ║');
    console.log('╚════════════════════════════════════════════════════════════════════╝\n');

    try {
        basicDatalogExample();
        recursiveDatalogExample();
        fraudDetectionExample();
        accessControlExample();
        rdfIntegrationExample();

        console.log('════════════════════════════════════════════════════════════════════');
        console.log('  All Datalog examples completed successfully!');
        console.log('════════════════════════════════════════════════════════════════════');
    } catch (error) {
        console.error('Error running examples:', error);
        process.exit(1);
    }
}

main();
