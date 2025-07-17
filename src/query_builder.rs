use crate::paramvalue::ParamsMap;
use anyhow::{anyhow, bail};
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ArrayKey, ZendClassObject, ZendHashTable};

/// Registers the `PaginateClause` and `PaginateClauseRendered` classes
/// with the provided PHP module builder.
pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<OrClause>().function(wrap_function!(any))
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
    Item((String, Option<ParamsMap>)),
}

#[php_function]
#[php(name = "Sqlx\\any")]
pub fn any(or: &ZendHashTable) -> anyhow::Result<OrClause> {
    let mut inner = Vec::with_capacity(or.len());
    for (key, value) in or {
        if let ArrayKey::Long(_) = key {
            if let Some(value) = value.string() {
                inner.push(OrClauseItem::Item((value, None)));
            } else if let Some(or) = value
                .object()
                .and_then(ZendClassObject::<OrClause>::from_zend_obj)
                .and_then(|x| x.obj.clone())
            {
                inner.push(OrClauseItem::Nested(or));
            } else {
                bail!("element must be string or OrClause");
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
    ( $struct:ident, $class:literal, $driver_inner: ident ) => {
        use $crate::ast::Ast;
        use $crate::paramvalue::ParamsMap;
        use $crate::query_builder::{OrClause, OrClauseItem};
        use $crate::selectclause::SelectClauseRendered;
        use anyhow::anyhow;
        use anyhow::bail;
        use ext_php_rs::php_impl;
        use ext_php_rs::prelude::*;
        use ext_php_rs::types::ArrayKey;
        use ext_php_rs::types::ZendClassObject;
        use ext_php_rs::types::Zval;
        use std::collections::BTreeMap;
        use std::collections::BTreeSet;
        use std::collections::HashMap;
        use std::fmt::Debug;
        use std::fmt::Write;
        use std::sync::Arc;
        use $crate::paramvalue::ParameterValue;
        use $crate::utils::ColumnArgument;
        use ext_php_rs::convert::FromZval;

        /// A reusable prepared SQL query with parameter support. Created using `PgDriver::builder()`, shares context with original driver.
        #[php_class]
        #[php(name = $class)]
        pub struct $struct {
            pub(crate) query: String,
            pub(crate) driver_inner: Arc<$driver_inner>,
            pub(crate) placeholders: BTreeSet<String>,
            pub(crate) parameters: BTreeMap<String, ParameterValue>,
        }

        impl $struct {
            pub(crate) fn new(driver_inner: Arc<$driver_inner>) -> Self {
                Self {
                    driver_inner,
                    placeholders: Default::default(),
                    parameters: Default::default(),
                    query: Default::default(),
                }
            }

            pub fn _append_op(
                &mut self,
                left_operand: &str,
                operator: &str,
                right_operand: Option<ParameterValue>,
                placeholder_prefix: &str,
            ) -> anyhow::Result<()> {
                let op = match operator.to_ascii_lowercase().trim() {
                    "=" | "eq" => "=",
                    "!=" | "<>" | "neq" | "ne" => "!=",
                    ">" | "gt" => ">",
                    ">=" | "gte" => ">=",
                    "<" | "lt" => "<",
                    "<=" | "lte" => "<=",
                    "like" => "LIKE",
                    "not like" | "nlike" => "NOT LIKE",
                    "ilike" => "ILIKE",
                    "not ilike" | "nilike" => "NOT ILIKE",
                    "in" => "IN",
                    "not in" => "NOT IN",
                    "is null" => "IS NULL",
                    "is not null" => "IS NOT NULL",
                    _ => bail!("Operator {operator:?} is not supported"),
                };

                match op {
                    "IS NULL" | "IS NOT NULL" => {
                        if right_operand.is_some() {
                            bail!("Operator {op} must not be given a right-hand operand");
                        }
                        self._append(
                            &format!("{left_operand} {op}"),
                            None::<[(&str, ParameterValue); 0]>,
                            placeholder_prefix
                        )?;
                    }
                    "IN" | "NOT IN" => {
                        let value = right_operand.ok_or_else(|| anyhow!("Operator {op} requires a right-hand operand"))?;
                        self._append(
                            &format!("{left_operand} {op} (?)"),
                            Some([("0", value)]),
                            placeholder_prefix
                        )?;
                    }
                    _ => {
                        let value = right_operand.ok_or_else(|| anyhow!("Operator {op} requires a right-hand operand"))?;
                        self._append(
                            &format!("{left_operand} {op} ?"),
                            Some([("0", value)]),
                            placeholder_prefix
                        )?;
                    }
                }

                Ok(())
            }

            pub fn _append_or(&mut self, or: &OrClause, prefix: &str) -> anyhow::Result<()> {
                self.query.push('(');
                for (i, item) in or.inner.iter().enumerate() {
                    if i > 0 {
                        self.query.push_str(" OR ");
                    }
                    match item {
                        OrClauseItem::Item((part, parameters))  => {
                            self._append(
                                part.as_str(),
                                parameters.clone(),
                                prefix,
                            )?;
                        }
                        OrClauseItem::Nested(nested)  => {
                            self._append_or(
                                nested,
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
                I: IntoIterator<Item = (K, V)> + Debug,
                K: Into<String>,
                V: Into<ParameterValue>,
            {
                self._append_ast(&self.driver_inner.parse_query(part)?, parameters, prefix)
            }

            pub fn _append_ast<I, K, V>(
                &mut self,
                ast: &Ast,
                parameters: Option<I>,
                prefix: &str,
            ) -> anyhow::Result<()>
            where
                I: IntoIterator<Item = (K, V)> + Debug,
                K: Into<String>,
                V: Into<ParameterValue>,
            {
                fn walk(
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
                        Ast::Sql(s) => sql.push_str(s),
                        Ast::Placeholder(name) => {
                            let new_name = resolve_placeholder_name(
                                name,
                                placeholders,
                                positional_index,
                                prefix,
                            );
                            parameters_bucket.insert(
                                new_name.clone(),
                                param_map.remove(name).unwrap_or(ParameterValue::Null),
                            );
                            write!(sql, ":{new_name}")?;
                        }
                        Ast::ConditionalBlock { branches, .. } => {
                            for b in branches {
                                walk(
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
                            parameters_bucket.insert(
                                new_name.clone(),
                                param_map
                                    .remove(placeholder)
                                    .unwrap_or(ParameterValue::Null),
                            );
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
                            parameters_bucket.insert(
                                new_name.clone(),
                                param_map
                                    .remove(placeholder)
                                    .unwrap_or(ParameterValue::Null),
                            );
                            write!(sql, "PAGINATE :{}", new_name)?;
                        }
                    }
                    Ok(())
                }

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


            fn set<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                set: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                use ext_php_rs::types::ArrayKey;

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
                                        .ok_or_else(|| anyhow!("first element (field) must be string"))?;

                                    let param = array
                                        .get_index(1)
                                        .and_then(ParameterValue::from_zval)
                                        .ok_or_else(|| anyhow!("second element (value) must be valid parameter value"))?;

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


            /// Returns an array of all currently accumulated parameters.
            pub(crate) fn parameters(&self) -> BTreeMap<String, ParameterValue> {
                self.parameters.clone()
            }

            /// Appends a raw SQL fragment to the query without validation.
            ///
            /// # Arguments
            /// * `part` - The SQL string to append.
            /// * `parameters` - Optional parameters to associate with the fragment.
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
                if let Some(str) = fields.str() {
                    if !self_.query.is_empty() {
                        self_.query.push('\n');
                    }
                    write!(self_.query, "SELECT {str}")?;
                } else if let Some(scr) = fields
                    .object()
                    .and_then(ZendClassObject::<SelectClauseRendered>::from_zend_obj)
                {
                    if !self_.query.is_empty() {
                        self_.query.push('\n');
                    }

                    let mut clause = String::from("SELECT ");
                    scr.write_sql_to(&mut clause, &self_.driver_inner.settings)?;
                    self_.query.push_str(&clause);
                } else {
                    bail!("illegal select() argument")
                }
                Ok(self_)
            }

             /// Appends a `FROM` clause to the query.
             ///
             /// # Arguments
             /// * `from` - A raw string representing the source table(s).
             ///
             /// # Exceptions
             /// Throws an exception if the argument is not a string.
            fn from<'a>(
                self_: &'a mut ZendClassObject<$struct>,
                from: &Zval,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if let Some(str) = from.str() {
                    if !self_.query.is_empty() {
                        self_.query.push('\n');
                    }
                    write!(self_.query, "FROM {str}")?;
                } else {
                    bail!("illegal from() argument")
                }
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
                parameters: Option<HashMap<String, ParameterValue>>,
            ) -> anyhow::Result<&'a mut ZendClassObject<$struct>> {
                if let Some(part) = r#where.str() {
                    self_.query.push_str("\nWHERE ");
                    self_._append(part, parameters, "where")?;
                } else if let Some(array) = r#where.array() {
                    self_.query.push_str("\nWHERE ");
                    for (i, (key, value)) in array.iter().enumerate() {
                        if i > 0 {
                            self_.query.push_str(" AND ");
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
                                            .ok_or_else(|| anyhow!("first element (left operand) of #{i} must be string"))?;
                                        let operator = array.get_index(1).and_then(Zval::str)
                                            .ok_or_else(|| anyhow!("second element (operator) of #{i} must be string"))?;
                                        self_._append_op(left_operand, operator, if array_len > 2 {
                                            Some(
                                                array.get_index(2)
                                                    .and_then(ParameterValue::from_zval)
                                                    .ok_or_else(|| anyhow!("third element (value) must a valid parameter value"))?
                                            )
                                        } else {
                                            None
                                        }, "where")?;
                                    }
                                }
                                else if let Some(or) = value
                                    .object()
                                    .and_then(ZendClassObject::<OrClause>::from_zend_obj)
                                    .and_then(|x| x.obj.as_ref())
                                {
                                    //println!("{or:?}");
                                    self_._append_or(or, "where")?;
                                } else {
                                    bail!("element must be string or OrClause");
                                }
                            }
                            _ => {
                                let part = key.to_string();
                                let ast = self_.driver_inner.parse_query(&part)?;
                                if ast.has_placeholders() {
                                    let Some(parameters) = value.array() else {
                                        bail!("value must be array because the key string ({part:?}) contains placeholders: {ast:?}");
                                    };
                                    let parameters: HashMap<String, ParameterValue> =
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

            fn __to_string(&self) -> String {
                self.query.clone()
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
