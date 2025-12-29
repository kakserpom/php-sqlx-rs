<?php
require __DIR__ . '/../interfaces.php';
use Sqlx\DriverFactory;
use Sqlx\SelectClause;
use function Sqlx\OR_;
use function Sqlx\JSON;
$driver = DriverFactory::make('postgres://localhost/postgres');
//var_dump(class_implements($driver->builder()));

var_dump( $driver->builder()
    ->select("*")
    ->from("tbl")
    ->where([
        'pg_sleep(?)' => [1],
    ])
);

return;
$builder = $driver->builder();
var_dump((string )$builder->insertInto("users")->set([
  "name" => "Alice",
  "created_at = NOW()"
]));

exit;

$driver->builder()->insertInto('test');

var_dump((string) $driver->builder()->having(["COUNT(id) > ?" => [JSON(["zzz" => "ffff"])]]));

$query = $driver->builder()
    ->with(
        'active_users',
        $driver->builder()
            ->select(['id', 'name', 'created_at'])
            ->from('users')
            ->where([
                ['status', '=', 'active'],
                ['deleted_at', 'IS NULL'],
            ])
    )
    ->select(
        SelectClause::allowed([
            'comment_id' => 'comments.id',
            'comment_body' => 'comments.body',
            'user_name' => 'active_users.name',
            'created_at' => 'comments.created_at',
            'num_likes' => 'COUNT(likes.id)',
        ])->input(['comment_id', 'user_name', 'comment_body', 'num_likes'])
    )
    ->from('comments')
    ->leftJoin('active_users', 'comments.user_id = active_users.id')
    ->leftJoin('likes', 'likes.comment_id = comments.id')
    ->WHERE([
        ['comments.created_at', '>', '2024-01-01'],
        ['comments.status', '=', 'published'],
        OR_([
            ['active_users.name', 'LIKE', '%john%'],
            ['active_users.name', 'ICONTAINS', 'jane'],
            ['active_users.name', 'NOT ICONTAINS', 'j_ack'],
        ]),
    ])
    ->orderBy(['comments.created_at' => 'DESC']);

echo (string) $query . PHP_EOL . PHP_EOL;

return;


$query = $driver->builder()
    ->update('comments')
    ->set([
        'timestamp' => time(),
        'counter = counter + 1',
        ['counter = counter + ?' => [1]],
    ])
    ->where([
        'id' => '1'
    ]);
var_dump((string) $query, $query->parameters());
var_dump($query->dry());
var_dump($query->dryInline());

/*class Builder {
    public $values = [];
    public $query;
    public function __invoke($query, $values = []) {
        $this->values = array_merge($this->values, $values);
        $this->query = $query;
        return $query;
    }
    public function __toString() {
        return $this->query;
    }
}
$builder = new Builder();
$q = $builder(
     'SELECT department, name FROM employees ' .
    $builder('WHERE department IN (?)')
);
var_dump($driver->queryColumnDictionary($q));*/

return;
$driver = new Sqlx\MssqlDriver([
    Sqlx\DriverOptions::OPT_URL => 'mssql://sa:S1p9M3qsVAUxHS@localhost:1433',
]);

/*$driver = new Sqlx\PgDriver([
    Sqlx\DriverOptions::OPT_URL => 'postgres://localhost/test',
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'test',
]);*/
var_dump($driver->queryAll('SELECT 123, null, $1 as c', ['zzz']));
return;

$driver->begin(function($driver) {
    var_dump($driver->insert('people', ['name' => 'John', 'age' => 30]));
    return false;
});

return;
$pagination = new Sqlx\PaginateClause;
$pagination->perPage(5);
$pagination->maxPerPage(20);

var_dump($driver->dry(
  'SELECT * FROM people ORDER by id WHERE foo = \'PAGINATE \'\':pagination\' PAGINATE :pagination', [
    'pagination' => $pagination()
  ]
));

return;

$driver = new Sqlx\MySqlDriver([
    Sqlx\DriverOptions::OPT_URL => 'mysql://root@localhost/mysql',
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'test',
]);
$orderBy = new Sqlx\ByClause([
    'name',
    'created_at',
    'random' => 'RAND()'
]);

$r = $orderBy([
     ['name', Sqlx\ByClause::ASC],
         'random'
       ]);

var_dump($r);
var_dump($driver->dry('SELECT * FROM users ORDER BY :order_by', [
  'order_by' => (object)['abc' => 'sfsdf'],
]));
return;


$driver = new Sqlx\MySqlDriver([
    Sqlx\DriverOptions::OPT_URL => 'mysql://root@localhost/mysql',
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'test',
]);
var_dump($driver->queryValue("SELECT JSON_OBJECT('name', 'John', 'age', 30)"));
return;





var_dump($driver->queryAll("SELECT '12\\'3'"));
return;




$pagination = new Sqlx\PaginateClause;
$pagination->perPage(5);
$pagination->maxPerPage(20);

var_dump($driver->dry(
  'SELECT * FROM people ORDER by id PAGINATE :pagination', [
    'pagination' => $pagination()
  ]
));
return;
$row = $driver->queryRowAssoc(
           'SELECT $1::json AS col',
           ["{'foo': ['bar', 'baz']}"]
       );

       var_dump($row);
       var_dump($row['col']['foo']);
//var_dump($col->decode()->foo[0]);
//var_dump($col->decode()->foo[0]);
return;
var_dump($driver->dry('SELECT * FROM users WHERE age >= ? AND gender IN (:a, :b, :c)', [
    20,
    'a' => [1]
]));
return;
/*var_dump($driver->dry('SELECT * FROM users WHERE age >= ? AND gender IN ?', [
    20,
    ['M','F'],
]));*/
return;

$orderBy = new Sqlx\ByClause([
    'name',
    'created_at',
    'random' => 'RANDOM()'
]);

// Equivalent to: SELECT * FROM users ORDER BY `name` ASC, RANDOM()
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', [
  'order_by' => $orderBy([
    ['name', Sqlx\ByClause::ASC],
    'random'
  ])

]);
return;
$columns = new Sqlx\SelectClause([
  'name', 'age'
]);
var_dump($driver->queryAll('SELECT :columns FROM people', [
    'columns' => $columns(['age', 'name'])
]));
/*var_dump($driver->queryColumnDictionary('SELECT
            gs AS id,
            md5(random()::TEXT) AS random_string
          FROM generate_series(1, 1000) AS gs'));*/
return;
var_dump($driver->queryColumnDictionary(
    'SELECT name, age FROM people WHERE name in (?)',
     [
        ['Peter', 'John', 'Jane'],
    ]
));
//var_dump($driver->queryRow('SELECT NULL::BIGINT as nul'));
/*
var_dump($driver->queryRow(
    'SELECT name, * FROM people WHERE name in (?)',
     [
        ['Peter', 'John', 'Jane'],
    ]
));
return;*/
$driver = new Sqlx\MySqlDriver([
    Sqlx\DriverOptions::OPT_URL => 'mysql://root@localhost/mysql',
    Sqlx\DriverOptions::OPT_PERSISTENT_NAME => 'test',
]);

return;
//var_dump($driver->queryValue('SELECT JSON_OBJECT('name', 'John', 'age', 30)'));
//return;

/*var_dump($driver->queryRow(
    'SELECT name, * FROM people WHERE name in (?)',
     [
        ['Peter', 'John', 'Jane'],
    ]
));*/
/* Output:
array(1) {
  ['John']=>
  object(stdClass)#2 (2) {
    ['name']=>
    string(4) 'John'
    ['age']=>
    int(22)
  }
}

*/
/* Output:
array(1) {
  ['John']=>
  int(22)
}
*/

return;

var_dump($driver->queryValue('SELECT ((1::BIGINT << 62) - 1) * 2 + 1'));
return;


$orderBy = new Sqlx\ByClause([
    'name',
    'created_at',
    'random' => 'RANDOM()'
]);

// Equivalent to: SELECT * FROM users ORDER BY ORDER BY name ASC, COUNT(posts.*) DESC
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', ['order_by' => $orderBy([
    ['zzzz', Sqlx\ByClause::ASC],
])]);


return;
//var_dump($driver->queryValue('select $1::json', ['{'foo': ['bar', 'baz']}']));
// Equivalent to: SELECT * FROM users ORDER BY ORDER BY name ASC, COUNT(posts.*) DESC
$driver->queryAll('SELECT * FROM users ORDER BY :order_by', ['order_by' => $orderBy([
    ['name', Sqlx\ByClause::ASC],
    ['posts', Sqlx\ByClause::DESC]
])]);
return;
var_dump($driver->queryRow(
    'SELECT $1::json AS col',
    ["{'foo': ['bar', 'baz']}"]
)->col->foo[0]);
return;
/*$order_by = new Sqlx\ByClause(['name', 'age', 'total_posts' => 'COUNT(posts.*)'])
    ->input([['age','desc'], ['total_posts', 'ASC']]);
var_dump($order_by);*/

//var_dump(new RenderedByClause());

$order_by = new Sqlx\ByClause([
    'name',
    'age',
]);
var_dump($driver->queryAll(
    'SELECT * FROM people WHERE name in (?) ORDER BY :order_by LIMIT :limit',
     [
        ['Peter', 'John', 'Jane'],
                'limit' => 1,
        'order_by' => $order_by([
            ['age','desc'],
            ['total_posts', 'ASC'],
        ])
    ]
));
/*array(2) {
    [0]=>
    string(85) 'SELECT * FROM people WHERE name in ($1, $2, $3) ORDER BY age DESC, COUNT(posts.*) ASC'
    [1]=>
    array(3) {
      [0]=>
      string(5) 'Peter'
      [1]=>
      string(4) 'John'
      [2]=>
      string(4) 'Jane'
    }
  }
*/

//var_dump($driver->queryRow('select $1::json', ['{'foo': ['bar', 'baz']}']));
//var_dump($driver->queryRowAssoc('select $1::json', ['{'foo': ['bar', 'baz']}']));
//var_dump($driver->queryRowAssoc('select 9223372036854775807'));
//var_dump($driver->queryRow('SELECT ((1::BIGINT << 62) - 1) * 2 + 1 largest')->largest);
// int(9223372036854775807)

//$driver->query('SELECT $1 {{WHERE id = $id}}', [123]);
//var_dump($driver->queryRow('SELECT ? IN (?) as result', [123, [111, 123, 333]]));
//var_dump($driver->prepare('SELECT ? IN (?) as result')->execute([123, [111, 123, 333]]));
//var_dump($driver->queryRow('SELECT 1 + $1 {{WHERE id = $id}}', [1, 'id']));
//var_dump()$driver->prepare('SELECT 1212313 as test')->queryRow())
//ORDER BY :order_by(id, username, birthdate) :order_dir(ASC)
