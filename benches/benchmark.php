<?php
/**
 */
class AnnotatedBench
{
    private $queryBig;
    private $driver;
    public function __construct() {
        $this->driver = new Sqlx\PgDriver([
            Sqlx\DriverOptions::OPT_URL => 'postgres://localhost/test',
        ]);
        $this->queryBig = "SELECT
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
{{ OFFSET :offset }} ";
    }
    /**
     * @Revs(1000000)
     */
    public function benchDryBig(): void
    {
        [$sql, $values] = $this->driver->dry($this->queryBig, ["status" => "accepted", "created_after" => "1111111", "limit" => "10"]);
    }
}