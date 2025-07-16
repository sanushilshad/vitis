use super::{
    models::RoleModel,
    schemas::{AccountRole, BulkRoleInsert, CreateRoleData},
};
use crate::schemas::Status;
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::{Execute, PgPool, QueryBuilder};
use uuid::Uuid;
// test case not needed
#[tracing::instrument(name = "get_role_model_by_type", skip(pool))]
pub async fn get_role_model(
    pool: &PgPool,
    query: &str,
) -> Result<Option<RoleModel>, anyhow::Error> {
    let row: Option<RoleModel> = sqlx::query_as!(
        RoleModel,
        r#"SELECT id, is_editable,  name, status as "status!:Status", created_on, created_by, is_deleted from role where name = $1 OR id::TEXT=$1"#,
        query
    )
    .fetch_optional(pool)
    .await.map_err(|e: sqlx::Error| {
            tracing::error!("Failed to execute query: {:?}", e);
            anyhow!(e).context("A database failure occurred while fetching role")
        })?;

    Ok(row)
}

// test case not needed
#[tracing::instrument(name = "get_role", skip(pool))]
pub async fn get_role(pool: &PgPool, query: &str) -> Result<Option<AccountRole>, anyhow::Error> {
    let role_model = get_role_model(pool, query).await?;
    match role_model {
        Some(role) => Ok(Some(role.into_schema())),
        None => Ok(None),
    }
}

#[tracing::instrument(name = "get_role_models", skip(pool))]
pub async fn get_role_models(
    pool: &PgPool,
    business_id: Option<Uuid>,
    department_id: Option<Uuid>,
    id_list: Option<Vec<Uuid>>,
    name_list: Option<Vec<&str>>,
    show_global: bool,
) -> Result<Vec<RoleModel>, anyhow::Error> {
    let mut query_builder = QueryBuilder::new(
        r#"
        SELECT 
            id, 
            name, 
            is_editable,
            status, 
            created_on, 
            created_by, 
            is_deleted
        FROM role 
        WHERE is_deleted = false AND name !='superadmin'
        "#,
    );

    if let Some(names) = name_list {
        if !names.is_empty() {
            query_builder.push(" AND name = ANY(");
            query_builder.push_bind(names);
            query_builder.push(")");
        }
    }

    if let Some(ids) = id_list {
        if !ids.is_empty() {
            tracing::info!(" IDS {:?}", ids);
            query_builder.push(" AND id = ANY(");
            query_builder.push_bind(ids);
            query_builder.push(")");
        }
    }

    if let Some(bid) = business_id {
        query_builder.push(" AND (business_id = ");
        tracing::info!("BUSINESS ID {}", bid);
        query_builder.push_bind(bid);
        if show_global {
            query_builder.push(
                " OR (business_id IS NULL AND department_id IS NULL AND is_editable = false)",
            );
        }
        query_builder.push(")");
    } else if let Some(did) = department_id {
        query_builder.push(" AND (department_id = ");
        query_builder.push_bind(did);
        if show_global {
            query_builder
                .push("OR (business_id IS NULL AND department_id IS NULL AND is_editable = false)");
        }
        query_builder.push(")");
    }

    let query = query_builder.build_query_as::<RoleModel>();
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    let roles = query.fetch_all(pool).await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while fetching roles")
    })?;

    Ok(roles)
}

#[tracing::instrument(name = "get_role_models", skip(pool))]
pub async fn get_roles(
    pool: &PgPool,
    business_id: Option<Uuid>,
    department_id: Option<Uuid>,
    id_list: Option<Vec<Uuid>>,
    name_list: Option<Vec<&str>>,
    show_default: bool,
) -> Result<Vec<AccountRole>, anyhow::Error> {
    let roles = get_role_models(
        pool,
        business_id,
        department_id,
        id_list,
        name_list,
        show_default,
    )
    .await?;
    let data = roles.into_iter().map(|a| a.into_schema()).collect();

    Ok(data)
}

#[tracing::instrument(name = "prepare bulk role data", skip(created_by))]
fn prepare_bulk_role_data<'a>(
    role_list: &'a Vec<CreateRoleData>,
    business_id: Option<Uuid>,
    created_by: Uuid,
    created_on: DateTime<Utc>,
) -> Option<BulkRoleInsert<'a>> {
    let mut name_list = vec![];
    let mut created_on_list = vec![];
    let mut id_list = vec![];
    let mut business_id_list = vec![];
    let mut created_by_list = vec![];
    let mut is_editable_list = vec![];
    if role_list.is_empty() {
        return None;
    }
    for role in role_list.iter() {
        created_on_list.push(created_on);
        created_by_list.push(created_by);
        if let Some(id) = role.id {
            id_list.push(id);
        } else {
            id_list.push(Uuid::new_v4());
        }
        name_list.push(role.name.as_ref());
        business_id_list.push(business_id);
        is_editable_list.push(true);
    }
    Some(BulkRoleInsert {
        id: id_list,
        name: name_list,
        created_on: created_on_list,
        created_by: created_by_list,
        business_id: business_id_list,
        is_editable: is_editable_list,
    })
}

// test case not needed
#[tracing::instrument(name = "save role to database", skip(pool, data))]
async fn save_role_to_database<'a>(
    pool: &PgPool,
    data: BulkRoleInsert<'a>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        INSERT INTO role (id, created_by, created_on, name, business_id, is_editable)
        SELECT * FROM UNNEST($1::uuid[], $2::uuid[], $3::TIMESTAMP[],  $4::TEXT[], $5::uuid[], $6::bool[]) 
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
        updated_by = EXCLUDED.created_by,
        updated_on = EXCLUDED.created_on
        "#,
        &data.id[..] as &[Uuid],
        &data.created_by[..] as &[Uuid],
        &data.created_on[..] as &[DateTime<Utc>],
        &data.name[..] as &[&str],
        &data.business_id[..] as &[Option<Uuid>],
        &data.is_editable[..] as &[bool]
    );
    let query_string = query.sql();
    println!("Generated SQL query for: {}", query_string);
    query.execute(pool).await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while saving role ")
    })?;

    Ok(())
}

pub async fn save_role(
    pool: &PgPool,
    role_list: &Vec<CreateRoleData>,
    business_id: Option<Uuid>,
    created_by: Uuid,
    created_on: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let data = prepare_bulk_role_data(role_list, business_id, created_by, created_on);
    if let Some(data) = data {
        save_role_to_database(pool, data).await?;
    }

    Ok(())
}

#[tracing::instrument(name = "soft deleted role", skip(pool))]
pub async fn soft_delete_role(
    pool: &PgPool,
    business_id: Uuid,
    role_id: Uuid,
    deleted_by: Uuid,
    deleted_on: DateTime<Utc>,
) -> Result<(), anyhow::Error> {
    let query = sqlx::query!(
        r#"
        UPDATE role
        SET
        is_deleted=true, 
        deleted_by = $1,
        deleted_on = $2
        Where id = $3
        "#,
        deleted_by,
        deleted_on,
        role_id
    );
    query.execute(pool).await.map_err(|e: sqlx::Error| {
        tracing::error!("Failed to execute query: {:?}", e);
        anyhow!(e).context("A database failure occurred while deleting role ")
    })?;
    Ok(())
}
