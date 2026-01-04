FROM postgres:16-bookworm

RUN apt-get update && \
    apt-get -y install postgresql-16-cron && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

COPY init-scripts/ /docker-entrypoint-initdb.d/

CMD ["postgres", "-c", "shared_preload_libraries=pg_cron", "-c", "cron.database_name=postgres"]