use ext_php_rs::ZvalConvert;

/// Represents a column reference, either by numeric index or string name.
///
/// Used to specify how to extract a value from a SQL row result.
#[derive(Debug, ZvalConvert)]
pub enum ColumnArgument<'a> {
    /// Column by numeric index (0-based).
    Index(usize),
    /// Column by name.
    Name(&'a str),
}
