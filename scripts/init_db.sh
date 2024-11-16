#!/usr/bin/env bash

set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "    cargo install --version='~0.6' sqlx-cli \
--no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

ZERO2PROD_DB_USER=postgres
ZERO2PROD_DB_PASSWORD=postgres
ZERO2PROD_DB_NAME=newsletter
ZERO2PROD_DB_PORT=6002

# Keep pinging Postgres until it's ready to accept commands
export PGPASSWORD="${ZERO2PROD_DB_PASSWORD}"
until psql -h "localhost" -U "${ZERO2PROD_DB_USER}" -p "${ZERO2PROD_DB_PORT}" -d "postgres" -c '\q'; do
  >&2 echo "Postgres is still unavailable - sleeping"
  sleep 1
done

>&2 echo "Postgres is up and running on port ${ZERO2PROD_DB_PORT}!"

export DATABASE_URL=postgres://${ZERO2PROD_DB_USER}:${ZERO2PROD_DB_PASSWORD}@localhost:${ZERO2PROD_DB_PORT}/${ZERO2PROD_DB_NAME}
sqlx database create
sqlx migrate run

>&2 echo "Postgres has been migrated, ready to go!"
