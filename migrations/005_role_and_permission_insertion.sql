

INSERT INTO  role(id, name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'superadmin', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO  role(id, name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'admin', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
INSERT INTO  role(id, name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'user', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);


INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'associate:user-business', 'User Business Association', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:users', 'list users', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false, false);


INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:user-business', 'List User Business', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:user-business:self', 'List User Business Self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'delete:business', 'Delete Business', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);



INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:leave-request:self', 'create leave request self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:leave-request', 'create leave request', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'update:leave-request-status', 'update leave request status', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'approve:leave-request', 'approve leave request', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:leave-request', 'list leave requests', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:leave-request:self', 'list leave requests self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:leave-type', 'create Leave Type', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:user-leave:self', 'List Leave', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'list:user-leave', 'List Leave Self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);



INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:user-setting', 'create user setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, false, true);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:user-setting:self', 'create user setting self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, false, true);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:business-setting:self', 'create business setting self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:business-setting', 'create business setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:department-setting:self', 'create department setting self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:department-setting', 'create department setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);

INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:global-setting', 'create global setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, false, true);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:user-business-setting', 'create User-Business setting', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'send:business-invite', 'send business invite', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'update:business', 'update business', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);

INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'disassociate:user-business', 'disassociate business self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'disassociate:user-business:self', 'disassociate business', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);

INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:business-role', 'create business role', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);

INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:department', 'Create Department', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, true, false, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'disassociate:user-department:self', 'Disassociate Department Self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'disassociate:user-department', 'Disassociate Department', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'associate:user-department', 'Associate Department', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'delete:department', 'Delete Department', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'update:department', 'Update Department', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);
INSERT INTO permission(id, name, description, created_on, created_by,  is_business, is_department, is_user)VALUES(uuid_generate_v4(), 'create:department-role', 'Create Department Role', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid, false, true, false);




WITH superadmin_role AS (SELECT "id" FROM "role" WHERE "name" = 'superadmin' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), superadmin_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid  FROM superadmin_role, "permission" WHERE "permission"."name" NOT LIKE '%:self';
WITH admin_role AS (SELECT "id" FROM "role" WHERE "name" = 'admin' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by")SELECT uuid_generate_v4(), admin_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(),'00000000-0000-0000-0000-000000000000'::uuid FROM admin_role, "permission" WHERE  "permission"."name" NOT LIKE '%:self'  AND "permission"."name"  NOT IN ('create:global-setting', 'create:user-business-setting', 'list:user-business', 'list:users');
WITH user_role AS (SELECT "id" FROM "role" WHERE "name" = 'user' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), user_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid FROM user_role, "permission" WHERE "permission"."name" IN ('create:user-setting:self', 'create:user-business-setting', 'create:leave-request:self', 'update:leave-request-status', 'list:leave-request:self',  'list:user-business:self', 'list:user-leave:self');

-- INSERT INTO  role(id, name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'developer', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO  role(id, name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'qa', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO  role(id, name, role_status, created_on, created_by) VALUES(uuid_generate_v4(), 'lead', 'active'::status, CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'associate:user-department', 'User Department Association', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);


-- WITH lead_role AS (SELECT "id" FROM "role" WHERE "name" = 'lead' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by")SELECT uuid_generate_v4(), lead_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(),'00000000-0000-0000-0000-000000000000'::uuid FROM lead_role, "permission" WHERE "permission"."name" NOT LIKE '%:self' AND "permission"."name"  NOT IN ('create:department', 'create:global-setting', 'create:project-setting',  'list:leave-request');
-- WITH developer_role AS (SELECT "id" FROM "role" WHERE "name" = 'developer' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), developer_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid FROM developer_role, "permission" WHERE "permission"."name" IN ('create:user-setting:self', 'create:leave-request:self', 'update:leave-request-status', 'list:leave-request:self');
-- WITH qa_role AS (SELECT "id" FROM "role" WHERE "name" = 'qa' LIMIT 1) INSERT INTO "role_permission" ("id", "role_id", "permission_id", "created_on", "created_by") SELECT uuid_generate_v4(), qa_role."id" AS "role_id", "permission"."id" AS "permission_id", NOW(), '00000000-0000-0000-0000-000000000000'::uuid FROM qa_role, "permission" WHERE "permission"."name" IN ('create:user-setting:self', 'create:leave-request:self', 'update:leave-request-status', 'list:leave-request:self');


-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:user', 'download user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'share:user', 'share user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'archive:user', 'archive user', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:business-account', 'read business-account', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:business-account', 'list business-account', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:business-account', 'update business-account', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:sku', 'create:sku', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:sku:self', 'delete:sku:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:sku:self', 'download:sku:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:asset', 'read:asset', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:asset', 'list:asset', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'create:asset', 'create:asset', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'update:asset:self', 'update:asset:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'delete:asset:self', 'delete:asset:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'download:asset:self', 'download:asset:self', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);

-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:invoice', 'read:invoice', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:policy', 'read:policy', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:policy', 'list:policy', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'read:log', 'read:log', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);
-- INSERT INTO permission(id, name, description, created_on, created_by)VALUES(uuid_generate_v4(), 'list:log', 'list:log', CURRENT_TIMESTAMP, '00000000-0000-0000-0000-000000000000'::uuid);




