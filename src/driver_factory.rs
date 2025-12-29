use crate::dbms::mssql::MssqlDriver;
use crate::dbms::mysql::MySqlDriver;
use crate::dbms::postgres::PgDriver;
use crate::error::Error as SqlxError;
use crate::options::{DriverOptions, DriverOptionsArg};
use ext_php_rs::convert::IntoZval;
use ext_php_rs::prelude::*;
use ext_php_rs::types::{ZendClassObject, Zval};
use url::Url;

pub fn build(module: ModuleBuilder) -> ModuleBuilder {
    module.class::<DriverFactory>().class::<DriverOptions>()
}

#[php_class]
#[php(name = "Sqlx\\DriverFactory")]
pub struct DriverFactory;

#[php_impl]
impl DriverFactory {
    /// Creates a driver instance based on the DSN or config array.
    ///
    /// # Arguments
    /// - `$config`: Either a DSN string (`"mysql://..."`, `"pgsql://..."`, etc.) or an array of driver options.
    ///
    /// # Example
    /// ```php
    /// $driver = Sqlx\DriverFactory::make("postgres://user:pass@localhost/db");
    /// $driver = Sqlx\DriverFactory::make([
    ///     Sqlx\DriverOptions::OPT_URL => "mysql://root@localhost/test",
    ///     Sqlx\DriverOptions::OPT_ASSOC_ARRAYS => true
    /// ]);
    /// ```
    ///
    /// # Returns
    /// Instance of `Sqlx\PgDriver`, `Sqlx\MySqlDriver`, or `Sqlx\MssqlDriver`
    pub fn make(url_or_options: DriverOptionsArg) -> crate::error::Result<Zval> {
        let options = url_or_options.parse()?;
        let url = Url::parse(options.url.as_ref().ok_or(SqlxError::UrlRequired)?)?;
        let scheme = url.scheme();
        match scheme.to_lowercase().as_str() {
            "postgres" | "postgresql" | "pgsql" => {
                Ok(ZendClassObject::new(PgDriver::new(options)?)
                    .into_zval(false)
                    .map_err(|err| SqlxError::Conversion {
                        message: format!("{err}"),
                    })?)
            }
            "mysql" => Ok(ZendClassObject::new(MySqlDriver::new(options)?)
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err}"),
                })?),
            "mssql" | "sqlserver" => Ok(ZendClassObject::new(MssqlDriver::new(options)?)
                .into_zval(false)
                .map_err(|err| SqlxError::Conversion {
                    message: format!("{err}"),
                })?),
            _ => Err(SqlxError::Other(format!("Unsupported scheme: {scheme}"))),
        }
    }
}
