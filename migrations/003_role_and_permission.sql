
CREATE TABLE IF NOT EXISTS role (
  id uuid PRIMARY KEY,
  name TEXT NOT NULL,
  status status DEFAULT 'active'::status NOT NULL,
  is_editable BOOLEAN NOT NULL default false,
  created_on TIMESTAMPTZ NOT NULL,
  business_id  uuid,
  department_id uuid,
  updated_on TIMESTAMPTZ,
  deleted_on TIMESTAMPTZ,
  created_by uuid NOT NULL,
  updated_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN  NOT NULL DEFAULT false
);

ALTER TABLE role ADD CONSTRAINT fk_role_business_id FOREIGN KEY (business_id) REFERENCES business_account(id) ON DELETE CASCADE;
ALTER TABLE role ADD CONSTRAINT unique_role_name UNIQUE NULLS NOT DISTINCT(business_id, department_id, name);

ALTER TABLE role ADD CONSTRAINT unique_role_name UNIQUE (role_name);
ALTER TABLE setting_value ADD CONSTRAINT fk_business_id FOREIGN KEY (business_id) REFERENCES business_account(id) ON DELETE CASCADE;

CREATE TABLE IF NOT EXISTS user_role(
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
  name TEXT NOT NULL,
  description TEXT,
  is_business BOOLEAN,
  is_user BOOLEAN,
  is_department BOOLEAN,
  is_global BOOLEAN,
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
  deleted_on TIMESTAMPTZ,
  created_by uuid,
  deleted_by uuid,
  is_deleted BOOLEAN NOT NULL DEFAULT false
);


ALTER TABLE role_permission ADD CONSTRAINT "fk_permission_id" FOREIGN KEY ("permission_id") REFERENCES permission ("id") ON DELETE CASCADE;
ALTER TABLE role_permission ADD CONSTRAINT "fk_role_id" FOREIGN KEY ("role_id") REFERENCES role ("id") ON DELETE CASCADE;
ALTER TABLE permission ADD CONSTRAINT permission_name UNIQUE (permission_name);
ALTER TABLE role_permission ADD CONSTRAINT permission_role_id UNIQUE (permission_id, role_id);


