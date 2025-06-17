use criterion::{Criterion, criterion_group, criterion_main};
use php_sqlx::driver::postgres::ast::PgAst;
use std::collections::HashMap;
use std::hint::black_box;

const QUERY_SMALL: &str = "SELECT id, name, meta
FROM users
WHERE TRUE
  {{ AND status = $status }}
  {{ AND created_at >= $since }}
  {{ AND id IN (?) }}
ORDER BY $order_by
LIMIT :limit";
const QUERY_BIG: &str = "SELECT 
    u.id, 
    u.name, 
    u.email, 
    COUNT(p.id) AS post_count,
    jsonb_agg(jsonb_build_object('id', p.id, 'title', p.title)) AS posts
FROM users u
LEFT JOIN posts p ON p.user_id = u.id
WHERE TRUE
    {{ AND u.status = $status }}
    {{ AND u.role IN (?) }}
    {{ AND (
        {{ u.created_at >= $created_after
            {{ AND u.created_at <= $created_before }}
         }}
    ) }}
GROUP BY u.id
{{ ORDER BY $order_by }}
{{ LIMIT :limit }}
{{ OFFSET :offset }}";

fn bench_pg_ast(c: &mut Criterion) {
    c.bench_function("PgAst::parse_small", |b| {
        b.iter(|| {
            let _res = black_box(PgAst::parse(QUERY_SMALL));
        })
    });
    c.bench_function("PgAst::parse_big", |b| {
        b.iter(|| {
            let _res = black_box(PgAst::parse(QUERY_BIG));
        })
    });

    c.bench_function("PgAst::render_big", |b| {
        let ast = PgAst::parse(QUERY_BIG).unwrap();
        b.iter(|| {
            let _res = black_box(ast.render(HashMap::from([
                ("status", "accepted"),
                ("created_after", "1111111"),
                ("limit", "10"),
            ])));
        })
    });
}

criterion_group!(benches, bench_pg_ast);
criterion_main!(benches);
