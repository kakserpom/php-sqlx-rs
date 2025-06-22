use criterion::{Criterion, criterion_group, criterion_main};
use php_sqlx::ast::{Ast, ParsingSettings, RenderingSettings};
use std::collections::HashMap;
use std::hint::black_box;
use std::sync::LazyLock;

// Postgres settings
const PARSING_SETTINGS: LazyLock<ParsingSettings> = LazyLock::new(|| ParsingSettings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: false,
    comment_hash: false,
});
const RENDERING_SETTINGS: LazyLock<RenderingSettings> = LazyLock::new(|| RenderingSettings {
    column_backticks: false,
    dollar_sign_placeholders: true,
});

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
    c.bench_function("Ast::parse_small", |b| {
        b.iter(|| {
            let _res = black_box(Ast::parse(QUERY_SMALL, &PARSING_SETTINGS));
        })
    });
    c.bench_function("Ast::parse_big", |b| {
        b.iter(|| {
            let _res = black_box(Ast::parse(QUERY_BIG, &PARSING_SETTINGS));
        })
    });

    c.bench_function("Ast::render_big", |b| {
        let ast = Ast::parse(QUERY_BIG, &PARSING_SETTINGS).unwrap();
        b.iter(|| {
            let _res = black_box(ast.render(
                HashMap::from([
                    ("status", "accepted"),
                    ("created_after", "1111111"),
                    ("limit", "10"),
                ]),
                &RENDERING_SETTINGS,
            ));
        })
    });
}

criterion_group!(benches, bench_ast);
criterion_main!(benches);
