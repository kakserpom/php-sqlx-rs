<?php
/**
 */
class AnnotatedBench
{
    private $driver;
    public function __construct() {
        $this->driver = new Sqlx\PgDriver([
            Sqlx\DriverOptions::OPT_URL => 'postgres://localhost/postgres',
        ]);
    }
    /**
     * @Revs(1000000)
     */
    public function benchDryBig(): void
    {
        $order_by = new Sqlx\ByClause([
            "name" => "u.name",
            "email" => "u.email",
        ]);
        [$sql, $values] = $this->driver->dry("SELECT
            u.id,
            u.name,
            u.email,
            COUNT(p.id) AS post_count,
            jsonb_agg(jsonb_build_object('id', p.id, 'title', p.title)) AS posts
         FROM users u
         LEFT JOIN posts p ON p.user_id = u.id
         WHERE TRUE
            {{ AND u.status = :status }}
            {{ AND u.role IN (?) }}
            {{ AND (
                {{ u.created_at >= :created_after
                    {{ AND u.created_at <= :created_before }}
                 }}
            ) }}
         GROUP BY u.id
         {{ ORDER BY :order_by }}
         {{ LIMIT :limit }}
         {{ OFFSET :offset }}",
         ["status" => "accepted", "created_after" => "1111111", "order_by" => $order_by(["name", "desc"]), "limit" => "10"]);
    }
    /**
     * @Revs(1000)
     */
    public function benchSelect1kRows(): void
    {
        $this->driver->queryColumnDictionary("SELECT
            gs AS id,
            md5(random()::TEXT) AS random_string
          FROM generate_series(1, 1000) AS gs");
    }
}