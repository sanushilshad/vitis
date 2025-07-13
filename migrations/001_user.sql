

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";




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
    full_mobile_no TEXT NOT NULL,
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




CREATE TYPE vector_type AS ENUM (
    'mobile_no',
    'email'
);








