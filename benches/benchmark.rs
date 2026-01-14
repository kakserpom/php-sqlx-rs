use criterion::{Criterion, criterion_group, criterion_main};
use php_sqlx::ast::{Ast, IdentifierQuoteStyle, Settings, UpsertStyle};
use std::collections::HashMap;
use std::hint::black_box;

// Postgres settings
const SETTINGS: Settings = Settings {
    collapsible_in_enabled: true,
    escaping_double_single_quotes: false,
    comment_hash: false,
    column_backticks: false,
    placeholder_dollar_sign: true,
    placeholder_at_sign: false,
    max_placeholders: 65535,
    booleans_as_literals: false,
    strings_as_ntext: false,
    cast_json: None,
    escape_backslash: false,
    upsert_style: UpsertStyle::OnConflict,
    identifier_quote_style: IdentifierQuoteStyle::DoubleQuote,
};

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

fn bench_ast(c: &mut Criterion) {
    c.bench_function("Ast::parse_small", |b| {
        b.iter(|| {
            let _res = black_box(Ast::parse(QUERY_SMALL, &SETTINGS));
        });
    });
    c.bench_function("Ast::parse_big", |b| {
        b.iter(|| {
            let _res = black_box(Ast::parse(QUERY_BIG, &SETTINGS));
        });
    });

    c.bench_function("Ast::render_big", |b| {
        let ast = Ast::parse(QUERY_BIG, &SETTINGS).unwrap();
        b.iter(|| {
            let _res = black_box(ast.render(
                HashMap::from([
                    ("status", "accepted"),
                    ("created_after", "1111111"),
                    ("limit", "10"),
                ]),
                &SETTINGS,
            ));
        });
    });
}

criterion_group!(benches, bench_ast);
criterion_main!(benches);
