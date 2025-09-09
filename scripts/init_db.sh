#!/usr/bin/env bash
set -x  # print each command before executing it.
set -eo pipefail    # exit on error and if a command in a pipeline fails

if ! [ -x "$(command -v docker)" ]; then
  echo >&2 "Error: docker is not installed."
  exit 1
fi

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  exit 1
fi

# check if a custom user has been set, otherwise default to postgres
DB_USER=${POSTGRES_USER:=postgres}
# check if a custom database name has been set, otherwise default to postgres
DB_NAME="${POSTGRES_DB:=newsletter}"
# check if a custom password has been set, otherwise default to postgres
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
# check if a custom port has been set, otherwise default to postgres
DB_PORT="${POSTGRES_PORT:=5432}"

# Launch postgres using Docker
if [[ -z "${SKIP_DOCKER}" ]]
then
    docker run \
        -e POSTGRES_USER=${DB_USER} \
        -e POSTGRES_PASSWORD=${DB_PASSWORD} \
        -e POSTGRES_DB=${DB_NAME} \
        -p "${DB_PORT}":5432 \
        -d postgres \
        postgres -N 100
        # increase max_connections to 100
fi

# wait until Postgres is ready
export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"