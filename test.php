<?php
//$sql = new Sqlx("postgres://localhost/postgres");
$sql = new Sqlx([
    'url' => 'postgres://localhost/postgres',
    'persistent_name' => 'test',
    'assoc_arrays' => false
]);
var_dump($sql->queryOne('select $1::json', ['{"a":1,"b":2}']));

//$sql->query('SELECT $1 {{WHERE id = $id}}', [123]);
//var_dump($sql->queryOne('SELECT ? IN (?) as result', [123, [111, 123, 333]]));
//var_dump($sql->prepare('SELECT ? IN (?) as result')->execute([123, [111, 123, 333]]));
//var_dump($sql->queryOne('SELECT 1 + $1 {{WHERE id = $id}}', [1, 'id']));
//var_dump()$sql->prepare("SELECT 1212313 as test")->queryOne())
//ORDER BY :order_by(id, username, birthdate) :order_dir(ASC)