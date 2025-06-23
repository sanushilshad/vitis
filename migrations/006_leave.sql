CREATE TYPE leave_status AS ENUM (
  'approved',
  'rejected',
  'cancelled',
  'requested'
);


CREATE TYPE leave_period AS ENUM (
  'half_day',
  'full_day'
);


CREATE TYPE alert_status AS ENUM (
  'pending',
  'success',
  'failed'
);


CREATE TABLE IF NOT EXISTS leave_type (
    id uuid PRIMARY KEY,
    label TEXT NOT NULL,
    business_id uuid NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    created_by uuid NOT NULL,
    updated_by uuid,
    updated_on TIMESTAMPTZ
);

ALTER TABLE leave_type ADD CONSTRAINT fk_business_id FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;
ALTER TABLE leave_type ADD CONSTRAINT uq_business_label UNIQUE (business_id, label);

CREATE TABLE IF NOT EXISTS leave_group(
    id uuid PRIMARY KEY,
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    business_id uuid NOT NULL,
    label TEXT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    created_by uuid NOT NULL,
    updated_on TIMESTAMPTZ,
    updated_by uuid
);

ALTER TABLE leave_group ADD CONSTRAINT leave_group_uq UNIQUE(business_id, start_date, end_date);
ALTER TABLE leave_group ADD CONSTRAINT leave_group_label_uq UNIQUE(business_id, label);
ALTER TABLE leave_group ADD CONSTRAINT fk_business_id FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;

CREATE TABLE IF NOT EXISTS user_leave_relationship(
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL,
    leave_group_id uuid NOT NULL,
    leave_type_id uuid NOT NULL,
    used_count DECIMAL(20, 1) NOT NULL,
    allocated_count DECIMAL(20, 1) NOT NULL,
    is_active status DEFAULT 'active'::status NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    created_by uuid NOT NULL,
    updated_by uuid,
    updated_on TIMESTAMPTZ
);

ALTER TABLE user_leave_relationship ADD CONSTRAINT fk_user_id FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE user_leave_relationship ADD CONSTRAINT fk_leave_type_id FOREIGN KEY (leave_type_id)  REFERENCES leave_type(id) ON DELETE CASCADE;
ALTER TABLE user_leave_relationship ADD CONSTRAINT fk_leave_group_id FOREIGN KEY (leave_group_id)  REFERENCES leave_group(id) ON DELETE CASCADE;
ALTER TABLE user_leave_relationship ADD CONSTRAINT leave_uq UNIQUE(leave_group_id, leave_type_id, user_id);


CREATE TABLE IF NOT EXISTS leave_request (
  id uuid PRIMARY KEY,
  sender_id uuid NOT NULL,
  business_id uuid NOT NULL,
  receiver_id uuid NOT NULL,
  leave_type_id uuid NOT NULL,
  status leave_status NOT NULL,
  period leave_period NOT NULL,
  date TIMESTAMPTZ NOT NULL,
  reason TEXT,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  email_message_id TEXT,
  alert_status alert_status DEFAULT 'pending'::alert_status NOT NULL,
  cc JSONB,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE leave_request ADD CONSTRAINT fk_leave_type_id FOREIGN KEY (leave_type_id)  REFERENCES leave_type(id) ON DELETE CASCADE;
ALTER TABLE leave_request ADD CONSTRAINT fk_user_id FOREIGN KEY ("sender_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE leave_request ADD CONSTRAINT fk_business_id FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;
CREATE UNIQUE INDEX leave_uq ON leave_request (sender_id, period, date) WHERE is_deleted = false;
ALTER TABLE leave_request ADD CONSTRAINT fk_user_id FOREIGN KEY (sender_id)  REFERENCES user_account(id) ON DELETE CASCADE;
CREATE INDEX leave_user_idx ON leave_request (sender_id);
CREATE INDEX leave_created_on_idx ON leave_request (created_on);