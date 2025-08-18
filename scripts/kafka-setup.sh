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

# Define topics with configurations
declare -A topics=(
  ["logical-agent-events"]="partitions=3,replication-factor=1"
  ["technical-agent-events"]="partitions=3,replication-factor=1"
  ["kinematic-agent-events"]="partitions=3,replication-factor=1"
  ["path-planning-events"]="partitions=3,replication-factor=1"
  ["dynamics-events"]="partitions=3,replication-factor=1"
  ["gui-events"]="partitions=3,replication-factor=1"
)

# Create each topic
for topic in "${!topics[@]}"; do
  echo "Creating topic: $topic"
  kafka-topics --bootstrap-server localhost:9092 \
    --create \
    --topic "$topic" \
    --config "${topics[$topic]}" \
    --if-not-exists
done

echo "All topics created successfully!"

# List all topics to verify
echo -e "\nAvailable topics:"
kafka-topics --bootstrap-server localhost:9092 --list
