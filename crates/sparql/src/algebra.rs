
//! SPARQL Query Algebra
//!
//! Zero-copy representation of SPARQL queries as algebraic operators.
//! Based on W3C SPARQL 1.1 specification and Apache Jena ARQ.
//!
//! Design principles:
//! - Zero-copy with lifetimes
//! - Visitor pattern for traversal
//! - Strongly typed (no stringly-typed operations)
//! - Grammar-driven structure

use rdf_model::Node;
use std::fmt;

/// Query algebra operator
///
/// Represents the complete SPARQL algebra as defined in the W3C specification.
/// All operators are zero-copy with borrowed lifetimes.
#[derive(Debug, Clone, PartialEq)]
pub enum Algebra<'a> {
    /// Basic Graph Pattern - set of triple patterns
    BGP(Vec<TriplePattern<'a>>),

    /// Join - combination of two patterns
    Join {
        /// Left operand of the join
        left: Box<Algebra<'a>>,
        /// Right operand of the join
        right: Box<Algebra<'a>>,
    },

    /// Left Join (OPTIONAL) - optional pattern matching
    LeftJoin {
        /// Required left pattern
        left: Box<Algebra<'a>>,
        /// Optional right pattern
        right: Box<Algebra<'a>>,
        /// Optional filter expression
        expr: Option<Expression<'a>>,
    },

    /// Filter - constraint on solutions
    Filter {
        /// Filter expression to evaluate
        expr: Expression<'a>,
        /// Input algebra to filter
        input: Box<Algebra<'a>>,
    },

    /// Union - alternative patterns
    Union {
        /// First alternative pattern
        left: Box<Algebra<'a>>,
        /// Second alternative pattern
        right: Box<Algebra<'a>>,
    },

    /// Minus - remove solutions
    Minus {
        /// Base pattern to subtract from
        left: Box<Algebra<'a>>,
        /// Pattern whose solutions to remove
        right: Box<Algebra<'a>>,
    },

    /// Graph - named graph pattern
    Graph {
        /// Graph identifier (variable or IRI)
        graph: VarOrNode<'a>,
        /// Pattern to match within the graph
        input: Box<Algebra<'a>>,
    },

    /// Service - federated query
    Service {
        /// Service endpoint (variable or IRI)
        endpoint: VarOrNode<'a>,
        /// Pattern to evaluate at the service
        input: Box<Algebra<'a>>,
        /// Whether to silently ignore service errors
        silent: bool,
    },

    /// Extend - bind expression to variable
    Extend {
        /// Variable to bind the expression result to
        var: Variable<'a>,
        /// Expression to evaluate
        expr: Expression<'a>,
        /// Input algebra to extend
        input: Box<Algebra<'a>>,
    },

    /// Project - select variables
    Project {
        /// Variables to project
        vars: Vec<Variable<'a>>,
        /// Input algebra to project from
        input: Box<Algebra<'a>>,
    },

    /// Distinct - eliminate duplicates
    Distinct {
        /// Input algebra to deduplicate
        input: Box<Algebra<'a>>,
    },

    /// Reduced - permit elimination of some duplicates
    Reduced {
        /// Input algebra to reduce
        input: Box<Algebra<'a>>,
    },

    /// OrderBy - sort solutions
    OrderBy {
        /// Ordering conditions
        conditions: Vec<OrderCondition<'a>>,
        /// Input algebra to sort
        input: Box<Algebra<'a>>,
    },

    /// Slice - limit and offset
    Slice {
        /// Offset (number of solutions to skip)
        start: Option<usize>,
        /// Limit (maximum number of solutions)
        length: Option<usize>,
        /// Input algebra to slice
        input: Box<Algebra<'a>>,
    },

    /// Group - group solutions for aggregation
    Group {
        /// Variables to group by
        vars: Vec<Variable<'a>>,
        /// Aggregate functions with result variables
        aggregates: Vec<(Variable<'a>, Aggregate<'a>)>,
        /// Input algebra to group
        input: Box<Algebra<'a>>,
    },

    /// Table - inline data (VALUES)
    Table {
        /// Variables in the table
        vars: Vec<Variable<'a>>,
        /// Rows of values (None for UNDEF)
        rows: Vec<Vec<Option<Node<'a>>>>,
    },

    /// Path - property path between subject and object
    Path {
        /// Subject of the path
        subject: VarOrNode<'a>,
        /// Property path expression
        path: PropertyPath<'a>,
        /// Object of the path
        object: VarOrNode<'a>,
    },
}

/// Triple pattern - pattern with variables
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TriplePattern<'a> {
    /// Subject position (variable or node)
    pub subject: VarOrNode<'a>,
    /// Predicate position (variable or node)
    pub predicate: VarOrNode<'a>,
    /// Object position (variable or node)
    pub object: VarOrNode<'a>,
}

/// Variable or node (for patterns)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum VarOrNode<'a> {
    /// Variable reference
    Var(Variable<'a>),
    /// Concrete RDF node
    Node(Node<'a>),
}

/// Variable name
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Variable<'a> {
    /// Variable name without the ? or $ prefix
    pub name: &'a str,
}

impl<'a> Variable<'a> {
    /// Creates a new variable with the given name
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl<'a> fmt::Display for Variable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "?{}", self.name)
    }
}

/// Expression in SPARQL query
#[derive(Debug, Clone, PartialEq)]
pub enum Expression<'a> {
    /// Variable reference
    Var(Variable<'a>),

    /// Constant node
    Constant(Node<'a>),

    /// Logical OR
    Or(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Logical AND
    And(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Equality
    Equal(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Inequality
    NotEqual(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Less than
    Less(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Greater than
    Greater(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Less than or equal
    LessOrEqual(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Greater than or equal
    GreaterOrEqual(Box<Expression<'a>>, Box<Expression<'a>>),

    /// IN operator
    In(Box<Expression<'a>>, Vec<Expression<'a>>),

    /// NOT IN operator
    NotIn(Box<Expression<'a>>, Vec<Expression<'a>>),

    /// Addition
    Add(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Subtraction
    Subtract(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Multiplication
    Multiply(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Division
    Divide(Box<Expression<'a>>, Box<Expression<'a>>),

    /// Unary negation
    Negate(Box<Expression<'a>>),

    /// Unary plus
    Plus(Box<Expression<'a>>),

    /// Logical NOT
    Not(Box<Expression<'a>>),

    /// Function call
    FunctionCall {
        /// Function name (IRI)
        function: &'a str,
        /// Function arguments
        args: Vec<Expression<'a>>,
    },

    /// Builtin function
    Builtin(BuiltinFunction<'a>),

    /// Aggregate function
    Aggregate(Aggregate<'a>),

    /// EXISTS pattern
    Exists(Box<Algebra<'a>>),

    /// NOT EXISTS pattern
    NotExists(Box<Algebra<'a>>),
}

/// Builtin SPARQL functions
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltinFunction<'a> {
    // String functions
    /// STR - converts value to string
    Str(Box<Expression<'a>>),
    /// LANG - returns language tag of literal
    Lang(Box<Expression<'a>>),
    /// DATATYPE - returns datatype IRI of literal
    Datatype(Box<Expression<'a>>),
    /// IRI - constructs IRI from string
    IRI(Box<Expression<'a>>),
    /// URI - alias for IRI
    URI(Box<Expression<'a>>),
    /// STRLEN - returns string length
    StrLen(Box<Expression<'a>>),
    /// SUBSTR - extracts substring (string, start, optional length)
    Substr(Box<Expression<'a>>, Box<Expression<'a>>, Option<Box<Expression<'a>>>),
    /// UCASE - converts string to uppercase
    UCase(Box<Expression<'a>>),
    /// LCASE - converts string to lowercase
    LCase(Box<Expression<'a>>),
    /// STRSTARTS - tests if string starts with substring
    StrStarts(Box<Expression<'a>>, Box<Expression<'a>>),
    /// STRENDS - tests if string ends with substring
    StrEnds(Box<Expression<'a>>, Box<Expression<'a>>),
    /// CONTAINS - tests if string contains substring
    Contains(Box<Expression<'a>>, Box<Expression<'a>>),
    /// STRBEFORE - returns substring before first occurrence
    StrBefore(Box<Expression<'a>>, Box<Expression<'a>>),
    /// STRAFTER - returns substring after first occurrence
    StrAfter(Box<Expression<'a>>, Box<Expression<'a>>),
    /// ENCODE_FOR_URI - encodes string for use in URI
    EncodeForURI(Box<Expression<'a>>),
    /// CONCAT - concatenates strings
    Concat(Vec<Expression<'a>>),
    /// LANGMATCHES - tests if language tag matches pattern
    LangMatches(Box<Expression<'a>>, Box<Expression<'a>>),
    /// REPLACE - replaces pattern in string (string, pattern, replacement, optional flags)
    Replace(Box<Expression<'a>>, Box<Expression<'a>>, Box<Expression<'a>>, Option<Box<Expression<'a>>>),
    /// REGEX - tests if string matches regular expression
    Regex(Box<Expression<'a>>, Box<Expression<'a>>, Option<Box<Expression<'a>>>),

    // Numeric functions
    /// ABS - absolute value
    Abs(Box<Expression<'a>>),
    /// ROUND - rounds to nearest integer
    Round(Box<Expression<'a>>),
    /// CEIL - rounds up to integer
    Ceil(Box<Expression<'a>>),
    /// FLOOR - rounds down to integer
    Floor(Box<Expression<'a>>),
    /// RAND - returns random number between 0 and 1
    Rand,

    // Date/Time functions
    /// NOW - returns current datetime
    Now,
    /// YEAR - extracts year from datetime
    Year(Box<Expression<'a>>),
    /// MONTH - extracts month from datetime
    Month(Box<Expression<'a>>),
    /// DAY - extracts day from datetime
    Day(Box<Expression<'a>>),
    /// HOURS - extracts hours from datetime
    Hours(Box<Expression<'a>>),
    /// MINUTES - extracts minutes from datetime
    Minutes(Box<Expression<'a>>),
    /// SECONDS - extracts seconds from datetime
    Seconds(Box<Expression<'a>>),
    /// TIMEZONE - returns timezone component
    Timezone(Box<Expression<'a>>),
    /// TZ - returns timezone string
    TZ(Box<Expression<'a>>),

    // Hash functions
    /// MD5 - computes MD5 hash
    MD5(Box<Expression<'a>>),
    /// SHA1 - computes SHA-1 hash
    SHA1(Box<Expression<'a>>),
    /// SHA256 - computes SHA-256 hash
    SHA256(Box<Expression<'a>>),
    /// SHA384 - computes SHA-384 hash
    SHA384(Box<Expression<'a>>),
    /// SHA512 - computes SHA-512 hash
    SHA512(Box<Expression<'a>>),

    // Test functions
    /// isIRI - tests if value is an IRI
    IsIRI(Box<Expression<'a>>),
    /// isURI - alias for isIRI
    IsURI(Box<Expression<'a>>),
    /// isBLANK - tests if value is a blank node
    IsBlank(Box<Expression<'a>>),
    /// isLITERAL - tests if value is a literal
    IsLiteral(Box<Expression<'a>>),
    /// isNUMERIC - tests if value is numeric
    IsNumeric(Box<Expression<'a>>),
    /// BOUND - tests if variable is bound
    Bound(Variable<'a>),
    /// sameTerm - tests if terms are identical
    SameTerm(Box<Expression<'a>>, Box<Expression<'a>>),

    // Other functions
    /// BNODE - creates blank node with optional identifier
    BNode(Option<Box<Expression<'a>>>),
    /// UUID - generates random UUID as IRI
    UUID,
    /// STRUUID - generates random UUID as string
    StrUUID,
    /// COALESCE - returns first non-error value
    Coalesce(Vec<Expression<'a>>),
    /// IF - conditional expression (condition, then, else)
    If(Box<Expression<'a>>, Box<Expression<'a>>, Box<Expression<'a>>),
    /// STRLANG - constructs language-tagged literal
    StrLang(Box<Expression<'a>>, Box<Expression<'a>>),
    /// STRDT - constructs typed literal
    StrDT(Box<Expression<'a>>, Box<Expression<'a>>),
}

/// Aggregate functions
#[derive(Debug, Clone, PartialEq)]
pub enum Aggregate<'a> {
    /// COUNT - counts solutions (expression optional for COUNT(*))
    Count {
        /// Expression to count (None for COUNT(*))
        expr: Option<Box<Expression<'a>>>,
        /// Whether to count only distinct values
        distinct: bool,
    },
    /// SUM - sums numeric values
    Sum {
        /// Expression to sum
        expr: Box<Expression<'a>>,
        /// Whether to sum only distinct values
        distinct: bool,
    },
    /// MIN - finds minimum value
    Min {
        /// Expression to find minimum of
        expr: Box<Expression<'a>>,
        /// Whether to consider only distinct values
        distinct: bool,
    },
    /// MAX - finds maximum value
    Max {
        /// Expression to find maximum of
        expr: Box<Expression<'a>>,
        /// Whether to consider only distinct values
        distinct: bool,
    },
    /// AVG - computes average of numeric values
    Avg {
        /// Expression to average
        expr: Box<Expression<'a>>,
        /// Whether to average only distinct values
        distinct: bool,
    },
    /// SAMPLE - returns arbitrary sample value
    Sample {
        /// Expression to sample
        expr: Box<Expression<'a>>,
        /// Whether to sample from distinct values
        distinct: bool,
    },
    /// GROUP_CONCAT - concatenates values into string
    GroupConcat {
        /// Expression to concatenate
        expr: Box<Expression<'a>>,
        /// Separator string (default is space)
        separator: Option<&'a str>,
        /// Whether to concatenate only distinct values
        distinct: bool,
    },
}

/// Order condition
#[derive(Debug, Clone, PartialEq)]
pub struct OrderCondition<'a> {
    /// Expression to order by
    pub expr: Expression<'a>,
    /// Whether to sort in ascending order (false for descending)
    pub ascending: bool,
}

/// Property path expression
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyPath<'a> {
    /// Direct predicate
    Predicate(Node<'a>),

    /// Inverse path (^p)
    Inverse(Box<PropertyPath<'a>>),

    /// Sequence path (p1 / p2)
    Sequence(Box<PropertyPath<'a>>, Box<PropertyPath<'a>>),

    /// Alternative path (p1 | p2)
    Alternative(Box<PropertyPath<'a>>, Box<PropertyPath<'a>>),

    /// Zero or more (p*)
    ZeroOrMore(Box<PropertyPath<'a>>),

    /// One or more (p+)
    OneOrMore(Box<PropertyPath<'a>>),

    /// Zero or one (p?)
    ZeroOrOne(Box<PropertyPath<'a>>),

    /// Negated property set (!(p1|p2|...))
    NegatedPropertySet(Vec<Node<'a>>),
}

/// Query type
#[derive(Debug, Clone, PartialEq)]
pub enum Query<'a> {
    /// SELECT query - returns variable bindings
    Select {
        /// Whether to eliminate duplicate solutions
        distinct: bool,
        /// Whether to permit elimination of some duplicates
        reduced: bool,
        /// Variables or expressions to project
        projection: Projection<'a>,
        /// Dataset specification (FROM/FROM NAMED)
        dataset: Dataset<'a>,
        /// Query pattern (WHERE clause)
        pattern: Algebra<'a>,
        /// ORDER BY conditions
        order: Vec<OrderCondition<'a>>,
        /// LIMIT (maximum number of results)
        limit: Option<usize>,
        /// OFFSET (number of results to skip)
        offset: Option<usize>,
    },
    /// CONSTRUCT query - returns RDF graph
    Construct {
        /// Template for constructing triples
        template: Vec<TriplePattern<'a>>,
        /// Dataset specification (FROM/FROM NAMED)
        dataset: Dataset<'a>,
        /// Query pattern (WHERE clause)
        pattern: Algebra<'a>,
        /// ORDER BY conditions
        order: Vec<OrderCondition<'a>>,
        /// LIMIT (maximum number of results)
        limit: Option<usize>,
        /// OFFSET (number of results to skip)
        offset: Option<usize>,
    },
    /// DESCRIBE query - returns description of resources
    Describe {
        /// Resources to describe
        resources: Vec<VarOrNode<'a>>,
        /// Dataset specification (FROM/FROM NAMED)
        dataset: Dataset<'a>,
        /// Optional query pattern (WHERE clause)
        pattern: Option<Algebra<'a>>,
        /// ORDER BY conditions
        order: Vec<OrderCondition<'a>>,
        /// LIMIT (maximum number of results)
        limit: Option<usize>,
        /// OFFSET (number of results to skip)
        offset: Option<usize>,
    },
    /// ASK query - returns boolean
    Ask {
        /// Dataset specification (FROM/FROM NAMED)
        dataset: Dataset<'a>,
        /// Query pattern (WHERE clause)
        pattern: Algebra<'a>,
    },
}

/// Update operation type (SPARQL 1.1 UPDATE)
#[derive(Debug, Clone, PartialEq)]
pub enum Update<'a> {
    /// INSERT DATA - insert concrete quads
    InsertData {
        /// Quads to insert (must be concrete, no variables)
        quads: Vec<QuadPattern<'a>>,
    },

    /// DELETE DATA - delete concrete quads
    DeleteData {
        /// Quads to delete (must be concrete, no variables)
        quads: Vec<QuadPattern<'a>>,
    },

    /// DELETE/INSERT - conditional update with WHERE clause
    DeleteInsert {
        /// Quad patterns to delete
        delete: Vec<QuadPattern<'a>>,
        /// Quad patterns to insert
        insert: Vec<QuadPattern<'a>>,
        /// WHERE clause pattern for matching
        pattern: Algebra<'a>,
        /// USING/USING NAMED dataset
        using: Option<Dataset<'a>>,
    },

    /// DELETE WHERE - shorthand for DELETE/INSERT with same pattern
    DeleteWhere {
        /// Quad patterns to delete (used for matching and deletion)
        quads: Vec<QuadPattern<'a>>,
    },

    /// LOAD - load RDF document into graph
    Load {
        /// Source IRI to load from
        source: &'a str,
        /// Target graph (None for default graph)
        target: Option<Node<'a>>,
        /// Whether to silently ignore errors
        silent: bool,
    },

    /// CLEAR - remove all triples from graph
    Clear {
        /// Target graph to clear
        graph: GraphTarget<'a>,
        /// Whether to silently ignore errors
        silent: bool,
    },

    /// CREATE - create a new graph
    Create {
        /// Graph IRI to create
        graph: Node<'a>,
        /// Whether to silently ignore if graph exists
        silent: bool,
    },

    /// DROP - remove graph
    Drop {
        /// Target graph to drop
        graph: GraphTarget<'a>,
        /// Whether to silently ignore if graph doesn't exist
        silent: bool,
    },
}

/// Quad pattern with optional graph component
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuadPattern<'a> {
    /// Subject position (variable or node)
    pub subject: VarOrNode<'a>,
    /// Predicate position (variable or node)
    pub predicate: VarOrNode<'a>,
    /// Object position (variable or node)
    pub object: VarOrNode<'a>,
    /// Optional graph component (None for default graph)
    pub graph: Option<VarOrNode<'a>>,
}

impl<'a> QuadPattern<'a> {
    /// Create a new quad pattern
    pub fn new(
        subject: VarOrNode<'a>,
        predicate: VarOrNode<'a>,
        object: VarOrNode<'a>,
        graph: Option<VarOrNode<'a>>,
    ) -> Self {
        Self {
            subject,
            predicate,
            object,
            graph,
        }
    }

    /// Convert to triple pattern (ignoring graph)
    pub fn as_triple(&self) -> TriplePattern<'a> {
        TriplePattern {
            subject: self.subject.clone(),
            predicate: self.predicate.clone(),
            object: self.object.clone(),
        }
    }

    /// Check if this is a concrete quad (no variables)
    pub fn is_concrete(&self) -> bool {
        matches!(self.subject, VarOrNode::Node(_))
            && matches!(self.predicate, VarOrNode::Node(_))
            && matches!(self.object, VarOrNode::Node(_))
            && self
                .graph
                .as_ref()
                .map_or(true, |g| matches!(g, VarOrNode::Node(_)))
    }
}

/// Graph target for CLEAR/DROP operations
#[derive(Debug, Clone, PartialEq)]
pub enum GraphTarget<'a> {
    /// Specific named graph
    Named(Node<'a>),
    /// Default graph
    Default,
    /// All named graphs (NAMED keyword)
    Named_,
    /// All graphs - both default and named (ALL keyword)
    All,
}

/// Projection specification
#[derive(Debug, Clone, PartialEq)]
pub enum Projection<'a> {
    /// SELECT * - all variables
    All,
    /// SELECT ?var1 ?var2 ... - specific variables
    Variables(Vec<Variable<'a>>),
    /// SELECT (expr AS ?var) - computed expressions
    Expressions(Vec<(Expression<'a>, Variable<'a>)>),
}

/// Dataset specification
#[derive(Debug, Clone, PartialEq)]
pub struct Dataset<'a> {
    /// Default graph IRIs (FROM clause)
    pub default: Vec<&'a str>,
    /// Named graph IRIs (FROM NAMED clause)
    pub named: Vec<&'a str>,
}

impl<'a> Default for Dataset<'a> {
    fn default() -> Self {
        Self {
            default: Vec::new(),
            named: Vec::new(),
        }
    }
}

/// Visitor pattern for traversing algebra
pub trait AlgebraVisitor<'a> {
    /// Output type produced by visitor methods
    type Output;

    /// Visits a basic graph pattern
    fn visit_bgp(&mut self, patterns: &[TriplePattern<'a>]) -> Self::Output;
    /// Visits a join operator
    fn visit_join(&mut self, left: &Algebra<'a>, right: &Algebra<'a>) -> Self::Output;
    /// Visits a left join (OPTIONAL) operator
    fn visit_left_join(&mut self, left: &Algebra<'a>, right: &Algebra<'a>, expr: &Option<Expression<'a>>) -> Self::Output;
    /// Visits a filter operator
    fn visit_filter(&mut self, expr: &Expression<'a>, input: &Algebra<'a>) -> Self::Output;
    /// Visits a union operator
    fn visit_union(&mut self, left: &Algebra<'a>, right: &Algebra<'a>) -> Self::Output;
    /// Visits a minus operator
    fn visit_minus(&mut self, left: &Algebra<'a>, right: &Algebra<'a>) -> Self::Output;
    /// Visits a graph operator
    fn visit_graph(&mut self, graph: &VarOrNode<'a>, input: &Algebra<'a>) -> Self::Output;
    /// Visits a service operator
    fn visit_service(&mut self, endpoint: &VarOrNode<'a>, input: &Algebra<'a>, silent: bool) -> Self::Output;
    /// Visits an extend operator
    fn visit_extend(&mut self, var: &Variable<'a>, expr: &Expression<'a>, input: &Algebra<'a>) -> Self::Output;
    /// Visits a project operator
    fn visit_project(&mut self, vars: &[Variable<'a>], input: &Algebra<'a>) -> Self::Output;
    /// Visits a distinct operator
    fn visit_distinct(&mut self, input: &Algebra<'a>) -> Self::Output;
    /// Visits a reduced operator
    fn visit_reduced(&mut self, input: &Algebra<'a>) -> Self::Output;
    /// Visits an order by operator
    fn visit_order_by(&mut self, conditions: &[OrderCondition<'a>], input: &Algebra<'a>) -> Self::Output;
    /// Visits a slice operator (LIMIT/OFFSET)
    fn visit_slice(&mut self, start: Option<usize>, length: Option<usize>, input: &Algebra<'a>) -> Self::Output;
    /// Visits a group operator
    fn visit_group(&mut self, vars: &[Variable<'a>], aggregates: &[(Variable<'a>, Aggregate<'a>)], input: &Algebra<'a>) -> Self::Output;
    /// Visits a table operator (VALUES)
    fn visit_table(&mut self, vars: &[Variable<'a>], rows: &[Vec<Option<Node<'a>>>]) -> Self::Output;
    /// Visits a property path operator
    fn visit_path(&mut self, subject: &VarOrNode<'a>, path: &PropertyPath<'a>, object: &VarOrNode<'a>) -> Self::Output;
}

impl<'a> Algebra<'a> {
    /// Accept a visitor
    pub fn accept<V: AlgebraVisitor<'a>>(&self, visitor: &mut V) -> V::Output {
        match self {
            Algebra::BGP(patterns) => visitor.visit_bgp(patterns),
            Algebra::Join { left, right } => visitor.visit_join(left, right),
            Algebra::LeftJoin { left, right, expr } => visitor.visit_left_join(left, right, expr),
            Algebra::Filter { expr, input } => visitor.visit_filter(expr, input),
            Algebra::Union { left, right } => visitor.visit_union(left, right),
            Algebra::Minus { left, right } => visitor.visit_minus(left, right),
            Algebra::Graph { graph, input } => visitor.visit_graph(graph, input),
            Algebra::Service { endpoint, input, silent } => visitor.visit_service(endpoint, input, *silent),
            Algebra::Extend { var, expr, input } => visitor.visit_extend(var, expr, input),
            Algebra::Project { vars, input } => visitor.visit_project(vars, input),
            Algebra::Distinct { input } => visitor.visit_distinct(input),
            Algebra::Reduced { input } => visitor.visit_reduced(input),
            Algebra::OrderBy { conditions, input } => visitor.visit_order_by(conditions, input),
            Algebra::Slice { start, length, input } => visitor.visit_slice(*start, *length, input),
            Algebra::Group { vars, aggregates, input } => visitor.visit_group(vars, aggregates, input),
            Algebra::Table { vars, rows } => visitor.visit_table(vars, rows),
            Algebra::Path { subject, path, object } => visitor.visit_path(subject, path, object),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_creation() {
        let var = Variable::new("x");
        assert_eq!(var.name, "x");
        assert_eq!(format!("{}", var), "?x");
    }

    #[test]
    fn test_triple_pattern() {
        let s = VarOrNode::Var(Variable::new("s"));
        let p = VarOrNode::Var(Variable::new("p"));
        let o = VarOrNode::Var(Variable::new("o"));

        let pattern = TriplePattern {
            subject: s,
            predicate: p,
            object: o,
        };

        assert!(matches!(pattern.subject, VarOrNode::Var(_)));
    }

    #[test]
    fn test_bgp_algebra() {
        let pattern = TriplePattern {
            subject: VarOrNode::Var(Variable::new("s")),
            predicate: VarOrNode::Var(Variable::new("p")),
            object: VarOrNode::Var(Variable::new("o")),
        };

        let bgp = Algebra::BGP(vec![pattern]);

        assert!(matches!(bgp, Algebra::BGP(_)));
    }
}
