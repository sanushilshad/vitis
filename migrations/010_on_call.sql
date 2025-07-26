CREATE TABLE IF NOT EXISTS on_call(
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL,
    department_id uuid NOT NULL,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    updated_by uuid,
    updated_on TIMESTAMPTZ,
    deleted_by uuid,
    deleted_on TIMESTAMPTZ,
    is_deleted BOOLEAN NOT NULL DEFAULT false,
    created_by uuid NOT NULL,
    created_on TIMESTAMPTZ NOT NULL

);