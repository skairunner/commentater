#!/bin/bash

set -a && source .env && set +a
export DATABASE_URL="postgres://$DATABASE_USER:$DATABASE_PASSWORD@localhost:5432/$DATABASE_DB"
sqlx database create
sqlx migrate run
