APP         := tolerance-api
PORT        := 3000
IMAGE       := $(APP):latest

.PHONY: dev run build start stop restart logs lint test clean help

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'

dev: ## Run with cargo watch (auto-reload)
	cargo watch -x 'run --bin $(APP)'

run: ## Run the API locally
	cargo run --bin $(APP)

lint: ## Run clippy and check formatting
	cargo clippy -- -D warnings
	cargo fmt --check

test: ## Run tests
	cargo test

build: ## Build and start the production container
	docker compose up -d --build

start: ## Start an existing container
	docker compose up -d

stop: ## Stop the container and kill any leaked port process
	docker compose down --remove-orphans
	-lsof -ti :$(PORT) | xargs kill -9 2>/dev/null

rebuild: stop build ## Rebuild from scratch

logs: ## Tail container logs
	docker compose logs -f

clean: stop ## Remove container, image, and build artifacts
	-docker rmi $(IMAGE) 2>/dev/null
	cargo clean
