use anyhow::bail;
use ext_php_rs::{ZvalConvert, php_class, php_impl, prelude::ModuleBuilder};

#[php_class]
#[php(name = "Sqlx\\PaginateClause")]
#[php(rename = "none")]
pub struct PaginateClause {
    pub(crate) min_per_page: i64,
    pub(crate) max_per_page: i64,
    pub(crate) default_per_page: i64,
}
impl PaginateClause {
    pub fn new() -> Self {
        Self {
            min_per_page: 1,
            max_per_page: 20,
            default_per_page: 20,
        }
    }

    #[must_use]
    #[inline(always)]
    fn internal_apply(
        &self,
        page_number: Option<i64>,
        per_page: Option<i64>,
    ) -> PaginateClauseRendered {
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
    pub fn __construct() -> Self {
        Self::new()
    }

    /// __invoke magic for apply()
    #[must_use]
    pub fn __invoke(
        &self,
        page_number: Option<i64>,
        per_page: Option<i64>,
    ) -> PaginateClauseRendered {
        self.internal_apply(page_number, per_page)
    }

    pub fn per_page(&mut self, per_page: i64) -> anyhow::Result<()> {
        if per_page < 1 {
            bail!("par_page must be greater than 0");
        }
        self.min_per_page = per_page;
        self.max_per_page = per_page;
        self.default_per_page = per_page;
        Ok(())
    }
    pub fn min_per_page(&mut self, min_per_page: i64) -> anyhow::Result<()> {
        if min_per_page < 1 {
            bail!("min_per_page must be greater than 0");
        }
        self.min_per_page = min_per_page;
        self.max_per_page = self.max_per_page.max(min_per_page);
        self.default_per_page = self.default_per_page.max(min_per_page);
        Ok(())
    }

    pub fn max_per_page(&mut self, max_per_page: i64) -> anyhow::Result<()> {
        if max_per_page < 1 {
            bail!("max_per_page must be greater than 0");
        }
        self.min_per_page = self.min_per_page.min(max_per_page);
        self.max_per_page = max_per_page;
        self.default_per_page = self.default_per_page.min(max_per_page);
        Ok(())
    }

    #[must_use]
    pub fn apply(&self, page_number: Option<i64>, per_page: Option<i64>) -> PaginateClauseRendered {
        self.internal_apply(page_number, per_page)
    }
}
#[derive(Clone, PartialEq, Debug)]
#[php_class]
pub struct PaginateClauseRendered {
    // @TODO: make it impossible to alter PaginateClauseRendered from PHP side
    pub(crate) limit: i64,
    pub(crate) offset: i64,
}
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<PaginateClause>()
}
