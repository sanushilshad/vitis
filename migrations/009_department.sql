
CREATE TABLE IF NOT EXISTS department_account (
  id uuid PRIMARY KEY,
  display_name TEXT NOT NULL,
  is_active status DEFAULT 'active'::status NOT NULL,
  created_by  uuid NOT NULL,
  created_on TIMESTAMPTZ NOT NULL,
  updated_by uuid,
  updated_on TIMESTAMPTZ,
  deleted_by uuid,
  deleted_on TIMESTAMPTZ,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  is_test_account BOOLEAN NOT NULL DEFAULT false

);


CREATE TABLE IF NOT EXISTS business_user_department_relationship (
  id uuid PRIMARY KEY,
  user_id uuid NOT NULL,
  department_id uuid NOT NULL,
  business_id uuid NOT NULL,
  role_id uuid NOT NULL,
  verified BOOLEAN NOT NULL DEFAULT false,
  created_on TIMESTAMPTZ NOT NULL,
  updated_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid
);


ALTER TABLE business_user_department_relationship ADD CONSTRAINT "fk_user_id" FOREIGN KEY ("user_id") REFERENCES user_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_department_relationship ADD CONSTRAINT "fk_department_id" FOREIGN KEY ("department_id") REFERENCES department_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_department_relationship ADD CONSTRAINT "fk_business_id" FOREIGN KEY ("business_id") REFERENCES business_account ("id") ON DELETE CASCADE;
ALTER TABLE business_user_department_relationship ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE business_user_department_relationship ADD CONSTRAINT user_department_role UNIQUE (user_id, department_id, business_id);