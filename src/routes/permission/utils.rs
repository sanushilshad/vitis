use chrono::{DateTime, Utc};
use sqlx::{PgPool, QueryBuilder};
use uuid::Uuid;

use super::{
    models::PermissionModel,
    schemas::{BulkPermissionInsert, Permission, PermissionLevel},
};
use anyhow::anyhow;

pub async fn fetch_permission_models_for_role(
    pool: &PgPool,
    role_id: Uuid,
    business_id: Uuid,
) -> Result<Vec<PermissionModel>, anyhow::Error> {
    let permissions = sqlx::query_as!(
        PermissionModel,
        r#"
        SELECT p.id, p.name, p.description
        FROM role_permission rp
        INNER JOIN permission p ON rp.permission_id = p.id
        INNER JOIN role r ON rp.role_id = r.id
        WHERE rp.role_id = $1
          AND r.business_id = $2
          AND rp.is_deleted = false
          AND p.is_deleted = false
          AND r.is_deleted = false
        "#,
        role_id,
        business_id
    )
    .fetch_all(pool)
    .await?;

    Ok(permissions)
}
pub async fn fetch_permissions_for_role(
    pool: &PgPool,
    role_id: Uuid,
    business_id: Uuid,
) -> Result<Vec<Permission>, anyhow::Error> {
    let models = fetch_permission_models_for_role(pool, role_id, business_id).await?;

    let permissions = models
        .into_iter()
        .map(|a| a.into_schema())
        .collect::<Vec<Permission>>();

    Ok(permissions)
}

pub async fn fetch_permission_models_by_scope(
    pool: &PgPool,
    types: Vec<PermissionLevel>,
    id_list: Option<&Vec<Uuid>>,
) -> Result<Vec<PermissionModel>, anyhow::Error> {
    let mut query_builder =
        QueryBuilder::new("SELECT id, name, description FROM permission WHERE is_deleted = false");

    // Add scope-based filtering
    if !types.is_empty() {
        query_builder.push(" AND (");

        let mut first = true;
        for permission_type in &types {
            if !first {
                query_builder.push(" OR ");
            }

            match permission_type {
                PermissionLevel::Global => {
                    query_builder.push("is_global = true");
                }
                PermissionLevel::User => {
                    query_builder.push("is_user = true");
                }
                PermissionLevel::Business => {
                    query_builder.push("is_business = true");
                }
                PermissionLevel::Department => {
                    query_builder.push("is_department = true");
                }
            }

            first = false;
        }

        query_builder.push(")");
    }

    if let Some(ref ids) = id_list {
        if !ids.is_empty() {
            query_builder
                .push(" AND id = ANY(")
                .push_bind(ids)
                .push(")");
        }
    }

    let query = query_builder.build_query_as::<PermissionModel>();
    let result = query.fetch_all(pool).await?;

    Ok(result)
}

pub async fn fetch_permissions_by_scope(
    pool: &PgPool,
    types: Vec<PermissionLevel>,
    id_list: Option<&Vec<Uuid>>,
) -> Result<Vec<Permission>, anyhow::Error> {
    let models = fetch_permission_models_by_scope(pool, types, id_list).await?;
    let permissions = models
        .into_iter()
        .map(|a| a.into_schema())
        .collect::<Vec<Permission>>();

    Ok(permissions)
}

#[tracing::instrument(name = "prepare bulk role data", skip(created_by))]
fn prepare_bulk_role_data(
    permission_id_list: Vec<Uuid>,
    role_id: Uuid,
    created_by: Uuid,
    created_on: DateTime<Utc>,
) -> BulkPermissionInsert {
    let mut role_id_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut created_by_list = vec![];
    for _ in permission_id_list.iter() {
        created_on_list.push(created_on);
        created_by_list.push(created_by);
        id_list.push(Uuid::new_v4());
        role_id_list.push(role_id);
    }
    BulkPermissionInsert {
        id: id_list,
        role_id: role_id_list,
        created_on: created_on_list,
        created_by: created_by_list,
        permission_id: permission_id_list,
    }
}

// test case not needed
#[tracing::instrument(name = "save role to database", skip(pool, data))]
async fn save_role_permissions_to_database(
    pool: &PgPool,
    data: BulkPermissionInsert,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO role_permission (id, created_by, created_on, role_id, permission_id)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::TIMESTAMP[],  $4::uuid[], $5::uuid[]) 
        ON CONFLICT (id) DO UPDATE
        SET updated_by = EXCLUDED.created_by,
        updated_on = EXCLUDED.created_on
        "#,
        &data.id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.role_id[..] as &[Uuid],
        &data.permission_id[..] as &[Uuid],
    );

    query.execute(pool).await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving role permission association")
    })?;

    Ok(())
}

pub async fn associate_permission_to_role(
    pool: &PgPool,
    role_id: Uuid,
    permission_id_list: Vec<Uuid>,
    user_id: Uuid,
) -> Result<(), anyhow::Error> {
    if !permission_id_list.is_empty() {
        let data = prepare_bulk_role_data(permission_id_list, role_id, user_id, Utc::now());
        save_role_permissions_to_database(pool, data).await?
    }
    Ok(())
}

pub async fn delete_role_permission_associations(
    pool: &PgPool,
    permission_id_list: Vec<Uuid>,
    role_id: Uuid,
) -> Result<(), anyhow::Error> {
    let mut query_builder = QueryBuilder::new("DELETE FROM role_permission WHERE role_id = ");

    query_builder.push_bind(role_id);

    if !permission_id_list.is_empty() {
        query_builder
            .push(" AND permission_id = ANY(")
            .push_bind(permission_id_list)
            .push(")");
    }

    // query_builder.push(")");

    let query = query_builder.build();

    query.execute(pool).await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while deleting role permission association")
    })?;

    Ok(())
}
