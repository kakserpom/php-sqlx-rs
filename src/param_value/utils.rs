use crate::param_value::ParameterValue;
use anyhow::bail;
use sqlx_oldapi::database::HasArguments;
use sqlx_oldapi::query::Query;
use sqlx_oldapi::{Database, Encode, Type};

/// Binds a list of `ParameterValue` items to an `SQLx` query.
///
/// This function recursively traverses and binds all primitive values from the input slice,
/// supporting nested arrays and objects. Each primitive is passed to `query.bind()`, in left-to-right order.
///
/// # Supported types
/// - `Str`, `Int`, `Float`, `Bool` — bound directly
/// - `Array`, `Object` — recursively expanded and flattened into positional bindings
///
/// # Unsupported types
/// - `ByClauseRendered`, `SelectClauseRendered`, `PaginateClauseRendered`, and `Null` are not bindable and will
///   result in an error
///
/// # Errors
/// Returns an `anyhow::Error` if an unsupported value is encountered or if recursive binding fails.
///
/// # Example
/// ```rust
///  use sqlx_oldapi::Postgres;
///  use php_sqlx::param_value::{utils::bind_values, ParameterValue};
///  let query = sqlx_oldapi::query::<Postgres>("SELECT * FROM users WHERE id = $1 AND active = $2");
///  let values = &[ParameterValue::Int(1), ParameterValue::Bool(true)];
///  let query = bind_values(query, values).expect("Cannot bind values");
/// ```
pub fn bind_values<'a, D: Database>(
    query: Query<'a, D, <D as HasArguments<'a>>::Arguments>,
    values: &'a [ParameterValue],
) -> anyhow::Result<Query<'a, D, <D as HasArguments<'a>>::Arguments>>
where
    f64: Type<D>,
    f64: Encode<'a, D>,
    i64: Type<D>,
    i64: Encode<'a, D>,
    bool: Type<D>,
    bool: Encode<'a, D>,
    String: Type<D>,
    String: Encode<'a, D>,
{
    fn walker<'a, D: Database>(
        q: Query<'a, D, <D as HasArguments<'a>>::Arguments>,
        value: &'a ParameterValue,
    ) -> anyhow::Result<Query<'a, D, <D as HasArguments<'a>>::Arguments>>
    where
        f64: Type<D>,
        f64: Encode<'a, D>,
        i64: Type<D>,
        i64: Encode<'a, D>,
        bool: Type<D>,
        bool: Encode<'a, D>,
        String: Type<D>,
        String: Encode<'a, D>,
    {
        Ok(match value {
            ParameterValue::Json(pv) => q.bind(pv.to_json()?),
            ParameterValue::String(s) => q.bind(s),
            ParameterValue::Int(s) => q.bind(s),
            ParameterValue::Bool(s) => q.bind(s),
            ParameterValue::Float(s) => q.bind(s),
            ParameterValue::Array(s) => s.iter().try_fold(q, walker)?,
            // @TODO: values()?
            ParameterValue::Object(s) => s.values().try_fold(q, walker)?,
            ParameterValue::ByClauseRendered(_)
            | ParameterValue::SelectClauseRendered(_)
            | ParameterValue::PaginateClauseRendered(_)
            | ParameterValue::Builder(_)
            | ParameterValue::Null => bail!("Internal error: cannot bind parameter of this type"),
        })
    }

    values.iter().try_fold(query, walker)
}
