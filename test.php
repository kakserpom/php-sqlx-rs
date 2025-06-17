<?php
$driver = new Sqlx\PgDriver([
    Sqlx\DriverOptions::OPT_URL => 'postgres://localhost/test',
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'test',
]);
var_dump($driver->queryRow('SELECT NULL::BIGINT as nul'));
/*
var_dump($driver->queryRow(
    'SELECT name, * FROM people WHERE name in (?)',
     [
        ["Peter", "John", "Jane"],
    ]
));
return;*/
$driver = new Sqlx\MySqlDriver([
    Sqlx\DriverOptions::OPT_URL => 'mysql://root@localhost/mysql',
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'test',
]);

var_dump($driver->queryValue('SELECT "131231"'));
return;

var_dump($driver->queryRow(
    'SELECT name, * FROM people WHERE name in (?)',
     [
        ["Peter", "John", "Jane"],
    ]
));
return;
/* Output:
array(1) {
  ["John"]=>
  object(stdClass)#2 (2) {
    ["name"]=>
    string(4) "John"
    ["age"]=>
    int(22)
  }
}

*/
var_dump($driver->queryColumnDictionary(
    'SELECT name, age FROM people WHERE name in (?)',
     [
        ["Peter", "John", "Jane"],
    ]
));
/* Output:
array(1) {
  ["John"]=>
  int(22)
}
*/

return;

var_dump($driver->queryValue('SELECT ((1::BIGINT << 62) - 1) * 2 + 1'));
return;


$orderBy = new Sqlx\OrderBy([
    'name',
    'created_at',
    'posts' => 'COUNT(posts.*)'
]);

// Equivalent to: SELECT * FROM users ORDER BY ORDER BY name ASC, COUNT(posts.*) DESC
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', ['order_by' => $orderBy([
    ['zzzz', Sqlx\OrderBy::ASC],
])]);


return;
//var_dump($driver->queryValue('select $1::json', ['{"foo": ["bar", "baz"]}']));
// Equivalent to: SELECT * FROM users ORDER BY ORDER BY name ASC, COUNT(posts.*) DESC
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', ['order_by' => $orderBy([
    ['name', Sqlx\OrderBy::ASC],
    ['posts', Sqlx\OrderBy::DESC]
])]);
return;
var_dump($driver->queryRow(
    'SELECT $1::json AS col',
    ['{"foo": ["bar", "baz"]}']
)->col->foo[0]);
return;
/*$order_by = new Sqlx\OrderBy(["name", "age", "total_posts" => "COUNT(posts.*)"])
    ->apply([["age","desc"], ["total_posts", "ASC"]]);
var_dump($order_by);*/

//var_dump(new RenderedOrderBy());

$order_by = new Sqlx\OrderBy([
    "name",
    "age",
]);
var_dump($driver->queryAll(
    'SELECT * FROM people WHERE name in (?) ORDER BY :order_by LIMIT :limit',
     [
        ["Peter", "John", "Jane"],
                "limit" => 1,
        "order_by" => $order_by([
            ["age","desc"],
            ["total_posts", "ASC"],
        ])
    ]
));
/*array(2) {
    [0]=>
    string(85) "SELECT * FROM people WHERE name in ($1, $2, $3) ORDER BY age DESC, COUNT(posts.*) ASC"
    [1]=>
    array(3) {
      [0]=>
      string(5) "Peter"
      [1]=>
      string(4) "John"
      [2]=>
      string(4) "Jane"
    }
  }
*/

//var_dump($driver->queryRow('select $1::json', ['{"foo": ["bar", "baz"]}']));
//var_dump($driver->queryRowAssoc('select $1::json', ['{"foo": ["bar", "baz"]}']));
//var_dump($driver->queryRowAssoc('select 9223372036854775807'));
//var_dump($driver->queryRow('SELECT ((1::BIGINT << 62) - 1) * 2 + 1 largest')->largest);
// int(9223372036854775807)

//$driver->query('SELECT $1 {{WHERE id = $id}}', [123]);
//var_dump($driver->queryRow('SELECT ? IN (?) as result', [123, [111, 123, 333]]));
//var_dump($driver->prepare('SELECT ? IN (?) as result')->execute([123, [111, 123, 333]]));
//var_dump($driver->queryRow('SELECT 1 + $1 {{WHERE id = $id}}', [1, 'id']));
//var_dump()$driver->prepare("SELECT 1212313 as test")->queryRow())
//ORDER BY :order_by(id, username, birthdate) :order_dir(ASC)