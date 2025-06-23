
CREATE TABLE IF NOT EXISTS pending_notification(
    id uuid PRIMARY KEY,
    data JSONB NOT NULL,
    connection_id TEXT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);