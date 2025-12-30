//! Query profiling hook for monitoring query execution.
//!
//! This module provides a callback mechanism for logging and profiling SQL queries.
//! When a hook is registered, it will be called after each query with:
//! - The rendered SQL query
//! - The bound parameters (as PHP array)
//! - The execution duration in milliseconds
//!
//! # Example
//!
//! ```php
//! $driver->onQuery(function(string $sql, array $params, float $durationMs) {
//!     Logger::debug("Query took {$durationMs}ms: $sql");
//! });
//! ```

use ext_php_rs::types::{ZendCallable, Zval};
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
    /// - `array $params` - The bound parameters
    /// - `float $durationMs` - Execution time in milliseconds
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

        // Call the hook, ignoring any errors
        let _ = callable.try_call(vec![&sql, &sql_inline, &duration]);
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
    pub fn finish(self) {
        let duration = self.start.elapsed();
        let duration_ms = duration.as_secs_f64() * 1000.0;

        self.hook.call(&QueryInfo {
            sql: self.sql,
            sql_inline: self.sql_inline,
            duration_ms,
        });
    }
}
