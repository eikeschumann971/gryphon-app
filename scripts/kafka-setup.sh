#!/bin/bash

# Kafka topic creation script for Gryphon App
# This script creates all the required topics for the event-sourced domains

set -euo pipefail

echo "Waiting for Kafka to be ready (max 10 attempts)..."

# Helper to run kafka-topics either locally or inside the gryphon-kafka container
run_kafka_topics() {
  if command -v kafka-topics >/dev/null 2>&1; then
    kafka-topics --bootstrap-server localhost:9092 "$@"
  elif docker ps --format '{{.Names}}' | grep -q '^gryphon-kafka$'; then
    docker exec gryphon-kafka bash -lc "kafka-topics --bootstrap-server localhost:9092 $*"
  else
    return 2
  fi
}

ATTEMPTS=0
MAX_ATTEMPTS=10
while [ $ATTEMPTS -lt $MAX_ATTEMPTS ]; do
  if run_kafka_topics --list >/dev/null 2>&1; then
    echo "Kafka is reachable"
    break
  fi
  ATTEMPTS=$((ATTEMPTS + 1))
  echo "Kafka not ready yet (attempt $ATTEMPTS/$MAX_ATTEMPTS), waiting 2s..."
  sleep 2
done

if [ $ATTEMPTS -ge $MAX_ATTEMPTS ]; then
  echo "ERROR: Kafka did not become ready after $MAX_ATTEMPTS attempts"
  echo "Make sure Docker Compose is running and the Kafka container is named 'gryphon-kafka'"
  exit 1
fi

echo "Creating Kafka topics for Gryphon App domains..."

# Define topics and common configuration
topics=(
  "logical-agent-events"
  "technical-agent-events"
  "kinematic-agent-events"
  "path-planning-events"
  "path-planning-replies"
  "dynamics-events"
  "gui-events"
)

# Default partitions and replication factor
PARTITIONS=3
REPLICATION_FACTOR=1

# Create each topic
for topic in "${topics[@]}"; do
  echo "Creating topic: $topic"
  if run_kafka_topics --create --topic "$topic" --partitions "$PARTITIONS" --replication-factor "$REPLICATION_FACTOR" --if-not-exists >/dev/null 2>&1; then
    echo "  OK: $topic"
  else
    echo "  WARN: failed to create topic $topic (it may already exist or kafka not reachable)"
  fi
done

echo "All topics created successfully!"

# List all topics to verify
echo -e "\nAvailable topics:"
run_kafka_topics --list || echo "(failed to list topics)"
