
CREATE TABLE IF NOT EXISTS business_account (
  id uuid PRIMARY KEY,
  display_name TEXT NOT NULL,
  vectors jsonb NOT NULL,
  email TEXT, 
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


CREATE TABLE IF NOT EXISTS business_user_relationship (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  business_id uuid NOT NULL,
  role_id uuid NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT false,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid
);


ALTER TABLE business_user_relationship ADD CONSTRAINT "fk_user_id" FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_relationship ADD CONSTRAINT "fk_business_id" FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_relationship ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE business_user_relationship ADD CONSTRAINT user_business UNIQUE (user_id, business_id);





CREATE TABLE IF NOT EXISTS business_account_invitation_request (
  id uuid PRIMARY KEY,
  email TEXT NOT NULL,
  business_id uuid NOT NULL,
  role_id uuid NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT false,
  created_on TIMESTAMPTZ NOT NULL,
  created_by uuid NOT NULL,
  updated_on TIMESTAMPTZ,
  updated_by uuid
);

ALTER TABLE business_account_invitation_request ADD CONSTRAINT "fk_business_invitation_req" FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;
ALTER TABLE business_account_invitation_request ADD CONSTRAINT unique_business_email_invite UNIQUE (email, business_id);
CREATE INDEX idx_invite_email_business ON business_account_invitation_request (email, business_id);
