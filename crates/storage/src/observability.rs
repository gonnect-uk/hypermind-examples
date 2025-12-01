//! Production Observability - Logging and Metrics
//!
//! Provides enterprise-grade observability for triple store operations:
//! - **Structured logging** with `tracing` (context-aware, filterable)
//! - **Metrics collection** with `metrics` (counters, histograms, gauges)
//! - **Performance tracking** (operation latency, throughput)
//! - **Error monitoring** (error rates, types, stack traces)
//! - **Health checks** (system health, resource usage)
//!
//! # Usage
//!
//! ```rust
//! use storage::{track_operation, record_error, OperationType};
//!
//! // Track operation with automatic timing
//! let result = track_operation(OperationType::Put, || -> Result<String, String> {
//!     // Your operation here
//!     Ok("success".to_string())
//! });
//! assert!(result.is_ok());
//!
//! // Record error
//! let error = "test error";
//! record_error(OperationType::Get, &error);
//! ```

use std::time::Instant;
use metrics::{counter, histogram, gauge};
use tracing::{error, warn, info, debug, trace, instrument};

/// Operation type for metrics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationType {
    /// Single get operation
    Get,
    /// Single put operation
    Put,
    /// Batch put operation
    BatchPut,
    /// Delete operation
    Delete,
    /// Range scan operation
    RangeScan,
    /// Prefix scan operation
    PrefixScan,
    /// Contains check
    Contains,
    /// Transaction commit
    TransactionCommit,
    /// Transaction rollback
    TransactionRollback,
}

impl OperationType {
    /// Get metric name for this operation
    pub fn metric_name(&self) -> &'static str {
        match self {
            OperationType::Get => "storage.get",
            OperationType::Put => "storage.put",
            OperationType::BatchPut => "storage.batch_put",
            OperationType::Delete => "storage.delete",
            OperationType::RangeScan => "storage.range_scan",
            OperationType::PrefixScan => "storage.prefix_scan",
            OperationType::Contains => "storage.contains",
            OperationType::TransactionCommit => "storage.transaction.commit",
            OperationType::TransactionRollback => "storage.transaction.rollback",
        }
    }

    /// Get counter name for this operation
    pub fn counter_name(&self) -> String {
        format!("{}.count", self.metric_name())
    }

    /// Get latency histogram name for this operation
    pub fn latency_name(&self) -> String {
        format!("{}.latency_ms", self.metric_name())
    }

    /// Get error counter name for this operation
    pub fn error_name(&self) -> String {
        format!("{}.errors", self.metric_name())
    }
}

/// Track an operation with automatic timing and metrics
///
/// Records:
/// - Operation counter
/// - Latency histogram
/// - Success/error tracking
/// - Structured logs
#[instrument(level = "debug", skip(f))]
pub fn track_operation<F, T, E>(op_type: OperationType, f: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: std::fmt::Debug,
{
    let start = Instant::now();

    // Get metric names (owned strings for 'static lifetime requirement)
    let counter_name = op_type.counter_name();
    let latency_name = op_type.latency_name();
    let error_name = op_type.error_name();

    // Increment operation counter
    counter!(counter_name.clone()).increment(1);

    // Execute operation
    let result = f();

    // Record latency
    let duration_ms = start.elapsed().as_micros() as f64 / 1000.0;
    histogram!(latency_name.clone()).record(duration_ms);

    // Record result
    match &result {
        Ok(_) => {
            debug!(
                op = ?op_type,
                latency_ms = duration_ms,
                "Operation completed successfully"
            );
        }
        Err(e) => {
            counter!(error_name.clone()).increment(1);
            error!(
                op = ?op_type,
                latency_ms = duration_ms,
                error = ?e,
                "Operation failed"
            );
        }
    }

    result
}

/// Record an error for monitoring
#[instrument(level = "error")]
pub fn record_error(op_type: OperationType, error: &dyn std::fmt::Debug) {
    let error_name = op_type.error_name();
    counter!(error_name.clone()).increment(1);
    error!(op = ?op_type, error = ?error, "Operation error recorded");
}

/// Record batch operation metrics
///
/// For operations that process multiple items (e.g., bulk insert).
#[instrument(level = "info", skip(items))]
pub fn track_batch<T>(op_type: OperationType, items: &[T]) {
    let count = items.len() as u64;

    counter!(format!("{}.items", op_type.metric_name())).increment(count);
    histogram!(format!("{}.batch_size", op_type.metric_name())).record(count as f64);

    info!(
        op = ?op_type,
        batch_size = count,
        "Batch operation tracked"
    );
}

/// Record throughput metric
///
/// For operations with measurable throughput (items/sec, bytes/sec).
pub fn record_throughput(op_type: OperationType, items_per_sec: f64) {
    gauge!(format!("{}.throughput", op_type.metric_name())).set(items_per_sec);

    debug!(
        op = ?op_type,
        throughput = items_per_sec,
        "Throughput recorded"
    );
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Is system healthy?
    pub healthy: bool,
    /// Total operations processed
    pub total_operations: u64,
    /// Total errors
    pub total_errors: u64,
    /// Error rate (errors / operations)
    pub error_rate: f64,
    /// Average latency (ms)
    pub avg_latency_ms: f64,
}

impl HealthStatus {
    /// Create a new health status
    pub fn new(
        total_operations: u64,
        total_errors: u64,
        avg_latency_ms: f64,
    ) -> Self {
        let error_rate = if total_operations > 0 {
            total_errors as f64 / total_operations as f64
        } else {
            0.0
        };

        let healthy = error_rate < 0.05 && avg_latency_ms < 1000.0;

        Self {
            healthy,
            total_operations,
            total_errors,
            error_rate,
            avg_latency_ms,
        }
    }

    /// Check if system is healthy
    pub fn is_healthy(&self) -> bool {
        self.healthy
    }

    /// Get status message
    pub fn status_message(&self) -> String {
        if self.healthy {
            format!(
                "Healthy: {:.2}% error rate, {:.2}ms avg latency",
                self.error_rate * 100.0,
                self.avg_latency_ms
            )
        } else {
            format!(
                "Unhealthy: {:.2}% error rate (>5%), {:.2}ms avg latency (>1000ms)",
                self.error_rate * 100.0,
                self.avg_latency_ms
            )
        }
    }
}

/// Performance metrics snapshot
#[derive(Debug, Clone, Default)]
pub struct PerformanceMetrics {
    /// Total GET operations
    pub get_count: u64,
    /// Total PUT operations
    pub put_count: u64,
    /// Total DELETE operations
    pub delete_count: u64,
    /// Total SCAN operations
    pub scan_count: u64,
    /// Total errors
    pub error_count: u64,
    /// Average GET latency (microseconds)
    pub avg_get_latency_us: f64,
    /// Average PUT latency (microseconds)
    pub avg_put_latency_us: f64,
    /// Average SCAN latency (milliseconds)
    pub avg_scan_latency_ms: f64,
    /// Throughput (operations/sec)
    pub throughput: f64,
}

impl PerformanceMetrics {
    /// Create metrics snapshot
    pub fn snapshot() -> Self {
        // In production, these would be read from metrics registry
        // For now, return defaults
        Self::default()
    }

    /// Get health status from metrics
    pub fn health_status(&self) -> HealthStatus {
        let total_ops = self.get_count + self.put_count + self.delete_count + self.scan_count;
        let avg_latency = if total_ops > 0 {
            (self.avg_get_latency_us + self.avg_put_latency_us) / 2.0 / 1000.0
        } else {
            0.0
        };

        HealthStatus::new(total_ops, self.error_count, avg_latency)
    }

    /// Print summary report
    pub fn summary(&self) -> String {
        format!(
            "Performance Summary:\n\
             - GET: {} ops ({:.2} µs avg)\n\
             - PUT: {} ops ({:.2} µs avg)\n\
             - DELETE: {} ops\n\
             - SCAN: {} ops ({:.2} ms avg)\n\
             - Errors: {}\n\
             - Throughput: {:.2} ops/sec",
            self.get_count,
            self.avg_get_latency_us,
            self.put_count,
            self.avg_put_latency_us,
            self.delete_count,
            self.scan_count,
            self.avg_scan_latency_ms,
            self.error_count,
            self.throughput
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_type_metrics() {
        assert_eq!(OperationType::Get.metric_name(), "storage.get");
        assert_eq!(OperationType::Put.counter_name(), "storage.put.count");
        assert_eq!(OperationType::Get.latency_name(), "storage.get.latency_ms");
        assert_eq!(OperationType::Delete.error_name(), "storage.delete.errors");
    }

    #[test]
    fn test_track_operation_success() {
        let result = track_operation(OperationType::Get, || -> Result<String, String> {
            Ok("success".to_string())
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_track_operation_error() {
        let result = track_operation(OperationType::Put, || -> Result<(), String> {
            Err("error".to_string())
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_health_status_healthy() {
        let status = HealthStatus::new(1000, 10, 50.0);
        assert!(status.is_healthy());
        assert_eq!(status.error_rate, 0.01); // 1%
    }

    #[test]
    fn test_health_status_unhealthy_error_rate() {
        let status = HealthStatus::new(1000, 60, 50.0);
        assert!(!status.is_healthy());
        assert_eq!(status.error_rate, 0.06); // 6% > 5% threshold
    }

    #[test]
    fn test_health_status_unhealthy_latency() {
        let status = HealthStatus::new(1000, 10, 1500.0);
        assert!(!status.is_healthy()); // Latency > 1000ms threshold
    }

    #[test]
    fn test_performance_metrics_summary() {
        let metrics = PerformanceMetrics {
            get_count: 1000,
            put_count: 500,
            delete_count: 100,
            scan_count: 50,
            error_count: 5,
            avg_get_latency_us: 2.78,
            avg_put_latency_us: 10.0,
            avg_scan_latency_ms: 5.0,
            throughput: 500.0,
        };

        let summary = metrics.summary();
        assert!(summary.contains("GET: 1000 ops"));
        assert!(summary.contains("2.78 µs"));
        assert!(summary.contains("Throughput: 500.00 ops/sec"));
    }

    #[test]
    fn test_performance_metrics_health() {
        let metrics = PerformanceMetrics {
            get_count: 1000,
            put_count: 500,
            error_count: 10,
            avg_get_latency_us: 2.78,
            avg_put_latency_us: 10.0,
            ..Default::default()
        };

        let health = metrics.health_status();
        assert!(health.is_healthy());
        assert_eq!(health.total_operations, 1500);
        assert_eq!(health.total_errors, 10);
    }
}
