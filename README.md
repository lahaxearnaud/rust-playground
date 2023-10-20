# rust-playground

## Deps

```bash
sudo apt-get install libpq-dev
```

```bash
cargo install diesel_cli --no-default-features --features postgres
```

# Dev postgres

```bash
docker run -d --rm --name postgres \
 -e POSTGRES_PASSWORD=postgres \
 -e POSTGRES_USER=postgres \
 -e POSTGRES_DB=postgres \
 -p 5432:5432 postgres:16
 ```

 ```bash
diesel setup
```

# Auth

Le jwt est affiché dans la console au start

# Swagger

http://127.0.0.1:8080/swagger-ui/

# Metrics (Prometheus)

http://127.0.0.1:8080/metrics


# Health check

http://127.0.0.1:8080/health

# build / publish docker image

@todo à mettre dans la CI

```bash
cargo build -r --all-features
docker build . -t lahaxearnaud/rust-playground
docker push lahaxearnaud/rust-playground
```