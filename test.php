<?php
//$sql = new Sqlx("postgres://localhost/postgres");
$sql = new Sqlx(['url' => 'postgres://localhost/postgres']);
//$sql->query('SELECT $1 {{WHERE id = $id}}', [123]);
//var_dump($sql->queryOne('SELECT ? IN (?) as result', [123, [111, 123, 333]]));
//var_dump($sql->prepare('SELECT ? IN (?) as result')->execute([123, [111, 123, 333]]));
var_dump($sql->queryOne('SELECT 1 + $1 {{WHERE id = $id}}', [1]));
$prepared = $sql->prepare("SELECT 2 + $1");
var_dump($prepared->queryOne([2]));
