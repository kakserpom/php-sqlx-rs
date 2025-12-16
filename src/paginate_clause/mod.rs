//! Pagination clause generation for php-sqlx.
//!
//! This module provides safe pagination that converts page numbers to SQL
//! LIMIT/OFFSET clauses with configurable constraints on items per page.
//!
//! # PHP Usage
//!
//! ```php
//! // Create paginator with defaults (1-20 items per page)
//! $paginate = new Sqlx\PaginateClause();
//!
//! // Configure limits
//! $paginate->minPerPage(5);   // At least 5 items
//! $paginate->maxPerPage(100); // At most 100 items
//!
//! // Apply user input (page 2, 25 items per page)
//! $rendered = $paginate->input(2, 25);
//! // Generates: LIMIT 25 OFFSET 50
//! ```
//!
//! # Safety
//!
//! The paginator clamps `per_page` to configured min/max values, preventing
//! users from requesting excessive result sets.

use crate::ast::Settings;
use crate::param_value::ParameterValue;
use anyhow::bail;
use ext_php_rs::{php_class, php_impl, prelude::ModuleBuilder};

/// Registers the `PaginateClause` and `PaginateClauseRendered` classes with the PHP module builder.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module
        .class::<PaginateClause>()
        .class::<PaginateClauseRendered>()
}

/// The `Sqlx\PaginateClause` class represents pagination settings
/// and provides methods to compute the appropriate SQL `LIMIT` and `OFFSET`
/// based on a given page number and items-per-page values.
#[php_class]
#[php(name = "Sqlx\\PaginateClause")]
#[allow(clippy::struct_field_names)]
pub struct PaginateClause {
    /// Minimum number of items allowed per page.
    pub(crate) min_per_page: i64,
    /// Maximum number of items allowed per page.
    pub(crate) max_per_page: i64,
    /// Default number of items per page when none is specified.
    pub(crate) default_per_page: i64,
}

impl Default for PaginateClause {
    /// Returns a `PaginateClause` initialized with default values:
    /// - `min_per_page = 1`
    /// - `max_per_page = 20`
    /// - `default_per_page = 20`
    fn default() -> Self {
        Self {
            min_per_page: 1,
            max_per_page: 20,
            default_per_page: 20,
        }
    }
}

impl PaginateClause {
    /// Creates a new `PaginateClause` with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Internal helper to calculate the `PaginateClauseRendered` result
    /// from optional `page_number` and `per_page` inputs.
    ///
    /// # Parameters
    /// - `page_number`: Optional zero-based page index. Defaults to 0 if `None`.
    /// - `per_page`: Optional number of items per page. Defaults to `default_per_page` if `None`.
    ///
    /// # Returns
    /// A `PaginateClauseRendered` containing clamped `limit` and computed `offset`.
    #[must_use]
    fn render(&self, page_number: Option<i64>, per_page: Option<i64>) -> PaginateClauseRendered {
        let per_page = per_page
            .unwrap_or(self.default_per_page)
            .clamp(self.min_per_page, self.max_per_page);
        PaginateClauseRendered {
            limit: per_page,
            offset: page_number.unwrap_or(0) * per_page,
        }
    }
}

#[php_impl]
impl PaginateClause {
    /// PHP constructor for `Sqlx\PaginateClause`.
    pub fn __construct() -> Self {
        Self::default()
    }

    /// Magic `__invoke` method allowing the object to be used as a callable
    /// for applying pagination.
    ///
    /// # Parameters
    /// - `page_number`: Optional page index.
    /// - `per_page`: Optional items per page.
    ///
    /// # Returns
    /// A `PaginateClauseRendered` with calculated `limit` and `offset`.
    #[must_use]
    pub fn __invoke(
        &self,
        page_number: Option<i64>,
        per_page: Option<i64>,
    ) -> PaginateClauseRendered {
        self.render(page_number, per_page)
    }

    /// Sets a fixed number of items per page.
    ///
    /// Updates `min_per_page`, `max_per_page`, and `default_per_page`
    /// to the provided value.
    ///
    /// # Errors
    /// Returns an error if `per_page < 1`.
    pub fn per_page(&mut self, per_page: i64) -> anyhow::Result<()> {
        if per_page < 1 {
            bail!("per_page must be greater than 0");
        }
        self.min_per_page = per_page;
        self.max_per_page = per_page;
        self.default_per_page = per_page;
        Ok(())
    }

    /// Sets the minimum number of items per page.
    ///
    /// Ensures `max_per_page` and `default_per_page` are at least
    /// the new minimum value.
    ///
    /// # Errors
    /// Returns an error if `min_per_page < 1`.
    pub fn min_per_page(&mut self, min_per_page: i64) -> anyhow::Result<()> {
        if min_per_page < 1 {
            bail!("min_per_page must be greater than 0");
        }
        self.min_per_page = min_per_page;
        self.max_per_page = self.max_per_page.max(min_per_page);
        self.default_per_page = self.default_per_page.max(min_per_page);
        Ok(())
    }

    /// Sets the maximum number of items per page.
    ///
    /// Ensures `min_per_page` and `default_per_page` do not exceed
    /// the new maximum value.
    ///
    /// # Errors
    /// Returns an error if `max_per_page < 1`.
    pub fn max_per_page(&mut self, max_per_page: i64) -> anyhow::Result<()> {
        if max_per_page < 1 {
            bail!("max_per_page must be greater than 0");
        }
        self.min_per_page = self.min_per_page.min(max_per_page);
        self.max_per_page = max_per_page;
        self.default_per_page = self.default_per_page.min(max_per_page);
        Ok(())
    }

    /// Applies pagination settings and returns a `PaginateClauseRendered`.
    ///
    /// # Parameters and behavior are identical to `render`.
    #[must_use]
    pub fn input(&self, page_number: Option<i64>, per_page: Option<i64>) -> PaginateClauseRendered {
        self.render(page_number, per_page)
    }
}

/// The `PaginateClauseRendered` struct holds the result of pagination:
/// - `limit`: Number of items to return (`LIMIT`).
/// - `offset`: Number of items to skip (`OFFSET`).
#[derive(Clone, PartialEq, Debug)]
#[php_class]
#[php(name = "Sqlx\\PaginateClauseRendered")]
pub struct PaginateClauseRendered {
    /// Number of items to return (LIMIT).
    pub(crate) limit: i64,
    /// Offset from the start (OFFSET).
    pub(crate) offset: i64,
}

impl PaginateClauseRendered {
    #[inline]
    pub(crate) fn write_sql_to(
        &self,
        sql: &mut String,
        out_vals: &mut Vec<ParameterValue>,
        settings: &Settings,
    ) -> anyhow::Result<()> {
        sql.push_str("LIMIT ");
        ParameterValue::Int(self.limit).write_sql_to(sql, out_vals, settings)?;
        sql.push_str(" OFFSET ");
        ParameterValue::Int(self.offset).write_sql_to(sql, out_vals, settings)
    }
}
