# php-sqlx

An alternative to PDO.

## EXPERIMENTAL
If you are reading this, the project is highly experimental. Use with caution.

## Docker
```shell
docker build -t php-sqlx .
docker run php-sql
```

## Build on Mac OS
```sh
export MACOSX_DEPLOYMENT_TARGET=15.5.0
export RUSTFLAGS="-C link-arg=-undefined -C link-arg=dynamic_lookup"
cargo php install --release --yes 
```
