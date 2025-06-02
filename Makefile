POSTGRES_CONTAINER_NAME := polyMarket_postgres
POSTGRES_PORT := 5432
POSTGRES_USER := polyMarket
POSTGRES_PASSWORD := polyMarket
POSTGRES_DB := polyMarket
POSTGRES_IMAGE := postgres:16.9-bookworm
POSTGRES_PUBLIC_SCHEMA := public

DATABASE_URL := postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost:$(POSTGRES_PORT)/$(POSTGRES_DB)


REDIS_CONTAINER_NAME := polyMarket_redis
REDIS_PORT := 6379
REDIS_IMAGE := redis:7.4.1-alpine


NATS_CONTAINER_NAME := polyMarket_nats
NATS_IMAGE := nats:2.11.3-alpine3.21
NATS_PORT := 4222

SERVICE_API_PORT := 8080

DEFAULT_TARGET := help

.PHONY: help

help:
	@echo "Available targets:"
	@echo "  start-pg-container: Start PostgreSQL container"
	@echo "  start-redis-container: Start Redis container"
	@echo "  create-new-migration: Create a new SQLx migration"
	@echo "  apply-sqlx-migrations: Apply SQLx migrations"
	@echo "  revert-migration: Revert the last SQLx migration"
	@echo "  print-db-url: Print the database URL"
	@echo "  reset-db: Reset the database"
	@echo "  help: Show this help message"

# definitions
define kill_process
@bash -c '\
PORT_TO_KILL=$(1); \
PID=$$(lsof -ti tcp:$$PORT_TO_KILL); \
if [ -n "$$PID" ]; then \
  kill -9 $$PID; \
  echo "Killed process on port $$PORT_TO_KILL"; \
else \
  echo "No process found on port $$PORT_TO_KILL"; \
fi'
endef

# services
start-service-api:
	$(call kill_process, 8080)
	@cd ./service-api && \
		cargo watch -x run

# Containers management

start-pg-container:
	@echo "Checking if PostgreSQL container is already running..."
	@if [ $$(docker ps -q -f name=$(POSTGRES_CONTAINER_NAME)) ]; then \
		echo "PostgreSQL container is already running."; \
	elif [ $$(docker ps -aq -f status=exited -f name=$(POSTGRES_CONTAINER_NAME)) ]; then \
		echo "PostgreSQL container is stopped. Starting it..."; \
		docker start $(POSTGRES_CONTAINER_NAME); \
	else \
		echo "Starting PostgreSQL container..."; \
		docker run --name $(POSTGRES_CONTAINER_NAME) -d -p $(POSTGRES_PORT):5432 \
			-e POSTGRES_USER=$(POSTGRES_USER) \
			-e POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) \
			-e POSTGRES_DB=$(POSTGRES_DB) \
			-v $(POSTGRES_VOLUME):$(POSTGRES_VOLUME_PATH) \
			$(POSTGRES_IMAGE); \
	fi


start-redis-container:
	@echo "Checking if Redis container is already running..."
	@if [ $$(docker ps -q -f name=$(REDIS_CONTAINER_NAME)) ]; then \
		echo "Redis container is already running."; \
	elif [ $$(docker ps -aq -f status=exited -f name=$(REDIS_CONTAINER_NAME)) ]; then \
		echo "Redis container is stopped. Starting it..."; \
		docker start $(REDIS_CONTAINER_NAME); \
	else \
		echo "Starting Redis container..."; \
		docker run --name $(REDIS_CONTAINER_NAME) -d -p $(REDIS_PORT):6379 $(REDIS_IMAGE); \
	fi

start-nats-container:
	@echo "Checking if NATS container is already running..."
	@if [ $$(docker ps -q -f name=$(NATS_CONTAINER_NAME)) ]; then \
		echo "NATS container is already running."; \
	elif [ $$(docker ps -aq -f status=exited -f name=$(NATS_CONTAINER_NAME)) ]; then \
		echo "NATS container is stopped. Starting it..."; \
		docker start $(NATS_CONTAINER_NAME); \
	else \
		echo "Starting NATS container..."; \
		docker run --name $(NATS_CONTAINER_NAME) -d -p $(NATS_PORT):4222 -p 8222:8222 $(NATS_IMAGE) -js; \
	fi

start-required-containers: start-pg-container start-redis-container start-nats-container


# Utility targets

create-new-migration:
	@echo "Enter migration name:"
	@read migration_name;
	@cd ./db-service && \
		cargo sqlx migrate add "$$migration_name" && \
		echo "Migration created successfully."
	

apply-sqlx-migrations:
	@cd ./db-service && export DATABASE_URL=$(DATABASE_URL) && cargo sqlx migrate run

revert-migration:
	@echo "Reverting migration"
	@export DATABASE_URL=$(DATABASE_URL) && \
		cd ./db-service && \
		cargo sqlx migrate revert

print-db-url:
	@echo "DATABASE_URL: $(DATABASE_URL)"

reset-db:
	@echo "Dropping database..."
	@docker exec -it $(POSTGRES_CONTAINER_NAME) psql -U $(POSTGRES_USER) -c "DROP SCHEMA $(POSTGRES_DB) CASCADE;"
	@docker exec -it $(POSTGRES_CONTAINER_NAME) psql -U $(POSTGRES_USER) -c "DROP SCHEMA $(POSTGRES_PUBLIC_SCHEMA) CASCADE;"
	@docker exec -it $(POSTGRES_CONTAINER_NAME) psql -U $(POSTGRES_USER) -c "CREATE SCHEMA $(POSTGRES_DB);"
	@docker exec -it $(POSTGRES_CONTAINER_NAME) psql -U $(POSTGRES_USER) -c "CREATE SCHEMA $(POSTGRES_PUBLIC_SCHEMA);"
	@echo "Database dropped."

move-proto-files-to-client:
	@cp -r ./service-api/proto/*.proto ./app/public/proto/
	@echo "Proto files moved to client directory."

start-order-service:
	@echo "Starting order service..."
	@cd ./order-service && \
		cargo watch -x run


run-test-with-output:
	@cargo test -- --nocapture

run-particular-test:
	@cargo test --package order-service --bin order-service -- order_book_v2::outcome_book::test