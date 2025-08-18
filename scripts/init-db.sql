-- Initialize the Gryphon App database schema

-- Create snapshots table (already handled by PostgresSnapshotStore)

-- Create additional tables for application-specific data
CREATE TABLE IF NOT EXISTS event_log (
    id BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    event_version BIGINT NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_event_log_aggregate_id ON event_log(aggregate_id);
CREATE INDEX IF NOT EXISTS idx_event_log_aggregate_type ON event_log(aggregate_type);
CREATE INDEX IF NOT EXISTS idx_event_log_event_type ON event_log(event_type);
CREATE INDEX IF NOT EXISTS idx_event_log_occurred_at ON event_log(occurred_at);

-- Create projection tables
CREATE TABLE IF NOT EXISTS logical_agent_overview (
    agent_id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    objectives_count INTEGER DEFAULT 0,
    completed_objectives INTEGER DEFAULT 0,
    facts_count INTEGER DEFAULT 0,
    rules_count INTEGER DEFAULT 0,
    last_activity TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS technical_agent_overview (
    agent_id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    agent_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    capabilities_count INTEGER DEFAULT 0,
    sensors_count INTEGER DEFAULT 0,
    actuators_count INTEGER DEFAULT 0,
    last_activity TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL
);

-- Add more projection tables as needed for other domains
