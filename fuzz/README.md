# Fuzzing the AST Parser

This directory contains fuzz targets for testing the SQL AST parser against malformed or unexpected input.

## Prerequisites

Install `cargo-fuzz`:

```bash
cargo install cargo-fuzz
```

Note: Fuzzing requires a nightly Rust toolchain:

```bash
rustup install nightly
```

## Available Fuzz Targets

| Target | Description |
|--------|-------------|
| `ast_postgres` | Fuzzes AST parsing with PostgreSQL settings |
| `ast_mysql` | Fuzzes AST parsing with MySQL settings |
| `ast_mssql` | Fuzzes AST parsing with MSSQL settings |
| `ast_render` | Fuzzes both parsing AND rendering with random parameters |

## Running the Fuzzer

From the project root directory:

```bash
# Fuzz PostgreSQL parser (recommended to start with)
cargo +nightly fuzz run ast_postgres

# Fuzz MySQL parser
cargo +nightly fuzz run ast_mysql

# Fuzz MSSQL parser
cargo +nightly fuzz run ast_mssql

# Fuzz parsing + rendering (more comprehensive)
cargo +nightly fuzz run ast_render
```

### Useful Options

```bash
# Run for a specific duration (e.g., 60 seconds)
cargo +nightly fuzz run ast_postgres -- -max_total_time=60

# Use multiple cores (e.g., 4 jobs)
cargo +nightly fuzz run ast_postgres --jobs 4

# Run with specific number of iterations
cargo +nightly fuzz run ast_postgres -- -runs=100000

# Show only new coverage
cargo +nightly fuzz run ast_postgres -- -print_final_stats=1
```

## Seed Corpus

The `corpus/` directory contains seed inputs to help the fuzzer get started. These include:

- Simple SELECT statements
- Named and positional parameters
- Conditional blocks (single and nested)
- IN/NOT IN clauses
- Typed placeholders
- String literals with escapes
- Comments (line and block)

## Investigating Crashes

When the fuzzer finds a crash, it saves the input to `fuzz/artifacts/<target>/`. To reproduce:

```bash
cargo +nightly fuzz run ast_postgres fuzz/artifacts/ast_postgres/crash-<hash>
```

## Coverage Report

Generate a coverage report:

```bash
cargo +nightly fuzz coverage ast_postgres
```

## Security

The AST parser handles untrusted SQL input. Fuzzing helps catch:

- Panics from unexpected input
- Stack overflows from deeply nested structures
- Infinite loops
- Memory safety issues

Any crashes found should be investigated and fixed promptly.
