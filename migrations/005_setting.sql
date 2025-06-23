CREATE TABLE IF NOT EXISTS setting_enum(
  id uuid PRIMARY KEY,
  label TEXT NOT NULL,
  values JSONB NOT NULL,
  created_by TEXT NOT NULL,
  created_on TIMESTAMPTZ NOT NULL
);
ALTER TABLE setting_enum ADD CONSTRAINT enum_label_uq UNIQUE (label);

CREATE TABLE IF NOT EXISTS setting (
  id uuid PRIMARY KEY,
  label TEXT NOT NULL,
  key TEXT NOT NULL,
  description TEXT,
  enum_id uuid,
  is_editable BOOLEAN NOT NULL DEFAULT true,
  value_type TEXT NOT NULL,
  is_deleted BOOLEAN NOT NULL,
  created_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  is_user BOOLEAN NOT NULL,
  is_user_business BOOLEAN NOT NULL,
  is_business BOOLEAN NOT NULL,
  is_global BOOLEAN NOT NULL,
  created_by TEXT,
  deleted_by TEXT

);

ALTER TABLE setting ADD CONSTRAINT fk_enum_id FOREIGN KEY (enum_id) REFERENCES setting_enum(id) ON DELETE SET NULL;
ALTER TABLE setting ADD CONSTRAINT label_uq UNIQUE (label);
ALTER TABLE setting ADD CONSTRAINT key_uq UNIQUE (key);

CREATE TABLE IF NOT EXISTS setting_value (
  id uuid PRIMARY KEY,
  user_id uuid, 
  business_id uuid,
  setting_id uuid,
  value TEXT NOT NULL,
  scope_ttl TIMESTAMPTZ,
  scope_retry_count INT,
  file_path TEXT,
  template TEXT,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by TEXT NOT NULL,
  updated_by TEXT,
  deleted_by TEXT
);

ALTER TABLE setting_value ADD CONSTRAINT fk_user_id FOREIGN KEY (user_id)  REFERENCES user_account(id) ON DELETE CASCADE;
ALTER TABLE setting_value ADD CONSTRAINT fk_business_id FOREIGN KEY (business_id) REFERENCES business_account(id) ON DELETE CASCADE;
ALTER TABLE setting_value ADD CONSTRAINT fk_setting FOREIGN KEY (setting_id) REFERENCES setting (id) ON DELETE CASCADE;
ALTER TABLE setting_value ADD CONSTRAINT user_business_id_uq UNIQUE NULLS NOT DISTINCT(setting_id, user_id, business_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_setting_business ON setting_value (setting_id, business_id) WHERE user_id IS NULL;



INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description, enum_id) VALUES(uuid_generate_v4(), 'Time Zone', 'time_zone', 'string', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', true, true, true, true, true, null, 'bf5195d6-a5d0-4899-b7d5-faa16b09a209');
INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Email App Password', 'email_app_password', 'string', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', true, false, true, false, false,  'App password for personal email. This is used to send emails from the application.');
INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Financial Year Start', 'financial_year_start', 'date_time', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', true, true, false, false, false,  null);

INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Leave Request Template', 'leave_request_template', 'text', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', true, true, false, false,false,  null);
INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Leave Request Status Update Template', 'leave_request_status_update_template', 'text', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', true, true, false, false,false,  null);
INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Email OTP template', 'email_otp_template', 'text', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', true, true, false, false, false, null);

-- INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Unpaid Leave Count', 'unpaid_leave_count', 'integer', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', false, false, false, true, false, null);
-- INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Total Common Leave Count', 'total_common_leave_count', 'integer', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', false, false, false, true, false, null);
-- INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Total Restricted Leave Count', 'total_restricted_leave_count', 'integer', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', false, false, false, true, false, null);
-- INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Total Medical Leave Count', 'total_medical_leave_count', 'integer', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', false, false, false, true, false,  null);
-- INSERT INTO setting(id, label, key, value_type,  is_deleted, created_on, created_by, is_editable, is_global, is_user, is_business, is_user_business, description) VALUES(uuid_generate_v4(), 'Total Casual Leave Count', 'total_casual_leave_count', 'integer', false, CURRENT_TIMESTAMP,  '00000000-0000-0000-0000-000000000000', false, false, false, true, false, null);
