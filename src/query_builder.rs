use crate::param_value::{ParameterValue, ParamsMap};
use anyhow::{anyhow, bail};
use ext_php_rs::convert::FromZval;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendClassObject, ZendHashTable, Zval};
use strum_macros::Display;

/// Registers the `OrClause` class and `OR_` function
/// with the provided PHP module builder.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<OrClause>().function(wrap_function!(or_))
}

#[php_class]
#[php(name = "Sqlx\\OrClause")]
#[derive(Debug, Clone)]
pub struct OrClause {
    pub(crate) inner: Vec<OrClauseItem>,
}
#[derive(Debug, Clone)]
pub enum OrClauseItem {
    Nested(OrClause),
    Op((String, String, Option<ParameterValue>)),
    Item((String, Option<ParamsMap>)),
}

/// Represents the supported SQL `JOIN` types.
#[derive(Debug, Clone, Display)]
pub enum JoinType {
    #[strum(to_string = "INNER")]
    Inner,

    #[strum(to_string = "LEFT")]
    Left,

    #[strum(to_string = "RIGHT")]
    Right,

    #[strum(to_string = "NATURAL")]
    Natural,

    #[strum(to_string = "FULL OUTER")]
    FullOuter,

    #[strum(to_string = "CROSS")]
    Cross,
}

#[php_function]
#[php(name = "Sqlx\\OR_")]
pub fn or_(or: &ZendHashTable) -> anyhow::Result<OrClause> {
    let mut inner = Vec::with_capacity(or.len());
    for (i, (key, value)) in or.iter().enumerate() {
        if let ArrayKey::Long(_) = key {
            if let Some(value) = value.string() {
                inner.push(OrClauseItem::Item((value, None)));
            } else if let Some(array) = value.array() {
                if array.has_sequential_keys() {
                    let array_len = array.len();
                    if array_len > 3 {
                        bail!("condition #{i}: array cannot contain more than 3 elements");
                    }
                    let left_operand =
                        array.get_index(0).and_then(Zval::string).ok_or_else(|| {
                            anyhow!("first element (left operand) of #{i} must be a string")
                        })?;
                    let operator = array.get_index(1).and_then(Zval::string).ok_or_else(|| {
                        anyhow!("second element (operator) of #{i} must be a string")
                    })?;
                    let right_operand = if array_len > 2 {
                        Some(
                            array
                                .get_index(2)
                                .and_then(ParameterValue::from_zval)
                                .ok_or_else(|| {
                                    anyhow!("third element (value) must a valid parameter value")
                                })?,
                        )
                    } else {
                        None
                    };
                    inner.push(OrClauseItem::Op((left_operand, operator, right_operand)));
                }
            } else if let Some(or) = value
                .object()
                .and_then(ZendClassObject::<OrClause>::from_zend_obj)
                .and_then(|x| x.obj.clone())
            {
                inner.push(OrClauseItem::Nested(or));
            } else {
                bail!("element must be a string or OrClause");
            }
        } else {
            let Some(parameters) = value.array() else {
                bail!("keyed element's value must be array");
            };
            let parameters: ParamsMap = parameters.try_into().map_err(|err| anyhow!("{err}"))?;
            inner.push(OrClauseItem::Item((key.to_string(), Some(parameters))));
        }
    }

    Ok(OrClause { inner })
}

#[macro_export]
macro_rules! php_sqlx_impl_query_builder {
    ( $struct:ident, $class:literal, $interface:literal, $driver: ident, $driver_inner: ident ) => {
        use $crate::ast::Ast;
        use $crate::param_value::ParamsMap;
        use $crate::query_builder::{OrClause, OrClauseItem};
        use $crate::select_clause::SelectClauseRendered;
        use $crate::by_clause::ByClauseRendered;
        use $crate::param_value::ParameterValue;
        use $crate::utils::types::ColumnArgument;
        use $crate::utils::indent_sql::IndentSql;
        use $crate::query_builder::JoinType;
        use $crate::utils::strip_prefix::StripPrefixWordIgnoreAsciiCase;
        use anyhow::anyhow;
        use anyhow::bail;
        use ext_php_rs::php_impl;
        use ext_php_rs::prelude::*;
        use ext_php_rs::types::{ArrayKey, ZendClassObject, Zval};
        use std::collections::{BTreeSet, BTreeMap, HashMap};
        use std::fmt::Write;
        use std::sync::{Once, Arc};
        use ext_php_rs::convert::FromZval;
        use ext_php_rs::flags::DataType;
        use trim_in_place::TrimInPlace;

        /// A prepared SQL query builder.
        ///
        /// Holds the generated query string, parameters, and placeholder tracking
        /// used during safe, composable query construction via AST rendering.
        #[php_class]
        #[php(name = $class)]
        pub struct $struct {
            pub(crate) query: String,
            pub(crate) readonly: bool,
            pub(crate) driver_inner: Arc<$driver_inner>,
            pub(crate) placeholders: BTreeSet<String>,
            pub(crate) parameters: BTreeMap<String, ParameterValue>,
        }

        struct ParameterValueWrapper(ParameterValue);
        impl FromZval<'_> for ParameterValueWrapper {
            const TYPE: DataType = DataType::Mixed;
            fn from_zval(zval: &Zval) -> Option<Self> {
                if let Some(builder) = zval
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    Some(Self(ParameterValue::Builder((builder.query.indent_sql(true), builder.parameters.clone()))))
                } else if let Some(pv) = ParameterValue::from_zval(zval) {
                    Some(Self(pv))
                } else {
                    None
                }
            }
        }
        impl From<ParameterValueWrapper> for ParameterValue {
            fn from(wrapper: ParameterValueWrapper) -> Self {
                wrapper.0
            }
        }

        impl $struct {
            /// Creates a new prepared query builder.
            ///
            /// This method initializes a fresh instance of the prepared query structure,
            /// linking it to the provided `DriverInner`. It starts with an empty query
            /// string, no parameters, and an empty set of used placeholders.
            ///
            /// # Arguments
            /// * `driver_inner` – A shared reference to the internal driver state.
            ///
            /// # Returns
            /// A new instance of the prepared query builder.
            pub(crate) fn new(driver_inner: Arc<$driver_inner>) -> Self {
                static INIT: Once = Once::new();
                INIT.call_once(|| {
                    $crate::utils::adhoc_php_class_implements($class, $interface);
                });
                Self {
                    readonly: driver_inner.is_readonly(),
                    driver_inner,
                    placeholders: Default::default(),
                    parameters: Default::default(),
                    query: Default::default(),
                }
            }

            fn _write_op_guard(&self) -> anyhow::Result<()> {
                if self.readonly {
                    bail!("You cannot write to a replica.");
                }
                Ok(())
            }

            /// Appends a SQL `JOIN` clause to the query with an `ON` condition.
            ///
            /// # Arguments
            /// * `join_type` – Enum value specifying the join type (e.g. `JoinType::Left`).
            /// * `table` – The name of the table to join.
            /// * `on` – SQL expression used in the `ON` clause.
            /// * `parameters` – Optional map of parameters for the `ON` clause.
            ///
            /// # Errors
            /// Returns an error if placeholder resolution fails.
            fn _join_clause(
                &mut self,
                join_type: JoinType,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<()> {
                if !self.query.is_empty() {
                    self.query.push('\n');
                }
                write!(self.query, "{} JOIN {} ON ", join_type, table)?;
                self._append(on, parameters, "join")?;
                Ok(())
            }


            /// Internal helper to append a UNION clause.
            ///
            /// # Arguments
            /// * `keyword` – Either "UNION" or "UNION ALL"
            /// * `query` – SQL string or Builder
            /// * `parameters` – Optional bound parameters
            fn _append_union_clause(
                &mut self,
                keyword: &str,
                query: &Zval,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<()> {
                use ext_php_rs::types::ZendClassObject;

                write!(self.query, "\n{keyword}\n")?;

                if let Some(part) = query.str() {
                    self._append(&part.indent_sql(true), parameters, "union")?;
                } else if let Some($struct { query, parameters, .. }) = query
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    self._append(
                        &format!("({})", query.indent_sql(true)),
                        Some(parameters.clone()),
                        "union",
                    )?;
                } else {
                    bail!("argument to union must be a string or Builder");
                }

                Ok(())
            }

            /// Appends a SQL comparison or null-check operator to the query.
            ///
            /// Supports standard SQL operators (e.g., `=`, `!=`, `<`, `IN`, `IS NULL`, etc.).
            /// Automatically binds parameters unless the operator is `IS NULL`/`IS NOT NULL`,
            /// which must not receive a value.
            pub fn _append_op(
                &mut self,
                left_operand: &str,
                operator: &str,
                right_operand: Option<ParameterValue>,
                placeholder_prefix: &str,
            ) -> anyhow::Result<()> {
                let mut op = operator.to_ascii_uppercase();
                op.trim_in_place();

                if op.eq("NLIKE") {
                    op = String::from("NOT LIKE")
                }
                if op.eq("NILIKE") {
                    op = String::from("NOT ILIKE")
                }

                let not = if let Some(stripped) = op.strip_prefix_word_ignore_ascii_case(&["NOT"; 1]) {
                    op = stripped.trim_start().to_owned();
                    true
                } else {
                    false
                };
                if let Some(to) = match op.as_str() {
                    "=" | "EQ" => Some("="),
                    "!=" | "<>" | "NEQ" | "NE" => Some("!="),
                    ">" | "GT" => Some(">"),
                    ">=" | "GTE" => Some(">="),
                    "<" | "LT" => Some("<"),
                    "<=" | "LTE" => Some("<="),
                    _ => None
                } {
                   op = to.to_owned();
                }
                if matches!(op.as_str(), "IS NULL" | "IS NOT NULL") {
                    if not {
                        bail!("Invalid operator");
                    }
                    if right_operand.is_some() {
                            bail!("Operator {op} must not be given a right-hand operand");
                        }
                        self._append(
                            &format!("{left_operand} {op}"),
                            None::<[(&str, ParameterValue); 0]>,
                            placeholder_prefix
                        )?;
                }
                else {
                    let value = right_operand.ok_or_else(|| anyhow!("Operator {op} requires a right-hand operand"))?;
                    match op.as_str() {
                        "=" | "!=" | ">" | ">=" | "<" | "<=" if !not => {
                            self._append(
                                &format!("{left_operand} {}{op} ?", if not {"NOT "} else {""}),
                                Some([("0", value)]),
                                placeholder_prefix
                            )?;
                        }
                       "LIKE" | "ILIKE" => {
                           self._append(
                                &format!("{left_operand} {}{op} ?", if not {"NOT "} else {""}),
                                Some([("0", value)]),
                                placeholder_prefix
                           )?;
                        }
                        "IN" => {
                            self._append(
                                &format!("{left_operand} {}{op} (?)", if not {"NOT "} else {""}),
                                Some([("0", value)]),
                                placeholder_prefix
                            )?;
                        }
                        "IEQ" => {
                            self._append(
                                &format!("{left_operand} {}ILIKE ?", if not {"NOT "} else {""}),
                                Some([("0", ParameterValue::String(format!("{}", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                       "CONTAINS" => {
                            self._append(
                                &format!("{left_operand} {}LIKE ?", if not {"NOT "} else {""}),
                                Some([("0", ParameterValue::String(format!("%{}%", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                        "ICONTAINS" => {
                            self._append(
                                &format!("{left_operand} {}ILIKE ?", if not {"NOT "} else {""}),
                                Some([("0", ParameterValue::String(format!("%{}%", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                        "STARTSWITH" => {
                            self._append(
                                &format!("{left_operand} {}LIKE ?", if not {"NOT "} else {""}),
                                Some([("0", ParameterValue::String(format!("{}%", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                        "ISTARTSWITH" => {
                            self._append(
                                &format!("{left_operand} {}ILIKE ?", if not {"NOT "} else {""}),
                                Some([("0", ParameterValue::String(format!("{}%", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                        "ENDSWITH" => {
                            self._append(
                                &format!("{left_operand} {}LIKE ?", if not {"NOT "} else {""}),
                                Some([("0", ParameterValue::String(format!("%{}", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                        "IENDSWITH" => {
                            self._append(
                                &format!("{left_operand} ILIKE ?"),
                                Some([("0", ParameterValue::String(format!("%{}", value.meta_quote_like()?)))]),
                                placeholder_prefix
                            )?;
                        }
                        _ => bail!("Operator {operator:?} is not supported"),
                    }
                }
                Ok(())
            }

            /// Appends a grouped `OR` clause composed of multiple expressions.
            ///
            /// This method recursively renders nested `OrClause` groups and ensures
            /// proper formatting within parenthesis.
            pub fn _append_or(&mut self, or: &OrClause, prefix: &str) -> anyhow::Result<()> {
                self.query.push('(');
                for (i, item) in or.inner.iter().enumerate() {
                    if i > 0 {
                        self.query.push_str(" OR ");
                    }
                    match item {
                        OrClauseItem::Nested(nested)  => {
                            self._append_or(
                                nested,
                                prefix,
                            )?;
                        }
                        OrClauseItem::Op((left_operand, operator, right_operand))  => {
                            self._append_op(
                                left_operand,
                                operator,
                                right_operand.clone(),
                                prefix,
                            )?;
                        }
                        OrClauseItem::Item((part, parameters))  => {
                            self._append(
                                part.as_str(),
                                parameters.clone(),
                                prefix,
                            )?;
                        }
                    }
                }
                self.query.push(')');
                Ok(())
            }

            /// Transforms the AST into a SQL string using only named placeholders, replacing all positional
            /// placeholders (`?`, `:1`, `$1`) with unique names if they conflict with existing `placeholders`.
            ///
            /// Named placeholders that do not conflict will be preserved.
            /// Also extends the `parameters_bucket` with values from `parameters`, accounting for
            /// renaming of positional or conflicting placeholders.
            pub fn _append<I, K, V>(
                &mut self,
                part: &str,
                parameters: Option<I>,
                prefix: &str,
            ) -> anyhow::Result<()>
            where
                I: IntoIterator<Item = (K, V)>,
                K: Into<String>,
                V: Into<ParameterValue>,
            {
                self._append_ast(&self.driver_inner.parse_query(part)?, parameters, prefix)
            }

            /// Appends a parsed SQL AST fragment into the query string.
            ///
            /// Resolves placeholders (named or positional) and merges parameter values.
            /// Ensures that each placeholder is unique and consistent throughout the query.
            pub fn _append_ast<I, K, V>(
                &mut self,
                ast: &Ast,
                parameters: Option<I>,
                prefix: &str,
            ) -> anyhow::Result<()>
            where
                I: IntoIterator<Item = (K, V)>,
                K: Into<String>,
                V: Into<ParameterValue>,
            {
                #[allow(clippy::too_many_arguments)]
                fn walk(
                    driver_inner: &Arc<$driver_inner>,
                    node: &Ast,
                    sql: &mut String,
                    placeholders: &mut BTreeSet<String>,
                    param_map: &mut ParamsMap,
                    parameters_bucket: &mut ParamsMap,
                    positional_index: &mut usize,
                    prefix: &str,
                ) -> anyhow::Result<()> {
                    match node {
                        Ast::Root { branches, .. } | Ast::Nested(branches) => {
                            for b in branches {
                                walk(
                                    driver_inner,
                                    b,
                                    sql,
                                    placeholders,
                                    param_map,
                                    parameters_bucket,
                                    positional_index,
                                    prefix,
                                )?;
                            }
                        }
                        Ast::Raw(s) => sql.push_str(s),
                        Ast::Placeholder(name) => {
                            let param = param_map.remove(name);
                            if let Some(ParameterValue::Builder((part, mut params))) = param {
                                 sql.push('(');
                                 walk(
                                    driver_inner,
                                    &driver_inner.parse_query(&part)?,
                                    sql,
                                    placeholders,
                                    &mut params,
                                    parameters_bucket,
                                    positional_index,
                                    prefix,
                                )?;
                                sql.push_str("\n)");
                            } else {
                                let new_name = resolve_placeholder_name(
                                    name,
                                    placeholders,
                                    positional_index,
                                    prefix,
                                );
                                if let Some(value) = param {
                                    parameters_bucket.insert(new_name.clone(), value);
                                }
                                write!(sql, ":{new_name}")?;
                            }
                        }
                        Ast::ConditionalBlock { branches, .. } => {
                            for b in branches {
                                walk(
                                    driver_inner,
                                    b,
                                    sql,
                                    placeholders,
                                    param_map,
                                    parameters_bucket,
                                    positional_index,
                                    prefix,
                                )?;
                            }
                        }
                        Ast::InClause { expr, placeholder }
                        | Ast::NotInClause { expr, placeholder } => {
                            let new_name = resolve_placeholder_name(
                                placeholder,
                                placeholders,
                                positional_index,
                                prefix,
                            );
                            if let Some(value) = param_map.remove(placeholder) {
                                parameters_bucket.insert(new_name.clone(), value);
                            }
                            let keyword = if matches!(node, Ast::InClause { .. }) {
                                "IN"
                            } else {
                                "NOT IN"
                            };
                            write!(sql, "{} {} (:{} )", expr, keyword, new_name)?;
                        }
                        Ast::PaginateClause { placeholder } => {
                            let new_name = resolve_placeholder_name(
                                placeholder,
                                placeholders,
                                positional_index,
                                prefix,
                            );
                            if let Some(value) = param_map.remove(placeholder) {
                                parameters_bucket.insert(new_name.clone(), value);
                            }
                            write!(sql, "PAGINATE :{}", new_name)?;
                        }
                    }
                    Ok(())
                }

                /// Generates a unique placeholder name to avoid name collisions.
                ///
                /// Reuses the original name if not yet taken; otherwise generates a unique
                /// variant with a numeric suffix.
                fn resolve_placeholder_name(
                    name: &str,
                    placeholders: &mut BTreeSet<String>,
                    positional_index: &mut usize,
                    prefix: &str,
                ) -> String {
                    if name.parse::<usize>().is_ok() {
                        loop {
                            let candidate = format!("{prefix}__{positional_index}");
                            *positional_index += 1;
                            if placeholders.insert(candidate.clone()) {
                                break candidate;
                            }
                        }
                    } else if !placeholders.contains(name) {
                        placeholders.insert(name.to_string());
                        name.to_string()
                    } else {
                        let mut index = 2;
                        loop {
                            let candidate = format!("{name}_{index}");
                            index += 1;
                            if placeholders.insert(candidate.clone()) {
                                break candidate;
                            }
                        }
                    }
                }
                let mut positional_index = 0;
                let mut param_map: ParamsMap = parameters
                    .map(|x| {
                        x.into_iter()
                            .map(|(k, v)| {
                                let mut k = k.into();
                                if let Ok(n) = k.parse::<u32>() {
                                    k = (n + 1).to_string();
                                }
                                (k, v.into())
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                walk(
                    &self.driver_inner,
                    &ast,
                    &mut self.query,
                    &mut self.placeholders,
                    &mut param_map,
                    &mut self.parameters,
                    &mut positional_index,
                    prefix,
                )?;

                Ok(())
            }
        }

        #[php_impl]
        impl $struct {

            /// Creates a query builder object
            ///
            ///
            /// # Returns
            /// Query builder object
            #[must_use]
            pub fn builder(&self,) -> $struct {
                $struct::new(self.driver_inner.clone())
            }

            pub fn factory(driver: &ZendClassObject<$driver>) -> anyhow::Result<$struct> {
                if let Some(obj) = driver.obj.as_ref() {
                     Ok($struct::new(obj.driver_inner.clone()))
                } else {
                    bail!("You cannot do this now.");
                }
            }

            /// Quotes a single scalar value for safe embedding into SQL.
            ///
            /// This method renders the given `ParameterValue` into a properly escaped SQL literal,
            /// using the driver's configuration (e.g., quoting style, encoding).
            ///
            /// ⚠️ **Warning:** Prefer using placeholders and parameter binding wherever possible.
            /// This method should only be used for debugging or generating static fragments,
            /// not for constructing dynamic SQL with user input.
            ///
            /// # Arguments
            /// * `param` – The parameter to quote (must be a scalar: string, number, or boolean).
            ///
            /// # Returns
            /// Quoted SQL string (e.g., `'abc'`, `123`, `TRUE`)
            ///
            /// # Errors
            /// Returns an error if the parameter is not a scalar or if rendering fails.
            ///
            /// # Example
            /// ```php
            /// $driver->builder()->quote("O'Reilly"); // "'O''Reilly'"
            /// ```
            pub fn quote(&self, param: ParameterValue) -> anyhow::Result<String> {
                param.quote(&self.driver_inner.settings)
            }

            /// Escapes `%` and `_` characters in a string for safe use in a LIKE/ILIKE pattern.
            ///
            /// This helper is designed for safely preparing user input for use with
            /// pattern-matching operators like `CONTAINS`, `STARTS_WITH`, or `ENDS_WITH`.
            ///
            /// ⚠️ **Warning:** This method does **not** quote or escape the full string for raw SQL.
            /// It only escapes `%` and `_` characters. You must still pass the result as a bound parameter,
            /// not interpolate it directly into the query string.
            ///
            /// # Arguments
            /// * `param` – The parameter to escape (must be a string).
            ///
            /// # Returns
            /// A string with `%` and `_` escaped for LIKE (e.g., `foo\%bar\_baz`)
            ///
            /// # Errors
            /// Returns an error if the input is not a string.
            ///
            /// # Example
            /// ```php
            /// $escaped = $builder->metaQuoteLike("100%_safe");
            /// // Use like:
            /// $builder->where([["name", "LIKE", "%$escaped%"]]);
            /// ```
            pub fn meta_quote_like(&self, param: ParameterValue) -> anyhow::Result<String> {
                param.meta_quote_like()
            }

            /// Appends an `ON CONFLICT` clause to the query.
            ///
            /// # Arguments
            /// * `target` – A string or array of column names to specify the conflict target.
            /// * `set` – Optional `SET` clause. If `null`, generates `DO NOTHING`; otherwise uses the `SET` values.
            ///
            /// # Example
            /// ```php
            /// $builder->onConflict("id", null);
            /// $builder->onConflict(["email", "tenant_id"], ["name" => "New"]);
            /// ```
            fn on_conflict<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                target: &Zval,
                set: Option<&Zval>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                let target_str = if let Some(scalar) = target.str() {
                    scalar.to_string()
                } else if let Some(arr) = target.array() {
                    arr
                        .iter()
                        .map(|(_, val)| {
                            val.str().ok_or_else(|| anyhow!("ON CONFLICT array elements must be strings"))
                        })
                        .collect::<anyhow::Result<Vec<_>>>()?.join(", ")
                } else {
                    bail!("Conflict target must be a string or array of strings");
                };

                if let Some(set_val) = set {
                    write!(self_.query, "\nON CONFLICT ({target_str}) DO UPDATE")?;
                    $struct::set(self_, set_val)?;
                } else {
                    write!(self_.query, "\nON CONFLICT ({target_str}) DO NOTHING")?;
                }

                Ok(self_)
            }


            /// Appends an `ON DUPLICATE KEY UPDATE` clause to the query (MySQL).
            ///
            /// # Arguments
            /// * `set` – An array representing fields and values to update.
            ///
            /// # Example
            /// ```php
            /// $builder->onDuplicateKeyUpdate(["email" => "new@example.com"]);
            /// ```
            fn on_duplicate_key_update<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                set: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_.query.push_str("\nON DUPLICATE KEY UPDATE");
                $struct::set(self_, set)?;
                Ok(self_)
            }

            /// Appends an `INNER JOIN` clause to the query.
            fn inner_join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::Inner, table, on, parameters)?;
                Ok(self_)
            }

            /// Appends an `INNER JOIN` clause to the query.
            /// Alias for `inner_join()`.
            fn join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::Inner, table, on, parameters)?;
                Ok(self_)
            }

            /// Appends a `LEFT JOIN` clause to the query.
            fn left_join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::Left, table, on, parameters)?;
                Ok(self_)
            }

            /// Appends a `RIGHT JOIN` clause to the query.
            fn right_join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::Right, table, on, parameters)?;
                Ok(self_)
            }

            /// Appends a `FULL OUTER JOIN` clause to the query.
            fn full_join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::FullOuter, table, on, parameters)?;
                Ok(self_)
            }

            /// Appends a `CROSS JOIN` clause to the query.
            fn cross_join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                on: &str,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::Cross, table, on, parameters)?;
                Ok(self_)
            }

            /// Appends a `NATURAL JOIN` clause to the query.
            ///
            /// # Note
            /// `NATURAL JOIN` does not accept `ON` conditions. The `on` argument is ignored.
            fn natural_join<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._join_clause(JoinType::Natural, table, "", None)?;
                Ok(self_)
            }

            /// Appends a `WITH` clause to the query.
            ///
            /// # Arguments
            /// * `table` - Name of the CTE (common table expression).
            /// * `as` - The query body to be rendered for the CTE.
            /// * `parameters` - Optional parameters for the query body.
            ///
            /// # Example
            /// ```php
            /// $builder->with("cte_name", "SELECT * FROM users WHERE active = $active", [
            ///     "active" => true,
            /// ]);
            /// ```
            fn with<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
                r#as: &Zval,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "WITH {table} AS (")?;
                if let Some(part) = r#as.str() {
                     self_._append(
                        part,
                        parameters,
                        "with",
                    )?;
                } else if let Some($struct {query, parameters, ..}) = r#as
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    self_._append(&query.indent_sql(true), Some(parameters.clone()), "with")?;
                }
                else {
                    bail!("`as` must be a string or Builder");
                }
                self_.query.push_str("\n)");
                Ok(self_)
            }

            /// Appends a `WHERE` clause to the query.
            ///
            /// # Arguments
            /// * `where` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
            /// * `parameters` - Optional parameters associated with the `WHERE` condition.
            ///
            /// # Exceptions
            /// Throws an exception if the input is not valid.
            fn _where<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                r#where: &Zval,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if let Some(part) = r#where.str() {
                    self_.query.push_str("\nWHERE ");
                    self_._append(&part.indent_sql(false), parameters, "where")?;
                } else if let Some(array) = r#where.array() {
                    self_.query.push_str("\nWHERE ");
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str("\n   AND ");
                        }
                        match key {
                            ArrayKey::Long(_) => {
                                if let Some(part) = value.str() {
                                    self_._append(
                                        part,
                                        None::<[(&str, &str); 0]>,
                                        "where",
                                    )?;
                                } else if let Some(array) = value.array() {
                                    if array.has_sequential_keys() {
                                        let array_len = array.len();
                                        if array_len > 3 {
                                            bail!("condition #{i}: array cannot contain more than 3 elements");
                                        }
                                        let left_operand = array.get_index(0)
                                            .and_then(Zval::str)
                                            .ok_or_else(|| anyhow!("first element (left operand) of #{i} must be a string"))?;
                                        let operator = array.get_index(1).and_then(Zval::str)
                                            .ok_or_else(|| anyhow!("second element (operator) of #{i} must be a string"))?;
                                        self_._append_op(left_operand, operator, if array_len > 2 {
                                            Some(
                                                array.get_index(2)
                                                    .and_then(ParameterValue::from_zval)
                                                    .ok_or_else(|| anyhow!("third element (value) must a valid parameter value"))?
                                            )
                                        } else {
                                            None
                                        }, "where")?;
                                    } else {
                                        bail!("condition #{i}: array must be a list");
                                    }
                                }
                                else if let Some(or) = value
                                    .object()
                                    .and_then(ZendClassObject::<OrClause>::from_zend_obj)
                                    .and_then(|x| x.obj.as_ref())
                                {
                                    self_._append_or(or, "where")?;
                                } else {
                                    bail!("element must be a string or OrClause");
                                }
                            }
                            _ => {
                                let part = key.to_string();
                                let ast = self_.driver_inner.parse_query(&part)?;
                                if ast.has_placeholders() {
                                    let Some(parameters) = value.array() else {
                                        bail!("value must be array because the key string ({part:?}) contains placeholders: {ast:?}");
                                    };
                                    let parameters: HashMap<String, ParameterValueWrapper> =
                                        parameters.try_into().map_err(|err| anyhow!("{err}"))?;
                                    self_._append_ast(&ast, Some(parameters), "where")?;
                                } else {
                                    self_._append(
                                        &format!("{part} = ?"),
                                        Some([
                                            ("0", ParameterValue::from_zval(value)
                                                .ok_or_else(|| anyhow!("element value must a valid parameter value"))?
                                            ); 1]
                                        ),
                                        "where"
                                    )?;
                                }
                            }
                        }
                    }
                } else {
                    bail!("illegal where() argument");
                }
                Ok(self_)
            }


            /// Appends a `UNION` clause to the query.
            ///
            /// # Arguments
            /// * `query` – A raw SQL string or another Builder instance (subquery).
            /// * `parameters` – Optional parameters to bind to the unioned subquery.
            ///
            /// # Example
            /// ```php
            /// $builder->union("SELECT id FROM users");
            /// $builder->union($other_builder);
            /// ```
            fn union<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                query: &Zval,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._append_union_clause("UNION", query, parameters)?;
                Ok(self_)
            }

            /// Appends a `UNION ALL` clause to the query.
            ///
            /// # Arguments
            /// * `query` – A raw SQL string or another Builder instance (subquery).
            /// * `parameters` – Optional parameters to bind to the unioned subquery.
            ///
            /// # Example
            /// ```php
            /// $builder->union_all("SELECT id FROM users");
            /// $builder->union_all($other_builder);
            /// ```
            fn union_all<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                query: &Zval,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._append_union_clause("UNION ALL", query, parameters)?;
                Ok(self_)
            }

            /// Appends a `HAVING` clause to the query.
            ///
            /// # Arguments
            /// * `having` - Either a raw string, a structured array of conditions, or a disjunction (`OrClause`).
            /// * `parameters` - Optional parameters associated with the `WHERE` condition.
            ///
            /// # Exceptions
            /// Throws an exception if the input is not valid.
            fn having<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                having: &Zval,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if let Some(part) = having.str() {
                    self_.query.push_str("\nHAVING ");
                    self_._append(&part.indent_sql(false), parameters, "having")?;
                } else if let Some(array) = having.array() {
                    self_.query.push_str("\nHAVING ");
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str("\n   AND ");
                        }
                        match key {
                            ArrayKey::Long(_) => {
                                if let Some(part) = value.str() {
                                    self_._append(
                                        part,
                                        None::<[(&str, &str); 0]>,
                                        "having",
                                    )?;
                                } else if let Some(array) = value.array() {
                                    if array.has_sequential_keys() {
                                        let array_len = array.len();
                                        if array_len > 3 {
                                            bail!("condition #{i}: array cannot contain more than 3 elements");
                                        }
                                        let left_operand = array.get_index(0)
                                            .and_then(Zval::str)
                                            .ok_or_else(|| anyhow!("first element (left operand) of #{i} must be a string"))?;
                                        let operator = array.get_index(1).and_then(Zval::str)
                                            .ok_or_else(|| anyhow!("second element (operator) of #{i} must be a string"))?;
                                        self_._append_op(left_operand, operator, if array_len > 2 {
                                            Some(
                                                array.get_index(2)
                                                    .and_then(ParameterValue::from_zval)
                                                    .ok_or_else(|| anyhow!("third element (value) must a valid parameter value"))?
                                            )
                                        } else {
                                            None
                                        }, "having")?;
                                    } else {
                                        bail!("condition #{i}: array must be a list");
                                    }
                                }
                                else if let Some(or) = value
                                    .object()
                                    .and_then(ZendClassObject::<OrClause>::from_zend_obj)
                                    .and_then(|x| x.obj.as_ref())
                                {
                                    self_._append_or(or, "having")?;
                                } else {
                                    bail!("element must be a string or OrClause");
                                }
                            }
                            _ => {
                                let part = key.to_string();
                                let ast = self_.driver_inner.parse_query(&part)?;
                                if ast.has_placeholders() {
                                    let Some(parameters) = value.array() else {
                                        bail!("value must be array because the key string ({part:?}) contains placeholders: {ast:?}");
                                    };
                                    let parameters: HashMap<String, ParameterValueWrapper> =
                                        parameters.try_into().map_err(|err| anyhow!("{err}"))?;
                                    self_._append_ast(&ast, Some(parameters), "having")?;
                                } else {
                                    self_._append(
                                        &format!("{part} = ?"),
                                        Some([
                                            ("0", ParameterValue::from_zval(value)
                                                .ok_or_else(|| anyhow!("element value must a valid parameter value"))?
                                            ); 1]
                                        ),
                                        "having"
                                    )?;
                                }
                            }
                        }
                    }
                } else {
                    bail!("illegal having() argument");
                }
                Ok(self_)
            }

            /// Appends a `LIMIT` (and optional `OFFSET`) clause to the query.
            ///
            /// # Arguments
            /// * `limit` – Maximum number of rows to return.
            /// * `offset` – Optional number of rows to skip before starting to return rows.
            ///
            /// # Example
            /// ```php
            /// $builder->limit(10);
            /// $builder->limit(10, 20); // LIMIT 10 OFFSET 20
            /// ```
            fn limit(
                self_: &mut ZendClassObject<$struct>,
                limit: i64,
                offset: Option<i64>,
            ) -> anyhow::Result<&mut ZendClassObject<$struct>> {
                if limit < 0 {
                    bail!("LIMIT must be non-negative");
                }
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "LIMIT {limit}")?;
                if let Some(offset) = offset {
                    if offset < 0 {
                        bail!("OFFSET must be non-negative");
                    }
                    write!(self_.query, " OFFSET {offset}")?;
                }
                Ok(self_)
            }

            /// Appends an `OFFSET` clause to the query independently.
            ///
            /// # Arguments
            /// * `offset` – Number of rows to skip before returning results.
            ///
            /// # Example
            /// ```php
            /// $builder->offset(30); // OFFSET 30
            /// ```
            fn offset(
                self_: &mut ZendClassObject<$struct>,
                offset: i64,
            ) -> anyhow::Result<&mut ZendClassObject<$struct>> {
                if offset < 0 {
                    bail!("OFFSET must be non-negative");
                }
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "OFFSET {offset}")?;
                Ok(self_)
            }


            /// Appends a `DELETE FROM` clause to the query.
            ///
            /// # Arguments
            /// * `from` - A string table name or a nested builder object.
            /// * `parameters` - Optional parameters if the `from` is a raw string.
            ///
            /// # Examples
            /// ```php
            /// $builder->deleteFrom("users");
            /// $builder->deleteFrom($builder->select("id")->from("temp_users"));
            /// ```
            fn delete_from<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                from: &Zval,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;

                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }

                if let Some(str) = from.str() {
                    self_._append(&format!("DELETE FROM ({})", str.indent_sql(true)), parameters, "delete")?;
                } else if from.array().is_some() {
                    bail!("deleteFrom() does not support arrays");
                } else if let Some($struct { query, parameters, .. }) = from
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    self_._append(&format!("DELETE FROM ({})", query.indent_sql(true)), Some(parameters.clone()), "delete")?;
                } else {
                    bail!("invalid deleteFrom() argument");
                }
                Ok(self_)
            }

            /// Appends a `USING` clause to the query.
            ///
            /// # Arguments
            /// * `from` - Either a string table name or a subquery builder.
            /// * `parameters` - Optional parameters if `from` is a string.
            fn using<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                from: &Zval,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_.query.push_str("\nUSING ");

                if let Some(str) = from.str() {
                    self_._append(&str.indent_sql(true), parameters, "using")?;
                } else if from.array().is_some() {
                    bail!("using() does not support arrays");
                } else if let Some($struct { query, parameters, .. }) = from
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    self_._append(&format!("({})", query.indent_sql(true)), Some(parameters.clone()), "using")?;
                } else {
                    bail!("invalid using() argument");
                }

                Ok(self_)
            }

            /// Appends a `PAGINATE` clause to the query using a `PaginateClauseRendered` object.
            ///
            /// This expands into the appropriate SQL `LIMIT` and `OFFSET` syntax during rendering,
            /// using the values stored in the given `PaginateClauseRendered` instance.
            ///
            /// # Arguments
            /// * `paginate` – An instance of `Sqlx\PaginateClauseRendered`, produced by invoking a `PaginateClause`
            ///               (e.g., `$paginate = (new PaginateClause)($page, $perPage)`).
            ///
            /// # Errors
            /// Returns an error if the argument is not an instance of `PaginateClauseRendered`.
            ///
            /// # Example
            /// ```php
            /// $paginate = (new \Sqlx\PaginateClause())->__invoke(2, 10); // page 2, 10 per page
            /// $builder->select("*")->from("users")->paginate($paginate);
            /// // SELECT * FROM users LIMIT 10 OFFSET 20
            /// ```
            fn paginate<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                paginate: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {

                let pv =  ParameterValue::from_zval(paginate);
                if !matches!(pv, Some(ParameterValue::PaginateClauseRendered(_))) {
                    bail!("argument must be PaginateClauseRendered");
                }
                self_._append(&format!("PAGINATE ?"), pv.map(|x| [("0", x); 1]), "paginate")?;
                Ok(self_)
            }

            /// Appends a `WITH RECURSIVE` clause to the query.
            ///
            /// # Arguments
            /// * `table_and_fields` - Table name with field list, e.g. `cte(col1, col2)`.
            /// * `as` - The recursive query body.
            /// * `parameters` - Optional parameters for the recursive body.
            ///
            /// # Example
            /// ```php
            /// $builder->withRecursive("cte(id, parent)", "SELECT ...", [...]);
            /// ```
            fn with_recursive<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table_and_fields: &str,
                r#as: &Zval,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "WITH RECURSIVE {table_and_fields} AS (\n")?;
                if let Some(part) = r#as.str() {
                     self_._append(
                        part,
                        parameters,
                        "with_recursive",
                    )?;
                } else if let Some($struct {query, parameters, ..}) = r#as
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    self_._append(&query.indent_sql(true), Some(parameters.clone()), "with")?;
                }
                else {
                    bail!("`as` must be a string or Builder");
                }
                self_.query.push_str("\n)");
                Ok(self_)
            }

            /// Appends a `UPDATE` clause to the query.
            ///
            /// # Arguments
            /// * `table` - A raw string representing the table(s).
            ///
            /// # Exceptions
            /// Throws an exception if the argument is not a string.
            fn update<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                if let Some(str) = table.str() {
                    if !self_.query.is_empty() {
                        self_.query.push('\n');
                    }
                    write!(self_.query, "UPDATE {str}")?;
                } else {
                    bail!("illegal update() argument")
                }
                Ok(self_)
            }

            /// Appends a `SET` clause to the query, supporting both keyed and indexed formats.
            ///
            /// # Arguments
            /// * `set` - An associative array mapping fields to values, or a sequential array
            ///   of `[field, value]` pairs or raw fragments.
            ///
            /// # Supported Formats
            /// ```php
            /// $builder->set(["name" => "John", "age" => 30]);
            /// $builder->set([["name", "John"], ["age", 30]]);
            /// $builder->set(["updated_at = NOW()"]);
            /// ```
            ///
            /// # Exceptions
            /// - When the input array is malformed or contains invalid value types.
            fn set<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                set: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                self_.query.push_str("\nSET ");
                let mut first = true;

                let set_array = set.array().ok_or_else(|| anyhow!("Argument to set() must be an array"))?;
                for (key, value) in set_array.iter() {
                    if !first {
                        self_.query.push_str(", ");
                    } else {
                        first = false;
                    }

                    match key {
                        ArrayKey::Long(_) => {
                            if let Some(array) = value.array() {
                                if array.has_sequential_keys() {
                                    if array.len() != 2 {
                                        bail!("each element must be [field, value]");
                                    }
                                    let field = array
                                        .get_index(0)
                                        .and_then(Zval::str)
                                        .ok_or_else(|| anyhow!("first element (field) must be a string"))?;

                                    let param = array
                                        .get_index(1)
                                        .and_then(ParameterValue::from_zval)
                                        .ok_or_else(|| anyhow!("second element (value) must be a valid parameter value"))?;

                                    self_._append_op(field, "=", Some(param), "set")?;
                                } else {
                                    if array.len() != 1 {
                                        bail!("keyed array must contain a single element");
                                    }
                                    for (key, value) in array {
                                        let Some(parameters) = value.array() else {
                                            bail!("value of keyed element {key:?} must be array");
                                        };
                                        let parameters: HashMap<String, ParameterValue> =
                                            parameters.try_into().map_err(|err| anyhow!("invalid parameters: {err}"))?;
                                        self_._append(&key.to_string(), Some(parameters), "set")?;
                                    }
                                }
                            } else if let Some(part) = value.str() {
                                self_._append(
                                    part,
                                    None::<[(&str, &str); 0]>,
                                    "set",
                                )?;
                            }
                            else {
                                bail!("numeric element #{key} must be string (raw expression) or array like [field, value]");
                            }
                        }
                        ArrayKey::Str(_) | ArrayKey::String(_) => {
                            let field = key.to_string();
                            let param = ParameterValue::from_zval(&value)
                                .ok_or_else(|| anyhow!("value for key `{field}` must be a valid parameter value"))?;

                            self_._append_op(field.as_str(), "=", Some(param), "set")?;
                        }
                    }
                }

                Ok(self_)
            }

            /// Renders the SQL query using named parameters and returns the fully rendered SQL with values injected inline.
            ///
            /// # Returns
            /// The rendered SQL query as a string with all parameters interpolated.
            ///
            /// # Exceptions
            /// - If rendering or encoding of parameters fails.
            pub(crate) fn dry(&self) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner.dry(&self.query, Some(self.parameters.clone().into_iter().collect()))
            }

            /// Returns the parameter map currently accumulated in the builder.
            ///
            /// # Returns
            /// A cloned map of all parameter names and their corresponding `ParameterValue`.
            ///
            /// # Exceptions
            /// - If rendering or encoding of parameters fails.
            pub(crate) fn dry_inline(&self) -> anyhow::Result<String> {
                self.driver_inner.dry_inline(&self.query, Some(self.parameters.clone().into_iter().collect()))
            }

            /// Returns the fully rendered SQL query with parameters embedded as literals.
            ///
            /// Used for debugging or fallback rendering when the placeholder limit is exceeded.
            ///
            /// # Returns
            /// A string representing the complete SQL statement.
            ///
            /// # Exceptions
            /// - If rendering or encoding of parameters fails.
            fn __to_string(&self) -> anyhow::Result<String> {
                self.dry_inline()
            }

            /// Returns an array of all currently accumulated parameters.
            pub(crate) fn parameters(&self) -> BTreeMap<String, ParameterValue> {
                self.parameters.clone()
            }

            /// Appends a raw SQL fragment to the query without structural validation.
            ///
            /// This method allows injecting raw SQL clauses directly into the query. It's intended
            /// for advanced use cases such as vendor-specific syntax, subqueries, or complex expressions
            /// not yet supported by the structured builder methods.
            ///
            /// # Arguments
            ///
            /// * `part` - A raw SQL string to append to the query. It is inserted **verbatim** into the
            ///   final SQL output, so it must be syntactically correct and complete.
            /// * `parameters` - An optional map of named parameters to bind to placeholders within the SQL string.
            ///   These values **are safely escaped and bound** according to the driver’s placeholder style.
            ///
            /// # Returns
            ///
            /// Returns a mutable reference to the builder for fluent method chaining.
            ///
            /// # Example (PHP)
            ///
            /// ```php
            /// $builder
            ///     ->select("*")
            ///     ->from("users")
            ///     ->raw("WHERE created_at > :after", ["after" => "2024-01-01"]);
            /// ```
            ///
            /// The `:after` placeholder will be safely bound to the provided value,
            /// using the correct placeholder format for your database (e.g. `$1`, `?`, `@p1`).
            ///
            /// # Safety
            ///
            /// While `raw()` allows bypassing structural checks, **it remains safe when placeholders are used properly**.
            /// However, avoid interpolating raw user input directly into the SQL string — prefer bound parameters to
            /// protect against SQL injection.
            ///
            /// Prefer using structured methods like `where()`, `join()`, or `select()` where possible for readability and safety.
            fn raw<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                part: &str,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._append(
                    part,
                    parameters,
                    "raw"
                )?;
                Ok(self_)
            }

            /// Appends a `SELECT` clause to the query.
            ///
            /// # Arguments
            /// * `fields` - Either a raw string or a `SelectClauseRendered` object.
            ///
            /// # Exceptions
            /// Throws an exception if the argument is not a string or a supported object.
            fn select<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                fields: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                if let Some(str) = fields.str() {
                    write!(self_.query, "SELECT {str}")?;
                }
                else if let Some(array) = fields.array() {
                    self_.query.push_str("SELECT ");
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str(", ");
                        }
                        if let ArrayKey::Long(_) = key {
                            if let Some(field) = value.str() {
                                self_.query.push_str(field);
                            } else {
                                bail!("indexed element must be a string")
                            }
                        } else {
                            let field = key.to_string();
                            if let Some(expr) = value.str() {
                                write!(self_.query, "{expr} AS {field}")?;
                            } else {
                                bail!("keyed element value must be a string")
                            }
                        }
                    }
                }
                else if let Some(scr) = fields
                    .object()
                    .and_then(ZendClassObject::<SelectClauseRendered>::from_zend_obj)
                {
                    let mut clause = String::from("SELECT ");
                    scr.write_sql_to(&mut clause, &self_.driver_inner.settings)?;
                    self_.query.push_str(&clause);
                } else {
                    bail!("illegal select() argument")
                }
                Ok(self_)
            }

            /// Appends a `ORDER BY` clause to the query.
            ///
            /// # Arguments
            /// * `fields` - Either a raw string or a `ByClauseRendered` object.
            ///
            /// # Exceptions
            /// Throws an exception if the argument is not a string or a supported object.
            fn order_by<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                fields: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                if let Some(str) = fields.str() {
                    write!(self_.query, "ORDER BY {str}")?;
                }
                else if let Some(array) = fields.array() {
                    self_.query.push_str("ORDER BY ");
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str(", ");
                        }
                        if let ArrayKey::Long(_) = key {
                            if let Some(field) = value.str() {
                                self_.query.push_str(field);
                            } else {
                                bail!("indexed element must be a string")
                            }
                        } else {
                            let field = key.to_string();
                            if let Some(dir) = value.str() {
                                write!(self_.query, "{field} {dir}")?;
                            } else {
                                bail!("keyed element value must be a string")
                            }
                        }
                    }
                }
                else if let Some(bcr) = fields
                    .object()
                    .and_then(ZendClassObject::<ByClauseRendered>::from_zend_obj)
                {
                    let mut clause = String::from("ORDER BY ");
                    bcr.write_sql_to(&mut clause, &self_.driver_inner.settings)?;
                    self_.query.push_str(&clause);
                } else {
                    bail!("illegal order_by() argument")
                }
                Ok(self_)
            }

            /// Appends a `GROUP BY` clause to the query.
            ///
            /// # Arguments
            /// * `fields` - Either a raw string or a `ByClauseRendered` object.
            ///
            /// # Exceptions
            /// Throws an exception if the argument is not a string or a supported object.
            fn group_by<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                fields: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                if let Some(str) = fields.str() {
                    write!(self_.query, "GROUP BY {str}")?;
                }
                else if let Some(array) = fields.array() {
                    self_.query.push_str("GROUP BY ");
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str(", ");
                        }
                        if let ArrayKey::Long(_) = key {
                            if let Some(field) = value.str() {
                                self_.query.push_str(field);
                            } else {
                                bail!("indexed element must be a string")
                            }
                        } else {
                            let field = key.to_string();
                            if let Some(dir) = value.str() {
                                write!(self_.query, "{field} {dir}")?;
                            } else {
                                bail!("keyed element value must be a string")
                            }
                        }
                    }
                }
                else if let Some(bcr) = fields
                    .object()
                    .and_then(ZendClassObject::<ByClauseRendered>::from_zend_obj)
                {
                    let mut clause = String::from("GROUP BY ");
                    bcr.write_sql_to(&mut clause, &self_.driver_inner.settings)?;
                    self_.query.push_str(&clause);
                } else {
                    bail!("illegal group_by() argument")
                }
                Ok(self_)
            }


            /// Appends a `FOR UPDATE` locking clause to the query.
            ///
            /// This clause is used in `SELECT` statements to lock the selected rows
            /// for update, preventing other transactions from modifying or acquiring
            /// locks on them until the current transaction completes.
            ///
            /// # Example
            /// ```php
            /// $builder->select("*")->from("users")->for_update();
            /// // SELECT * FROM users FOR UPDATE
            /// ```
            ///
            /// # Notes
            /// - Only valid in transactional contexts (e.g., PostgreSQL, MySQL with InnoDB).
            /// - Useful for implementing pessimistic locking in concurrent systems.
            ///
            /// # Returns
            /// The query builder with `FOR UPDATE` appended.
            fn for_update(
                self_: &mut ZendClassObject<$struct>,
            ) -> anyhow::Result<&mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                self_.query.push_str("\nFOR UPDATE");
                Ok(self_)
            }

            /// Appends a `FOR SHARE` locking clause to the query.
            ///
            /// This clause is used in `SELECT` statements to acquire shared locks
            /// on the selected rows, allowing concurrent transactions to read but
            /// not modify the rows until the current transaction completes.
            ///
            /// # Example
            /// ```php
            /// $builder->select("*")->from("documents")->for_share();
            /// // SELECT * FROM documents FOR SHARE
            /// ```
            ///
            /// # Notes
            /// - Supported in PostgreSQL and some MySQL configurations.
            /// - Ensures rows cannot be updated or deleted by other transactions while locked.
            ///
            /// # Returns
            /// The query builder with `FOR SHARE` appended.
            fn for_share(
                self_: &mut ZendClassObject<$struct>,
            ) -> anyhow::Result<&mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                self_.query.push_str("\nFOR SHARE");
                Ok(self_)
            }

            /// Appends an `INSERT INTO` clause to the query.
            ///
            /// # Arguments
            /// * `table` - The name of the target table.
            ///
            /// # Example
            /// ```php
            /// $builder->insertInto("users");
            /// ```
            fn insert_into<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "INSERT INTO {table}")?;
                Ok(self_)
            }


            /// Appends an `REPLACE INTO` clause to the query.
            ///
            /// # Arguments
            /// * `table` - The name of the target table.
            ///
            /// # Example
            /// ```php
            /// $builder->insertInto("users");
            /// ```
            fn replace_into<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "REPLACE INTO {table}")?;
                Ok(self_)
            }

            /// Appends a `VALUES` clause to the query.
            ///
            /// # Arguments
            /// * `values` - Can be:
            ///     - An associative array: `["name" => "John", "email" => "j@example.com"]`
            ///     - A list of `[column, value]` pairs: `[["name", "John"], ["email", "j@example.com"]]`
            ///     - A raw SQL string or a subquery builder
            ///
            /// # Example
            /// ```php
            /// $builder->insert("users")->values(["name" => "John", "email" => "j@example.com"]);
            /// ```
            fn values<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                values: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                use ext_php_rs::types::ArrayKey;

                if let Some(array) = values.array() {
                    let mut columns = Vec::new();
                    let mut placeholders = Vec::new();
                    let mut param_index = 0;

                    // Case 1: associative array or list of [col, val]
                    for (key, val) in array.iter() {
                        match key {
                            ArrayKey::Long(_) => {
                                // Expecting list of [col, val]
                                let arr = val.array().ok_or_else(|| anyhow!("each element must be [column, value]"))?;
                                if arr.len() != 2 {
                                    bail!("each sub-array must be [column, value]");
                                }
                                let column = arr.get_index(0).and_then(Zval::str).ok_or_else(|| anyhow!("first element must be string"))?;
                                let value = arr.get_index(1).and_then(ParameterValue::from_zval).ok_or_else(|| anyhow!("invalid value"))?;
                                columns.push(column.to_string());
                                placeholders.push((String::from("0"), value));
                            }
                            _ => {
                                // Associative array
                                let column = key.to_string();
                                let value = ParameterValue::from_zval(&val).ok_or_else(|| anyhow!("invalid value"))?;
                                columns.push(column);
                                placeholders.push((param_index.to_string(), value));
                                param_index += 1;
                            }
                        }
                    }

                    if columns.is_empty() {
                        bail!("values() requires at least one column-value pair");
                    }

                    write!(self_.query, "\n({})\nVALUES (", columns.join(", "))?;
                    for i in 0..placeholders.len() {
                        if i > 0 {
                            self_.query.push_str(", ");
                        }
                        self_.query.push('?');
                    }
                    self_.query.push(')');
                    self_.parameters.extend(
                        placeholders
                            .into_iter()
                            .enumerate()
                            .map(|(i, (_, v))| (format!("values__{i}"), v)),
                    );
                    Ok(self_)
                } else if let Some(str) = values.str() {
                    // Case 2: raw subquery string
                    write!(self_.query, "\n{str}")?;
                    Ok(self_)
                } else if let Some($struct { query, parameters, .. }) = values
                    .object()
                    .and_then(ZendClassObject::<$struct>::from_zend_obj)
                    .and_then(|x| x.obj.as_ref())
                {
                    // Case 3: subquery builder
                    write!(self_.query, "\n({})", query.indent_sql(true))?;
                    self_.parameters.extend(parameters.clone());
                    Ok(self_)
                } else {
                    bail!("values() expects array, string or Builder");
                }
            }

            /// Appends a `TRUNCATE TABLE` statement to the query.
            ///
            /// This command removes all rows from the specified table quickly and efficiently.
            /// It is faster than `DELETE FROM` and usually does not fire triggers or return affected row counts.
            ///
            /// # Arguments
            /// * `table` – The name of the table to truncate.
            ///
            /// # Example
            /// ```php
            /// $builder->truncate_table("users");
            /// // TRUNCATE TABLE users
            /// ```
            ///
            /// # Notes
            /// - May require elevated privileges depending on the database.
            /// - This method can be chained with other query builder methods.
            ///
            /// # Errors
            /// Returns an error if appending the SQL fragment fails (e.g., formatting error).
            fn truncate_table<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                table: &str,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_._write_op_guard()?;
                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }
                write!(self_.query, "TRUNCATE TABLE {table}")?;
                Ok(self_)
            }

            /// Finalizes the query by appending a semicolon (`;`).
            ///
            /// This method is optional. Most databases do not require semicolons in prepared queries,
            /// but you may use it to explicitly terminate a query string.
            ///
            /// # Example
            /// ```php
            /// $builder->select("*")->from("users")->end();
            /// // SELECT * FROM users;
            /// ```
            ///
            /// # Returns
            /// The builder instance after appending the semicolon.
            ///
            /// # Notes
            /// - Only appends the semicolon character; does not perform any execution.
            /// - Useful when exporting a full query string with a terminating symbol (e.g., for SQL scripts).
            fn end<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                _table: &str,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                self_.query.push(';');
                Ok(self_)
            }

            /// Appends multiple rows to the `VALUES` clause of an `INSERT` statement.
            ///
            /// Each row must be:
            /// - an ordered list of values (indexed array),
            /// - or a map of column names to values (associative array) — only for the first row, to infer column order.
            ///
            /// # Arguments
            /// * `rows` – A sequential array of rows (arrays of values).
            ///
            /// # Example
            /// ```php
            /// $builder->insert("users")->values_many([
            ///     ["Alice", "alice@example.com"],
            ///     ["Bob", "bob@example.com"]
            /// ]);
            ///
            /// $builder->insert("users")->values_many([
            ///     ["name" => "Alice", "email" => "alice@example.com"],
            ///     ["name" => "Bob",   "email" => "bob@example.com"]
            /// ]);
            /// ```
            fn values_many<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                rows: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                use ext_php_rs::types::ArrayKey;

                let rows_array = rows
                    .array()
                    .ok_or_else(|| anyhow!("values_many() expects an array of rows"))?;

                if rows_array.len() == 0 {
                    bail!("values_many() array is empty");
                }

                let mut columns: Vec<String> = Vec::new();
                let mut all_placeholders: Vec<Vec<ParameterValue>> = Vec::new();

                for (row_index, (_, row_val)) in rows_array.iter().enumerate() {
                    let row = row_val
                        .array()
                        .ok_or_else(|| anyhow!("each row must be an array"))?;

                    if row_index == 0 {
                        // Infer column names or count
                        for (k, _) in row.iter() {
                            match k {
                                ArrayKey::Long(_) => columns.push(format!("col{}", columns.len() + 1)), // dummy name
                                _ => columns.push(k.to_string()),
                            }
                        }
                    } else if row.len() != columns.len() {
                        bail!("row #{row_index} has inconsistent column count");
                    }

                    let mut placeholder_row = Vec::with_capacity(columns.len());
                    for (_, v) in row.iter() {
                        let pv = ParameterValue::from_zval(&v).ok_or_else(|| {
                            anyhow!("row #{row_index}: failed to convert value to parameter")
                        })?;
                        placeholder_row.push(pv);
                    }
                    all_placeholders.push(placeholder_row);
                }

                // Append header
                write!(self_.query, "\n({})\nVALUES", columns.join(", "))?;

                // Append each VALUES (...)
                for (i, row) in all_placeholders.iter().enumerate() {
                    if i > 0 {
                        self_.query.push(',');
                    }
                    self_.query.push_str("\n(");
                    for j in 0..row.len() {
                        if j > 0 {
                            self_.query.push_str(", ");
                        }
                        self_.query.push('?');
                    }
                    self_.query.push(')');
                }

                // Register parameters
                for (i, row) in all_placeholders.into_iter().enumerate() {
                    for (j, val) in row.into_iter().enumerate() {
                        let name = format!("values__{}_{}", i, j);
                        self_.parameters.insert(name, val);
                    }
                }

                Ok(self_)
            }

            /// Appends a `RETURNING` clause to the query.
            ///
            /// # Arguments
            /// * `fields` - A string or array of column names to return.
            ///
            /// # Supported formats
            /// ```php
            /// $builder->returning("id");
            /// $builder->returning(["id", "name"]);
            /// ```
            ///
            /// # Notes
            /// - This is mainly supported in PostgreSQL.
            /// - Use with `INSERT`, `UPDATE`, or `DELETE`.
            fn returning<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                fields: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                use ext_php_rs::types::ArrayKey;

                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }

                self_.query.push_str("RETURNING ");

                if let Some(field_str) = fields.str() {
                    self_.query.push_str(field_str);
                } else if let Some(array) = fields.array() {
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str(", ");
                        }
                        match key {
                            ArrayKey::Long(_) => {
                                let Some(field) = value.str() else {
                                    bail!("returning[] values must be strings");
                                };
                                self_.query.push_str(field);
                            }
                            _ => {
                                let alias = key.to_string();
                                let Some(expr) = value.str() else {
                                    bail!("returning[key => value] must be strings");
                                };
                                write!(self_.query, "{expr} AS {alias}")?;
                            }
                        }
                    }
                } else {
                    bail!("Argument to returning() must be a string or array");
                }

                Ok(self_)
            }

            fn from<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                from: &Zval,
                parameters: Option<HashMap<String, ParameterValueWrapper>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                use ext_php_rs::types::ArrayKey;

                if !self_.query.is_empty() {
                    self_.query.push('\n');
                }

                self_.query.push_str("FROM ");

                if let Some(from_str) = from.str() {
                    self_._append(&from_str.indent_sql(false), parameters, "from")?;
                } else if let Some(from_array) = from.array() {
                    if parameters.is_some() {
                        bail!("parameters argument cannot be used with array-based `from()`");
                    }

                    let parts: Vec<_> = from_array
                        .iter()
                        .map(|(key, value)| {
                            let alias = match key {
                                ArrayKey::Long(_) => None,
                                _ => Some(key.to_string())
                            };
                            let source = value
                                .str()
                                .ok_or_else(|| anyhow!("`from` value must be a string"))?;

                            Ok(match alias {
                                Some(alias) => (format!("{source} AS {alias}"), ParamsMap::default()),
                                None => (source.to_string(), ParamsMap::default()),
                            })
                        })
                        .collect::<anyhow::Result<_>>()?;

                    for (i, (part, params)) in parts.into_iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str(", ");
                        }
                        self_._append(&part.indent_sql(true), Some(params), "from")?;
                    }
                } else {
                    bail!("illegal `from()` argument: must be string or array");
                }

                Ok(self_)
            }

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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(&self.query, parameters, None)
            }

            /// Same as `query_grouped_dictionary`, but forces rows to be decoded as associative arrays.
            pub fn query_grouped_dictionary_assoc(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_grouped_dictionary(&self.query, parameters, Some(true))
            }

            /// Same as `query_grouped_dictionary`, but forces rows to be decoded as PHP objects.
            pub fn query_grouped_dictionary_obj(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner.query_grouped_column_dictionary(
                    &self.query,
                    parameters,
                    Some(false),
                )
            }

            /// Executes the prepared query with optional parameters.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Number of affected rows
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the SQL query is invalid or fails to execute (e.g., due to syntax error, constraint violation, or connection issue);
            /// - parameters contain unsupported types or fail to bind correctly;
            /// - the runtime fails to execute the query (e.g., task panic or timeout).
            pub fn execute(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<u64> {
                self.driver_inner.execute(self.query.as_str(), parameters)
            }

            /// Executes the prepared query and returns a single result.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Single row as array or object depending on config
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or execution fails;
            /// - a parameter cannot be bound or has incorrect type;
            /// - the row contains unsupported database types;
            /// - conversion to PHP object fails.
            pub fn query_row(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner.query_row(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns one row as an associative array.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            pub fn query_row_assoc(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_row(&self.query, parameters, Some(true))
            }

            /// Executes the prepared query and returns one row as an object.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            pub fn query_row_obj(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_row(&self.query, parameters, Some(false))
            }

            /// Executes an SQL query and returns a single result, or `null` if no row matched.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Single row as array or object depending on config
            ///
            /// # Exceptions
            /// Throws an exception if the query fails for reasons other than no matching rows.
            /// For example, syntax errors, type mismatches, or database connection issues.
            pub fn query_maybe_row(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_maybe_row(&self.query, parameters, None)
            }

            /// Executes the SQL query and returns a single row as a PHP associative array, or `null` if no row matched.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// The result row as an associative array (`array<string, mixed>` in PHP), or `null` if no matching row is found.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query is invalid or fails to execute;
            /// - parameters are invalid or cannot be bound;
            /// - the row contains unsupported or unconvertible data types.
            pub fn query_maybe_row_assoc(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_maybe_row(&self.query, parameters, Some(true))
            }

            /// Executes an SQL query and returns a single row as a PHP object, or `null` if no row matched.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// The result row as a `stdClass` PHP object, or `null` if no matching row is found.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query is invalid or fails to execute;
            /// - parameters are invalid or cannot be bound;
            /// - the row contains unsupported or unconvertible data types.
            pub fn query_maybe_row_obj(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Zval> {
                self.driver_inner
                    .query_maybe_row(&self.query, parameters, Some(false))
            }

            /// Executes the SQL query and returns the specified column values from all result rows.
            ///
            /// # Arguments
            /// - `query`: SQL query string to execute.
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            /// - `column`: Optional column name or index to extract.
            ///
            /// # Returns
            /// An array of column values, one for each row.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - the query fails to execute;
            /// - the specified column is not found;
            /// - a column value cannot be converted to PHP.
            pub fn query_column(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(&self.query, parameters, column, None)
            }

            /// Executes the SQL query and returns the specified column values from all rows in associative array mode.
            ///
            /// # Arguments
            /// - `query`: SQL query string.
            /// - `parameters`: Optional named parameters.
            /// - `column`: Column index or name to extract.
            ///
            /// # Returns
            /// An array of column values (associative arrays for structured data).
            ///
            /// # Exceptions
            /// Same as `query_column`.
            pub fn query_column_assoc(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(&self.query, parameters, column, Some(true))
            }

            /// Executes the SQL query and returns the specified column values from all rows in object mode.
            ///
            /// # Arguments
            /// - `parameters`: Optional named parameters.
            /// - `column`: Column index or name to extract.
            ///
            /// # Returns
            /// An array of column values (objects for structured data).
            ///
            /// # Exceptions
            /// Same as `query_column`.
            pub fn query_column_obj(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
                column: Option<ColumnArgument>,
            ) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner
                    .query_column(&self.query, parameters, column, Some(false))
            }

            /// Executes the prepared query and returns all rows.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Returns
            /// Array of rows as array or object depending on config
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or fails to execute;
            /// - parameter binding fails;
            /// - row decoding fails due to an unsupported or mismatched database type;
            /// - conversion to PHP values fails (e.g., due to memory or encoding issues).
            pub fn query_all(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner.query_all(&self.query, parameters, None)
            }

            /// Executes the prepared query and returns all rows as associative arrays.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or fails to execute;
            /// - parameter binding fails;
            /// - row decoding fails due to an unsupported or mismatched database type;
            /// - conversion to PHP values fails (e.g., due to memory or encoding issues).
            pub fn query_all_assoc(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner
                    .query_all(&self.query, parameters, Some(true))
            }

            /// Executes the prepared query and returns all rows as objects.
            ///
            /// # Arguments
            /// - `parameters`: Optional array of indexed/named parameters to bind.
            ///
            /// # Exceptions
            /// Throws an exception if:
            /// - SQL query is invalid or fails to execute;
            /// - parameter binding fails;
            /// - row decoding fails due to an unsupported or mismatched database type;
            /// - conversion to PHP values fails (e.g., due to memory or encoding issues).
            pub fn query_all_obj(
                &self,
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<Vec<Zval>> {
                self.driver_inner
                    .query_all(&self.query, parameters, Some(false))
            }
        }
    };
}
