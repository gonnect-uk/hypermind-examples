# Batch SPARQL Execution with Statement-Level Error Handling

**Date**: 2025-11-18
**Context**: Sync batch writes with precise error reporting
**Status**: Design document for future implementation

---

## Problem Statement

When executing multiple SPARQL UPDATE statements in a batch:

```sparql
-- Batch of 3 statements
INSERT DATA { <s1> <p1> <o1> } ;  # Statement 1 ✅
INSERT DATA { <s2> invalid } ;     # Statement 2 ❌ Parse error
INSERT DATA { <s3> <p3> <o3> } ;  # Statement 3 - should this execute?
```

**Requirements**:
1. **Sync batch writes** - Execute statements sequentially (not async)
2. **Statement-level errors** - Report EXACTLY which statement failed (e.g., "Statement 2 failed: Parse error")
3. **Transaction semantics** - Choose between:
   - **All-or-nothing** (rollback all on any failure)
   - **Best-effort** (execute as many as possible, report failures)

---

## Architecture Options

### Option 1: All-or-Nothing Transaction (ACID) ⭐ **Recommended**

```rust
pub struct BatchExecutor {
    store: Arc<QuadStore>,
}

pub struct BatchResult {
    success: bool,
    executed_count: usize,
    failed_statement: Option<StatementError>,
}

pub struct StatementError {
    statement_index: usize,     // Which statement failed (0-based)
    statement_text: String,     // SPARQL text
    error: ExecutionError,      // What went wrong
}

impl BatchExecutor {
    /// Execute batch of SPARQL UPDATE statements (all-or-nothing)
    pub fn execute_batch(&mut self, statements: &[&str]) -> Result<BatchResult, BatchError> {
        // 1. Parse all statements first (fail fast)
        let parsed: Vec<_> = statements.iter().enumerate()
            .map(|(i, sql)| {
                parse_sparql(sql).map_err(|e| StatementError {
                    statement_index: i,
                    statement_text: sql.to_string(),
                    error: ExecutionError::ParseError(e),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // 2. Begin transaction (snapshot current state)
        let snapshot = self.store.snapshot();

        // 3. Execute statements one by one
        for (i, update) in parsed.iter().enumerate() {
            if let Err(e) = self.execute_update(update) {
                // Rollback on error
                self.store.restore(snapshot);
                return Err(BatchError::StatementFailed(StatementError {
                    statement_index: i,
                    statement_text: statements[i].to_string(),
                    error: e,
                }));
            }
        }

        // 4. Commit (all succeeded)
        Ok(BatchResult {
            success: true,
            executed_count: statements.len(),
            failed_statement: None,
        })
    }
}
```

**Pros**:
- ✅ ACID guarantees (atomicity)
- ✅ No partial state (all or nothing)
- ✅ Simple error semantics

**Cons**:
- ⚠️ Requires snapshot/rollback infrastructure (not yet implemented)
- ⚠️ Memory overhead for large transactions

---

### Option 2: Best-Effort with Error Collection

```rust
pub struct BestEffortBatchResult {
    success_count: usize,
    failure_count: usize,
    errors: Vec<StatementError>,  // All failed statements
}

impl BatchExecutor {
    /// Execute batch, collect all errors, continue on failure
    pub fn execute_batch_best_effort(&mut self, statements: &[&str])
        -> BestEffortBatchResult {

        let mut result = BestEffortBatchResult {
            success_count: 0,
            failure_count: 0,
            errors: Vec::new(),
        };

        for (i, statement) in statements.iter().enumerate() {
            match self.execute_single(statement) {
                Ok(_) => {
                    result.success_count += 1;
                }
                Err(e) => {
                    result.failure_count += 1;
                    result.errors.push(StatementError {
                        statement_index: i,
                        statement_text: statement.to_string(),
                        error: e,
                    });
                }
            }
        }

        result
    }
}
```

**Pros**:
- ✅ Maximum progress (don't fail entire batch)
- ✅ Simple implementation (no rollback needed)
- ✅ Collect all errors at once (better DX)

**Cons**:
- ❌ Partial state (database left in inconsistent state)
- ❌ Not ACID compliant

---

### Option 3: Checkpoint-Based Rollback (Middle Ground)

```rust
pub struct CheckpointBatchExecutor {
    store: Arc<QuadStore>,
    checkpoints: Vec<DatasetCheckpoint>,
}

impl CheckpointBatchExecutor {
    /// Execute batch with checkpoint every N statements
    pub fn execute_batch_with_checkpoints(
        &mut self,
        statements: &[&str],
        checkpoint_interval: usize,  // e.g., every 100 statements
    ) -> Result<BatchResult, BatchError> {

        for (i, chunk) in statements.chunks(checkpoint_interval).enumerate() {
            // Save checkpoint before chunk
            let checkpoint = self.store.checkpoint();
            self.checkpoints.push(checkpoint);

            // Execute chunk
            for (j, statement) in chunk.iter().enumerate() {
                let global_index = i * checkpoint_interval + j;

                if let Err(e) = self.execute_single(statement) {
                    // Rollback to last checkpoint
                    self.store.restore(self.checkpoints.last().unwrap());
                    return Err(BatchError::StatementFailed(StatementError {
                        statement_index: global_index,
                        statement_text: statement.to_string(),
                        error: e,
                    }));
                }
            }
        }

        Ok(BatchResult { success: true, .. })
    }
}
```

**Pros**:
- ✅ Balance between ACID and performance
- ✅ Limited rollback scope

**Cons**:
- ⚠️ Complex implementation
- ⚠️ Partial ACID (within checkpoints only)

---

## Recommended Implementation

### Phase 1: Best-Effort (Simple, Immediate Value)

```rust
// crates/sparql/src/batch.rs
pub struct BatchExecutor<'a, B: StorageBackend> {
    executor: UpdateExecutor<'a, B>,
}

impl<'a, B: StorageBackend> BatchExecutor<'a, B> {
    pub fn new(executor: UpdateExecutor<'a, B>) -> Self {
        Self { executor }
    }

    /// Execute multiple SPARQL UPDATE statements
    /// Returns success count and detailed errors for failures
    pub fn execute_batch(&mut self, statements: &[String]) -> BatchResult {
        let mut result = BatchResult {
            total: statements.len(),
            success_count: 0,
            failures: Vec::new(),
        };

        for (index, statement) in statements.iter().enumerate() {
            match self.execute_single(statement) {
                Ok(_) => {
                    result.success_count += 1;
                }
                Err(error) => {
                    result.failures.push(StatementFailure {
                        index,
                        statement: statement.clone(),
                        error: format!("{:?}", error),
                        position: error.position(),  // Line/column if parse error
                    });
                }
            }
        }

        result
    }

    fn execute_single(&mut self, statement: &str) -> Result<(), ExecutionError> {
        // Parse
        let update = parse_sparql_update(statement)?;

        // Execute
        self.executor.execute_update(&update)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct BatchResult {
    pub total: usize,
    pub success_count: usize,
    pub failures: Vec<StatementFailure>,
}

#[derive(Debug, Clone)]
pub struct StatementFailure {
    pub index: usize,          // 0-based statement index
    pub statement: String,     // SPARQL text
    pub error: String,         // Error message
    pub position: Option<(usize, usize)>,  // (line, column) if parse error
}

impl BatchResult {
    pub fn is_success(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn success_rate(&self) -> f64 {
        self.success_count as f64 / self.total as f64
    }
}
```

### Usage Example

```rust
let statements = vec![
    "INSERT DATA { <s1> <p1> <o1> }".to_string(),
    "INSERT DATA { <s2> invalid }".to_string(),    // ❌ Will fail
    "INSERT DATA { <s3> <p3> <o3> }".to_string(),
];

let mut batch_executor = BatchExecutor::new(update_executor);
let result = batch_executor.execute_batch(&statements);

if !result.is_success() {
    for failure in &result.failures {
        eprintln!(
            "Statement {} failed: {}\n  SPARQL: {}\n  Error: {}",
            failure.index,
            failure.position.map(|(l, c)| format!("at line {}, column {}", l, c))
                .unwrap_or_default(),
            failure.statement,
            failure.error
        );
    }
}

// Output:
// Statement 1 failed: at line 1, column 20
//   SPARQL: INSERT DATA { <s2> invalid }
//   Error: ParseError: Expected predicate, found 'invalid'
```

---

### Phase 2: Add Transaction Support (Future)

When implementing full ACID transactions:

```rust
pub trait TransactionalStore {
    type Snapshot;

    fn begin_transaction(&mut self) -> Self::Snapshot;
    fn commit(&mut self);
    fn rollback(&mut self, snapshot: Self::Snapshot);
}

impl BatchExecutor<'a, B: StorageBackend + TransactionalStore> {
    /// Execute batch with ACID guarantees (all-or-nothing)
    pub fn execute_batch_transactional(&mut self, statements: &[String])
        -> Result<BatchResult, BatchError> {

        // Begin transaction
        let snapshot = self.executor.store.begin_transaction();

        // Execute all statements
        for (index, statement) in statements.iter().enumerate() {
            if let Err(error) = self.execute_single(statement) {
                // Rollback on first error
                self.executor.store.rollback(snapshot);
                return Err(BatchError::StatementFailed {
                    index,
                    statement: statement.clone(),
                    error,
                });
            }
        }

        // Commit if all succeeded
        self.executor.store.commit();
        Ok(BatchResult {
            total: statements.len(),
            success_count: statements.len(),
            failures: Vec::new(),
        })
    }
}
```

---

## Error Reporting Format

### JSON API Response

```json
{
  "success": false,
  "total": 3,
  "executed": 1,
  "failures": [
    {
      "index": 1,
      "statement": "INSERT DATA { <s2> invalid }",
      "error": {
        "type": "ParseError",
        "message": "Expected predicate, found 'invalid'",
        "position": {
          "line": 1,
          "column": 20
        }
      }
    }
  ]
}
```

### CLI Output

```
Executing 3 SPARQL statements...
✅ Statement 0: Success
❌ Statement 1: ParseError at line 1, column 20
   INSERT DATA { <s2> invalid }
                      ^^^^^^^ Expected predicate
✅ Statement 2: Success

Summary: 2/3 succeeded (66.7%)
```

---

## Performance Considerations

### Batch Size Optimization

```rust
impl BatchExecutor {
    /// Recommend optimal batch size based on statement complexity
    pub fn recommend_batch_size(&self, statements: &[String]) -> usize {
        let avg_size = statements.iter()
            .map(|s| s.len())
            .sum::<usize>() / statements.len();

        match avg_size {
            0..=100 => 1000,      // Simple statements: large batches
            101..=1000 => 100,    // Medium statements: medium batches
            _ => 10,              // Complex statements: small batches
        }
    }

    /// Execute with auto-chunking
    pub fn execute_batch_auto(&mut self, statements: &[String]) -> BatchResult {
        let chunk_size = self.recommend_batch_size(statements);

        let mut total_result = BatchResult::default();

        for chunk in statements.chunks(chunk_size) {
            let result = self.execute_batch(chunk);
            total_result.merge(result);
        }

        total_result
    }
}
```

---

## Summary

**Recommended Approach**:
1. ✅ **Implement Phase 1 (Best-Effort)** - Simple, immediate value
2. ⏳ **Add Phase 2 (Transactional)** when needed - Full ACID guarantees

**Key Features**:
- ✅ Statement-level error reporting (index + error details)
- ✅ Sync batch execution (no async complexity)
- ✅ Position-aware errors (line/column for parse errors)
- ✅ JSON API friendly
- ✅ Performance optimizations (auto-chunking)

**Implementation Priority**:
- **Now**: Best-effort batch execution
- **Later**: Transaction support (when ACID required)

---

**Generated**: 2025-11-18
**Status**: Design document
**Next**: Implement Phase 1 (Best-Effort) if batch execution is a priority
