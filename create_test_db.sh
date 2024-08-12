#!/bin/bash

set -a && source .env && set +a
docker run --name commentatordb \
  --env POSTGRES_PASSWORD="$DATABASE_PASSWORD" \
  --env POSTGRES_USER=commentater \
  --env POSTGRES_DB=commentater \
  --publish 5432:5432 \
  -d postgres:15
