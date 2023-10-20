# rust-playground

## Quickstart

```bash
make boot
```

## Deps

```bash
sudo apt-get install libpq-dev
```

```bash
make diesel-setup
```

# Dev postgres

```bash
make pg-start
 ```

 ```bash
make setup-db
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

```bash
make publish-docker
```