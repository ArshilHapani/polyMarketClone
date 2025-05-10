POSTGRES_CONTAINER_NAME := polyMarket_postgres
POSTGRES_PORT := 5432
POSTGRES_USER := polyMarket
POSTGRES_PASSWORD := polyMarket
POSTGRES_DB := polyMarket
POSTGRES_IMAGE := postgres:16.9-bookworm
POSTGRES_PUBLIC_SCHEMA := public

DATABASE_URL := postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost:$(POSTGRES_PORT)/$(POSTGRES_DB)

start-pg-container:
	@echo "Checking if PostgreSQL container is already running..."
	@if [ $$(docker ps -q -f name=$(POSTGRES_CONTAINER_NAME)) ]; then \
		echo "PostgreSQL container is already running."; \
	else \
		echo "Starting PostgreSQL container..."; \
		docker run --name $(POSTGRES_CONTAINER_NAME) -d -p $(POSTGRES_PORT):5432 \
			-e POSTGRES_USER=$(POSTGRES_USER) \
			-e POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) \
			-e POSTGRES_DB=$(POSTGRES_DB) \
			-v $(POSTGRES_VOLUME):$(POSTGRES_VOLUME_PATH) \
			$(POSTGRES_IMAGE); \
	fi


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