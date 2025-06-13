

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE user_type AS ENUM (
  'guest',
  'developer',
  'maintainer',
  'superadmin',
  'admin'
);

CREATE TYPE status AS ENUM (
  'active',
  'inactive',
  'pending',
  'archived'
);


CREATE TABLE IF NOT EXISTS user_account(
    id uuid PRIMARY KEY,
    is_test_user BOOLEAN NOT NULL DEFAULT false,
    username TEXT NOT NULL UNIQUE,
    international_dialing_code TEXT NOT NULL,
    mobile_no TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT NOT NULL,
    is_active status DEFAULT 'active'::status NOT NULL,
    created_by uuid NOT NULL,
    vectors jsonb NOT NULL,
    updated_by uuid,
    deleted_by uuid,
    created_on TIMESTAMPTZ NOT NULL,
    updated_on TIMESTAMPTZ,
    deleted_on TIMESTAMPTZ,
    is_deleted BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE user_account ADD CONSTRAINT user_mobile_uq UNIQUE (mobile_no);
ALTER TABLE user_account ADD CONSTRAINT user_username_uq UNIQUE (username);
ALTER TABLE user_account ADD CONSTRAINT user_email_uq UNIQUE (email);

CREATE TYPE "user_auth_identifier_scope" AS ENUM (
  'otp',
  'password',
  'google',
  'facebook',
  'microsoft',
  'apple',
  'token',
  'auth_app',
  'qr',
  'email'
);

CREATE TABLE IF NOT EXISTS auth_mechanism (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  auth_scope user_auth_identifier_scope NOT NULL,
  auth_identifier text NOT NULL,
  secret TEXT,
  valid_upto TIMESTAMPTZ,
  is_active status DEFAULT 'active'::status NOT NULL,
  created_on TIMESTAMPTZ,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  retry_count INTEGER, 
  created_by TEXT,
  updated_by TEXT,
  deleted_by TEXT,
  is_deleted BOOLEAN DEFAULT false
);

ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user FOREIGN KEY (user_id) REFERENCES user_account (id) ON DELETE CASCADE;
ALTER TABLE auth_mechanism ADD CONSTRAINT fk_auth_user_id_auth_scope UNIQUE (user_id, auth_scope);


CREATE TABLE IF NOT EXISTS role (
  id uuid PRIMARY KEY,
  role_name TEXT NOT NULL,
  role_status status NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN  NOT NULL DEFAULT false
);

ALTER TABLE role ADD CONSTRAINT unique_role_name UNIQUE (role_name);

CREATE TABLE IF NOT EXISTS user_role (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  role_id uuid NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);

ALTER TABLE user_role ADD CONSTRAINT fk_role_id FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE user_role ADD CONSTRAINT fk_user_id FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE user_role ADD CONSTRAINT user_role_pk UNIQUE (user_id, role_id);

CREATE TABLE IF NOT EXISTS permission (
  id uuid PRIMARY KEY,
  permission_name TEXT NOT NULL,
  permission_description TEXT,
  created_on TIMESTAMPTZ,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS role_permission (
  id uuid PRIMARY KEY,
  role_id uuid,
  permission_id uuid,
  created_on TIMESTAMPTZ,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);


ALTER TABLE role_permission ADD CONSTRAINT "fk_permission_id" FOREIGN KEY ("permission_id") REFERENCES permission ("id") ON DELETE CASCADE;
ALTER TABLE role_permission ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE permission ADD CONSTRAINT permission_name UNIQUE (permission_name);
ALTER TABLE role_permission ADD CONSTRAINT permission_role_id UNIQUE (permission_id, role_id);


CREATE TYPE vector_type AS ENUM (
    'mobile_no',
    'email'
);

CREATE TABLE IF NOT EXISTS project_account (
  id uuid PRIMARY KEY,
  name TEXT NOT NULL,
  vectors jsonb NOT NULL,
  tags TEXT[],
  is_active status DEFAULT 'inactive'::status NOT NULL,
  created_by  uuid NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_by uuid,
  updated_on TIMESTAMPTZ,
  deleted_by uuid,
  deleted_on TIMESTAMPTZ,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  is_test_account BOOLEAN NOT NULL DEFAULT false

);


CREATE TABLE IF NOT EXISTS project_user_relationship (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  project_id uuid NOT NULL,
  role_id uuid NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT false,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid
);


ALTER TABLE project_user_relationship ADD CONSTRAINT "fk_user_id" FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE project_user_relationship ADD CONSTRAINT "fk_project_id" FOREIGN KEY ("project_id") REFERENCES project_account ("id") ON DELETE CASCADE;
ALTER TABLE project_user_relationship ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE project_user_relationship ADD CONSTRAINT user_project_role UNIQUE (user_id, project_id, role_id);


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
  is_project BOOLEAN NOT NULL,
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
  project_id uuid,
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
ALTER TABLE setting_value ADD CONSTRAINT fk_project_id FOREIGN KEY (project_id) REFERENCES project_account(id) ON DELETE CASCADE;
ALTER TABLE setting_value ADD CONSTRAINT fk_setting FOREIGN KEY (setting_id) REFERENCES setting (id) ON DELETE CASCADE;
ALTER TABLE setting_value ADD CONSTRAINT user_project_id_uq UNIQUE NULLS NOT DISTINCT(setting_id, user_id, project_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_setting_project ON setting_value (setting_id, project_id) WHERE user_id IS NULL;


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

CREATE TYPE leave_type AS ENUM (
  'medical',
  'casual',
  'restricted',
  'common',
  'unpaid'
);


CREATE TABLE IF NOT EXISTS leave (
  id uuid PRIMARY KEY,
  sender_id uuid NOT NULL,
  receiver_id uuid NOT NULL,
  status leave_status NOT NULL,
  type leave_type NOT NULL,
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
  cc JSONB,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);

CREATE UNIQUE INDEX leave_uq ON leave (sender_id, period, date) WHERE is_deleted = false;
ALTER TABLE leave ADD CONSTRAINT fk_user_id FOREIGN KEY (sender_id)  REFERENCES user_account(id) ON DELETE CASCADE;
CREATE INDEX leave_user_idx ON leave (sender_id);
CREATE INDEX leave_created_on_idx ON leave (created_on);


CREATE TABLE IF NOT EXISTS pending_notification(
    id uuid PRIMARY KEY,
    data JSONB NOT NULL,
    connection_id TEXT NOT NULL,
    created_on TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);