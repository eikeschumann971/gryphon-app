#!/bin/bash

# Kafka topic creation script for Gryphon App
# This script creates all the required topics for the event-sourced domains

set -e

echo "Waiting for Kafka to be ready..."
until kafka-topics --bootstrap-server localhost:9092 --list &> /dev/null; do
  echo "Kafka not ready yet, waiting..."
  sleep 2
done

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
  kafka-topics --bootstrap-server localhost:9092 \
    --create \
    --topic "$topic" \
    --partitions "$PARTITIONS" \
    --replication-factor "$REPLICATION_FACTOR" \
    --if-not-exists
done

echo "All topics created successfully!"

# List all topics to verify
echo -e "\nAvailable topics:"
kafka-topics --bootstrap-server localhost:9092 --list
