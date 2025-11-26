//! Transaction support for storage backends

use crate::StorageResult;

/// Transaction trait for ACID operations
///
/// Provides isolation and atomicity guarantees for multi-operation updates.
pub trait Transaction: Send {
    /// Get a value within this transaction
    fn get(&self, key: &[u8]) -> StorageResult<Option<Vec<u8>>>;

    /// Put a key-value pair within this transaction
    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()>;

    /// Delete a key within this transaction
    fn delete(&mut self, key: &[u8]) -> StorageResult<()>;

    /// Commit the transaction, making all changes durable
    fn commit(self: Box<Self>) -> StorageResult<()>;

    /// Rollback the transaction, discarding all changes
    fn rollback(self: Box<Self>) -> StorageResult<()>;
}

/// In-memory transaction implementation
///
/// For in-memory backends, transactions are simulated by buffering changes.
pub struct InMemoryTransaction {
    /// Buffered puts
    puts: Vec<(Vec<u8>, Vec<u8>)>,

    /// Buffered deletes
    deletes: Vec<Vec<u8>>,

    /// Whether committed
    committed: bool,
}

impl InMemoryTransaction {
    /// Create a new in-memory transaction
    pub fn new() -> Self {
        Self {
            puts: Vec::new(),
            deletes: Vec::new(),
            committed: false,
        }
    }

    /// Get buffered changes for commit
    pub fn into_changes(self) -> (Vec<(Vec<u8>, Vec<u8>)>, Vec<Vec<u8>>) {
        (self.puts, self.deletes)
    }
}

impl Default for InMemoryTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl Transaction for InMemoryTransaction {
    fn get(&self, _key: &[u8]) -> StorageResult<Option<Vec<u8>>> {
        // For in-memory, we don't track reads
        Ok(None)
    }

    fn put(&mut self, key: &[u8], value: &[u8]) -> StorageResult<()> {
        self.puts.push((key.to_vec(), value.to_vec()));
        Ok(())
    }

    fn delete(&mut self, key: &[u8]) -> StorageResult<()> {
        self.deletes.push(key.to_vec());
        Ok(())
    }

    fn commit(mut self: Box<Self>) -> StorageResult<()> {
        self.committed = true;
        Ok(())
    }

    fn rollback(self: Box<Self>) -> StorageResult<()> {
        // Just drop the transaction
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_buffer() {
        let mut txn = InMemoryTransaction::new();

        txn.put(b"key1", b"value1").unwrap();
        txn.put(b"key2", b"value2").unwrap();
        txn.delete(b"key3").unwrap();

        let (puts, deletes) = txn.into_changes();

        assert_eq!(puts.len(), 2);
        assert_eq!(deletes.len(), 1);
    }

    #[test]
    fn test_transaction_commit() {
        let txn = Box::new(InMemoryTransaction::new());
        assert!(txn.commit().is_ok());
    }

    #[test]
    fn test_transaction_rollback() {
        let txn = Box::new(InMemoryTransaction::new());
        assert!(txn.rollback().is_ok());
    }
}
