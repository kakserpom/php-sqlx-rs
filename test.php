<?php
$sql = new Sqlx();
//$sql->query('SELECT $1 {{WHERE id = $id}}', [123]);
var_dump($sql->queryOne('SELECT ? IN (?) as result', [123, [111, 123, 333]]));
