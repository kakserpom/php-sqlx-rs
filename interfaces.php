<?php
namespace Sqlx
{
   /**
    * Represents a database driver.
    *
    * Implementations of this interface provide a connection to a specific database backend
    * (e.g. PostgreSQL, MySQL, SQL Server) and expose query building and execution methods.
    */
   interface DriverInterface {}

   /**
    * Represents a prepared query that can be executed or fetched.
    *
    * Prepared queries allow for bound parameter execution and safe SQL interaction.
    */
   interface PreparedQueryInterface {}

   /**
    * Provides factory methods to obtain query builders and connections.
    *
    * Used to abstract the process of creating read/write query builders and handling
    * master/slave connections.
    */
   interface FactoryInterface {}

   /**
    * Base interface for all query builders.
    *
    * Defines core query-building methods like `from()`, `where()`, `groupBy()`, `orderBy()`, etc.
    * Can be used as a type hint for logic that is agnostic to query type (read/write).
    */
   interface QueryBuilderInterface {}

   /**
    * Extends QueryBuilderInterface with read-only query support.
    *
    * Adds methods like `select()`, `limit()`, `offset()`, and locking clauses like `forUpdate()`.
    */
   interface ReadQueryBuilderInterface extends QueryBuilderInterface {}

   /**
    * Extends ReadQueryBuilderInterface with write capabilities.
    *
    * Adds methods like `insertInto()`, `update()`, `deleteFrom()`, `set()`, etc.
    */
   interface WriteQueryBuilderInterface extends ReadQueryBuilderInterface {}
}