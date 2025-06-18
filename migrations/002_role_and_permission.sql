-- INSERT INTO  role(id, role_name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'developer', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO  role(id, role_name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'qa', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO  role(id, role_name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'lead', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

INSERT INTO  role(id, role_name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'superadmin', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO  role(id, role_name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'admin', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

INSERT INTO  role(id, role_name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'employee', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);


INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'associate:user-project', 'User Project Association', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:users', 'list users', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);


INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:leave-request:self', 'create leave request self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:leave-request', 'create leave request', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:leave-request-status', 'update leave request status', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'approve:leave-request', 'approve leave request', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:leave-request', 'list leave requests', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:leave-request:self', 'list leave requests self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);



INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:user-setting', 'create user setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:user-setting:self', 'create user setting self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:project-setting:self', 'create project setting self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:project-setting', 'create project setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:global-setting', 'create global setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'associate:user-department', 'User Department Association', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:department', 'Create Department', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

WITH superadmin_role AS (SELECT "id" FROM "role" WHERE "role_name" = 'superadmin' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), superadmin_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid  FROM superadmin_role, "permission" WHERE "permission"."permission_name" NOT LIKE '%:self';
WITH admin_role AS (SELECT "id" FROM "role" WHERE "role_name" = 'admin' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by")SELECT uuid_generate_v4(), admin_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(),'00000000-0000-0000-0000-000000000000'::uuid FROM admin_role, "permission" WHERE  "permission"."permission_name" NOT LIKE '%:self'  AND "permission"."permission_name"  NOT IN ('create:global-setting');
WITH employee_role AS (SELECT "id" FROM "role" WHERE "role_name" = 'qa' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), employee_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid FROM employee_role, "permission" WHERE "permission"."permission_name" IN ('create:user-setting:self', 'create:leave-request:self', 'update:leave-request-status', 'list:leave-request:self');

-- WITH lead_role AS (SELECT "id" FROM "role" WHERE "role_name" = 'lead' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by")SELECT uuid_generate_v4(), lead_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(),'00000000-0000-0000-0000-000000000000'::uuid FROM lead_role, "permission" WHERE "permission"."permission_name" NOT LIKE '%:self' AND "permission"."permission_name"  NOT IN ('create:department', 'create:global-setting', 'create:project-setting',  'list:leave-request');
-- WITH developer_role AS (SELECT "id" FROM "role" WHERE "role_name" = 'developer' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), developer_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid FROM developer_role, "permission" WHERE "permission"."permission_name" IN ('create:user-setting:self', 'create:leave-request:self', 'update:leave-request-status', 'list:leave-request:self');
-- WITH qa_role AS (SELECT "id" FROM "role" WHERE "role_name" = 'qa' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), qa_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid FROM qa_role, "permission" WHERE "permission"."permission_name" IN ('create:user-setting:self', 'create:leave-request:self', 'update:leave-request-status', 'list:leave-request:self');



-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:user', 'read user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:user', 'list user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:user', 'create user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:user', 'update user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:user', 'download user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'share:user', 'share user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'archive:user', 'archive user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'delete:user', 'delete user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:business-account', 'read business-account', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:business-account', 'list business-account', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:business-account', 'update business-account', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:product', 'read:product', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:product', 'list:product', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:product', 'create:product', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:product:self', 'update:product:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'delete:product:self', 'delete:product:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:product:self', 'download:product:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:sku', 'read:sku', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:sku', 'list:sku', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:sku', 'create:sku', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:sku:self', 'delete:sku:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:sku:self', 'download:sku:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:asset', 'read:asset', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:asset', 'list:asset', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:asset', 'create:asset', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:asset:self', 'update:asset:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'delete:asset:self', 'delete:asset:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:asset:self', 'download:asset:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'delete:order:self', 'delete:order:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:order:self', 'download:order:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:invoice', 'read:invoice', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:policy', 'read:policy', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:policy', 'list:policy', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:log', 'read:log', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, permission_name, permission_description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:log', 'list:log', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);




