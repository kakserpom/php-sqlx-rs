#[macro_export]
macro_rules! php_sqlx_impl_prepared_query {
    ( $struct:ident, $class:literal, $driver_inner: ident ) => {
        use ext_php_rs::php_impl;
        use ext_php_rs::prelude::*;
        use ext_php_rs::types::Zval;
        use std::collections::BTreeMap;
        use std::sync::Arc;
        use $crate::interfaces::PreparedQueryInterface;
        use $crate::param_value::ParameterValue;
        use $crate::utils::types::ColumnArgument;

        /// A reusable prepared SQL query with parameter support. Created using `PgDriver::prepare()`, shares context with original driver.
        #[php_class]
        #[php(name = $class)]
        pub struct $struct {
            pub(crate) query: String,
            pub(crate) driver_inner: Arc<$driver_inner>,
        }

        #[php_impl_interface]
        impl $crate::interfaces::PreparedQueryInterface for $struct {
            /// Executes the prepared statement and returns affected rows.
            fn execute(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<u64> {
                self.driver_inner.execute(self.query.as_str(), parameters)
            }

            /// Executes the prepared query and returns a single row.
            fn query_row(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_row(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns a single row as associative array.
            fn query_row_assoc(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_row(&self.query, parameters, Some(true))
            }

            /// Executes the prepared query and returns a single row as object.
            fn query_row_obj(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_row(&self.query, parameters, Some(false))
            }

            /// Executes the prepared query and returns a single row or null.
            fn query_maybe_row(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_maybe_row(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns all rows.
            fn query_all(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner.query_all(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns all rows as associative arrays.
            fn query_all_assoc(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner
                    .query_all(&self.query, parameters, Some(true))
            }

            /// Executes the prepared query and returns all rows as objects.
            fn query_all_obj(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner
                    .query_all(&self.query, parameters, Some(false))
            }

            /// Executes the prepared query and returns a single column value.
            fn query_value(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_value(&self.query, parameters, column, None)
            }

            /// Executes the prepared query and returns a column from all rows.
            fn query_column(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> $crate::error::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(&self.query, parameters, column, None)
            }
        }

        impl $struct {
            pub fn new(query: &str, driver_inner: Arc<$driver_inner>) -> Self {
                Self {
                    query: query.to_owned(),
                    driver_inner,
                }
            }
        }
        #[php_impl]
        impl $struct {
            /// Executes the prepared query and returns a dictionary mapping the first column to the second column.
            ///
            /// This method expects each result row to contain at least two columns. It converts the first column
            /// into a PHP string (used as the key), and the second column into a PHP value (used as the value).
            ///
            /// # Parameters
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// An associative array (`array<string, mixed>`) where each key is the first column (as string),
            /// and the value is the second column.
            ///
            /// # Errors
            /// Returns an error if:
            /// - the query fails to execute;
            /// - the first column cannot be converted to a PHP string;
            /// - the second column cannot be converted to a PHP value.
            ///
            /// # Notes
            /// - The order of dictionary entries is preserved.
            /// - The query must return at least two columns per row.
            pub fn query_column_dictionary(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_column_dictionary(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns a dictionary in associative array mode.
            ///
            /// Same as `query_column_dictionary`, but forces JSON objects to be represented as associative arrays.
            ///
            /// # Parameters
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// A dictionary where each key is the first column (as string),
            /// and each value is the second column as an associative PHP array.
            ///
            /// # Errors
            /// Same as `query_column_dictionary`.
            pub fn query_column_dictionary_assoc(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_column_dictionary(&self.query, parameters, Some(true))
            }

            /// Executes the prepared query and returns a dictionary in object mode.
            ///
            /// Same as `query_column_dictionary`, but forces JSON objects to be represented as objects.
            ///
            /// # Parameters
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// A dictionary where each key is the first column (as string),
            /// and each value is the second column as a PHP object.
            ///
            /// # Errors
            /// Same as `query_column_dictionary`.
            pub fn query_column_dictionary_obj(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_column_dictionary(&self.query, parameters, Some(false))
            }

            /// Executes the prepared query and returns a dictionary (map) indexed by the first column of each row.
            ///
            /// The result is a `HashMap` where the key is the stringified first column from each row,
            /// and the value is the full row, returned as array or object depending on config.
            ///
            /// # Parameters
            /// - `parameters`: Optional map of named parameters to bind.
            ///
            /// # Returns
            /// A map from the first column (as string) to the corresponding row.
            ///
            /// # Errors
            /// Returns an error if:
            /// - the query fails to execute;
            /// - the first column cannot be converted to a string;
            /// - any row cannot be decoded or converted to a PHP value.
            ///
            /// # Notes
            /// - The iteration order of the returned map is **not** guaranteed.
            pub fn query_dictionary(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_dictionary(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
            /// with each row returned as an associative array.
            ///
            /// # Parameters
            /// - `parameters`: Optional map of named parameters to bind.
            ///
            /// # Returns
            /// A map from the first column (as string) to the corresponding row as an associative array.
            ///
            /// # Errors
            /// Returns an error if:
            /// - the query fails to execute;
            /// - the first column cannot be converted to a string;
            /// - any row cannot be decoded or converted to a PHP value.
            ///
            /// # Notes
            /// - The iteration order of the returned map is **not** guaranteed.
            pub fn query_dictionary_assoc(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_dictionary(&self.query, parameters, Some(true))
            }

            /// Executes the prepared query and returns a dictionary (map) indexed by the first column of each row,
            /// with each row returned as an object (`stdClass`).
            ///
            /// # Parameters
            /// - `parameters`: Optional map of named parameters to bind.
            ///
            /// # Returns
            /// A map from the first column (as string) to the corresponding row as an object.
            ///
            /// # Errors
            /// Returns an error if:
            /// - the query fails to execute;
            /// - the first column cannot be converted to a string;
            /// - any row cannot be decoded or converted to a PHP value.
            ///
            /// # Notes
            /// - The iteration order of the returned map is **not** guaranteed.
            pub fn query_dictionary_obj(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_dictionary(&self.query, parameters, Some(false))
            }

            /// Executes a query and returns a grouped dictionary (Vec of rows per key).
            ///
            /// Same as [`queryGroupedDictionary`](crate::Driver::query_grouped_dictionary), but works on a prepared query.
            ///
            /// The first column is used as the key (must be scalar),
            /// and each resulting row is appended to the corresponding key's Vec.
            ///
            /// # Errors
            /// Fails if the query fails, or the first column is not scalar.
            pub fn query_grouped_dictionary(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(&self.query, parameters, None)
            }

            /// Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
            pub fn query_grouped_dictionary_assoc(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(&self.query, parameters, Some(true))
            }

            /// Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
            pub fn query_grouped_dictionary_obj(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(&self.query, parameters, Some(false))
            }

            /// Executes the prepared query and returns a grouped dictionary where:
            /// - the key is the **first column** (must be scalar),
            /// - the value is a list of values from the **second column** for each group.
            ///
            /// This variant uses the driver's default associative array option for JSON values.
            ///
            /// # Errors
            /// Returns an error if the first column is not convertible to a string.
            pub fn query_grouped_column_dictionary(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner
                    .query_grouped_column_dictionary(&self.query, parameters, None)
            }

            /// Same as `queryGroupedColumnDictionary()`, but forces associative arrays
            /// for the second column if it contains JSON objects.
            ///
            /// # Errors
            /// Returns an error if the first column is not convertible to a string.
            pub fn query_grouped_column_dictionary_assoc(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_grouped_column_dictionary(
                    &self.query,
                    parameters,
                    Some(true),
                )
            }

            /// Same as `queryGroupedColumnDictionary()`, but forces PHP objects
            /// for the second column if it contains JSON objects.
            ///
            /// # Errors
            /// Returns an error if the first column is not convertible to a string.
            pub fn query_grouped_column_dictionary_obj(
                &self,
                parameters: Option<BTreeMap<String, ParameterValue>>,
            ) -> $crate::error::Result<Zval> {
                self.driver_inner.query_grouped_column_dictionary(
                    &self.query,
                    parameters,
                    Some(false),
                )
            }
        }
    };
}
