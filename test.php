<?php
//$sql = new Sqlx("postgres://localhost/postgres");
$sql = new Sqlx\Driver([
    Sqlx\Driver::OPT_URL => 'postgres://localhost/test',
    Sqlx\Driver::OPT_PERSISTENT_NAME => 'test',
]);
//var_dump($sql->queryRowColumn('select $1::json', ['{"foo": ["bar", "baz"]}']));
var_dump($sql->queryColumn(
    'select $1::json as foo', ['{"foo": ["bar", "baz"]}'],
));
return;
/*$order_by = new Sqlx\OrderBy(["name", "age", "total_posts" => "COUNT(posts.*)"])
    ->apply([["age","desc"], ["total_posts", "ASC"]]);
var_dump($order_by);*/

//var_dump(new RenderedOrderBy());

$order_by = new Sqlx\OrderBy([
    "name",
    "age",
]);
var_dump($sql->queryAll(
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

//var_dump($sql->queryRow('select $1::json', ['{"foo": ["bar", "baz"]}']));
//var_dump($sql->queryRowAssoc('select $1::json', ['{"foo": ["bar", "baz"]}']));
//var_dump($sql->queryRowAssoc('select 9223372036854775807'));
//var_dump($sql->queryRow('SELECT ((1::BIGINT << 62) - 1) * 2 + 1 largest')->largest);
// int(9223372036854775807)

//$sql->query('SELECT $1 {{WHERE id = $id}}', [123]);
//var_dump($sql->queryRow('SELECT ? IN (?) as result', [123, [111, 123, 333]]));
//var_dump($sql->prepare('SELECT ? IN (?) as result')->execute([123, [111, 123, 333]]));
//var_dump($sql->queryRow('SELECT 1 + $1 {{WHERE id = $id}}', [1, 'id']));
//var_dump()$sql->prepare("SELECT 1212313 as test")->queryRow())
//ORDER BY :order_by(id, username, birthdate) :order_dir(ASC)