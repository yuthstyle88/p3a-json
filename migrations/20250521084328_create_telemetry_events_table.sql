-- Add migration script here
CREATE TABLE telemetry_events (
                                  id BIGSERIAL NOT NULL,
                                  cadence TEXT NOT NULL,
                                  channel TEXT NOT NULL,
                                  country_code TEXT NOT NULL,
                                  metric_name TEXT NOT NULL,
                                  metric_value INTEGER NOT NULL,
                                  platform TEXT NOT NULL,
                                  version TEXT NOT NULL,
                                  woi SMALLINT NOT NULL,
                                  wos SMALLINT NOT NULL,
                                  yoi SMALLINT NOT NULL,
                                  yos SMALLINT NOT NULL,
                                  received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                                  UNIQUE (id, received_at)
) PARTITION BY RANGE (received_at);

CREATE TABLE telemetry_events_y2025
    PARTITION OF telemetry_events
    FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

CREATE TABLE telemetry_events_default
    PARTITION OF telemetry_events DEFAULT;

CREATE INDEX idx_metric_time_y2025 ON telemetry_events_y2025 (metric_name, received_at);
CREATE INDEX idx_platform_y2025 ON telemetry_events_y2025 (platform);