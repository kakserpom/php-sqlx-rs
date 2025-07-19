namespace Sqlx;

interface DriverInterface {}
interface PreparedQueryInterface {}
interface FactoryInterface {}

interface QueryBuilderInterface {}
interface ReadQueryBuilderInterface extends QueryBuilderInterface {}
interface WriteQueryBuilderInterface extends ReadQueryBuilderInterface {}
