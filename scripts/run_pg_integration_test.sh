#!/usr/bin/env bash
set -euo pipefail

# Helper script to start a Postgres Docker container, run the gated pg integration test, and stop the container.
# Usage: ./scripts/run_pg_integration_test.sh [port]
# Default port: 5433

PORT=${1:-5433}
CONTAINER_NAME=gryphon-tests-pg

echo "Starting Postgres container on host port ${PORT}..."
# If a previous container with the same name exists, remove it so we can recreate it.
if docker ps -a --format '{{.Names}}' | grep -q "^${CONTAINER_NAME}$"; then
  echo "Found existing container named ${CONTAINER_NAME}, removing it..."
  docker rm -f ${CONTAINER_NAME} >/dev/null 2>&1 || true
fi

docker run --rm --name ${CONTAINER_NAME} -e POSTGRES_PASSWORD=postgres -p ${PORT}:5432 -d postgres:15

# Wait for Postgres to become available (10 retries)
for i in {1..10}; do
  if docker exec ${CONTAINER_NAME} pg_isready -U postgres >/dev/null 2>&1; then
    echo "Postgres is ready"
    break
  fi
  echo "Waiting for Postgres... ($i)"
  sleep 1
done

export PG_TEST_PORT=${PORT}

echo "Running Postgres integration test (feature: pg_integration)..."
cargo test --features pg_integration -- tests::pg_integration_tests -- --nocapture

# Capture exit code
RC=$?

echo "Stopping Postgres container..."
docker stop ${CONTAINER_NAME} >/dev/null || true

exit ${RC}
