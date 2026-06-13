//! Query profiling hook for monitoring query execution.
//!
//! This module provides a callback mechanism for logging and profiling SQL queries.
//! When a hook is registered, it will be called after each query with:
//! - The rendered SQL query
//! - The SQL with inlined parameters (for logging)
//! - The execution duration in milliseconds
//! - The number of rows affected (writes) or returned (reads), if known
//! - The error message, if the query failed (`null` on success)
//!
//! # Example
//!
//! ```php
//! $driver->onQuery(function(
//!     string $sql,
//!     string $sqlInline,
//!     float $durationMs,
//!     ?int $rows,
//!     ?string $error,
//! ) {
//!     if ($error !== null) {
//!         Logger::error("Query failed after {$durationMs}ms: $error");
//!     } else {
//!         Logger::debug("Query returned {$rows} row(s) in {$durationMs}ms: $sql");
//!     }
//! });
//! ```

use ext_php_rs::types::{ZendCallable, Zval};
use saturating_cast::SaturatingCast;
use std::cell::RefCell;
use std::time::Instant;

/// Information about an executed query, passed to the hook callback.
#[derive(Debug)]
pub struct QueryInfo {
    /// The rendered SQL query with placeholders.
    pub sql: String,
    /// The rendered SQL with inlined parameters (for logging).
    pub sql_inline: Option<String>,
    /// Execution duration in milliseconds.
    pub duration_ms: f64,
    /// Number of rows affected (for writes) or returned (for reads).
    ///
    /// `None` when the count is not applicable or the query failed.
    pub rows: Option<u64>,
    /// Error message if the query failed; `None` on success.
    pub error: Option<String>,
}

/// Storage for the query hook callback.
///
/// Uses `RefCell` because PHP is single-threaded and we need interior mutability.
/// The `Zval` stores the PHP callable (closure, function name, or [object, method]).
#[derive(Default)]
pub struct QueryHook {
    callback: RefCell<Option<Zval>>,
}

// SAFETY: PHP is single-threaded per request. The QueryHook is only accessed
// from the same thread that created it.
unsafe impl Send for QueryHook {}
unsafe impl Sync for QueryHook {}

impl QueryHook {
    /// Creates a new empty query hook.
    #[must_use]
    pub fn new() -> Self {
        Self {
            callback: RefCell::new(None),
        }
    }

    /// Sets the query hook callback.
    ///
    /// The callback should be a PHP callable that accepts:
    /// - `string $sql` - The rendered SQL query
    /// - `string $sqlInline` - The SQL with inlined parameters
    /// - `float $durationMs` - Execution time in milliseconds
    /// - `?int $rows` - Rows affected/returned, or `null` if unknown
    /// - `?string $error` - Error message, or `null` on success
    pub fn set(&self, callback: Zval) {
        *self.callback.borrow_mut() = Some(callback);
    }

    /// Clears the query hook callback.
    pub fn clear(&self) {
        *self.callback.borrow_mut() = None;
    }

    /// Returns true if a hook is registered.
    #[must_use]
    pub fn is_set(&self) -> bool {
        self.callback.borrow().is_some()
    }

    /// Calls the hook with query information.
    ///
    /// This is a no-op if no hook is registered.
    /// Errors from the callback are silently ignored to avoid disrupting query execution.
    pub fn call(&self, info: &QueryInfo) {
        let callback = self.callback.borrow();
        let Some(ref zval) = *callback else {
            return;
        };

        let Ok(callable) = ZendCallable::new(zval) else {
            return;
        };

        // Prepare arguments as string slices
        let sql: &str = &info.sql;
        let sql_inline: &str = info.sql_inline.as_deref().unwrap_or(&info.sql);
        let duration = info.duration_ms;
        // PHP integers are signed 64-bit; saturate on the (practically impossible)
        // overflow rather than wrapping.
        let rows: Option<i64> = info.rows.map(SaturatingCast::saturating_cast);
        let error: Option<&str> = info.error.as_deref();

        // Call the hook, ignoring any errors
        let _ = callable.try_call(vec![&sql, &sql_inline, &duration, &rows, &error]);
    }
}

impl std::fmt::Debug for QueryHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryHook")
            .field("is_set", &self.is_set())
            .finish()
    }
}

/// A guard that measures query execution time and calls the hook on drop.
pub struct QueryTimer<'a> {
    hook: &'a QueryHook,
    sql: String,
    sql_inline: Option<String>,
    start: Instant,
}

impl<'a> QueryTimer<'a> {
    /// Creates a new query timer.
    ///
    /// Returns `None` if no hook is registered (avoids timing overhead).
    #[must_use]
    pub fn new(hook: &'a QueryHook, sql: String, sql_inline: Option<String>) -> Option<Self> {
        if hook.is_set() {
            Some(Self {
                hook,
                sql,
                sql_inline,
                start: Instant::now(),
            })
        } else {
            None
        }
    }

    /// Completes the timer and calls the hook.
    ///
    /// - `rows`: rows affected (writes) or returned (reads), if known.
    /// - `error`: the error message if the query failed, or `None` on success.
    pub fn finish(self, rows: Option<u64>, error: Option<&str>) {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        self.hook.call(&QueryInfo {
            sql: self.sql,
            sql_inline: self.sql_inline,
            duration_ms,
            rows,
            error: error.map(ToOwned::to_owned),
        });
    }
}
