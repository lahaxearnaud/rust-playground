SHELL := /bin/bash
help:            ## Show this help.
	@fgrep -h "##" $(MAKEFILE_LIST) | fgrep -v fgrep | sed -e 's/\\$$//' | sed -e 's/##//'
.DEFAULT_GOAL:= help

start: ## Start app
	cargo run

watch: ## Start cargo watch
	cargo watch -x run

build: ## Build release app
	cargo build -r --all-features

build-docker: build ## Build docker image
	docker build . -t lahaxearnaud/rust-playground

publish-docker: build-docker ## Publish docker image
	docker push lahaxearnaud/rust-playground

pg-start: ## Start local pg server
	docker rm -f postgres
	docker run -d --rm --name postgres -e POSTGRES_PASSWORD=postgres -e POSTGRES_USER=postgres -e POSTGRES_DB=postgres -p 5432:5432 postgres:16
	@while [ -z "$(shell docker logs postgres 2>&1 | grep -o "accept connections")" ]; \
    do \
        sleep 2; \
    done; \
	sleep 5;

db-setup: pg-start ## Start PG and populate DB
	diesel setup

diesel-setup: ## Setup DB
	cargo install diesel_cli --no-default-features --features postgres

install: diesel-setup db-setup ## Boot all dev tools

lint: ## Lint code
	cargo clippy --fix
