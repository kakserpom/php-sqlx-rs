//! Query result iterator macro for php-sqlx.
//!
//! This module provides the [`php_sqlx_impl_query_result!`] macro that generates
//! a lazy, streaming query result iterator for each database type.
//!
//! Rows are fetched from the database on demand through a channel, with the
//! database streaming happening in a background task. This provides true lazy
//! loading without needing to fetch all rows upfront.

/// Default batch size for lazy row fetching (channel buffer size).
pub const DEFAULT_BATCH_SIZE: usize = 100;

/// Generates a database-specific `QueryResult` implementation with lazy streaming.
///
/// The generated `QueryResult` implements PHP's `Iterator` interface,
/// receiving rows from a background streaming task as iteration progresses.
///
/// # Arguments
///
/// - `$struct` - The Rust struct name for the query result (e.g., `PgQueryResult`)
/// - `$class` - The PHP class name as a string literal (e.g., `"Sqlx\\PgQueryResult"`)
/// - `$database` - The `SQLx` database type (e.g., `Postgres`)
/// - `$inner` - The inner driver type
#[macro_export]
macro_rules! php_sqlx_impl_query_result {
    ( $struct:ident, $class:literal, $database:ident, $inner:ident ) => {
        use ext_php_rs::prelude::*;
        use ext_php_rs::types::Zval;
        use ext_php_rs::zend::ce;
        use saturating_cast::SaturatingCast;

        /// A lazy query result iterator that streams rows on demand.
        ///
        /// This class implements PHP's `Iterator` interface, allowing it to be used
        /// with `foreach` loops. Rows are streamed from the database through a
        /// channel as you iterate, providing true lazy loading.
        ///
        /// # Example
        /// ```php
        /// $result = $driver->query('SELECT * FROM large_table');
        /// foreach ($result as $row) {
        ///     // Rows are fetched on demand as you iterate
        ///     process($row);
        /// }
        /// ```
        #[php_class]
        #[php(name = $class)]
        #[php(implements(ce = ce::iterator, stub = "\\Iterator"))]
        pub struct $struct {
            /// Channel receiver for streaming rows from background task
            receiver: tokio::sync::mpsc::Receiver<
                Result<
                    <sqlx_oldapi::$database as sqlx_oldapi::Database>::Row,
                    $crate::error::Error,
                >,
            >,
            /// Current row (converted to Zval)
            current: Option<Zval>,
            /// Current index (0-based)
            index: i64,
            /// Whether the stream has been exhausted
            exhausted: bool,
            /// Total rows fetched so far
            total_fetched: usize,
            /// Whether to use associative arrays for row conversion
            associative_arrays: bool,
            /// Whether we've done the initial fetch (rewind called)
            initialized: bool,
            /// Last error encountered (if any)
            last_error: Option<$crate::error::Error>,
            /// Configured buffer size for streaming
            buffer_size: usize,
        }

        impl $struct {
            /// Creates a new streaming `QueryResult`.
            #[allow(dead_code)]
            pub fn new(
                receiver: tokio::sync::mpsc::Receiver<
                    Result<
                        <sqlx_oldapi::$database as sqlx_oldapi::Database>::Row,
                        $crate::error::Error,
                    >,
                >,
                associative_arrays: bool,
                buffer_size: usize,
            ) -> Self {
                Self {
                    receiver,
                    current: None,
                    index: -1,
                    exhausted: false,
                    total_fetched: 0,
                    associative_arrays,
                    initialized: false,
                    last_error: None,
                    buffer_size,
                }
            }

            /// Fetches the next row from the channel.
            fn fetch_next(&mut self) {
                use $crate::conversion::Conversion;

                if self.exhausted {
                    self.current = None;
                    return;
                }

                // Try to receive the next row from the channel
                match self.receiver.blocking_recv() {
                    Some(Ok(row)) => {
                        // Convert the row to Zval
                        match row.into_zval(self.associative_arrays) {
                            Ok(zval) => {
                                self.current = Some(zval);
                                self.total_fetched += 1;
                            }
                            Err(e) => {
                                self.last_error = Some(e);
                                self.current = None;
                                self.exhausted = true;
                            }
                        }
                    }
                    Some(Err(e)) => {
                        // Error from the streaming task
                        self.last_error = Some(e);
                        self.current = None;
                        self.exhausted = true;
                    }
                    None => {
                        // Channel closed - stream exhausted
                        self.current = None;
                        self.exhausted = true;
                    }
                }
            }
        }

        #[php_impl]
        impl $struct {
            /// Returns the current row.
            ///
            /// Returns the row at the current iterator position, or null if
            /// the position is invalid.
            pub fn current(&self) -> Zval {
                self.current
                    .as_ref()
                    .map(Zval::shallow_clone)
                    .unwrap_or_else(Zval::null)
            }

            /// Returns the current index (0-based position).
            pub fn key(&self) -> i64 {
                self.index
            }

            /// Advances the iterator to the next row.
            pub fn next(&mut self) {
                self.index += 1;
                self.fetch_next();
            }

            /// Resets the iterator to the beginning.
            ///
            /// On first call, fetches the first row.
            /// Note: The stream cannot be truly rewound - this only works
            /// before any iteration has occurred.
            pub fn rewind(&mut self) {
                if !self.initialized {
                    self.initialized = true;
                    self.index = 0;
                    self.fetch_next();
                }
            }

            /// Returns true if the current position is valid.
            pub fn valid(&self) -> bool {
                self.current.is_some()
            }

            /// Returns the number of rows fetched so far.
            ///
            /// Note: This returns the count of rows fetched, not the total
            /// result set size (which may not be known until iteration completes).
            pub fn count(&self) -> i64 {
                self.total_fetched.saturating_cast::<i64>()
            }

            /// Returns true if the result set has been fully consumed.
            pub fn is_exhausted(&self) -> bool {
                self.exhausted
            }

            /// Returns the configured buffer size for streaming.
            pub fn get_batch_size(&self) -> i64 {
                self.buffer_size.saturating_cast::<i64>()
            }

            /// Consumes all remaining rows and returns them as an array.
            ///
            /// This will fetch all remaining rows from the stream.
            /// Use with caution on large result sets.
            #[allow(clippy::wrong_self_convention)]
            pub fn to_array(&mut self) -> $crate::error::Result<Vec<Zval>> {
                let mut all_rows = Vec::new();

                // Add current row if valid
                if let Some(ref current) = self.current {
                    all_rows.push(current.shallow_clone());
                }

                // Fetch remaining rows
                while !self.exhausted {
                    self.fetch_next();
                    if let Some(ref row) = self.current {
                        all_rows.push(row.shallow_clone());
                    }
                }

                // Return error if one occurred
                if let Some(err) = self.last_error.take() {
                    return Err(err);
                }

                Ok(all_rows)
            }

            /// Returns the last error that occurred, if any.
            ///
            /// This is useful for checking if iteration stopped due to an error.
            pub fn get_last_error(&self) -> Option<String> {
                self.last_error.as_ref().map(|e| e.to_string())
            }
        }
    };

    ( $( $t:tt )* ) => {
        compile_error!(
            "php_sqlx_impl_query_result! accepts 4 arguments: \
             ($struct, $class, $database, $inner)"
        );
    };
}
