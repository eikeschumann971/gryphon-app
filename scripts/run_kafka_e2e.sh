#!/usr/bin/env bash
set -euo pipefail

# Orchestrated end-to-end test for Kafka-based path planning components.
# Starts planner and worker in the background, runs client, collects logs and topic output,
# and verifies PlanAssigned and PlanCompleted events were emitted.

ROOT_DIR="$(pwd)"
OUT_DIR="$ROOT_DIR/target/e2e_kafka"
mkdir -p "$OUT_DIR/logs"

# Remove stale logs and pid files so each run starts clean
rm -f "$OUT_DIR/logs"/* || true
rm -f "$OUT_DIR"/planner.pid || true
rm -f "$OUT_DIR"/worker.pid || true

PLANNER_LOG="$OUT_DIR/logs/planner.log"
WORKER_LOG="$OUT_DIR/logs/worker.log"
CLIENT_LOG="$OUT_DIR/logs/client.log"
TOPIC_LOG="$OUT_DIR/logs/topic.log"

PLANNER_PID_FILE="$OUT_DIR/planner.pid"
WORKER_PID_FILE="$OUT_DIR/worker.pid"

echo "Starting planner (background) -> logs: $PLANNER_LOG"
nohup cargo run --bin pathplan_planner_kafka > "$PLANNER_LOG" 2>&1 &
PLANNER_PID=$!
echo $PLANNER_PID > "$PLANNER_PID_FILE"

# Give planner time to start
sleep 2

echo "Starting worker (background) -> logs: $WORKER_LOG"
nohup cargo run --bin pathplan_worker_kafka > "$WORKER_LOG" 2>&1 &
WORKER_PID=$!
echo $WORKER_PID > "$WORKER_PID_FILE"

# Give worker time to register
sleep 6

echo "Running client (foreground) -> logs: $CLIENT_LOG"
# Run client; it publishes PathPlanRequested and polls for responses
cargo run --bin pathplan_client_kafka > "$CLIENT_LOG" 2>&1 || true

# After client completes, capture topic contents to verify
echo "Dumping topic messages to $TOPIC_LOG"
docker exec -i gryphon-kafka bash -lc "kafka-console-consumer --bootstrap-server localhost:9092 --topic path-planning-events --from-beginning --timeout-ms 15000" > "$TOPIC_LOG" 2>&1 || true

# Grep for relevant events
echo "Scanning logs for PlanAssigned and PlanCompleted"
ASSIGNED_IN_TOPIC=$(grep -E '"event_type":"PlanAssigned"' -n "$TOPIC_LOG" || true)
COMPLETED_IN_TOPIC=$(grep -E '"event_type":"PlanCompleted"' -n "$TOPIC_LOG" || true)
ASSIGNED_IN_PLANNER=$(grep -E 'Published PlanAssigned' -n "$PLANNER_LOG" || true)
COMPLETED_IN_WORKER=$(grep -E 'Published PlanCompleted' -n "$WORKER_LOG" || true)
ASSIGNED_IN_CLIENT=$(grep -E 'Received PlanAssigned' -n "$CLIENT_LOG" || true)
COMPLETED_IN_CLIENT=$(grep -E 'Received PlanCompleted' -n "$CLIENT_LOG" || true)

echo "--- SUMMARY ---"
if [ -n "$ASSIGNED_IN_TOPIC" ]; then
  echo "PlanAssigned found in topic (sample lines):"
  echo "$ASSIGNED_IN_TOPIC" | sed -n '1,5p'
else
  echo "PlanAssigned NOT found in topic"
fi

if [ -n "$COMPLETED_IN_TOPIC" ]; then
  echo "PlanCompleted found in topic (sample lines):"
  echo "$COMPLETED_IN_TOPIC" | sed -n '1,5p'
else
  echo "PlanCompleted NOT found in topic"
fi

if [ -n "$ASSIGNED_IN_PLANNER" ]; then
  echo "Planner emitted PlanAssigned (sample lines):"
  echo "$ASSIGNED_IN_PLANNER" | sed -n '1,5p'
else
  echo "Planner did NOT emit PlanAssigned"
fi

if [ -n "$COMPLETED_IN_WORKER" ]; then
  echo "Worker emitted PlanCompleted (sample lines):"
  echo "$COMPLETED_IN_WORKER" | sed -n '1,5p'
else
  echo "Worker did NOT emit PlanCompleted"
fi

if [ -n "$ASSIGNED_IN_CLIENT" ]; then
  echo "Client consumed PlanAssigned (sample lines):"
  echo "$ASSIGNED_IN_CLIENT" | sed -n '1,5p'
else
  echo "Client did NOT observe PlanAssigned"
fi

if [ -n "$COMPLETED_IN_CLIENT" ]; then
  echo "Client consumed PlanCompleted (sample lines):"
  echo "$COMPLETED_IN_CLIENT" | sed -n '1,5p'
else
  echo "Client did NOT observe PlanCompleted"
fi

# Cleanup background processes
if [ -f "$PLANNER_PID_FILE" ]; then
  PL_PID=$(cat "$PLANNER_PID_FILE")
  echo "Stopping planner pid $PL_PID"
  kill "$PL_PID" || true
  rm -f "$PLANNER_PID_FILE"
fi
if [ -f "$WORKER_PID_FILE" ]; then
  W_PID=$(cat "$WORKER_PID_FILE")
  echo "Stopping worker pid $W_PID"
  kill "$W_PID" || true
  rm -f "$WORKER_PID_FILE"
fi

echo "Logs and topic dump are in $OUT_DIR/logs"

# Decide success: require PlanAssigned in topic *and* that the client consumed both PlanAssigned and PlanCompleted
if [ -n "$ASSIGNED_IN_TOPIC" ] && [ -n "$ASSIGNED_IN_CLIENT" ] && [ -n "$COMPLETED_IN_CLIENT" ]; then
  echo "E2E verification PASSED"
  exit 0
else
  echo "E2E verification FAILED"
  exit 2
fi
